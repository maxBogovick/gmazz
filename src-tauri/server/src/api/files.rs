use axum::{
    extract::{State, Path, Query, Request},
    body::Body,
    http::{header, HeaderValue, HeaderMap},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tower_http::services::ServeFile;
use tower::ServiceExt;
use utoipa::{IntoParams, ToSchema};

use crate::{AppState, auth::AuthenticatedApp, error::AppError};

#[derive(Serialize, ToSchema)]
pub struct FileResponse {
    /// Unique file identifier (UUID)
    pub id: String,
    /// Original filename
    pub original_name: String,
    /// Size in bytes
    pub size_bytes: i64,
    /// MIME type
    pub mime_type: Option<String>,
    /// SHA256 checksum
    pub checksum: Option<String>,
    /// Creation timestamp (Unix epoch)
    pub created_at: i64,
}

/// Upload a file
#[utoipa::path(
    post,
    path = "/v1/files",
    request_body(content = Vec<u8>, description = "File content stream"),
    params(
        ("X-File-Name" = String, Header, description = "Original filename"),
        ("Content-Type" = Option<String>, Header, description = "MIME type")
    ),
    responses(
        (status = 201, description = "File created successfully", body = FileResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 413, description = "Payload too large")
    ),
    security(("api_key" = []))
)]
pub async fn upload_handler(
    State(state): State<Arc<AppState>>,
    AuthenticatedApp(app): AuthenticatedApp,
    headers: HeaderMap,
    body: Body,
) -> Result<impl IntoResponse, AppError> {
    let original_name = headers
        .get("X-File-Name")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown.bin")
        .to_string();

    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let res = state.file_service.upload_file(&app.id, original_name, content_type, body).await?;
    
    Ok((axum::http::StatusCode::CREATED, Json(res)))
}

/// Download a file
#[utoipa::path(
    get,
    path = "/v1/files/{id}",
    params(("id" = String, Path, description = "File ID")),
    responses(
        (status = 200, description = "File stream", body = Vec<u8>),
        (status = 206, description = "Partial content"),
        (status = 404, description = "File not found")
    ),
)]
pub async fn download_handler(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
    request: Request,
) -> Result<Response, AppError> {
    // Allow reading ANY file publicly (UUID prevents enumeration)
    let meta = state.file_service.get_file_metadata_any_owner(&file_id).await?;
    let abs_path = state.file_service.get_absolute_path(&meta.stored_path);
    
    if !abs_path.exists() {
        return Err(AppError::NotFound("File missing on disk".to_string()));
    }

    let service = ServeFile::new(abs_path);
    match service.oneshot(request).await {
        Ok(mut response) => {
             if let Some(mime) = meta.mime_type {
                 if let Ok(val) = mime.parse::<HeaderValue>() {
                     response.headers_mut().insert(header::CONTENT_TYPE, val);
                 }
             }
             let safe_name = meta.original_name.replace("\"", "\\\"");
             if let Ok(val) = format!("attachment; filename=\"{}\"", safe_name).parse::<HeaderValue>() {
                 response.headers_mut().insert(header::CONTENT_DISPOSITION, val);
             }
             if let Some(sum) = meta.checksum {
                 if let Ok(val) = sum.parse::<HeaderValue>() {
                     response.headers_mut().insert(header::ETAG, val);
                 }
             }
             Ok(response.into_response())
        },
        Err(e) => Err(AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }
}

/// Get file metadata
#[utoipa::path(
    get,
    path = "/v1/files/{id}/meta",
    params(("id" = String, Path, description = "File ID")),
    responses(
        (status = 200, description = "File metadata", body = FileResponse),
        (status = 404, description = "File not found")
    ),
    security(("api_key" = []))
)]
pub async fn get_file_meta_handler(
    State(state): State<Arc<AppState>>,
    AuthenticatedApp(app): AuthenticatedApp,
    Path(file_id): Path<String>,
) -> Result<Json<FileResponse>, AppError> {
    let meta = state.file_service.get_file_metadata(&file_id, &app.id).await?;

    Ok(Json(FileResponse {
        id: meta.id,
        original_name: meta.original_name,
        size_bytes: meta.size_bytes,
        mime_type: meta.mime_type,
        checksum: meta.checksum,
        created_at: meta.created_at,
    }))
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List files
#[utoipa::path(
    get,
    path = "/v1/files",
    params(ListParams),
    responses((status = 200, description = "List of files", body = [FileResponse])),
    security(("api_key" = []))
)]
pub async fn list_handler(
    State(state): State<Arc<AppState>>,
    AuthenticatedApp(app): AuthenticatedApp,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<FileResponse>>, AppError> {
    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = params.offset.unwrap_or(0).max(0);

    let files = state.file_service.list_files(&app.id, limit, offset).await?;

    let response = files.into_iter().map(|f| FileResponse {
        id: f.id,
        original_name: f.original_name,
        size_bytes: f.size_bytes,
        mime_type: f.mime_type,
        checksum: f.checksum,
        created_at: f.created_at,
    }).collect();

    Ok(Json(response))
}

/// Delete a file
#[utoipa::path(
    delete,
    path = "/v1/files/{id}",
    params(("id" = String, Path, description = "File ID")),
    responses(
        (status = 204, description = "File deleted"),
        (status = 404, description = "File not found")
    ),
    security(("api_key" = []))
)]
pub async fn delete_handler(
    State(state): State<Arc<AppState>>,
    AuthenticatedApp(app): AuthenticatedApp,
    Path(file_id): Path<String>,
) -> Result<axum::http::StatusCode, AppError> {
    state.file_service.soft_delete_file(&file_id, &app.id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
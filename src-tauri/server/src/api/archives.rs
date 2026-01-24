use axum::{
    extract::{State, Path},
    http::{header, HeaderValue},
    response::{IntoResponse},
    Json,
    body::Body,
};
use std::sync::Arc;
use utoipa::ToSchema;
use serde::Serialize;

use crate::{AppState, auth::AuthenticatedApp, error::AppError};

#[derive(Serialize, ToSchema)]
pub struct ArchiveResponse {
    pub archive_id: String,
    pub status: String,
    pub expires_at: i64,
}

#[derive(Serialize, ToSchema)]
pub struct ArchiveStatusResponse {
    pub status: String,
}

/// Create an archive
#[utoipa::path(
    post,
    path = "/v1/archives",
    responses(
        (status = 202, description = "Archive creation started", body = ArchiveResponse),
        (status = 500, description = "Internal server error")
    ),
    security(("api_key" = []))
)]
pub async fn create_archive_handler(
    State(state): State<Arc<AppState>>,
    AuthenticatedApp(app): AuthenticatedApp,
) -> Result<impl IntoResponse, AppError> {
    let res = state.archive_service.start_archiving(&app.id).await?;
    Ok((axum::http::StatusCode::ACCEPTED, Json(res)))
}

/// Get archive status or download
#[utoipa::path(
    get,
    path = "/v1/archives/{id}",
    params(("id" = String, Path, description = "Archive ID")),
    responses(
        (status = 200, description = "Archive stream", body = Vec<u8>),
        (status = 202, description = "Archive pending", body = ArchiveStatusResponse),
        (status = 404, description = "Archive not found")
    ),
    security(("api_key" = []))
)]
pub async fn get_archive_handler(
    State(state): State<Arc<AppState>>,
    AuthenticatedApp(app): AuthenticatedApp,
    Path(archive_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let archive = state.archive_service.get_archive(&archive_id, &app.id).await?;

    match archive.status.as_str() {
        "pending" => Ok((axum::http::StatusCode::ACCEPTED, Json(ArchiveStatusResponse { status: "pending".to_string() })).into_response()),
        "failed" => Err(AppError::Anyhow(anyhow::anyhow!("Archive generation failed"))),
        "ready" => {
            if let Some(path) = archive.file_path {
                 let abs_path = state.storage.get_absolute_path(&path);
                 let file = tokio::fs::File::open(&abs_path).await.map_err(|e| AppError::Io(e))?;

                 let stream = tokio_util::io::ReaderStream::new(file);
                 let body = Body::from_stream(stream);

                 let mut headers = axum::http::HeaderMap::new();
                 headers.insert(header::CONTENT_TYPE, "application/zip".parse().unwrap());
                 headers.insert(
                    header::CONTENT_DISPOSITION, 
                    format!("attachment; filename=\"archive_{}.zip\"", archive_id).parse().unwrap()
                 );
                 
                 Ok((headers, body).into_response())
            } else {
                 Err(AppError::Anyhow(anyhow::anyhow!("Invalid archive state")))
            }
        },
        _ => Err(AppError::Anyhow(anyhow::anyhow!("Unknown status"))),
    }
}

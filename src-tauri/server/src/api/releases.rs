use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{AppState, auth::AuthenticatedApp, error::AppError};

#[derive(Serialize, ToSchema)]
pub struct ReleaseResponse {
    pub id: String,
    pub file_id: String,
    pub version_name: Option<String>,
    pub description: Option<String>,
    pub created_at: i64,
    pub file_original_name: String,
    pub file_size_bytes: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateReleaseRequest {
    pub file_id: String,
    pub version_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, IntoParams)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Create a new release
#[utoipa::path(
    post,
    path = "/v1/releases",
    request_body = CreateReleaseRequest,
    responses((status = 201, description = "Release created", body = ReleaseResponse)),
    security(("api_key" = []))
)]
pub async fn create_release_handler(
    State(state): State<Arc<AppState>>,
    AuthenticatedApp(app): AuthenticatedApp,
    Json(payload): Json<CreateReleaseRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.release_service.create_release(
        &app.id, 
        &payload.file_id, 
        payload.version_name, 
        payload.description
    ).await?;
    
    Ok((StatusCode::CREATED, Json(ReleaseResponse {
        id: res.id,
        file_id: res.file_id,
        version_name: res.version_name,
        description: res.description,
        created_at: res.created_at,
        file_original_name: res.file_original_name,
        file_size_bytes: res.file_size_bytes,
    })))
}

/// Get the latest active release
#[utoipa::path(
    get,
    path = "/v1/releases/latest",
    responses(
        (status = 200, description = "Latest release", body = ReleaseResponse),
        (status = 404, description = "No releases found")
    ),
    security(("api_key" = []))
)]
pub async fn get_latest_release_handler(
    State(state): State<Arc<AppState>>,
    AuthenticatedApp(app): AuthenticatedApp,
) -> Result<Json<ReleaseResponse>, AppError> {
    let res = state.release_service.get_latest_release(&app.id).await?;
    
    Ok(Json(ReleaseResponse {
        id: res.id,
        file_id: res.file_id,
        version_name: res.version_name,
        description: res.description,
        created_at: res.created_at,
        file_original_name: res.file_original_name,
        file_size_bytes: res.file_size_bytes,
    }))
}

/// List releases history
#[utoipa::path(
    get,
    path = "/v1/releases",
    params(ListParams),
    responses((status = 200, description = "List of releases", body = [ReleaseResponse])),
    security(("api_key" = []))
)]
pub async fn list_releases_handler(
    State(state): State<Arc<AppState>>,
    AuthenticatedApp(app): AuthenticatedApp,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<ReleaseResponse>>, AppError> {
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let offset = params.offset.unwrap_or(0).max(0);

    let releases = state.release_service.list_releases(&app.id, limit, offset).await?;

    let response = releases.into_iter().map(|r| ReleaseResponse {
        id: r.id,
        file_id: r.file_id,
        version_name: r.version_name,
        description: r.description,
        created_at: r.created_at,
        file_original_name: r.file_original_name,
        file_size_bytes: r.file_size_bytes,
    }).collect();

    Ok(Json(response))
}

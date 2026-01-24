use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{ToSchema, IntoParams};

use crate::{AppState, error::AppError};

#[derive(Deserialize, ToSchema)]
pub struct CreateKeyRequest {
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreateKeyResponse {
    pub id: String,
    pub name: String,
    pub api_key: String,
}

#[derive(Serialize, ToSchema)]
pub struct AppKeyResponse {
    pub id: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
}

/// Create a new API Key
#[utoipa::path(
    post,
    path = "/admin/keys",
    request_body = CreateKeyRequest,
    responses((status = 201, description = "Key created", body = CreateKeyResponse)),
    security(("admin_secret" = []))
)]
pub async fn create_key_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateKeyRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.admin_service.create_key(payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

/// List all API Keys
#[utoipa::path(
    get,
    path = "/admin/keys",
    responses((status = 200, description = "List of keys", body = [AppKeyResponse])),
    security(("admin_secret" = []))
)]
pub async fn list_keys_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AppKeyResponse>>, AppError> {
    let apps = state.admin_service.list_keys().await?;
    let response = apps.into_iter().map(|app| AppKeyResponse {
        id: app.id,
        name: app.name,
        is_active: app.is_active,
        created_at: app.created_at,
        last_used_at: app.last_used_at,
    }).collect();
    Ok(Json(response))
}

/// Revoke API Key
#[utoipa::path(
    put,
    path = "/admin/keys/{id}/revoke",
    params(("id" = String, Path, description = "App ID")),
    responses((status = 200, description = "Key revoked")),
    security(("admin_secret" = []))
)]
pub async fn revoke_key_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.admin_service.update_key_status(&id, false).await?;
    Ok(StatusCode::OK)
}

/// Activate API Key
#[utoipa::path(
    put,
    path = "/admin/keys/{id}/activate",
    params(("id" = String, Path, description = "App ID")),
    responses((status = 200, description = "Key activated")),
    security(("admin_secret" = []))
)]
pub async fn activate_key_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.admin_service.update_key_status(&id, true).await?;
    Ok(StatusCode::OK)
}
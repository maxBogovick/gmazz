use axum::{
    extract::{State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::{AppState, api::admin::{CreateKeyResponse}, auth::hash_api_key};

/// Generate a Guest API Key
#[utoipa::path(
    post,
    path = "/v1/auth/guest",
    responses((status = 201, description = "Guest Key created", body = CreateKeyResponse)),
)]
pub async fn create_guest_key_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    let max_keys = state.config.security.max_api_keys as i64;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM apps")
        .fetch_one(&state.repo.pool)
        .await
        .map_err(|e| crate::error::AppError::Anyhow(e.into()))?;

    if count >= max_keys {
        return Err(crate::error::AppError::BadRequest("Max API keys reached".to_string()));
    }

    // Generate a secure key
    let plain_key = Uuid::new_v4().to_string().replace("-", "") + &Uuid::new_v4().to_string().replace("-", "");
    let hash = hash_api_key(&state.config.security.server_secret, &plain_key);
    
    // Create "Guest" App
    // We append a random suffix to name to track sessions if we want, or just "Guest Client"
    let name = format!("Guest Client {}", Uuid::new_v4().to_string().chars().take(8).collect::<String>());
    
    let id = state.repo.create_app(&name, &hash).await?;
    
    Ok((StatusCode::CREATED, Json(CreateKeyResponse {
        id,
        name,
        api_key: plain_key,
    })))
}

use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::{AppState, auth::AuthenticatedApp, error::AppError};

/// Activate a specific release
#[utoipa::path(
    put,
    path = "/v1/releases/{id}/activate",
    params(("id" = String, Path, description = "Release ID")),
    responses(
        (status = 200, description = "Release activated"),
        (status = 404, description = "Release not found")
    ),
    security(("api_key" = []))
)]
pub async fn activate_release_handler(
    State(state): State<Arc<AppState>>,
    AuthenticatedApp(_app): AuthenticatedApp, // Ensure authenticated
    Path(release_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state.release_service.activate_release(&release_id).await?;
    
    // Instant Reload
    if let Err(e) = crate::services::ReleaseService::reload_active_db(&state).await {
        tracing::error!("Failed to reload active DB after activation: {}", e);
        return Err(AppError::Anyhow(anyhow::anyhow!("Failed to reload DB")));
    }

    Ok(StatusCode::OK)
}

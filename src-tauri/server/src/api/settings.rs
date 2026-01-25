use axum::{
    extract::{State, Path},
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{AppState, error::AppError};

#[derive(Serialize, ToSchema)]
pub struct SettingResponse {
    pub key: String,
    pub value: String,
    pub updated_at: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct SetSettingRequest {
    pub value: String,
}

#[derive(Serialize, ToSchema)]
pub struct AllSettingsResponse {
    pub settings: Vec<SettingResponse>,
}

/// Get all settings (public endpoint)
#[utoipa::path(
    get,
    path = "/v1/public/settings",
    responses((status = 200, description = "All settings", body = AllSettingsResponse)),
)]
pub async fn get_all_settings_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AllSettingsResponse>, AppError> {
    let settings = state.repo.get_all_settings()
        .await
        .map_err(|e| AppError::Anyhow(e))?;

    let settings = settings.into_iter().map(|s| SettingResponse {
        key: s.key,
        value: s.value,
        updated_at: s.updated_at,
    }).collect();

    Ok(Json(AllSettingsResponse { settings }))
}

/// Get a specific setting (public endpoint)
#[utoipa::path(
    get,
    path = "/v1/public/settings/{key}",
    params(("key" = String, Path, description = "Setting key")),
    responses(
        (status = 200, description = "Setting value", body = SettingResponse),
        (status = 404, description = "Setting not found")
    ),
)]
pub async fn get_setting_handler(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Result<Json<SettingResponse>, AppError> {
    let setting = state.repo.get_setting(&key)
        .await
        .map_err(|e| AppError::Anyhow(e))?
        .ok_or_else(|| AppError::NotFound(format!("Setting '{}' not found", key)))?;

    Ok(Json(SettingResponse {
        key: setting.key,
        value: setting.value,
        updated_at: setting.updated_at,
    }))
}

/// Set a setting (requires API key)
#[utoipa::path(
    put,
    path = "/v1/settings/{key}",
    params(("key" = String, Path, description = "Setting key")),
    request_body = SetSettingRequest,
    responses(
        (status = 200, description = "Setting updated", body = SettingResponse),
    ),
    security(("api_key" = []))
)]
pub async fn set_setting_handler(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Json(req): Json<SetSettingRequest>,
) -> Result<Json<SettingResponse>, AppError> {
    state.repo.set_setting(&key, &req.value)
        .await
        .map_err(|e| AppError::Anyhow(e))?;

    let setting = state.repo.get_setting(&key)
        .await
        .map_err(|e| AppError::Anyhow(e))?
        .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("Failed to retrieve setting after update")))?;

    Ok(Json(SettingResponse {
        key: setting.key,
        value: setting.value,
        updated_at: setting.updated_at,
    }))
}

/// Delete a setting (requires API key)
#[utoipa::path(
    delete,
    path = "/v1/settings/{key}",
    params(("key" = String, Path, description = "Setting key")),
    responses(
        (status = 200, description = "Setting deleted"),
        (status = 404, description = "Setting not found")
    ),
    security(("api_key" = []))
)]
pub async fn delete_setting_handler(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let deleted = state.repo.delete_setting(&key)
        .await
        .map_err(|e| AppError::Anyhow(e))?;

    if deleted {
        Ok(Json(serde_json::json!({ "deleted": true })))
    } else {
        Err(AppError::NotFound(format!("Setting '{}' not found", key)))
    }
}

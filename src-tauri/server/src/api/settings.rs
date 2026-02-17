use axum::{
    extract::{State, Path},
    http::header,
    response::IntoResponse,
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
) -> Result<impl IntoResponse, AppError> {
    let active_db = state.notebook_db.read().await;
    
    let settings = if let Some(pool) = active_db.as_ref() {
        sqlx::query_as::<_, crate::db::repo::Setting>("SELECT key, value, updated_at FROM settings ORDER BY key")
            .fetch_all(pool)
            .await
            .unwrap_or_else(|_| vec![])
    } else {
        state.repo.get_all_settings()
            .await
            .map_err(|e| AppError::Anyhow(e))?
    };

    let settings = settings.into_iter().map(|s| SettingResponse {
        key: s.key,
        value: s.value,
        updated_at: s.updated_at,
    }).collect();

    let mut response = Json(AllSettingsResponse { settings }).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("public, max-age=60"),
    );
    Ok(response)
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
) -> Result<impl IntoResponse, AppError> {
    let active_db = state.notebook_db.read().await;

    let setting = if let Some(pool) = active_db.as_ref() {
        sqlx::query_as::<_, crate::db::repo::Setting>("SELECT key, value, updated_at FROM settings WHERE key = ?")
            .bind(&key)
            .fetch_optional(pool)
            .await
            .unwrap_or(None)
    } else {
        state.repo.get_setting(&key)
            .await
            .map_err(|e| AppError::Anyhow(e))?
    };

    let setting = setting.ok_or_else(|| AppError::NotFound(format!("Setting '{}' not found", key)))?;

    let mut response = Json(SettingResponse {
        key: setting.key,
        value: setting.value,
        updated_at: setting.updated_at,
    })
    .into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("public, max-age=60"),
    );
    Ok(response)
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

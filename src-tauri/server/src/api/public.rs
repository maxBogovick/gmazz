use axum::{
    extract::{State, Path, Query},
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};
use crate::{AppState, error::AppError};

// Re-using models from main crate might be hard if not shared. 
// We define response structs here that match the DB structure.

#[derive(Serialize, ToSchema)]
pub struct PublicNote {
    pub id: String,
    pub note_type: String,
    pub content: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct NoteFilter {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List public notes
#[utoipa::path(
    get,
    path = "/v1/public/notes",
    params(NoteFilter),
    responses((status = 200, description = "List of public notes", body = [PublicNote])),
)]
pub async fn list_public_notes_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<NoteFilter>,
) -> Result<Json<Vec<PublicNote>>, AppError> {
    let pool_guard = state.notebook_db.read().await;
    let pool = pool_guard.as_ref()
        .ok_or_else(|| AppError::NotFound("Public database not loaded".to_string()))?;

    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let offset = params.offset.unwrap_or(0).max(0);

    let rows = sqlx::query_as::<_, (String, String, String, String, String, String)>(
        r#"
        SELECT id, note_type, content, metadata, created_at, updated_at
        FROM notes
        WHERE is_public = 1
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Anyhow(anyhow::anyhow!("DB Error: {}", e)))?;

    let notes = rows.into_iter().map(|row| {
        let metadata = serde_json::from_str(&row.3).unwrap_or_else(|_| Value::String(row.3));
        PublicNote {
        id: row.0,
        note_type: row.1,
        content: row.2,
        metadata,
        created_at: row.4,
        updated_at: row.5,
        }
    }).collect();

    Ok(Json(notes))
}

/// Get a single public note
#[utoipa::path(
    get,
    path = "/v1/public/notes/{id}",
    params(("id" = String, Path, description = "Note ID")),
    responses(
        (status = 200, description = "Public note", body = PublicNote),
        (status = 404, description = "Note not found")
    ),
)]
pub async fn get_public_note_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<PublicNote>, AppError> {
    let pool_guard = state.notebook_db.read().await;
    let pool = pool_guard.as_ref()
        .ok_or_else(|| AppError::NotFound("Public database not loaded".to_string()))?;

    let row = sqlx::query_as::<_, (String, String, String, String, String, String)>(
        r#"
        SELECT id, note_type, content, metadata, created_at, updated_at
        FROM notes
        WHERE id = ? AND is_public = 1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Anyhow(anyhow::anyhow!("DB Error: {}", e)))?;

    let row = row.ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    let metadata = serde_json::from_str(&row.3).unwrap_or_else(|_| Value::String(row.3));

    Ok(Json(PublicNote {
        id: row.0,
        note_type: row.1,
        content: row.2,
        metadata,
        created_at: row.4,
        updated_at: row.5,
    }))
}

use crate::models::{CreateNoteRequest, Note, NoteMetadata, NoteType, NotesFilter, Setting, UpdateNoteRequest};
use crate::sync::SyncClient;
use chrono::Utc;
use sqlx::{Pool, Sqlite};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

pub struct AppState {
    pub db: Pool<Sqlite>,
    pub sync_client: Option<SyncClient>,
    pub storage_dir: PathBuf,
}
#[tauri::command]
pub async fn get_all_settings(state: State<'_, AppState>) -> Result<Vec<Setting>, String> {
    sqlx::query_as::<_, Setting>("SELECT key, value, updated_at FROM settings ORDER BY key")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let result: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.map(|(v,)| v))
}

#[tauri::command]
pub async fn set_setting(state: State<'_, AppState>, key: String, value: String) -> Result<Setting, String> {
    let updated_at = Utc::now().timestamp();

    // SQLite upsert
    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at"
    )
        .bind(&key)
        .bind(&value)
        .bind(updated_at)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Setting { key, value, updated_at })
}

#[tauri::command]
pub async fn delete_setting(state: State<'_, AppState>, key: String) -> Result<bool, String> {
    let result = sqlx::query("DELETE FROM settings WHERE key = ?")
        .bind(key)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.rows_affected() > 0)
}

#[tauri::command]
pub async fn create_note(
    state: State<'_, AppState>,
    request: CreateNoteRequest,
) -> Result<Note, String> {
    let now = Utc::now();
    let note = Note {
        id: Uuid::new_v4(),
        note_type: request.note_type,
        content: request.content,
        metadata: request.metadata,
        created_at: now,
        updated_at: now,
        is_public: request.is_public,
    };

    let metadata_json = serde_json::to_string(&note.metadata).map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        INSERT INTO notes (id, note_type, content, metadata, created_at, updated_at, is_public)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
        .bind(note.id.to_string())
        .bind(note.note_type.as_str())
        .bind(&note.content)
        .bind(&metadata_json)
        .bind(note.created_at.to_rfc3339())
        .bind(note.updated_at.to_rfc3339())
        .bind(note.is_public)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(note)
}

#[tauri::command]
pub async fn update_note(
    state: State<'_, AppState>,
    id: String,
    request: UpdateNoteRequest,
) -> Result<Note, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    let row = sqlx::query_as::<_, (String, String, String, String, String, bool)>(
        "SELECT note_type, content, metadata, created_at, updated_at, is_public FROM notes WHERE id = ?",
    )
        .bind(uuid.to_string())
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Note not found".to_string())?;

    let note_type = NoteType::from_str(&row.0).ok_or_else(|| "Invalid note type".to_string())?;
    let content = request.content.unwrap_or(row.1);
    let metadata: NoteMetadata = if let Some(m) = request.metadata {
        m
    } else {
        serde_json::from_str(&row.2).map_err(|e| e.to_string())?
    };
    let created_at = chrono::DateTime::parse_from_rfc3339(&row.3)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);
    let is_public = request.is_public.unwrap_or(row.5);
    let updated_at = Utc::now();

    let metadata_json = serde_json::to_string(&metadata).map_err(|e| e.to_string())?;

    sqlx::query(
        "UPDATE notes SET content = ?, metadata = ?, updated_at = ?, is_public = ? WHERE id = ?",
    )
        .bind(&content)
        .bind(&metadata_json)
        .bind(updated_at.to_rfc3339())
        .bind(is_public)
        .bind(uuid.to_string())
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Note {
        id: uuid,
        note_type,
        content,
        metadata,
        created_at,
        updated_at,
        is_public,
    })
}

#[tauri::command]
pub async fn get_notes(
    state: State<'_, AppState>,
    filter: Option<NotesFilter>,
) -> Result<Vec<Note>, String> {
    let filter = filter.unwrap_or(NotesFilter {
        note_type: None,
        year: None,
        limit: Some(50),
        offset: Some(0),
    });

    let mut query = String::from(
        "SELECT id, note_type, content, metadata, created_at, updated_at, is_public FROM notes WHERE 1=1",
    );

    if let Some(ref note_type) = filter.note_type {
        query.push_str(&format!(" AND note_type = '{}'", note_type.as_str()));
    }

    if let Some(year) = filter.year {
        query.push_str(&format!(
            " AND strftime('%Y', created_at) = '{}'",
            year
        ));
    }

    query.push_str(" ORDER BY created_at DESC");

    if let Some(limit) = filter.limit {
        query.push_str(&format!(" LIMIT {}", limit));
    }

    if let Some(offset) = filter.offset {
        query.push_str(&format!(" OFFSET {}", offset));
    }

    let rows = sqlx::query_as::<_, (String, String, String, String, String, String, bool)>(&query)
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let notes: Result<Vec<Note>, String> = rows
        .into_iter()
        .map(|row| {
            let id = Uuid::parse_str(&row.0).map_err(|e| e.to_string())?;
            let note_type =
                NoteType::from_str(&row.1).ok_or_else(|| "Invalid note type".to_string())?;
            let metadata: NoteMetadata =
                serde_json::from_str(&row.3).map_err(|e| e.to_string())?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&row.4)
                .map_err(|e| e.to_string())?
                .with_timezone(&Utc);
            let updated_at = chrono::DateTime::parse_from_rfc3339(&row.5)
                .map_err(|e| e.to_string())?
                .with_timezone(&Utc);

            Ok(Note {
                id,
                note_type,
                content: row.2,
                metadata,
                created_at,
                updated_at,
                is_public: row.6,
            })
        })
        .collect();

    notes
}

#[tauri::command]
pub async fn get_note(state: State<'_, AppState>, id: String) -> Result<Note, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    let row = sqlx::query_as::<_, (String, String, String, String, String, String, bool)>(
        "SELECT id, note_type, content, metadata, created_at, updated_at, is_public FROM notes WHERE id = ?",
    )
        .bind(uuid.to_string())
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Note not found".to_string())?;

    let note_type = NoteType::from_str(&row.1).ok_or_else(|| "Invalid note type".to_string())?;
    let metadata: NoteMetadata = serde_json::from_str(&row.3).map_err(|e| e.to_string())?;
    let created_at = chrono::DateTime::parse_from_rfc3339(&row.4)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&row.5)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);

    Ok(Note {
        id: uuid,
        note_type,
        content: row.2,
        metadata,
        created_at,
        updated_at,
        is_public: row.6,
    })
}

#[tauri::command]
pub async fn get_random_note(state: State<'_, AppState>) -> Result<Note, String> {
    let row = sqlx::query_as::<_, (String, String, String, String, String, String, bool)>(
        "SELECT id, note_type, content, metadata, created_at, updated_at, is_public FROM notes ORDER BY RANDOM() LIMIT 1",
    )
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No notes found".to_string())?;

    let id = Uuid::parse_str(&row.0).map_err(|e| e.to_string())?;
    let note_type = NoteType::from_str(&row.1).ok_or_else(|| "Invalid note type".to_string())?;
    let metadata: NoteMetadata = serde_json::from_str(&row.3).map_err(|e| e.to_string())?;
    let created_at = chrono::DateTime::parse_from_rfc3339(&row.4)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&row.5)
        .map_err(|e| e.to_string())?
        .with_timezone(&Utc);

    Ok(Note {
        id,
        note_type,
        content: row.2,
        metadata,
        created_at,
        updated_at,
        is_public: row.6,
    })
}

#[tauri::command]
pub async fn delete_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(uuid.to_string())
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn upload_file(
    state: State<'_, AppState>,
    file_name: String,
    file_data: Vec<u8>,
    file_type: String,
) -> Result<String, String> {
    let id = Uuid::new_v4();
    let file_path = id.to_string(); // Storing as flat file in storage_dir
    let full_path = state.storage_dir.join(&file_path);

    // 1. Write to filesystem
    tokio::fs::write(&full_path, &file_data)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    // 2. Record in DB
    let now = Utc::now();
    let size_bytes = file_data.len() as i64;

    sqlx::query(
        r#"
        INSERT INTO files (id, file_path, original_name, mime_type, size_bytes, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
        .bind(id.to_string())
        .bind(&file_path)
        .bind(&file_name)
        .bind(&file_type)
        .bind(size_bytes)
        .bind(now.to_rfc3339())
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to record file in DB: {}", e))?;

    Ok(id.to_string())
}

#[derive(serde::Serialize)]
pub struct AssetResponse {
    pub path: String,
    pub mime_type: Option<String>,
}

#[tauri::command]
pub async fn get_asset_path(state: State<'_, AppState>, relative_path: String) -> Result<AssetResponse, String> {
    // Legacy local path - might still be needed if some assets are local
    // but ideally we should distinguish.
    // For now, if it starts with http, return as is.
    if relative_path.starts_with("http") {
        return Ok(AssetResponse { path: relative_path, mime_type: None });
    }

    let full_path = state.storage_dir.join(&relative_path);

    // Try to get mime_type from DB assuming relative_path is the ID
    let mime_type: Option<String> = sqlx::query_scalar("SELECT mime_type FROM files WHERE id = ?")
        .bind(&relative_path)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    Ok(AssetResponse {
        path: full_path.to_string_lossy().to_string(),
        mime_type,
    })
}

async fn prepare_release_db(
    state: &AppState,
    source_db_path: &PathBuf,
    release_db_path: &PathBuf,
) -> Result<(), String> {
    tokio::fs::copy(source_db_path, release_db_path)
        .await
        .map_err(|e| format!("Failed to copy DB: {}", e))?;

    let release_pool = Pool::<Sqlite>::connect(&format!("sqlite:{}", release_db_path.display()))
        .await
        .map_err(|e| format!("Failed to open release DB: {}", e))?;

    let file_ids: Vec<(String,)> = sqlx::query_as("SELECT id FROM files")
        .fetch_all(&release_pool)
        .await
        .map_err(|e| format!("Failed to list files: {}", e))?;

    for (id,) in file_ids {
        let file_path = state.storage_dir.join(&id);
        if file_path.exists() {
            match tokio::fs::read(&file_path).await {
                Ok(content) => {
                    sqlx::query("UPDATE files SET content = ? WHERE id = ?")
                        .bind(content)
                        .bind(&id)
                        .execute(&release_pool)
                        .await
                        .map_err(|e| format!("Failed to embed file {}: {}", id, e))?;
                }
                Err(e) => println!("Warning: File {} not found on disk: {}", id, e),
            }
        }
    }

    release_pool.close().await;
    Ok(())
}

#[tauri::command]
pub async fn sync_local_db_to_server(app: AppHandle, state: State<'_, AppState>, api_key: Option<String>) -> Result<String, String> {
    if let Some(client_ref) = &state.sync_client {
        // Use provided key or fallback to state key
        let client = if let Some(key) = api_key {
            if !key.is_empty() {
                client_ref.with_api_key(key)
            } else {
                client_ref.with_api_key(client_ref.api_key.clone())
            }
        } else {
            client_ref.with_api_key(client_ref.api_key.clone())
        };

        // Force Checkpoint on main DB
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&state.db)
            .await
            .map_err(|e| format!("Checkpoint failed: {}", e))?;

        let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        let db_path = app_data_dir.join("notebook.db");

        let temp_dir = app.path().temp_dir().map_err(|e| e.to_string())?;
        if !temp_dir.exists() {
            std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
        }
        let release_db_path = temp_dir.join(format!(
            "notebook_release_{}.db",
            Utc::now().timestamp_millis()
        ));

        prepare_release_db(&state, &db_path, &release_db_path).await?;

        // 4. Upload
        let res = match client.upload_db_release(&release_db_path).await {
            Ok(id) => Ok(id),
            Err(e) => Err(format!("Sync failed: {}", e))
        };

        // 5. Cleanup
        let _ = tokio::fs::remove_file(&release_db_path).await;

        // Also remove WAL/SHM if they exist
        let _ = tokio::fs::remove_file(release_db_path.with_extension("db-wal")).await;
        let _ = tokio::fs::remove_file(release_db_path.with_extension("db-shm")).await;

        res
    } else {
        Err("Sync not configured".to_string())
    }
}

#[tauri::command]
pub async fn export_local_db_to_downloads(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&state.db)
        .await
        .map_err(|e| format!("Checkpoint failed: {}", e))?;

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_path = app_data_dir.join("notebook.db");
    if !db_path.exists() {
        return Err("Local database not found".to_string());
    }

    let downloads_dir = app.path().download_dir().map_err(|e| e.to_string())?;
    let release_dir = downloads_dir.join("release");
    if !release_dir.exists() {
        std::fs::create_dir_all(&release_dir).map_err(|e| e.to_string())?;
    }

    let file_name = format!("notebook_release_{}.db", Utc::now().format("%Y%m%d_%H%M%S"));
    let export_path = release_dir.join(file_name);

    prepare_release_db(&state, &db_path, &export_path).await?;

    Ok(export_path.to_string_lossy().to_string())
}

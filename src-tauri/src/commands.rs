use crate::models::{CreateNoteRequest, Note, NoteMetadata, NoteType, NotesFilter, UpdateNoteRequest};
use chrono::Utc;
use sqlx::{Pool, Sqlite};
use tauri::{Manager, State};
use uuid::Uuid;

pub struct AppState {
    pub db: Pool<Sqlite>,
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
    app: tauri::AppHandle,
    file_name: String,
    file_data: Vec<u8>,
    file_type: String,
) -> Result<String, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| e.to_string())?;

    let assets_dir = app_data_dir.join("notebook_assets");
    let type_dir = assets_dir.join(&file_type);

    std::fs::create_dir_all(&type_dir).map_err(|e| e.to_string())?;

    let unique_name = format!("{}_{}", Uuid::new_v4(), file_name);
    let file_path = type_dir.join(&unique_name);

    std::fs::write(&file_path, file_data).map_err(|e| e.to_string())?;

    let relative_path = format!("notebook_assets/{}/{}", file_type, unique_name);
    Ok(relative_path)
}

#[tauri::command]
pub fn get_asset_path(app: tauri::AppHandle, relative_path: String) -> Result<String, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| e.to_string())?;

    let full_path = app_data_dir.join(&relative_path);
    Ok(full_path.to_string_lossy().to_string())
}

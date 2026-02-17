use std::sync::Arc;
use uuid::Uuid;
use sqlx::sqlite::SqlitePoolOptions;
use crate::db::repo::{Repo, Release, ReleaseWithFile, FileMetadata};
use crate::error::AppError;
use crate::AppState;

pub struct ReleaseService {
    repo: Repo,
}

impl ReleaseService {
    pub fn new(repo: Repo) -> Self {
        Self { repo }
    }

    pub async fn create_release(
        &self,
        app_id: &str,
        file_id: &str,
        version_name: Option<String>,
        description: Option<String>,
    ) -> Result<ReleaseWithFile, AppError> {
        // Verify file ownership/existence
        let file = self.repo.get_file(file_id, app_id).await?
            .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

        let release = Release {
            id: Uuid::new_v4().to_string(),
            app_id: app_id.to_string(),
            file_id: file.id,
            version_name: version_name.or_else(|| Some("Auto-Release".to_string())),
            description,
            is_active: true, // Initially true
            created_at: chrono::Utc::now().timestamp(),
        };

        // Create the record
        self.repo.create_release(&release).await?;
        
        // Ensure it is the ONLY active one
        self.repo.set_release_active(&release.id).await?;
        
        Ok(ReleaseWithFile {
            id: release.id,
            app_id: release.app_id,
            file_id: release.file_id,
            version_name: release.version_name,
            description: release.description,
            is_active: release.is_active,
            created_at: release.created_at,
            file_original_name: file.original_name,
            file_size_bytes: file.size_bytes,
        })
    }

    pub async fn activate_release(&self, release_id: &str) -> Result<(), AppError> {
        self.repo.set_release_active(release_id).await?;
        Ok(())
    }

    pub async fn get_latest_release(&self, app_id: &str) -> Result<ReleaseWithFile, AppError> {
        // For the client sync, we probably want the ACTIVE release, not just the latest created?
        // Or keep "latest created" for history but "active" for sync?
        // The prompt implies "Server starts with active", "Admin changes release -> Server reacts".
        // This implies clients should also sync to ACTIVE.
        
        if let Some(active) = self.repo.get_active_release().await? {
            if active.app_id == app_id {
                return Ok(active);
            }
        }
        
        // Fallback to latest created if no active? Or error?
        // Let's fallback to latest created for safety if none marked active.
        self.repo.get_latest_release(app_id).await?
            .ok_or_else(|| AppError::NotFound("No releases found".to_string()))
    }

    pub async fn list_releases(&self, app_id: &str, limit: i64, offset: i64) -> Result<Vec<ReleaseWithFile>, AppError> {
        Ok(self.repo.list_releases(app_id, limit, offset).await?)
    }

    pub async fn reload_active_db(state: &AppState) -> Result<(), anyhow::Error> {
        // Prefer explicit active release
        let release = if let Some(active) = state.repo.get_active_release().await? {
            active
        } else if let Some(latest) = state.repo.get_latest_global_release().await? {
            tracing::warn!("No active release found, falling back to latest global.");
            latest
        } else {
            tracing::warn!("No releases found to serve.");
            return Ok(());
        };

        // Get file path
        let meta = state.repo.get_file(&release.file_id, &release.app_id).await?
            .ok_or_else(|| anyhow::anyhow!("File metadata missing"))?;
        
        let abs_path = state.storage.get_absolute_path(&meta.stored_path);
        
        if !abs_path.exists() {
            return Err(anyhow::anyhow!("Database file missing on disk: {:?}", abs_path));
        }

        // Copy to temp folder to avoid locking the original file (if needed) 
        // or just read directly if read-only mode is fine.
        // Prompt says: "unzip it in temp folder and work with it". 
        // Since it's a single .db file (not a zip yet, though we support archives), we just copy it.
        // NOTE: If we want to support WRites from server, we MUST copy.
        // But we are Read-Only for now.
        // "разворачивать его у себя в темп папке" implies copy/extraction.
        
        let tmp_dir = std::env::temp_dir().join("gmazz_server_active_db");
        tokio::fs::create_dir_all(&tmp_dir).await?;
        cleanup_old_active_dbs(&tmp_dir).await;
        let active_db_path = tmp_dir.join(format!("active_{}.db", release.id));
        
        tokio::fs::copy(&abs_path, &active_db_path).await?;
        
        tracing::info!("Deployed Active DB to: {:?}", active_db_path);

        // Connect
        let db_url = format!("sqlite:{}?mode=ro", active_db_path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        // --- Dehydration: Extract embedded files ---
        // We do this optimistically. If it fails, we log and continue (the DB is still valid for notes).
        match sqlx::query_as::<_, (String, String, Option<String>, i64, Option<Vec<u8>>)>(
            "SELECT id, original_name, mime_type, size_bytes, content FROM files WHERE content IS NOT NULL"
        )
        .fetch_all(&pool)
        .await 
        {
            Ok(files) => {
                for (id, name, mime, size, content) in files {
                    if let Some(data) = content {
                        // Check if file is already registered in System DB to avoid overwriting/work
                        match state.repo.get_file(&id, &release.app_id).await {
                            Ok(Some(_)) => continue, // Already exists
                            Ok(None) => {
                                // Extract
                                match state.storage.prepare_file_path(&release.app_id, &id).await {
                                    Ok((rel_path, abs_path)) => {
                                        if let Err(e) = tokio::fs::write(&abs_path, &data).await {
                                            tracing::error!("Failed to write extracted file {}: {}", id, e);
                                            continue;
                                        }

                                        let metadata = FileMetadata {
                                            id: id.clone(),
                                            app_id: release.app_id.clone(),
                                            original_name: name,
                                            stored_path: rel_path,
                                            mime_type: mime,
                                            size_bytes: size,
                                            checksum: None, 
                                            created_at: chrono::Utc::now().timestamp(),
                                            deleted_at: None,
                                        };

                                        if let Err(e) = state.repo.create_file(&metadata).await {
                                            tracing::error!("Failed to register extracted file {}: {}", id, e);
                                        } else {
                                            tracing::info!("Dehydrated file from release: {}", id);
                                        }
                                    },
                                    Err(e) => tracing::error!("Failed to prepare path for {}: {}", id, e),
                                }
                            },
                            Err(e) => tracing::error!("DB error checking file {}: {}", id, e),
                        }
                    }
                }
            },
            Err(e) => {
                // It's possible the uploaded DB doesn't have 'files' table or 'content' column if it's an old version?
                // But we just created it.
                tracing::warn!("Failed to scan release DB for files (Dehydration skipped): {}", e);
            }
        }

        // Swap
        let mut lock = state.notebook_db.write().await;
        *lock = Some(pool);
        
        tracing::info!("Switched Public DB to Release: {} (File: {})", release.id, release.file_id);

        Ok(())
    }
}

async fn cleanup_old_active_dbs(tmp_dir: &std::path::Path) {
    if let Ok(mut entries) = tokio::fs::read_dir(tmp_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.starts_with("active_") && name.ends_with(".db") {
                    let _ = tokio::fs::remove_file(&path).await;
                }
            }
        }
    }
}

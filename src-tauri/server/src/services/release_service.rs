use std::sync::Arc;
use uuid::Uuid;
use sqlx::sqlite::SqlitePoolOptions;
use crate::db::repo::{Repo, Release, ReleaseWithFile};
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
        let active_db_path = tmp_dir.join(format!("active_{}.db", release.id));
        
        tokio::fs::copy(&abs_path, &active_db_path).await?;
        
        tracing::info!("Deployed Active DB to: {:?}", active_db_path);

        // Connect
        let db_url = format!("sqlite:{}?mode=ro", active_db_path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        // Swap
        let mut lock = state.notebook_db.write().await;
        *lock = Some(pool);
        
        tracing::info!("Switched Public DB to Release: {} (File: {})", release.id, release.file_id);

        Ok(())
    }
}

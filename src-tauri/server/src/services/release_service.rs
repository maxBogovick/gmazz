use std::sync::Arc;
use uuid::Uuid;
use crate::db::repo::{Repo, Release, ReleaseWithFile};
use crate::error::AppError;

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
            is_active: true, // Default to true for now as we use "latest" logic
            created_at: chrono::Utc::now().timestamp(),
        };

        self.repo.create_release(&release).await?;
        
        // Return full object with file info
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

    pub async fn get_latest_release(&self, app_id: &str) -> Result<ReleaseWithFile, AppError> {
        self.repo.get_latest_release(app_id).await?
            .ok_or_else(|| AppError::NotFound("No releases found".to_string()))
    }

    pub async fn list_releases(&self, app_id: &str, limit: i64, offset: i64) -> Result<Vec<ReleaseWithFile>, AppError> {
        Ok(self.repo.list_releases(app_id, limit, offset).await?)
    }
}

use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use futures::StreamExt;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use axum::body::Body;

use crate::db::repo::{Repo, FileMetadata};
use crate::storage::StorageManager;
use crate::error::AppError;
use crate::api::files::FileResponse;
use crate::config::Config;

pub struct FileService {
    repo: Repo,
    storage: StorageManager,
    config: Config,
}

impl FileService {
    pub fn new(repo: Repo, storage: StorageManager, config: Config) -> Self {
        Self { repo, storage, config }
    }

    pub async fn upload_file(
        &self, 
        app_id: &str, 
        original_name: String, 
        content_type: Option<String>, 
        body: Body
    ) -> Result<FileResponse, AppError> {
        let file_uuid = Uuid::new_v4().to_string();
        let tmp_path = self.storage.get_temp_path(&file_uuid);

        let file = File::create(&tmp_path).await?;
        let mut writer = BufWriter::new(file);
        let mut hasher = Sha256::new();
        let mut size_bytes: i64 = 0;
        
        let mut stream = body.into_data_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| AppError::BadRequest(format!("Stream error: {}", e)))?;
            
            writer.write_all(&chunk).await?;
            hasher.update(&chunk);
            size_bytes += chunk.len() as i64;

            if size_bytes > self.config.server.request_body_limit_bytes as i64 {
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Err(AppError::PayloadTooLarge);
            }
        }

        writer.flush().await?;
        let checksum = hex::encode(hasher.finalize());

        let (rel_path, abs_path) = self.storage.prepare_file_path(app_id, &file_uuid).await
            .map_err(|e| AppError::Anyhow(e))?;

        self.storage.move_file(&tmp_path, &abs_path).await
            .map_err(|e| AppError::Anyhow(e))?;

        let metadata = FileMetadata {
            id: file_uuid.clone(),
            app_id: app_id.to_string(),
            original_name: original_name.clone(),
            stored_path: rel_path,
            mime_type: content_type.clone(),
            size_bytes,
            checksum: Some(checksum.clone()),
            created_at: chrono::Utc::now().timestamp(),
            deleted_at: None,
        };

        self.repo.create_file(&metadata).await?;

        Ok(FileResponse {
            id: file_uuid,
            original_name,
            size_bytes,
            mime_type: content_type,
            checksum: Some(checksum),
            created_at: metadata.created_at,
        })
    }

    pub async fn get_file_metadata(&self, file_id: &str, app_id: &str) -> Result<FileMetadata, AppError> {
        self.repo.get_file(file_id, app_id).await?
            .ok_or_else(|| AppError::NotFound("File not found".to_string()))
    }

    pub async fn list_files(&self, app_id: &str, limit: i64, offset: i64) -> Result<Vec<FileMetadata>, AppError> {
        Ok(self.repo.list_files(app_id, limit, offset).await?)
    }

    pub async fn soft_delete_file(&self, file_id: &str, app_id: &str) -> Result<(), AppError> {
        let deleted = self.repo.soft_delete_file(file_id, app_id).await?;
        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound("File not found".to_string()))
        }
    }
    
    pub fn get_absolute_path(&self, relative_path: &str) -> std::path::PathBuf {
        self.storage.get_absolute_path(relative_path)
    }
}

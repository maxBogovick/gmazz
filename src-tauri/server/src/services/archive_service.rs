use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use tokio::fs::File;
use async_zip::tokio::write::ZipFileWriter;
use async_zip::Compression;

use crate::db::repo::{Repo, Archive};
use crate::storage::StorageManager;
use crate::error::AppError;
use crate::api::archives::ArchiveResponse;

pub struct ArchiveService {
    repo: Repo,
    storage: StorageManager,
}

impl ArchiveService {
    pub fn new(repo: Repo, storage: StorageManager) -> Self {
        Self { repo, storage }
    }

    pub async fn start_archiving(&self, app_id: &str) -> Result<ArchiveResponse, AppError> {
        let archive_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();
        let expires_at = now + 24 * 3600; 

        let archive = Archive {
            id: archive_id.clone(),
            app_id: app_id.to_string(),
            status: "pending".to_string(),
            file_path: None,
            created_at: now,
            expires_at: Some(expires_at),
        };

        self.repo.create_archive_record(&archive).await.map_err(|e| AppError::Anyhow(e))?;

        // Background task
        let repo = self.repo.clone();
        let storage = self.storage.clone();
        let app_id_owned = app_id.to_string();
        let archive_id_owned = archive_id.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::generate_archive_task(repo.clone(), storage, &app_id_owned, &archive_id_owned).await {
                tracing::error!("Archive generation failed for {}: {}", archive_id_owned, e);
                let _ = repo.update_archive_status(&archive_id_owned, "failed", None).await;
            }
        });

        Ok(ArchiveResponse {
            archive_id,
            status: "pending".to_string(),
            expires_at,
        })
    }

    async fn generate_archive_task(repo: Repo, storage: StorageManager, app_id: &str, archive_id: &str) -> anyhow::Result<()> {
        let (rel_path, abs_path) = storage.prepare_archive_path(app_id, archive_id).await?;
        let file = File::create(&abs_path).await?;
        let mut zip = ZipFileWriter::with_tokio(file);

        let files = repo.get_all_files_for_app(app_id).await?;

        for file_meta in files {
            let file_path = storage.get_absolute_path(&file_meta.stored_path);
            if let Ok(source_file) = File::open(&file_path).await {
                 let entry_name = file_meta.stored_path.replace('\\', "/");
                 let entry_options = async_zip::ZipEntryBuilder::new(entry_name.into(), Compression::Deflate);
                 let mut entry_writer = zip.write_entry_stream(entry_options).await?;
                 let mut source_compat = tokio_util::compat::TokioAsyncReadCompatExt::compat(source_file);
                 futures::io::copy(&mut source_compat, &mut entry_writer).await?;
                 entry_writer.close().await?;
            }
        }

        zip.close().await?;
        repo.update_archive_status(archive_id, "ready", Some(rel_path)).await?;
        Ok(())
    }

    pub async fn get_archive(&self, archive_id: &str, app_id: &str) -> Result<Archive, AppError> {
        self.repo.get_archive(archive_id, app_id).await.map_err(|e| AppError::Anyhow(e))?
            .ok_or_else(|| AppError::NotFound("Archive not found".to_string()))
    }
}

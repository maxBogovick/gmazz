use std::path::{Path, PathBuf};
use chrono::Utc;
use tokio::fs;
use anyhow::{Result, Context};

#[derive(Clone)]
pub struct StorageManager {
    base_dir: PathBuf,
}

impl StorageManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn get_temp_path(&self, id: &str) -> PathBuf {
        self.base_dir.join("tmp").join(format!("{}.upload", id))
    }

    /// Generates the destination path and ensures directories exist.
    /// Returns (relative_path_string, absolute_path_pathbuf)
    pub async fn prepare_file_path(&self, app_id: &str, file_uuid: &str) -> Result<(String, PathBuf)> {
        let now = Utc::now();
        let yyyy = now.format("%Y").to_string();
        let mm = now.format("%m").to_string();
        let dd = now.format("%d").to_string();

        let rel_dir = Path::new("files")
            .join(app_id)
            .join(&yyyy)
            .join(&mm)
            .join(&dd);
        
        let abs_dir = self.base_dir.join(&rel_dir);

        if !abs_dir.exists() {
            fs::create_dir_all(&abs_dir).await
                .context("Failed to create file directory")?;
        }

        let filename = format!("{}.bin", file_uuid);
        let rel_path = rel_dir.join(&filename);
        let abs_path = abs_dir.join(&filename);

        Ok((rel_path.to_string_lossy().to_string(), abs_path))
    }

    pub fn get_absolute_path(&self, relative_path: &str) -> PathBuf {
        self.base_dir.join(relative_path)
    }

    pub async fn ensure_structure(&self) -> Result<()> {
        let dirs = ["files", "archives", "tmp"];
        for d in dirs {
            let p = self.base_dir.join(d);
            if !p.exists() {
                fs::create_dir_all(p).await?;
            }
        }
        Ok(())
    }
    
    pub async fn move_file(&self, from: &Path, to: &Path) -> Result<()> {
        fs::rename(from, to).await.context("Failed to move file")
    }

    pub async fn prepare_archive_path(&self, app_id: &str, archive_id: &str) -> Result<(String, PathBuf)> {
        let rel_dir = Path::new("archives").join(app_id);
        let abs_dir = self.base_dir.join(&rel_dir);

        if !abs_dir.exists() {
            fs::create_dir_all(&abs_dir).await
                .context("Failed to create archive directory")?;
        }

        let filename = format!("{}.zip", archive_id);
        let rel_path = rel_dir.join(&filename);
        let abs_path = abs_dir.join(&filename);

        Ok((rel_path.to_string_lossy().to_string(), abs_path))
    }
}

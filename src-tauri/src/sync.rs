use std::path::PathBuf;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tokio::io::AsyncWriteExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileResponse {
    pub id: String,
    pub original_name: String,
    pub created_at: i64,
}

pub struct SyncClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReleaseResponse {
    pub id: String,
    pub file_id: String,
    pub version_name: Option<String>,
    pub created_at: i64,
}

#[derive(Serialize)]
pub struct CreateReleaseRequest {
    pub file_id: String,
    pub version_name: Option<String>,
    pub description: Option<String>,
}

impl SyncClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        }
    }

    pub async fn download_latest_db(&self, db_path: &PathBuf) -> Result<bool> {
        // 1. Get Latest Release Info
        let release_url = format!("{}/v1/releases/latest", self.base_url);
        let resp = self.client.get(&release_url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            println!("No active releases found on server.");
            return Ok(false);
        }

        let release: ReleaseResponse = resp.error_for_status()?.json().await?;

        // 2. Download the file associated with the release
        let download_url = format!("{}/v1/files/{}", self.base_url, release.file_id);
        let mut stream_resp = self.client.get(&download_url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .error_for_status()?;

        // Write to temp file
        let tmp_path = db_path.with_extension("db.tmp");
        let mut file = tokio::fs::File::create(&tmp_path).await?;
        
        while let Some(chunk) = stream_resp.chunk().await? {
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        drop(file);

        // Replace current DB
        tokio::fs::rename(&tmp_path, db_path).await?;
        
        println!("Downloaded latest released DB (Release: {}, File: {})", release.id, release.file_id);
        return Ok(true);
    }

    pub async fn upload_current_db(&self, db_path: &PathBuf) -> Result<String> {
        if !db_path.exists() {
            return Err(anyhow::anyhow!("Database file not found"));
        }

        // 1. Upload File
        let file_content = tokio::fs::read(db_path).await?;
        let part = reqwest::multipart::Part::bytes(file_content)
            .file_name("notebook.db")
            .mime_str("application/x-sqlite3")?;

        let form = reqwest::multipart::Form::new().part("file", part);

        let upload_url = format!("{}/v1/files", self.base_url);
        let resp = self.client.post(&upload_url)
            .header("X-API-KEY", &self.api_key)
            .header("X-File-Name", "notebook.db")
            .multipart(form)
            .send()
            .await?
            .error_for_status()?;

        let file_res: FileResponse = resp.json().await?;
        println!("Uploaded notebook.db (File ID: {})", file_res.id);

        // 2. Create Release (Auto-Release)
        // In a strict environment, maybe we don't do this automatically?
        // But for "sync", yes we do.
        let release_req = CreateReleaseRequest {
            file_id: file_res.id.clone(),
            version_name: Some("Auto-Sync".to_string()),
            description: Some("Uploaded from desktop client".to_string()),
        };

        let release_url = format!("{}/v1/releases", self.base_url);
        let rel_resp = self.client.post(&release_url)
            .header("X-API-KEY", &self.api_key)
            .json(&release_req)
            .send()
            .await?
            .error_for_status()?;

        let release_res: ReleaseResponse = rel_resp.json().await?;
        println!("Created Release (ID: {})", release_res.id);
        
        Ok(release_res.id)
    }
}

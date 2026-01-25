use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use anyhow::Result;
use chrono::Utc;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct App {
    pub id: String,
    pub api_key_hash: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct FileMetadata {
    pub id: String,
    pub app_id: String,
    pub original_name: String,
    pub stored_path: String,
    pub mime_type: Option<String>,
    pub size_bytes: i64,
    pub checksum: Option<String>,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Archive {
    pub id: String,
    pub app_id: String,
    pub status: String,
    pub file_path: Option<String>,
    pub created_at: i64,
    pub expires_at: Option<i64>,
}

#[derive(Clone)]
pub struct Repo {
    pub pool: Pool<Sqlite>,
}

impl Repo {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    // --- Apps ---

    pub async fn create_app(&self, name: &str, api_key_hash: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        sqlx::query(
            "INSERT INTO apps (id, api_key_hash, name, is_active, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(api_key_hash)
        .bind(name)
        .bind(true)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn get_app_by_key_hash(&self, hash: &str) -> Result<Option<App>> {
        let app = sqlx::query_as::<_, App>(
            "SELECT id, api_key_hash, name, is_active, created_at, last_used_at FROM apps WHERE api_key_hash = ?"
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(app)
    }

    pub async fn update_app_last_used(&self, app_id: &str) -> Result<()> {
        let now = Utc::now().timestamp();
        sqlx::query(
            "UPDATE apps SET last_used_at = ? WHERE id = ?"
        )
        .bind(now)
        .bind(app_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_apps(&self) -> Result<Vec<App>> {
        let apps = sqlx::query_as::<_, App>(
            "SELECT id, api_key_hash, name, is_active, created_at, last_used_at FROM apps ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(apps)
    }

    pub async fn update_app_status(&self, app_id: &str, is_active: bool) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE apps SET is_active = ? WHERE id = ?"
        )
        .bind(is_active)
        .bind(app_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    // --- Files ---

    pub async fn create_file(&self, file: &FileMetadata) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO files (id, app_id, original_name, stored_path, mime_type, size_bytes, checksum, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&file.id)
        .bind(&file.app_id)
        .bind(&file.original_name)
        .bind(&file.stored_path)
        .bind(&file.mime_type)
        .bind(file.size_bytes)
        .bind(&file.checksum)
        .bind(file.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_file(&self, file_id: &str, app_id: &str) -> Result<Option<FileMetadata>> {
        let file = sqlx::query_as::<_, FileMetadata>(
            r#"
            SELECT id, app_id, original_name, stored_path, mime_type, size_bytes, checksum, created_at, deleted_at
            FROM files
            WHERE id = ? AND app_id = ? AND deleted_at IS NULL
            "#
        )
        .bind(file_id)
        .bind(app_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(file)
    }

    pub async fn get_file_any_owner(&self, file_id: &str) -> Result<Option<FileMetadata>> {
        let file = sqlx::query_as::<_, FileMetadata>(
            r#"
            SELECT id, app_id, original_name, stored_path, mime_type, size_bytes, checksum, created_at, deleted_at
            FROM files
            WHERE id = ? AND deleted_at IS NULL
            "#
        )
        .bind(file_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(file)
    }

    pub async fn list_files(&self, app_id: &str, limit: i64, offset: i64) -> Result<Vec<FileMetadata>> {
        let files = sqlx::query_as::<_, FileMetadata>(
            r#"
            SELECT id, app_id, original_name, stored_path, mime_type, size_bytes, checksum, created_at, deleted_at
            FROM files
            WHERE app_id = ? AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(app_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(files)
    }

    pub async fn get_all_files_for_app(&self, app_id: &str) -> Result<Vec<FileMetadata>> {
        let files = sqlx::query_as::<_, FileMetadata>(
            r#"
            SELECT id, app_id, original_name, stored_path, mime_type, size_bytes, checksum, created_at, deleted_at
            FROM files
            WHERE app_id = ? AND deleted_at IS NULL
            "#
        )
        .bind(app_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(files)
    }

    pub async fn soft_delete_file(&self, file_id: &str, app_id: &str) -> Result<bool> {
        let now = Utc::now().timestamp();
        let result = sqlx::query(
            "UPDATE files SET deleted_at = ? WHERE id = ? AND app_id = ? AND deleted_at IS NULL"
        )
        .bind(now)
        .bind(file_id)
        .bind(app_id)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }

    // --- Archives ---

    pub async fn create_archive_record(&self, archive: &Archive) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO archives (id, app_id, status, file_path, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&archive.id)
        .bind(&archive.app_id)
        .bind(&archive.status)
        .bind(&archive.file_path)
        .bind(archive.created_at)
        .bind(archive.expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_archive(&self, archive_id: &str, app_id: &str) -> Result<Option<Archive>> {
        let archive = sqlx::query_as::<_, Archive>(
            r#"
            SELECT id, app_id, status, file_path, created_at, expires_at
            FROM archives
            WHERE id = ? AND app_id = ?
            "#
        )
        .bind(archive_id)
        .bind(app_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(archive)
    }

    pub async fn update_archive_status(&self, archive_id: &str, status: &str, file_path: Option<String>) -> Result<()> {
        sqlx::query(
            "UPDATE archives SET status = ?, file_path = ? WHERE id = ?"
        )
        .bind(status)
        .bind(file_path)
        .bind(archive_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // --- Releases ---

    pub async fn create_release(&self, release: &Release) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO releases (id, app_id, file_id, version_name, description, is_active, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&release.id)
        .bind(&release.app_id)
        .bind(&release.file_id)
        .bind(&release.version_name)
        .bind(&release.description)
        .bind(release.is_active)
        .bind(release.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_latest_release(&self, app_id: &str) -> Result<Option<ReleaseWithFile>> {
        let release = sqlx::query_as::<_, ReleaseWithFile>(
            r#"
            SELECT r.id, r.app_id, r.file_id, r.version_name, r.description, r.is_active, r.created_at,
                   f.original_name as file_original_name, f.size_bytes as file_size_bytes
            FROM releases r
            JOIN files f ON r.file_id = f.id
            WHERE r.app_id = ?
            ORDER BY r.created_at DESC
            LIMIT 1
            "#
        )
        .bind(app_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(release)
    }

    pub async fn list_releases(&self, app_id: &str, limit: i64, offset: i64) -> Result<Vec<ReleaseWithFile>> {
        let releases = sqlx::query_as::<_, ReleaseWithFile>(
            r#"
            SELECT r.id, r.app_id, r.file_id, r.version_name, r.description, r.is_active, r.created_at,
                   f.original_name as file_original_name, f.size_bytes as file_size_bytes
            FROM releases r
            JOIN files f ON r.file_id = f.id
            WHERE r.app_id = ?
            ORDER BY r.created_at DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(app_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(releases)
    }

    pub async fn get_latest_global_release(&self) -> Result<Option<ReleaseWithFile>> {
        let release = sqlx::query_as::<_, ReleaseWithFile>(
            r#"
            SELECT r.id, r.app_id, r.file_id, r.version_name, r.description, r.is_active, r.created_at,
                   f.original_name as file_original_name, f.size_bytes as file_size_bytes
            FROM releases r
            JOIN files f ON r.file_id = f.id
            ORDER BY r.created_at DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(release)
    }

    pub async fn get_active_release(&self) -> Result<Option<ReleaseWithFile>> {
        let release = sqlx::query_as::<_, ReleaseWithFile>(
            r#"
            SELECT r.id, r.app_id, r.file_id, r.version_name, r.description, r.is_active, r.created_at,
                   f.original_name as file_original_name, f.size_bytes as file_size_bytes
            FROM releases r
            JOIN files f ON r.file_id = f.id
            WHERE r.is_active = 1
            ORDER BY r.created_at DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(release)
    }

    pub async fn set_release_active(&self, release_id: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Deactivate all
        sqlx::query("UPDATE releases SET is_active = 0 WHERE is_active = 1")
            .execute(&mut *tx)
            .await?;

        // Activate target
        sqlx::query("UPDATE releases SET is_active = 1 WHERE id = ?")
            .bind(release_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    // --- Settings ---

    pub async fn get_setting(&self, key: &str) -> Result<Option<Setting>> {
        let setting = sqlx::query_as::<_, Setting>(
            "SELECT key, value, updated_at FROM settings WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(setting)
    }

    pub async fn get_all_settings(&self) -> Result<Vec<Setting>> {
        let settings = sqlx::query_as::<_, Setting>(
            "SELECT key, value, updated_at FROM settings ORDER BY key"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(settings)
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let now = Utc::now().timestamp();
        sqlx::query(
            r#"
            INSERT INTO settings (key, value, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
            "#
        )
        .bind(key)
        .bind(value)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_setting(&self, key: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM settings WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct Release {
    pub id: String,
    pub app_id: String,
    pub file_id: String,
    pub version_name: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ReleaseWithFile {
    pub id: String,
    pub app_id: String,
    pub file_id: String,
    pub version_name: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
    pub file_original_name: String,
    pub file_size_bytes: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: i64,
}

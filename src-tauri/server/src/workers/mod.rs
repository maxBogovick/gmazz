use std::sync::Arc;
use tokio::time::{interval, Duration};
use crate::AppState;
use chrono::Utc;

pub async fn start_background_workers(state: Arc<AppState>) {
    let state_cleanup = state.clone();
    tokio::spawn(async move {
        run_cleanup_loop(state_cleanup).await;
    });
}

async fn run_cleanup_loop(state: Arc<AppState>) {
    // Run every hour
    let mut timer = interval(Duration::from_secs(3600));

    loop {
        timer.tick().await;
        tracing::info!("Starting background cleanup...");

        if let Err(e) = cleanup_tmp_files(&state).await {
            tracing::error!("Cleanup tmp files failed: {}", e);
        }

        if let Err(e) = cleanup_expired_archives(&state).await {
            tracing::error!("Cleanup expired archives failed: {}", e);
        }
        
        if let Err(e) = cleanup_soft_deleted_files(&state).await {
             tracing::error!("Cleanup soft-deleted files failed: {}", e);
        }

        tracing::info!("Background cleanup finished.");
    }
}

async fn cleanup_tmp_files(state: &Arc<AppState>) -> anyhow::Result<()> {
    let tmp_dir = state.storage.get_absolute_path("tmp");
    if !tmp_dir.exists() { return Ok(()); }

    let mut entries = tokio::fs::read_dir(tmp_dir).await?;
    let now = std::time::SystemTime::now();
    let ttl = Duration::from_secs(state.config.storage.tmp_ttl_seconds);

    while let Some(entry) = entries.next_entry().await.ok().flatten() {
        if let Ok(metadata) = entry.metadata().await {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age > ttl {
                        tracing::info!("Removing old tmp file: {:?}", entry.path());
                        let _ = tokio::fs::remove_file(entry.path()).await;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn cleanup_expired_archives(state: &Arc<AppState>) -> anyhow::Result<()> {
    // 1. Get expired archives from DB
    let now = Utc::now().timestamp();
    
    // Using raw query here because we didn't add this specific method to Repo and it's worker logic.
    // Ideally should be in Repo, but for speed putting here.
    let expired: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT id, file_path FROM archives WHERE expires_at < ?"
    )
    .bind(now)
    .fetch_all(&state.repo.pool)
    .await?;

    for (id, path) in expired {
        tracing::info!("Removing expired archive: {}", id);
        
        // Remove file
        if let Some(p) = path {
            let abs_path = state.storage.get_absolute_path(&p);
            if abs_path.exists() {
                let _ = tokio::fs::remove_file(abs_path).await;
            }
        }

        // Remove from DB
        sqlx::query("DELETE FROM archives WHERE id = ?")
            .bind(id)
            .execute(&state.repo.pool)
            .await?;
    }

    Ok(())
}

async fn cleanup_soft_deleted_files(state: &Arc<AppState>) -> anyhow::Result<()> {
    let now = Utc::now().timestamp();
    let retention_days = state.config.retention.soft_delete_days as i64;
    let cutoff = now - (retention_days * 24 * 3600);

    let to_delete: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, stored_path FROM files WHERE deleted_at IS NOT NULL AND deleted_at < ?"
    )
    .bind(cutoff)
    .fetch_all(&state.repo.pool)
    .await?;

    for (id, path) in to_delete {
        tracing::info!("Permanently deleting file: {}", id);

        // Remove file
        let abs_path = state.storage.get_absolute_path(&path);
        if abs_path.exists() {
             let _ = tokio::fs::remove_file(abs_path).await;
        }

        // Remove from DB
        sqlx::query("DELETE FROM files WHERE id = ?")
            .bind(id)
            .execute(&state.repo.pool)
            .await?;
    }

    Ok(())
}

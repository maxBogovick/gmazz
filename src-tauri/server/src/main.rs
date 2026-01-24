use std::sync::Arc;
use clap::Parser;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use gmazz_file_server::{config::Config, db, storage::StorageManager, server, cli, workers, AppState};
use gmazz_file_server::db::repo::Repo;
use gmazz_file_server::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Config
    let config = Config::load();

    // 2. Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,file_server=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 3. Storage
    let storage = StorageManager::new(config.storage.data_dir.clone());
    storage.ensure_structure().await?;

    // 4. Database
    let db = db::Db::new(&config.db_path()).await?;
    let repo = Repo::new(db.pool.clone());

    let state = Arc::new(AppState {
        config: config.clone(),
        repo: repo.clone(),
        storage: storage.clone(),
        file_service: gmazz_file_server::services::FileService::new(repo.clone(), storage.clone(), config.clone()),
        archive_service: gmazz_file_server::services::ArchiveService::new(repo.clone(), storage.clone()),
        admin_service: gmazz_file_server::services::AdminService::new(repo.clone(), config.clone()),
        release_service: gmazz_file_server::services::ReleaseService::new(repo.clone()),
    });

    // 5. CLI Args
    let args = Cli::parse();

    if args.command.is_some() {
        // Run CLI command
        cli::handle_cli(state, args).await?;
    } else {
        // Start Workers
        workers::start_background_workers(state.clone()).await;

        // Run Server
        let app = server::app(state.clone()).await;
        let addr = format!("{}:{}", config.server.host, config.server.port);
        info!("Listening on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}

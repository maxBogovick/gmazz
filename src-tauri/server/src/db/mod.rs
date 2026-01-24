use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, migrate::MigrateDatabase};
use anyhow::Result;
use tracing::{info, warn};
use std::path::Path;

pub mod repo;

#[derive(Clone)]
pub struct Db {
    pub pool: Pool<Sqlite>,
}

impl Db {
    pub async fn new(db_url: &str) -> Result<Self> {
        if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
            info!("Creating database: {}", db_url);
            Sqlite::create_database(db_url).await?;
        } else {
            info!("Database already exists: {}", db_url);
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        // Run migrations
        info!("Running migrations...");
        sqlx::migrate!("./src/db/migrations")
            .run(&pool)
            .await?;
        
        info!("Migrations applied successfully.");

        // Apply PRAGMAs for performance and safety
        sqlx::query("PRAGMA journal_mode = WAL;").execute(&pool).await?;
        sqlx::query("PRAGMA synchronous = NORMAL;").execute(&pool).await?;
        sqlx::query("PRAGMA foreign_keys = ON;").execute(&pool).await?;
        sqlx::query("PRAGMA temp_store = MEMORY;").execute(&pool).await?;

        Ok(Self { pool })
    }
}

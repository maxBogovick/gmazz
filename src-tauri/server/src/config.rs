use serde::Deserialize;
use std::env;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub sqlite: SqliteConfig,
    pub security: SecurityConfig,
    pub retention: RetentionConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub request_body_limit_bytes: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub tmp_ttl_seconds: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SqliteConfig {
    pub db_name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SecurityConfig {
    pub server_secret: String,
    pub admin_secret: String,
    pub max_api_keys: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RetentionConfig {
    pub soft_delete_days: u64,
}

impl Config {
    pub fn load() -> Self {
        // Load .env file if it exists
        let _ = dotenvy::dotenv();

        let data_dir_str = env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string());
        let data_dir = PathBuf::from(data_dir_str);

        Self {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .expect("PORT must be a number"),
                request_body_limit_bytes: env::var("REQUEST_BODY_LIMIT_BYTES")
                    .unwrap_or_else(|_| (2 * 1024 * 1024).to_string()) // 2GB
                    .parse()
                    .expect("REQUEST_BODY_LIMIT_BYTES must be a number"),
            },
            storage: StorageConfig {
                data_dir: data_dir.clone(),
                tmp_ttl_seconds: env::var("TMP_TTL_SECONDS")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()
                    .expect("TMP_TTL_SECONDS must be a number"),
            },
            sqlite: SqliteConfig {
                db_name: env::var("DB_NAME").unwrap_or_else(|_| "metadata.db".to_string()),
            },
            security: SecurityConfig {
                server_secret: env::var("SERVER_SECRET")
                    .expect("SERVER_SECRET environment variable is required"),
                admin_secret: env::var("ADMIN_SECRET")
                    .expect("ADMIN_SECRET environment variable is required"),
                max_api_keys: env::var("MAX_API_KEYS")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .expect("MAX_API_KEYS must be a number"),
            },
            retention: RetentionConfig {
                soft_delete_days: env::var("SOFT_DELETE_DAYS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .expect("SOFT_DELETE_DAYS must be a number"),
            },
        }
    }

    pub fn db_path(&self) -> String {
        self.storage.data_dir.join(&self.sqlite.db_name).to_string_lossy().to_string()
    }
}

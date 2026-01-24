use uuid::Uuid;
use crate::db::repo::{Repo, App};
use crate::error::AppError;
use crate::auth::hash_api_key;
use crate::api::admin::{CreateKeyResponse, CreateKeyRequest};
use crate::config::Config;

pub struct AdminService {
    repo: Repo,
    config: Config,
}

impl AdminService {
    pub fn new(repo: Repo, config: Config) -> Self {
        Self { repo, config }
    }

    pub async fn create_key(&self, req: CreateKeyRequest) -> Result<CreateKeyResponse, AppError> {
        let plain_key = Uuid::new_v4().to_string().replace("-", "") + &Uuid::new_v4().to_string().replace("-", "");
        let hash = hash_api_key(&self.config.security.server_secret, &plain_key);

        let id = self.repo.create_app(&req.name, &hash).await.map_err(|e| AppError::Anyhow(e))?;

        Ok(CreateKeyResponse {
            id,
            name: req.name,
            api_key: plain_key,
        })
    }

    pub async fn list_keys(&self) -> Result<Vec<App>, AppError> {
        Ok(self.repo.list_apps().await.map_err(|e| AppError::Anyhow(e))?)
    }

    pub async fn update_key_status(&self, id: &str, is_active: bool) -> Result<(), AppError> {
        let affected = self.repo.update_app_status(id, is_active).await.map_err(|e| AppError::Anyhow(e))?;
        if affected {
            Ok(())
        } else {
            Err(AppError::NotFound("App ID not found".to_string()))
        }
    }
}

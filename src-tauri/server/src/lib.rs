pub mod config;
pub mod error;
pub mod auth;
pub mod server;
pub mod cli;
pub mod storage;
pub mod db;
pub mod api;
pub mod services;
pub mod workers;

use std::sync::Arc;
use config::Config;
use db::repo::Repo;
use storage::StorageManager;
use services::{FileService, ArchiveService, AdminService, ReleaseService};

pub struct AppState {
    pub config: Config,
    pub repo: Repo,
    pub storage: StorageManager,
    pub file_service: FileService,
    pub archive_service: ArchiveService,
    pub admin_service: AdminService,
    pub release_service: ReleaseService,
}
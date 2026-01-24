use file_server::{AppState, config::Config, db, storage::StorageManager, server};
use file_server::db::repo::Repo;
use std::sync::Arc;
use tokio::net::TcpListener;
use std::path::PathBuf;
use reqwest::Client;
use serde_json::Value;

// Helper struct to hold test context and cleanup on drop (if needed, though TempDir handles it)
struct TestApp {
    pub address: String,
    pub http_client: Client,
    pub _temp_dir: tempfile::TempDir, // Keep alive to prevent cleanup
    pub admin_secret: String,
}

impl TestApp {
    pub async fn spawn() -> Self {
        // 1. Create Temp Directory for isolation
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let data_path = temp_dir.path().to_path_buf();
        let db_path = data_path.join("test.db").to_string_lossy().to_string();

        // 2. Setup Config manually
        let admin_secret = "test_admin_secret".to_string();
        let config = Config {
            server: file_server::config::ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0, // OS assigns random port
                request_body_limit_bytes: 1024 * 1024 * 10,
            },
            storage: file_server::config::StorageConfig {
                data_dir: data_path.clone(),
                tmp_ttl_seconds: 60,
            },
            sqlite: file_server::config::SqliteConfig {
                db_name: "test.db".to_string(),
            },
            security: file_server::config::SecurityConfig {
                server_secret: "test_server_secret".to_string(),
                admin_secret: admin_secret.clone(),
                max_api_keys: 10,
            },
            retention: file_server::config::RetentionConfig {
                soft_delete_days: 1,
            },
        };

        // 3. Init DB and Storage
        let storage = StorageManager::new(config.storage.data_dir.clone());
        storage.ensure_structure().await.expect("Failed to init storage");

        let db = db::Db::new(&db_path).await.expect("Failed to init DB");
        let repo = Repo::new(db.pool.clone());

        let state = Arc::new(AppState {
            config: config.clone(),
            repo: repo.clone(),
            storage: storage.clone(),
            file_service: file_server::services::FileService::new(repo.clone(), storage.clone(), config.clone()),
            archive_service: file_server::services::ArchiveService::new(repo.clone(), storage.clone()),
            admin_service: file_server::services::AdminService::new(repo.clone(), config.clone()),
        });

        // 4. Start Server on random port
        let app = server::app(state).await;
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind random port");
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{}", port);

        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("Failed to start server");
        });

        Self {
            address,
            http_client: Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap(),
            _temp_dir: temp_dir,
            admin_secret,
        }
    }

    pub async fn create_api_key(&self, name: &str) -> String {
        let response = self.http_client.post(format!("{}/admin/keys", self.address))
            .header("X-ADMIN-SECRET", &self.admin_secret)
            .json(&serde_json::json!({ "name": name }))
            .send()
            .await
            .expect("Failed to execute request");
        
        assert_eq!(response.status(), 201);
        let json: Value = response.json().await.unwrap();
        json["api_key"].as_str().unwrap().to_string()
    }
}

#[tokio::test]
async fn test_admin_flow() {
    let app = TestApp::spawn().await;

    // 1. Fail without secret
    let response = app.http_client.get(format!("{}/admin/keys", app.address)).send().await.unwrap();
    assert_eq!(response.status(), 401);

    // 2. Success with secret
    let response = app.http_client.get(format!("{}/admin/keys", app.address))
        .header("X-ADMIN-SECRET", &app.admin_secret)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // 3. Create Key
    let api_key = app.create_api_key("Integration Test App").await;
    assert!(!api_key.is_empty());

    // 4. Verify in List
    let list_response: Vec<Value> = app.http_client.get(format!("{}/admin/keys", app.address))
        .header("X-ADMIN-SECRET", &app.admin_secret)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    assert!(list_response.iter().any(|k| k["name"] == "Integration Test App"));
}

#[tokio::test]
async fn test_file_upload_download() {
    let app = TestApp::spawn().await;
    let api_key = app.create_api_key("File App").await;

    // 1. Upload
    let file_content = "Hello Integration World";
    let response = app.http_client.post(format!("{}/v1/files", app.address))
        .header("X-API-KEY", &api_key)
        .header("X-File-Name", "test.txt")
        .header("Content-Type", "text/plain")
        .body(file_content)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 201);
    let json: Value = response.json().await.unwrap();
    let file_id = json["id"].as_str().unwrap();

    // 2. Get Metadata
    let meta_res = app.http_client.get(format!("{}/v1/files/{}/meta", app.address, file_id))
        .header("X-API-KEY", &api_key)
        .send()
        .await
        .unwrap();
    assert_eq!(meta_res.status(), 200);
    let meta_json: Value = meta_res.json().await.unwrap();
    assert_eq!(meta_json["original_name"], "test.txt");

    // 3. Download
    let dl_res = app.http_client.get(format!("{}/v1/files/{}", app.address, file_id))
        .header("X-API-KEY", &api_key)
        .send()
        .await
        .unwrap();
    assert_eq!(dl_res.status(), 200);
    assert_eq!(dl_res.text().await.unwrap(), file_content);

    // 4. Soft Delete
    let del_res = app.http_client.delete(format!("{}/v1/files/{}", app.address, file_id))
        .header("X-API-KEY", &api_key)
        .send()
        .await
        .unwrap();
    assert_eq!(del_res.status(), 204);

    // 5. Verify 404 after delete
    let dl_res_404 = app.http_client.get(format!("{}/v1/files/{}", app.address, file_id))
        .header("X-API-KEY", &api_key)
        .send()
        .await
        .unwrap();
    assert_eq!(dl_res_404.status(), 404);
}

#[tokio::test]
async fn test_archives() {
    let app = TestApp::spawn().await;
    let api_key = app.create_api_key("Archive App").await;

    // Upload 2 files
    for i in 0..2 {
        app.http_client.post(format!("{}/v1/files", app.address))
            .header("X-API-KEY", &api_key)
            .header("X-File-Name", format!("file{}.txt", i))
            .body(format!("content {}", i))
            .send()
            .await
            .unwrap();
    }

    // Trigger Archive
    let res = app.http_client.post(format!("{}/v1/archives", app.address))
        .header("X-API-KEY", &api_key)
        .send()
        .await
        .unwrap();
    
    assert_eq!(res.status(), 202);
    let json: Value = res.json().await.unwrap();
    let archive_id = json["archive_id"].as_str().unwrap();

    // Poll for completion (max 5 seconds)
    let mut ready = false;
    for _ in 0..10 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let check = app.http_client.get(format!("{}/v1/archives/{}", app.address, archive_id))
            .header("X-API-KEY", &api_key)
            .send()
            .await
            .unwrap();
        
        if check.status() == 200 {
            // Verify Content Type
            assert_eq!(check.headers().get("content-type").unwrap(), "application/zip");
            let bytes = check.bytes().await.unwrap();
            // Basic zip check
            assert!(bytes.len() > 0);
            ready = true;
            break;
        }
    }
    assert!(ready, "Archive did not complete in time");
}

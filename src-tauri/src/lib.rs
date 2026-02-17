mod commands;
mod database;
mod models;
mod sync;

use commands::AppState;
use tauri::Manager;
use sync::SyncClient;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            
            // --- Sync Config ---
            // Ideally load from config file or env.
            // Using placeholder defaults for now.
            let server_url = std::env::var("GMAZZ_SERVER_URL").unwrap_or("http://localhost:8080".to_string());
            let api_key = std::env::var("GMAZZ_API_KEY").unwrap_or("d44b36ed5bc340778871680b49e089f729c7b08939514940848d69b2c6b730f0".to_string());
            
            let sync_client = if !api_key.is_empty() {
                Some(SyncClient::new(server_url, api_key))
            } else {
                println!("Sync disabled: GMAZZ_API_KEY not set.");
                None
            };
            
            // Offline-first: no automatic network sync on startup.

            let db_data_dir = app_data_dir.clone();
            let db = tauri::async_runtime::block_on(async move {
                match database::init_database(db_data_dir.clone()).await {
                    Ok(pool) => Ok(pool),
                    Err(_) => {
                        eprintln!("Database corrupted. backing up and recreating...");
                        let db_path = db_data_dir.join("notebook.db");
                        let backup_path = db_data_dir.join(format!("notebook.db.corrupt.{}", chrono::Utc::now().timestamp()));
                        if db_path.exists() {
                            let _ = std::fs::rename(&db_path, &backup_path);
                        }
                        // Retry init
                        database::init_database(db_data_dir).await
                    }
                }
            }).expect("Failed to initialize database");

            let storage_dir = app_data_dir.join("files");
            if !storage_dir.exists() {
                std::fs::create_dir_all(&storage_dir).expect("Failed to create storage directory");
            }

            app.manage(AppState { db, sync_client, storage_dir });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_note,
            commands::update_note,
            commands::get_notes,
            commands::get_note,
            commands::get_random_note,
            commands::delete_note,
            commands::upload_file,
            commands::get_asset_path,
            commands::sync_local_db_to_server,
            commands::export_local_db_to_downloads,
            commands::get_all_settings,
            commands::get_setting,
            commands::set_setting,
            commands::delete_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

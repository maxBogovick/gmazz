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
            
            // --- Initial Download ---
            if let Some(client) = &sync_client {
                let db_path = app_data_dir.join("notebook.db");
                // Blocking async for setup is tricky in Tauri, but we can verify DB existence
                // Usually we run this in a separate thread, but for DB init we want it before.
                // We use tauri::async_runtime::block_on
                if let Err(e) = tauri::async_runtime::block_on(async {
                    client.download_latest_db(&db_path).await
                }) {
                   eprintln!("Failed to download DB: {}", e); 
                }
            }
            // -----------------------

            let db = tauri::async_runtime::block_on(async {
                database::init_database(app_data_dir)
                    .await
                    .expect("Failed to initialize database")
            });

            app.manage(AppState { db, sync_client });

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

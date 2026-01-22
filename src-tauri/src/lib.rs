mod commands;
mod database;
mod models;

use commands::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;

            let db = tauri::async_runtime::block_on(async {
                database::init_database(app_data_dir)
                    .await
                    .expect("Failed to initialize database")
            });

            app.manage(AppState { db });

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

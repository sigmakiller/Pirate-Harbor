mod commands;
mod db;
mod models;

use std::sync::Mutex;
use tauri::Manager;

use db::DbState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            let conn = db::init_db(&app_data_dir)
                .expect("Failed to initialize database");

            app.manage(DbState(Mutex::new(conn)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Game CRUD
            commands::games::get_all_games,
            commands::games::get_game,
            commands::games::add_game,
            commands::games::update_game,
            commands::games::delete_game,
            commands::games::toggle_favorite,
            // Settings
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_all_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

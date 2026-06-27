mod commands;
mod db;
mod models;

use std::sync::Mutex;

use tauri::Manager;

use commands::launcher::LauncherState;
use db::DbState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // ── Database ──────────────────────────────────────────────────────
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            let conn = db::init_db(&app_data_dir)
                .expect("Failed to initialize database");

            app.manage(DbState(Mutex::new(conn)));

            // ── Launcher state ────────────────────────────────────────────────
            app.manage(LauncherState(Mutex::new(None)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // ── Game CRUD ─────────────────────────────────────────────────────
            commands::games::get_all_games,
            commands::games::get_game,
            commands::games::add_game,
            commands::games::update_game,
            commands::games::delete_game,
            commands::games::toggle_favorite,
            // ── Settings ──────────────────────────────────────────────────────
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_all_settings,
            // ── Launcher ──────────────────────────────────────────────────────
            commands::launcher::launch_game,
            commands::launcher::get_running_game,
            // ── Sessions ──────────────────────────────────────────────────────
            commands::sessions::get_sessions,
            // ── Scanner ───────────────────────────────────────────────────────
            commands::scanner::get_scan_directories,
            commands::scanner::add_scan_directory,
            commands::scanner::remove_scan_directory,
            commands::scanner::scan_directory,
            commands::scanner::scan_all_directories,
            commands::scanner::batch_add_games,
            // ── Metadata ──────────────────────────────────────────────────────
            commands::metadata::search_game_metadata,
            commands::metadata::get_rawg_api_key,
            // ── Collections ───────────────────────────────────────────────────
            commands::collections::get_collections,
            commands::collections::get_collection,
            commands::collections::create_collection,
            commands::collections::update_collection,
            commands::collections::delete_collection,
            commands::collections::add_game_to_collection,
            commands::collections::remove_game_from_collection,
            commands::collections::get_game_collections,
            // ── Journal ───────────────────────────────────────────────────────────
            commands::journal::get_journal_entries,
            commands::journal::create_journal_entry,
            commands::journal::update_journal_entry,
            commands::journal::delete_journal_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

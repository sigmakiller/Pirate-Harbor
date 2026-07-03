mod analytics;
mod api;
mod assets;
mod background;
mod commands;
mod db;
mod images;
mod metadata;
mod models;

use std::sync::Mutex;

use tauri::Manager;

use assets::AssetManager;
use background::JobScheduler;
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

            // ── Asset manager ─────────────────────────────────────────────────
            let asset_manager = AssetManager::new(&app_data_dir)
                .expect("Failed to initialize asset manager");
            app.manage(asset_manager);

            // ── Background job scheduler ───────────────────────────────────────
            let scheduler = JobScheduler::new();
            app.manage(scheduler.clone());

            // Determine the DB path so the worker can open its own connection.
            let db_path = app_data_dir.join("pirate_harbor.db");

            // Start the worker loop — runs for the lifetime of the app.
            background::start_worker(
                scheduler.state.clone(),
                db_path,
                app.handle().clone(),
            );

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
            commands::metadata::enrich_game_metadata,
            commands::metadata::bulk_enrich_library,
            commands::metadata::get_enrichment_status,
            commands::metadata::get_rawg_api_key,
            commands::metadata::download_game_images,
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
            // ── Milestones ────────────────────────────────────────────────────────
            commands::milestones::create_milestone,
            commands::milestones::get_milestones,
            commands::milestones::delete_milestone,
            commands::milestones::get_milestone_templates,
            commands::milestones::create_milestone_template,
            commands::milestones::create_milestone_from_template,
            commands::milestones::seed_default_templates,
            commands::milestones::get_milestone_statistics,
            commands::milestones::migrate_journal_to_milestones,
            // ── Identity ──────────────────────────────────────────────────────
            commands::identity::get_gaming_identity,
            // ── Background jobs ───────────────────────────────────────────────
            commands::background::get_job_status,
            commands::background::cancel_job,
            commands::background::list_active_jobs,
            commands::background::list_all_jobs,
            commands::background::queue_depth,
            // ── Assets (T28) ──────────────────────────────────────────────────
            commands::assets::store_cover,
            commands::assets::store_background,
            commands::assets::get_cover_path,
            commands::assets::delete_cover,
            commands::assets::store_gallery_image,
            commands::assets::get_gallery_images,
            commands::assets::delete_gallery_image,
            commands::assets::delete_game_gallery,
            commands::assets::get_storage_stats,
            commands::assets::cleanup_orphan_assets,
            commands::assets::check_duplicate,
            // ── Search / FTS5 (T29) ───────────────────────────────────────────
            commands::search::search_global,
            commands::search::rebuild_search_index,
            // ── Recommendations (T31) ─────────────────────────────────────────
            commands::recommendations::get_recommendations,
            commands::recommendations::get_game_recommendations,
            // ── Analytics engines (T30) ───────────────────────────────────────
            commands::analytics::get_library_summary,
            commands::analytics::get_most_played_games,
            commands::analytics::get_playtime_trend,
            commands::analytics::get_activity_heatmap,
            commands::analytics::get_genre_distribution,
            commands::analytics::get_completion_stats,
            commands::analytics::get_year_in_review,
            commands::analytics::get_related_games,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

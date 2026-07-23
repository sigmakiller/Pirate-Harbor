mod analytics;
mod api;
mod assets;
mod background;
mod commands;
mod db;
mod images;
mod metadata;
mod models;
mod steam_bridge;
#[cfg(test)]
mod tests;

use std::sync::Mutex;

use tauri::Manager;

use assets::AssetManager;
use background::JobScheduler;
use commands::launcher::LauncherState;
use db::DbState;
use steam_bridge::achievement_watcher::WatcherRegistry;

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

            // C1 fix: persist app_data_dir so backup/restore/diagnostics commands
            // can resolve the correct asset root without guessing from path heuristics.
            conn.execute(
                "INSERT INTO settings (key, value) VALUES ('app_data_dir', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                rusqlite::params![app_data_dir.to_string_lossy()],
            ).expect("Failed to write app_data_dir to settings");

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

            // ── Achievement watcher registry (T40) ────────────────────────────
            // Holds active notify watchers keyed by game_id. Populated by
            // T41 Tauri commands (enable_achievement_tracking / disable_tracking).
            app.manage(WatcherRegistry::new());

            // ── T49: Startup scheduled jobs ────────────────────────────────────
            // Both jobs are idempotent — AutoBackupJob checks `is_auto_backup_due`
            // internally, and MetadataRefreshJob skips if the RAWG key is absent.
            {
                // Open a short-lived connection just to read settings; the
                // primary connection is already managed by DbState above.
                if let Ok(cfg_conn) = db::init_db(&app_data_dir) {
                    // Auto-backup — skipped when `auto_backup_enabled = "false"`.
                    let enabled: bool = cfg_conn
                        .query_row(
                            "SELECT value FROM settings WHERE key = 'auto_backup_enabled'",
                            [],
                            |r| r.get::<_, String>(0),
                        )
                        .ok()
                        .map(|v| v != "false")
                        .unwrap_or(true);

                    if enabled {
                        scheduler.enqueue_with_priority(
                            commands::backup::AutoBackupJob {
                                app_data_dir: app_data_dir.clone(),
                            },
                            background::Priority::Normal,
                        );
                    }

                    // Metadata refresh — self-skips when api_key is None.
                    let rawg_key: Option<String> = cfg_conn
                        .query_row(
                            "SELECT value FROM settings WHERE key = 'rawg_api_key'",
                            [],
                            |r| r.get(0),
                        )
                        .ok();

                    scheduler.enqueue_with_priority(
                        commands::metadata::MetadataRefreshJob { api_key: rawg_key },
                        background::Priority::Normal,
                    );
                }
            }

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
            commands::metadata::start_bulk_enrichment_job,   // T50
            commands::metadata::get_stale_games_count,       // T51
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
            commands::milestones::get_recent_milestones,     // T53
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
            commands::analytics::get_session_years,        // T52
            commands::analytics::get_monthly_playtime,     // T52
            commands::analytics::get_date_heatmap,         // T53
            commands::analytics::get_milestone_streak_stats, // T54
            // ── Export (T32) ──────────────────────────────────────────────────────
            commands::export::get_export_preview,
            commands::export::export_library_json,
            commands::export::export_profile_markdown,
            // ── Backup (T33) ──────────────────────────────────────────────────────
            commands::backup::create_backup,
            commands::backup::restore_backup,
            commands::backup::list_auto_backups,
            commands::backup::get_auto_backup_enabled,
            commands::backup::set_auto_backup_enabled,
            // ── Diagnostics (T35) ────────────────────────────────────────────────
            commands::diagnostics::get_diagnostics,
            commands::diagnostics::run_integrity_check,
            commands::diagnostics::get_db_path,
            // ── Achievement tracking (T42) ────────────────────────────────────────
            commands::achievements::enable_achievement_tracking,
            commands::achievements::disable_achievement_tracking,
            commands::achievements::get_achievement_tracking_status,
            commands::achievements::add_achievement_mapping,
            commands::achievements::remove_achievement_mapping,
            commands::achievements::get_achievement_mappings,
            commands::achievements::import_achievements_from_steam,
            // ── Steam App ID detection (T43) ──────────────────────────────────────
            commands::achievements::detect_steam_app_id,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

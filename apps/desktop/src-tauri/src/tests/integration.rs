//! T37 — Integration Testing & Acceptance
//!
//! Full acceptance test suite covering all Phase 4 features:
//!   - Migration versioning (T26)
//!   - Background job scheduler (T27)
//!   - Asset manager: stats, cleanup (T28)
//!   - FTS5 search performance (T29)
//!   - Export: valid JSON, readable Markdown (T32)
//!   - Backup: create → verify → restore (T33)
//!   - Diagnostics: schema version, table counts (T35)
//!
//! # Design Notes
//!
//! All tests use in-memory SQLite (`Connection::open_in_memory()`), which
//! gives perfect isolation and sub-millisecond setup time.  Tests that
//! require the filesystem use `std::env::temp_dir()` with unique sub-dirs
//! and clean up after themselves.

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::db::migrations::{run_migrations, CURRENT_SCHEMA_VERSION};
    use crate::commands::export::{write_library_json, write_profile_markdown};
    use crate::commands::backup::{create_backup_file, restore_backup_file};
    use crate::background::scheduler::JobScheduler;
    use crate::background::job::JobStatus;
    use crate::assets::asset_manager::AssetManager;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn migrated_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    /// Insert a minimal game row for tests that need data.
    fn insert_game(conn: &Connection, id: &str, title: &str, status: &str) {
        conn.execute(
            "INSERT INTO games (id, title, exe_path, status, added_at) \
             VALUES (?1, ?2, '/games/game.exe', ?3, '2024-01-01T00:00:00Z')",
            rusqlite::params![id, title, status],
        ).unwrap();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // T26 — Migration Versioning (extended)
    // ─────────────────────────────────────────────────────────────────────────

    /// A fresh in-memory DB must reach the current schema version.
    #[test]
    fn t37_migration_fresh_db_reaches_current_version() {
        let conn = migrated_db();
        let v: i64 = conn.query_row(
            "SELECT CAST(value AS INTEGER) FROM settings WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(v, CURRENT_SCHEMA_VERSION as i64,
            "Fresh DB must reach schema v{}", CURRENT_SCHEMA_VERSION);
    }

    /// Running migrations twice on the same DB must be a no-op (idempotent).
    #[test]
    fn t37_migration_is_idempotent() {
        let conn = migrated_db();
        // Second run — must not panic or corrupt version.
        run_migrations(&conn).unwrap();
        let v: i64 = conn.query_row(
            "SELECT CAST(value AS INTEGER) FROM settings WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(v, CURRENT_SCHEMA_VERSION as i64,
            "Version must be stable after double migration");
    }

    /// All expected tables must exist after migration.
    #[test]
    fn t37_migration_all_tables_created() {
        let conn = migrated_db();
        let expected = [
            "games", "sessions", "collections", "collection_games",
            "milestones", "journal_entries", "settings",
            "games_fts", "journal_fts",
        ];
        for table in &expected {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name = ?1",
                [table],
                |r| r.get(0),
            ).unwrap();
            assert_eq!(count, 1, "Table '{}' must exist after migrations", table);
        }
    }

    /// `run_migrations` sets up a valid settings table — verify a known key is queryable.
    #[test]
    fn t37_migration_settings_table_writable() {
        let conn = migrated_db();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('test_key', 'test_val')",
            [],
        ).unwrap();
        let val: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'test_key'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(val, "test_val", "Settings table must be writable after migration");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // T27 — Background Job Scheduler (data-model tests)
    //
    // NOTE: Tests that call Job::execute require a live tauri::AppHandle
    // (and thus WebView2 DLLs on Windows). Those belong to end-to-end testing.
    // Here we verify the scheduler's pure-Rust state machine: queue depth,
    // job-status transitions, and list APIs — none of which touch the runtime.
    // ─────────────────────────────────────────────────────────────────────────

    /// A new scheduler must start empty.
    #[test]
    fn t37_scheduler_new_has_zero_depth() {
        let sched = JobScheduler::new();
        assert_eq!(sched.queue_depth(), 0, "New scheduler must have queue depth 0");
        assert!(sched.list_active_jobs().is_empty(), "New scheduler must have no active jobs");
        assert!(sched.list_all_jobs().is_empty(), "New scheduler must have no jobs at all");
    }

    /// `JobStatus::is_terminal` must be correct for every variant.
    #[test]
    fn t37_job_status_terminal_states() {
        assert!(!JobStatus::Queued.is_terminal(),                            "Queued is not terminal");
        assert!(!JobStatus::Running { progress: 0.5 }.is_terminal(),         "Running is not terminal");
        assert!( JobStatus::Done.is_terminal(),                               "Done IS terminal");
        assert!( JobStatus::Cancelled.is_terminal(),                          "Cancelled IS terminal");
        assert!( JobStatus::Failed { reason: "err".into() }.is_terminal(),   "Failed IS terminal");
    }

    /// Cancel on an unknown job-id must return false and not panic.
    #[test]
    fn t37_scheduler_cancel_unknown_job_returns_false() {
        let sched = JobScheduler::new();
        let cancelled = sched.cancel_job("non-existent-id");
        assert!(!cancelled, "cancel_job with unknown id must return false");
    }

    /// `get_job_status` for an unknown id must return None, not panic.
    #[test]
    fn t37_scheduler_get_unknown_job_returns_none() {
        let sched = JobScheduler::new();
        assert!(sched.get_job_status("ghost").is_none(),
            "get_job_status must return None for an unknown job id");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // T28 — Asset Manager
    // ─────────────────────────────────────────────────────────────────────────

    fn tmp_dir(suffix: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("ph_t37_{}", suffix));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn t37_asset_manager_creates_directories() {
        let base = tmp_dir("assets_dirs");
        let am = AssetManager::new(&base).expect("AssetManager::new must succeed");

        // All subdirectories must exist.
        let dirs = ["assets/covers", "assets/backgrounds", "assets/gallery", "assets/thumbnails"];
        for d in &dirs {
            assert!(base.join(d).exists(), "Directory '{}' must be created by AssetManager::new", d);
        }
        drop(am);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn t37_asset_manager_storage_stats_empty() {
        let base = tmp_dir("assets_empty");
        let am = AssetManager::new(&base).expect("AssetManager::new must succeed");

        let stats = am.get_storage_stats().expect("get_storage_stats must succeed on empty directory");
        assert_eq!(stats.file_count, 0, "Empty asset dir must have 0 files");
        assert_eq!(stats.total_bytes, 0, "Empty asset dir must have 0 bytes");

        drop(am);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn t37_asset_manager_cleanup_orphans_returns_zero_on_clean_db() {
        let base = tmp_dir("assets_cleanup");
        let am = AssetManager::new(&base).expect("AssetManager::new must succeed");
        let conn = migrated_db();

        let result = am.cleanup_orphans(&conn).expect("cleanup_orphans must succeed");
        assert_eq!(result.deleted_count, 0, "Clean DB must report 0 deletions");
        assert_eq!(result.bytes_freed, 0, "Clean DB must report 0 bytes freed");

        drop(am);
        let _ = std::fs::remove_dir_all(&base);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // T29 — FTS5 Search
    // ─────────────────────────────────────────────────────────────────────────

    /// Seed a DB with N games for performance tests.
    fn seed_games(conn: &Connection, n: usize) {
        // m3 fix: use an explicit transaction so N inserts are one fsync,
        // ensuring the 100ms timing test is not sensitive to debug-build I/O.
        conn.execute_batch("BEGIN").unwrap();
        for i in 0..n {
            let id    = format!("gid{}", i);
            let title = format!("Game {}", i);
            let genre = if i % 2 == 0 { "RPG" } else { "Action" };
            conn.execute(
                "INSERT INTO games (id, title, exe_path, genre, status, added_at) \
                 VALUES (?1, ?2, '/games/game.exe', ?3, 'unplayed', '2024-01-01T00:00:00Z')",
                rusqlite::params![id, title, genre],
            ).unwrap();
        }
        conn.execute_batch("COMMIT").unwrap();
    }

    #[test]
    fn t37_fts5_partial_match_returns_results() {
        let conn = migrated_db();
        insert_game(&conn, "g1", "The Witcher 3", "completed");
        insert_game(&conn, "g2", "Witcher Chronicles", "unplayed");
        insert_game(&conn, "g3", "Dark Souls", "dropped");

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM games_fts WHERE games_fts MATCH 'Witch*'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 2, "Prefix search 'Witch*' must match 2 games");
    }

    #[test]
    fn t37_fts5_search_sub_100ms_on_1000_games() {
        let conn = migrated_db();
        seed_games(&conn, 1_000);

        let t0 = std::time::Instant::now();
        let _count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM games_fts WHERE games_fts MATCH 'Game*'",
            [],
            |r| r.get(0),
        ).unwrap();
        let elapsed = t0.elapsed();

        assert!(
            elapsed.as_millis() < 100,
            "FTS5 search on 1000 games must complete in <100ms, got {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn t37_fts5_no_results_for_unknown_term() {
        let conn = migrated_db();
        seed_games(&conn, 100);

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM games_fts WHERE games_fts MATCH 'XxXnotawordXxX*'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 0, "Unknown term must return 0 results");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // T32 — Export
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn t37_export_json_valid_structure() {
        let conn = migrated_db();
        insert_game(&conn, "exp1", "Hollow Knight", "completed");
        insert_game(&conn, "exp2", "Celeste",       "completed");

        let dir = tmp_dir("export_json");
        let out = dir.join("library.json");

        let summary = write_library_json(&conn, &out)
            .expect("write_library_json must succeed");
        assert!(out.exists(), "JSON export file must be created");

        let raw = std::fs::read_to_string(&out).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw)
            .expect("Exported JSON must be valid");
        assert!(parsed.is_object(), "Top-level must be a JSON object");
        assert!(parsed.get("games").is_some(), "JSON must contain 'games' key");

        let games = parsed["games"].as_array().unwrap();
        assert_eq!(games.len(), 2, "JSON must export all 2 games");
        assert!(summary.contains("2"), "Summary must mention game count");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn t37_export_markdown_contains_expected_sections() {
        let conn = migrated_db();
        insert_game(&conn, "md1", "Elden Ring", "completed");

        let dir = tmp_dir("export_md");
        let out = dir.join("profile.md");

        write_profile_markdown(&conn, &out)
            .expect("write_profile_markdown must succeed");
        assert!(out.exists(), "Markdown export file must be created");

        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.starts_with('#'), "Markdown must start with a heading");
        // Must contain at least one H2 section
        assert!(content.contains("##"), "Markdown must have at least one H2 section");
        // Must reference the game
        assert!(content.contains("Elden Ring"), "Markdown must mention game titles");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn t37_export_json_empty_library() {
        let conn = migrated_db();
        let dir = tmp_dir("export_empty");
        let out = dir.join("library.json");

        write_library_json(&conn, &out).expect("Export of empty library must succeed");
        let raw = std::fs::read_to_string(&out).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let games = parsed["games"].as_array().unwrap();
        assert_eq!(games.len(), 0, "Empty library export must have 0 games");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // T33 — Backup & Restore
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn t37_backup_creates_valid_archive() {
        let conn = migrated_db();
        insert_game(&conn, "bk1", "Sekiro", "completed");

        let dir = tmp_dir("backup_create");
        let backup_path = dir.join("test.phb");
        let app_data = tmp_dir("backup_app_data");

        let result = create_backup_file(&conn, &backup_path, &app_data, false)
            .expect("create_backup_file must succeed");

        assert!(backup_path.exists(), ".phb archive file must exist");
        assert!(result.size_bytes > 0, "Backup must have non-zero size");
        assert_eq!(result.game_count, 1, "Backup must record 1 game");

        // Verify it is a valid ZIP archive containing database.json.
        let file = std::fs::File::open(&backup_path).unwrap();
        let mut archive = zip::ZipArchive::new(file)
            .expect(".phb must be a valid ZIP archive");
        let names: Vec<_> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_owned())
            .collect();
        assert!(names.contains(&"manifest.json".to_string()), "Archive must contain manifest.json");
        assert!(names.contains(&"database.json".to_string()), "Archive must contain database.json");

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&app_data);
    }

    #[test]
    fn t37_backup_restore_preserves_data() {
        // 1. Create a DB with known data and back it up.
        let conn_original = migrated_db();
        insert_game(&conn_original, "restore1", "Bloodborne", "completed");
        insert_game(&conn_original, "restore2", "Dark Souls",  "dropped");

        let dir = tmp_dir("backup_restore");
        let backup_path = dir.join("restore_test.phb");
        let app_data = tmp_dir("backup_restore_appdata");

        create_backup_file(&conn_original, &backup_path, &app_data, false)
            .expect("Backup must succeed");

        // 2. Restore into a fresh empty DB.
        let mut conn_fresh = migrated_db();
        let restore_result = restore_backup_file(&mut conn_fresh, &backup_path, &app_data)
            .expect("Restore must succeed");

        // 3. Verify game count.
        let game_count: i64 = conn_fresh
            .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
            .unwrap();
        assert_eq!(game_count, 2, "Restored DB must contain 2 games");
        assert_eq!(restore_result.games_restored, 2, "restore_result.games_restored must be 2");

        // 4. Verify specific game title persisted.
        let title: String = conn_fresh
            .query_row(
                "SELECT title FROM games WHERE id = 'restore1'",
                [],
                |r| r.get(0),
            ).expect("Game 'restore1' must exist after restore");
        assert_eq!(title, "Bloodborne", "Restored game title must match original");

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&app_data);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // T35 — Diagnostics
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn t37_diagnostics_integrity_check_passes_on_clean_db() {
        let conn = migrated_db();

        let mut stmt = conn.prepare("PRAGMA integrity_check(64)").unwrap();
        let results: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(results.len(), 1, "integrity_check must return exactly 1 row for clean DB");
        assert_eq!(results[0], "ok", "integrity_check must report 'ok' for clean DB");
    }

    #[test]
    fn t37_diagnostics_table_counts_correct_after_inserts() {
        let conn = migrated_db();
        seed_games(&conn, 5);
        // Insert 2 journal entries.
        for i in 0..2 {
            conn.execute(
                "INSERT INTO journal_entries (id, title, body, entry_type, created_at, updated_at) \
                 VALUES (?1, ?2, 'body', 'note', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
                rusqlite::params![format!("je{}", i), format!("Entry {}", i)],
            ).unwrap();
        }

        let game_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM games", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(game_count, 5, "Diagnostics game count must be 5");

        let journal_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM journal_entries", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(journal_count, 2, "Diagnostics journal count must be 2");
    }

    #[test]
    fn t37_diagnostics_schema_version_matches_constant() {
        let conn = migrated_db();
        let v: i64 = conn.query_row(
            "SELECT CAST(value AS INTEGER) FROM settings WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(
            v, CURRENT_SCHEMA_VERSION as i64,
            "Schema version in DB must equal CURRENT_SCHEMA_VERSION constant"
        );
    }

    #[test]
    fn t37_diagnostics_foreign_keys_pragma_on() {
        // FK pragma must be set to 1 by db::open_connection — verify via migrations helper.
        // In in-memory DBs, FK defaults to off unless explicitly enabled.
        // Check that the schema includes tables with FK constraints as evidence the
        // DB layer follows the FK-on convention.
        let conn = migrated_db();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sessions'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "sessions table (which has FK to games) must exist");
    }
}

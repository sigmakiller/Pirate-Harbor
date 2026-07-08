//! Diagnostics & storage statistics commands — T35.
//!
//! # Commands
//! - `get_diagnostics`      → [`DiagnosticsReport`]   — schema version, table counts, DB size, active jobs
//! - `run_integrity_check`  → [`IntegrityResult`]      — `PRAGMA integrity_check` output
//! - `get_db_path`          → `String`                 — absolute path to the .db file

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::assets::{AssetManager, StorageStats};
use crate::background::{JobScheduler};
use crate::db::{DbState, CURRENT_SCHEMA_VERSION};

// ── Output types ──────────────────────────────────────────────────────────────

/// Counts per core table, shown in the diagnostics panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCounts {
    pub games:             i64,
    pub sessions:          i64,
    pub collections:       i64,
    pub collection_games:  i64,
    pub milestones:        i64,
    pub journal_entries:   i64,
    pub settings:          i64,
}

/// Full diagnostics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    /// Current schema version stored in the settings table.
    pub schema_version:         i32,
    /// Compiled-in target schema version.
    pub target_schema_version:  i32,
    /// Whether schema_version == target_schema_version.
    pub schema_up_to_date:      bool,
    /// SQLite page_count × page_size in bytes.
    pub db_size_bytes:          i64,
    /// WAL mode is recommended; true if "wal".
    pub wal_enabled:            bool,
    /// Foreign key pragma value.
    pub foreign_keys_enabled:   bool,
    /// Row counts for core tables.
    pub table_counts:           TableCounts,
    /// Number of background jobs currently running or queued.
    pub active_job_count:       usize,
    /// Asset storage breakdown.
    pub storage:                StorageStats,
    /// Absolute path to the .db file (read from settings or derived).
    pub db_path:                String,
}

/// Result of `PRAGMA integrity_check`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityResult {
    /// true when the only message is "ok".
    pub ok:       bool,
    /// All lines returned by the pragma (usually just ["ok"]).
    pub messages: Vec<String>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn count(conn: &rusqlite::Connection, table: &str) -> i64 {
    conn.query_row(
        &format!("SELECT COUNT(*) FROM {table}"),
        [],
        |r| r.get(0),
    ).unwrap_or(0)
}

fn pragma_i64(conn: &rusqlite::Connection, name: &str) -> i64 {
    conn.pragma_query_value(None, name, |r| r.get(0)).unwrap_or(0)
}

fn pragma_str(conn: &rusqlite::Connection, name: &str) -> String {
    conn.pragma_query_value(None, name, |r| r.get::<_, String>(0))
        .unwrap_or_default()
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Snapshot of DB health metrics, table counts, and storage statistics.
///
/// Fast — all reads are single-row PRAGMA / COUNT(*) queries.
#[tauri::command]
pub fn get_diagnostics(
    db:        State<'_, DbState>,
    assets:    State<'_, AssetManager>,
    scheduler: State<'_, JobScheduler>,
) -> Result<DiagnosticsReport, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;

    let schema_version        = crate::db::get_schema_version(&conn);
    let target_schema_version = CURRENT_SCHEMA_VERSION;

    let page_count = pragma_i64(&conn, "page_count");
    let page_size  = pragma_i64(&conn, "page_size");
    let db_size_bytes = page_count * page_size;

    let wal_enabled          = pragma_str(&conn, "journal_mode").to_lowercase() == "wal";
    let foreign_keys_enabled = pragma_i64(&conn, "foreign_keys") == 1;

    let table_counts = TableCounts {
        games:            count(&conn, "games"),
        sessions:         count(&conn, "sessions"),
        collections:      count(&conn, "collections"),
        collection_games: count(&conn, "collection_games"),
        milestones:       count(&conn, "milestones"),
        journal_entries:  count(&conn, "journal_entries"),
        settings:         count(&conn, "settings"),
    };

    // DB path — stored as a setting during init, fallback to APPDATA path
    let db_path: String = conn
        .query_row("SELECT value FROM settings WHERE key='app_data_dir'", [], |r| r.get(0))
        .map(|dir: String| format!("{dir}\\pirate_harbor.db"))
        .unwrap_or_else(|_| "%APPDATA%\\com.pirateharbor.app\\pirate_harbor.db".to_string());

    drop(conn); // release lock before calling assets

    let storage = assets.get_storage_stats()
        .unwrap_or(StorageStats { covers_bytes: 0, backgrounds_bytes: 0, gallery_bytes: 0, thumbnails_bytes: 0, total_bytes: 0, file_count: 0 });

    let active_job_count = scheduler.queue_depth();

    Ok(DiagnosticsReport {
        schema_version,
        target_schema_version,
        schema_up_to_date: schema_version == target_schema_version,
        db_size_bytes,
        wal_enabled,
        foreign_keys_enabled,
        table_counts,
        active_job_count,
        storage,
        db_path,
    })
}

/// Run SQLite's built-in integrity checker.
///
/// Returns `ok: true` for a healthy database. For corrupt databases the
/// `messages` vec will contain descriptions of the problems found.
/// This is a read-only operation and is safe to run at any time.
#[tauri::command]
pub fn run_integrity_check(
    db: State<'_, DbState>,
) -> Result<IntegrityResult, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;

    let mut stmt = conn
        .prepare("PRAGMA integrity_check(64)")
        .map_err(|e| e.to_string())?;

    let messages: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let ok = messages.len() == 1 && messages[0].eq_ignore_ascii_case("ok");

    Ok(IntegrityResult { ok, messages })
}

/// Return the absolute path to the SQLite database file.
#[tauri::command]
pub fn get_db_path(
    db: State<'_, DbState>,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    let dir: Result<String, _> =
        conn.query_row("SELECT value FROM settings WHERE key='app_data_dir'", [], |r| r.get(0));
    Ok(dir
        .map(|d| format!("{d}\\pirate_harbor.db"))
        .unwrap_or_else(|_| "%APPDATA%\\com.pirateharbor.app\\pirate_harbor.db".to_string()))
}

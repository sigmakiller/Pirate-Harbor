//! Watched-folder scanner commands — T10.
//!
//! The scanner walks registered directories looking for `.exe` files,
//! cross-references them against the library, and returns candidates
//! the user can choose to import.
//!
//! Scan directories are stored as a JSON-encoded `Vec<String>` in the
//! `settings` table under the key `"scan_directories"`.

use std::collections::HashSet;
use std::path::Path;

use tauri::State;
use walkdir::WalkDir;

use crate::db::DbState;
use crate::models::ScanResult;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum directory depth to recurse. Games are rarely nested > 4 levels.
const MAX_DEPTH: usize = 4;

/// Well-known utility executables that are never game launchers.
const BLOCKLIST: &[&str] = &[
    "unitycrashandler64",
    "unitycrashandler",
    "crashhandler",
    "crashreporter",
    "unins000",
    "uninstall",
    "setup",
    "install",
    "dxsetup",
    "vcredist_x64",
    "vcredist_x86",
    "ue4prereqsetup_x64",
    "dotnetfx",
    "directx",
    "redist",
    "easyanticheat_setup",
    "battleye_setup",
    "registrator",
    "activation",
    "launcher_setup",
    "update",
    "updater",
    "patcher",
];

// ── Settings helpers ──────────────────────────────────────────────────────────

fn load_directories(conn: &rusqlite::Connection) -> Vec<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'scan_directories'",
        [],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
    .unwrap_or_default()
}

fn save_directories(conn: &rusqlite::Connection, dirs: &[String]) -> Result<(), String> {
    let json = serde_json::to_string(dirs).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('scan_directories', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![json],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Load all exe_paths already registered in the library.
///
/// Extracted as a standalone function so that the `conn` and `stmt` borrows
/// are fully contained inside it — avoids E0597 lifetime errors when using `?`
/// inside blocks that need to return owned values.
fn load_known_paths(db_state: &State<'_, DbState>) -> Result<HashSet<String>, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT exe_path FROM games")
        .map_err(|e| e.to_string())?;
    let paths: HashSet<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(paths)
}

// ── Directory management commands ─────────────────────────────────────────────

/// Return all registered scan directories.
#[tauri::command]
pub fn get_scan_directories(db_state: State<'_, DbState>) -> Result<Vec<String>, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    Ok(load_directories(&conn))
}

/// Add a new directory to the watch list (idempotent).
#[tauri::command]
pub fn add_scan_directory(
    db_state: State<'_, DbState>,
    path: String,
) -> Result<Vec<String>, String> {
    if !Path::new(&path).is_dir() {
        return Err(format!("'{}' is not a valid directory.", path));
    }
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let mut dirs = load_directories(&conn);
    if !dirs.iter().any(|d| d == &path) {
        dirs.push(path);
        save_directories(&conn, &dirs)?;
    }
    Ok(dirs)
}

/// Remove a directory from the watch list.
#[tauri::command]
pub fn remove_scan_directory(
    db_state: State<'_, DbState>,
    path: String,
) -> Result<Vec<String>, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let mut dirs = load_directories(&conn);
    dirs.retain(|d| d != &path);
    save_directories(&conn, &dirs)?;
    Ok(dirs)
}

// ── Core scanner ──────────────────────────────────────────────────────────────

/// Walk a single root directory and return candidate executables.
fn do_scan(root: &str, known_paths: &HashSet<String>) -> Vec<ScanResult> {
    let mut results: Vec<ScanResult> = WalkDir::new(root)
        .max_depth(MAX_DEPTH)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("exe"))
                    .unwrap_or(false)
        })
        .filter_map(|e| {
            let path = e.path();
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            let stem_lower = stem.to_lowercase();
            if BLOCKLIST.iter().any(|b| stem_lower.contains(b)) {
                return None;
            }

            let exe_path = path.to_string_lossy().into_owned();
            let already_added = known_paths.contains(&exe_path);
            Some(ScanResult { name: stem, exe_path, already_added })
        })
        .collect();

    results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    results.dedup_by(|a, b| a.exe_path == b.exe_path);
    results
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Scan a single directory and return discovered executables.
#[tauri::command]
pub fn scan_directory(
    db_state: State<'_, DbState>,
    path: String,
) -> Result<Vec<ScanResult>, String> {
    if !Path::new(&path).is_dir() {
        return Err(format!("'{}' is not a valid directory.", path));
    }
    let known_paths = load_known_paths(&db_state)?;
    Ok(do_scan(&path, &known_paths))
}

/// Scan ALL registered directories and return a merged, deduplicated result set.
#[tauri::command]
pub fn scan_all_directories(
    db_state: State<'_, DbState>,
) -> Result<Vec<ScanResult>, String> {
    let dirs = {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;
        load_directories(&conn)
        // conn drops here — no borrow crosses the block boundary
    };

    if dirs.is_empty() {
        return Ok(vec![]);
    }

    let known_paths = load_known_paths(&db_state)?;

    let mut all: Vec<ScanResult> = dirs
        .iter()
        .filter(|d| Path::new(d.as_str()).is_dir())
        .flat_map(|d| do_scan(d, &known_paths))
        .collect();

    let mut seen = HashSet::new();
    all.retain(|r| seen.insert(r.exe_path.clone()));
    all.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(all)
}

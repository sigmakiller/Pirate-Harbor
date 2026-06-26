//! Watched-folder scanner commands — T10 (M3 updated: confidence scoring).
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

/// Minimum executable size in bytes (20 MB). Files smaller are skipped.
const MIN_EXE_SIZE: u64 = 20 * 1024 * 1024;

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

// ── Confidence scoring ────────────────────────────────────────────────────────

/// Compute a heuristic confidence score (0.0–1.0) for a candidate executable.
///
/// Factors:
///   +0.3 — exe is inside a named subfolder (not the scan root itself)
///   +0.2 — folder contains typical game file extensions (.dll, .pak, etc.)
///   +0.2 — exe stem matches the parent folder name (e.g. witcher3/witcher3.exe)
///   +0.2 — exe lives inside a /bin/ or /binaries/ subdirectory
///   +0.1 — file size > 50 MB
fn compute_confidence(path: &Path, exe_stem: &str) -> f64 {
    let mut score: f64 = 0.0;

    // +0.3 if exe is inside a named subfolder (has a parent with a name)
    if path.parent().and_then(|p| p.file_name()).is_some() {
        score += 0.3;
    }

    // +0.2 if folder contains typical game file extensions
    if let Some(parent) = path.parent() {
        if let Ok(entries) = std::fs::read_dir(parent) {
            let has_game_files = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.to_lowercase())
                })
                .any(|ext| matches!(ext.as_str(), "dll" | "pak" | "uasset" | "unity3d" | "pck"));
            if has_game_files {
                score += 0.2;
            }
        }
    }

    // +0.2 if exe stem matches parent folder name (case-insensitive)
    if let Some(folder_name) = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|f| f.to_str())
    {
        if exe_stem.to_lowercase() == folder_name.to_lowercase() {
            score += 0.2;
        }
    }

    // +0.2 if exe is inside a known game binary directory
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains("\\bin\\")
        || path_str.contains("\\binaries\\")
        || path_str.contains("/bin/")
        || path_str.contains("/binaries/")
    {
        score += 0.2;
    }

    // +0.1 if file size > 50 MB
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > 50 * 1024 * 1024 {
            score += 0.1;
        }
    }

    score.min(1.0)
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

/// Walk a single root directory and return candidate executables, sorted by
/// confidence descending (highest first).
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

            // Blocklist filter — use exact stem match to avoid "setuptown" false positives
            let stem_lower = stem.to_lowercase();
            if BLOCKLIST.iter().any(|b| stem_lower == *b) {
                return None;
            }

            // Size filter — skip exes under 20 MB
            let metadata = std::fs::metadata(path).ok()?;
            let size_bytes = metadata.len();
            if size_bytes < MIN_EXE_SIZE {
                return None;
            }

            let size_mb      = size_bytes as f64 / (1024.0 * 1024.0);
            let confidence   = compute_confidence(path, &stem);
            let folder_name  = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|f| f.to_str())
                .unwrap_or("")
                .to_string();
            let exe_path     = path.to_string_lossy().into_owned();
            let already_added = known_paths.contains(&exe_path);

            Some(ScanResult {
                name: stem,
                exe_path,
                already_added,
                confidence,
                size_mb,
                folder_name,
            })
        })
        .collect();

    // Sort by confidence descending (highest probability first)
    results.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
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

    // Global dedup by exe_path, then re-sort by confidence
    let mut seen = HashSet::new();
    all.retain(|r| seen.insert(r.exe_path.clone()));
    all.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(all)
}

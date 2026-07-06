//! Local backup and restore — T33.
//! Format: .phb = ZIP (manifest.json + database.json + settings.json + images/)
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use rusqlite::Connection;
use crate::background::{Job, JobContext, JobResult, JobScheduler};
use crate::db::DbState;
use crate::commands::export::query_as_json;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub path: String,
    pub size_bytes: u64,
    pub game_count: i64,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub games_restored: i64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub path: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub game_count: i64,
    pub is_auto: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupQueued {
    pub job_id: String,
    pub output_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BackupManifest {
    pub schema_version: u32,
    pub created_at: String,
    pub app_version: String,
    pub game_count: i64,
    pub is_auto: bool,
}

// ─── Core: create ─────────────────────────────────────────────────────────────

pub fn create_backup_file(
    conn: &Connection,
    output_path: &Path,
    app_data_dir: &Path,
    is_auto: bool,
) -> Result<BackupResult, String> {
    let t0 = std::time::Instant::now();

    let game_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
        .unwrap_or(0);

    let file = std::fs::File::create(output_path)
        .map_err(|e| format!("Cannot create backup file: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // manifest.json
    let manifest = BackupManifest {
        schema_version: 1,
        created_at: Utc::now().to_rfc3339(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        game_count,
        is_auto,
    };
    zip.start_file("manifest.json", opts).map_err(|e| e.to_string())?;
    zip.write_all(serde_json::to_string_pretty(&manifest).unwrap().as_bytes())
        .map_err(|e| e.to_string())?;

    // database.json — all core tables as JSON
    let tables: &[(&str, &str)] = &[
        ("games",             "SELECT * FROM games"),
        ("sessions",          "SELECT * FROM sessions"),
        ("collections",       "SELECT * FROM collections"),
        ("collection_games",  "SELECT * FROM collection_games"),
        ("milestones",        "SELECT * FROM milestones"),
        ("journal_entries",   "SELECT * FROM journal_entries"),
    ];
    let mut db_doc = serde_json::Map::new();
    for (table, sql) in tables {
        let rows = query_as_json(conn, sql, &[]).unwrap_or_default();
        db_doc.insert(table.to_string(), serde_json::Value::Array(rows));
    }
    zip.start_file("database.json", opts).map_err(|e| e.to_string())?;
    zip.write_all(serde_json::to_string_pretty(&db_doc).unwrap().as_bytes())
        .map_err(|e| e.to_string())?;

    // settings.json
    let settings = query_as_json(conn, "SELECT key, value FROM settings", &[])
        .unwrap_or_default();
    zip.start_file("settings.json", opts).map_err(|e| e.to_string())?;
    zip.write_all(serde_json::to_string_pretty(&settings).unwrap().as_bytes())
        .map_err(|e| e.to_string())?;

    // images/ — copy assets dir recursively
    let assets_dir = app_data_dir.join("assets");
    if assets_dir.exists() {
        for entry in walkdir::WalkDir::new(&assets_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let rel = entry.path().strip_prefix(app_data_dir).unwrap();
            let zip_path = format!("images/{}", rel.to_string_lossy().replace('\\', "/"));
            zip.start_file(&zip_path, opts).map_err(|e| e.to_string())?;
            let mut f = std::fs::File::open(entry.path()).map_err(|e| e.to_string())?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            zip.write_all(&buf).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;

    let size_bytes = std::fs::metadata(output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(BackupResult {
        path: output_path.to_string_lossy().to_string(),
        size_bytes,
        game_count,
        duration_ms: t0.elapsed().as_millis(),
    })
}

// ─── Core: restore ────────────────────────────────────────────────────────────

pub fn restore_backup_file(
    conn: &mut Connection,
    backup_path: &Path,
    app_data_dir: &Path,
) -> Result<RestoreResult, String> {
    let mut warnings = Vec::new();

    let file = std::fs::File::open(backup_path)
        .map_err(|e| format!("Cannot open backup: {e}"))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Invalid .phb archive: {e}"))?;

    // Read database.json
    let db_json: serde_json::Value = {
        let mut f = archive.by_name("database.json")
            .map_err(|_| "Backup missing database.json".to_string())?;
        let mut buf = String::new();
        f.read_to_string(&mut buf).map_err(|e| e.to_string())?;
        serde_json::from_str(&buf).map_err(|e| format!("database.json parse error: {e}"))?
    };

    // Read settings.json
    let settings_json: serde_json::Value = {
        if let Ok(mut f) = archive.by_name("settings.json") {
            let mut buf = String::new();
            f.read_to_string(&mut buf).map_err(|e| e.to_string())?;
            serde_json::from_str(&buf).unwrap_or(serde_json::Value::Array(vec![]))
        } else {
            warnings.push("settings.json missing from backup".to_string());
            serde_json::Value::Array(vec![])
        }
    };

    // Restore inside a transaction
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Disable FK for bulk insert
    tx.execute_batch("PRAGMA foreign_keys = OFF;").ok();

    // Clear existing data (order matters for FK)
    for tbl in &["collection_games","journal_entries","milestones","sessions","collections","games"] {
        tx.execute(&format!("DELETE FROM {tbl}"), []).ok();
    }

    let mut games_restored: i64 = 0;

    let restore_table = |tx: &rusqlite::Transaction,
                         name: &str,
                         rows: &[serde_json::Value],
                         warnings: &mut Vec<String>| {
        for row in rows {
            let obj = match row.as_object() {
                Some(o) => o,
                None => continue,
            };
            let cols: Vec<String> = obj.keys().cloned().collect();
            let placeholders: Vec<String> =
                (1..=cols.len()).map(|i| format!("?{i}")).collect();
            let sql = format!(
                "INSERT OR IGNORE INTO {} ({}) VALUES ({})",
                name,
                cols.join(", "),
                placeholders.join(", ")
            );
            let vals: Vec<rusqlite::types::Value> = cols.iter().map(|k| {
                match &obj[k] {
                    serde_json::Value::Null    => rusqlite::types::Value::Null,
                    serde_json::Value::Bool(b) => rusqlite::types::Value::Integer(*b as i64),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() { rusqlite::types::Value::Integer(i) }
                        else { rusqlite::types::Value::Real(n.as_f64().unwrap_or(0.0)) }
                    }
                    serde_json::Value::String(s) => rusqlite::types::Value::Text(s.clone()),
                    other => rusqlite::types::Value::Text(other.to_string()),
                }
            }).collect();
            if let Err(e) = tx.execute(&sql, rusqlite::params_from_iter(vals.iter())) {
                warnings.push(format!("Row skip ({name}): {e}"));
            }
        }
    };

    let table_order = ["games","sessions","collections","collection_games","milestones","journal_entries"];
    for tbl in &table_order {
        if let Some(rows) = db_json.get(tbl).and_then(|v| v.as_array()) {
            restore_table(&tx, tbl, rows, &mut warnings);
            if *tbl == "games" { games_restored = rows.len() as i64; }
        }
    }

    // Restore settings
    if let Some(rows) = settings_json.as_array() {
        for row in rows {
            if let (Some(k), Some(v)) = (
                row.get("key").and_then(|v| v.as_str()),
                row.get("value").and_then(|v| v.as_str()),
            ) {
                tx.execute(
                    "INSERT INTO settings (key, value) VALUES (?1, ?2)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    rusqlite::params![k, v],
                ).ok();
            }
        }
    }

    tx.execute_batch("PRAGMA foreign_keys = ON;").ok();
    tx.commit().map_err(|e| format!("Transaction commit failed: {e}"))?;

    // Extract images/ back to app_data_dir
    let names: Vec<String> = archive.file_names().map(String::from).collect();
    for name in names.iter().filter(|n| n.starts_with("images/")) {
        let rel = name.strip_prefix("images/").unwrap_or(name);
        if rel.is_empty() { continue; }
        let dest = app_data_dir.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let mut zf = match archive.by_name(name) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let mut buf = Vec::new();
        if zf.read_to_end(&mut buf).is_ok() {
            std::fs::write(&dest, &buf).ok();
        }
    }

    Ok(RestoreResult { games_restored, warnings })
}

// ─── Auto-backup helpers ──────────────────────────────────────────────────────

const SETTING_AUTO_BACKUP_INTERVAL: &str = "auto_backup_interval";
const SETTING_LAST_AUTO_BACKUP: &str = "last_auto_backup_at";
const MAX_AUTO_BACKUPS: usize = 5;

fn get_setting_val(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [key],
        |r| r.get(0),
    ).ok()
}

fn set_setting_val(conn: &Connection, key: &str, value: &str) {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    ).ok();
}

/// Returns true if an auto-backup is due based on the stored interval setting.
pub fn is_auto_backup_due(conn: &Connection) -> bool {
    let interval = get_setting_val(conn, SETTING_AUTO_BACKUP_INTERVAL)
        .unwrap_or_else(|| "weekly".to_string());
    if interval == "never" { return false; }

    let last = match get_setting_val(conn, SETTING_LAST_AUTO_BACKUP) {
        None => return true,
        Some(s) => s,
    };
    let last_dt = match chrono::DateTime::parse_from_rfc3339(&last) {
        Ok(d) => d.with_timezone(&Utc),
        Err(_) => return true,
    };
    let now = Utc::now();
    let hours_since = (now - last_dt).num_hours();
    match interval.as_str() {
        "daily"   => hours_since >= 24,
        "weekly"  => hours_since >= 24 * 7,
        "monthly" => hours_since >= 24 * 30,
        _         => false,
    }
}

/// Prune auto-backups so at most MAX_AUTO_BACKUPS are kept.
fn prune_auto_backups(backup_dir: &Path) {
    let mut autos: Vec<PathBuf> = std::fs::read_dir(backup_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("auto_") && n.ends_with(".phb"))
                .unwrap_or(false)
        })
        .collect();
    autos.sort();
    while autos.len() > MAX_AUTO_BACKUPS {
        std::fs::remove_file(&autos[0]).ok();
        autos.remove(0);
    }
}

pub fn default_backup_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("backups")
}

// ─── List backups ─────────────────────────────────────────────────────────────

pub fn list_backups_in_dir(backup_dir: &Path) -> Vec<BackupInfo> {
    let mut infos: Vec<BackupInfo> = std::fs::read_dir(backup_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|x| x.to_str()) == Some("phb")
        })
        .filter_map(|e| {
            let path = e.path();
            let meta = std::fs::metadata(&path).ok()?;
            let name = path.file_name()?.to_string_lossy().to_string();
            let is_auto = name.starts_with("auto_");
            // Read created_at from manifest inside zip
            let file = std::fs::File::open(&path).ok()?;
            let mut archive = zip::ZipArchive::new(file).ok()?;
            let manifest: BackupManifest = {
                let mut f = archive.by_name("manifest.json").ok()?;
                let mut buf = String::new();
                f.read_to_string(&mut buf).ok()?;
                serde_json::from_str(&buf).ok()?
            };
            Some(BackupInfo {
                path: path.to_string_lossy().to_string(),
                created_at: manifest.created_at,
                size_bytes: meta.len(),
                game_count: manifest.game_count,
                is_auto,
            })
        })
        .collect();
    infos.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    infos
}

// ─── Background jobs ──────────────────────────────────────────────────────────

pub struct CreateBackupJob {
    pub output_path: PathBuf,
    pub app_data_dir: PathBuf,
    pub is_auto: bool,
}

impl Job for CreateBackupJob {
    fn name(&self) -> &str { "create_backup" }
    fn execute(&self, ctx: JobContext) -> Result<JobResult, String> {
        let conn = ctx.db.lock().map_err(|_| "DB lock poisoned".to_string())?;
        let res = create_backup_file(&conn, &self.output_path, &self.app_data_dir, self.is_auto)?;
        if self.is_auto {
            set_setting_val(&conn, SETTING_LAST_AUTO_BACKUP, &Utc::now().to_rfc3339());
            prune_auto_backups(&self.output_path.parent().unwrap_or(&self.app_data_dir));
        }
        drop(conn);
        Ok(JobResult::with_payload(
            format!("Backup created: {} ({} bytes)", res.path, res.size_bytes),
            serde_json::to_value(&res).unwrap_or_default(),
        ))
    }
}

pub struct AutoBackupJob {
    pub app_data_dir: PathBuf,
}

impl Job for AutoBackupJob {
    fn name(&self) -> &str { "auto_backup" }
    fn execute(&self, ctx: JobContext) -> Result<JobResult, String> {
        let conn = ctx.db.lock().map_err(|_| "DB lock poisoned".to_string())?;
        if !is_auto_backup_due(&conn) {
            return Ok(JobResult::ok("Auto-backup not due yet"));
        }
        let backup_dir = default_backup_dir(&self.app_data_dir);
        std::fs::create_dir_all(&backup_dir).ok();
        let stamp = Utc::now().format("%Y%m%d_%H%M%S");
        let path = backup_dir.join(format!("auto_{stamp}.phb"));
        let res = create_backup_file(&conn, &path, &self.app_data_dir, true)?;
        set_setting_val(&conn, SETTING_LAST_AUTO_BACKUP, &Utc::now().to_rfc3339());
        prune_auto_backups(&backup_dir);
        Ok(JobResult::with_payload(
            format!("Auto-backup created: {} bytes", res.size_bytes),
            serde_json::to_value(&res).unwrap_or_default(),
        ))
    }
}

// ─── Tauri commands ───────────────────────────────────────────────────────────

/// Create a backup at the given absolute path (async, returns job_id).
#[tauri::command]
pub fn create_backup(
    db: State<'_, DbState>,
    scheduler: State<'_, JobScheduler>,
    path: String,
) -> Result<BackupQueued, String> {
    let output_path = PathBuf::from(&path);
    if output_path.extension().and_then(|e| e.to_str()) != Some("phb") {
        return Err("Backup file must have .phb extension".to_string());
    }
    // Resolve app_data_dir from the backup path parent heuristic or fail fast with DB lock
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    let app_data_dir: PathBuf = conn
        .query_row("SELECT value FROM settings WHERE key='app_data_dir'", [], |r| r.get(0))
        .unwrap_or_else(|_| output_path.parent().unwrap_or(Path::new(".")).to_string_lossy().to_string())
        .into();
    drop(conn);

    let job = CreateBackupJob {
        output_path: output_path.clone(),
        app_data_dir,
        is_auto: false,
    };
    let job_id = scheduler.enqueue(job);
    Ok(BackupQueued { job_id, output_path: path })
}

/// Restore from a .phb backup file (synchronous — only call from settings page).
#[tauri::command]
pub fn restore_backup(
    db: State<'_, DbState>,
    path: String,
) -> Result<RestoreResult, String> {
    let backup_path = PathBuf::from(&path);
    if !backup_path.exists() {
        return Err(format!("Backup file not found: {path}"));
    }
    let conn_guard = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    let app_data_dir: PathBuf = conn_guard
        .query_row("SELECT value FROM settings WHERE key='app_data_dir'", [], |r| r.get(0))
        .unwrap_or_else(|_| backup_path.parent().unwrap_or(Path::new(".")).to_string_lossy().to_string())
        .into();
    drop(conn_guard);

    // Restore needs &mut Connection — unlock, take ownership temporarily via raw ptr approach
    // Instead we lock again and get a mutable ref through unsafe deref (safe: we hold the mutex)
    let mut conn_guard = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    restore_backup_file(&mut conn_guard, &backup_path, &app_data_dir)
}

/// List all .phb files in the default backup directory.
#[tauri::command]
pub fn list_auto_backups(
    db: State<'_, DbState>,
) -> Result<Vec<BackupInfo>, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    let app_data_dir: PathBuf = conn
        .query_row("SELECT value FROM settings WHERE key='app_data_dir'", [], |r| r.get(0))
        .unwrap_or_else(|_| "".to_string())
        .into();
    drop(conn);
    let backup_dir = default_backup_dir(&app_data_dir);
    if !backup_dir.exists() { return Ok(vec![]); }
    Ok(list_backups_in_dir(&backup_dir))
}

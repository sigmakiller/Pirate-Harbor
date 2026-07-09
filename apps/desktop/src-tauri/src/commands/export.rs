//! Data export commands — T32.
//!
//! Exports library data in two formats:
//! - **JSON** — machine-readable, full-fidelity dump (games + collections +
//!   milestones + journal entries + play sessions + settings)
//! - **Markdown** — human-readable profile report suitable for sharing
//!
//! Both exports run as background jobs (T27) for large libraries so the UI
//! stays responsive.  A lightweight `get_export_preview` command is provided
//! so the UI can show a size estimate before the user commits.
//!
//! # Commands
//! - `get_export_preview` → [`ExportPreview`] — fast (single query)
//! - `export_library_json(path)` → job_id (runs async)
//! - `export_profile_markdown(path)` → job_id (runs async)

use std::io::Write;
use std::path::PathBuf;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::background::{Job, JobContext, JobResult, JobScheduler};
use crate::db::DbState;

// ── Output types ──────────────────────────────────────────────────────────────

/// Lightweight summary shown in the Export UI before the user starts a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPreview {
    /// Total number of games in the library.
    pub game_count: i64,
    /// Total number of milestone records.
    pub milestone_count: i64,
    /// Total number of journal entries.
    pub journal_count: i64,
    /// Total number of play sessions.
    pub session_count: i64,
    /// Rough estimate of the uncompressed JSON export size in bytes.
    pub estimated_size_bytes: i64,
}

/// Returned immediately after queueing an export job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportQueued {
    /// Job ID — poll `get_job_status(job_id)` for progress.
    pub job_id: String,
    /// Absolute path that the file will be written to.
    pub output_path: String,
}

// ── JSON export payload ───────────────────────────────────────────────────────

/// Root of the JSON export document.
#[derive(Debug, Serialize, Deserialize)]
struct LibraryExport {
    /// Export schema version — increment if structure changes.
    pub schema_version: u32,
    /// RFC 3339 timestamp of when the export was created.
    pub exported_at: String,
    pub app_version: String,
    pub games: Vec<serde_json::Value>,
    pub collections: Vec<serde_json::Value>,
    pub collection_memberships: Vec<serde_json::Value>,
    pub milestones: Vec<serde_json::Value>,
    pub journal_entries: Vec<serde_json::Value>,
    pub sessions: Vec<serde_json::Value>,
    pub settings: Vec<serde_json::Value>,
}

// ── Internal query helpers ────────────────────────────────────────────────────

/// Run a SQL query and return each row as a JSON object.
///
/// Column names from the prepared statement become JSON keys.
/// SQLite types map as: TEXT → String, INTEGER → i64, REAL → f64, NULL → null.
pub fn query_as_json(
    conn: &Connection,
    sql: &str,
    params: &[&dyn rusqlite::types::ToSql],
) -> Result<Vec<serde_json::Value>, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let column_names: Vec<String> = stmt
        .column_names()
        .into_iter()
        .map(String::from)
        .collect();

    let rows = stmt
        .query_map(params, |row| {
            let mut obj = serde_json::Map::new();
            for (i, name) in column_names.iter().enumerate() {
                let val: serde_json::Value = match row.get_ref(i)? {
                    rusqlite::types::ValueRef::Null    => serde_json::Value::Null,
                    rusqlite::types::ValueRef::Integer(n) => serde_json::json!(n),
                    rusqlite::types::ValueRef::Real(f)    => serde_json::json!(f),
                    rusqlite::types::ValueRef::Text(b)    => {
                        serde_json::Value::String(String::from_utf8_lossy(b).into_owned())
                    }
                    rusqlite::types::ValueRef::Blob(_) => serde_json::Value::Null,
                };
                obj.insert(name.clone(), val);
            }
            Ok(serde_json::Value::Object(obj))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

// ── Preview ───────────────────────────────────────────────────────────────────

/// Build an `ExportPreview` from the live database.
///
/// Uses four `COUNT(*)` queries — fast even on large libraries.
pub fn build_export_preview(conn: &Connection) -> Result<ExportPreview, String> {
    let game_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
        .unwrap_or(0);
    let milestone_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM milestones", [], |r| r.get(0))
        .unwrap_or(0);
    let journal_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM journal_entries", [], |r| r.get(0))
        .unwrap_or(0);
    let session_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
        .unwrap_or(0);

    // Rough size estimate: ~2 KB per game + ~512 B each for others.
    let estimated_size_bytes = game_count * 2048
        + milestone_count * 512
        + journal_count   * 512
        + session_count   * 256;

    Ok(ExportPreview {
        game_count,
        milestone_count,
        journal_count,
        session_count,
        estimated_size_bytes,
    })
}

// ── JSON export engine ────────────────────────────────────────────────────────

/// Dump the entire library to a pretty-printed JSON file at `output_path`.
///
/// All data is exported — games, collections, memberships, milestones,
/// journal entries, sessions, and settings.  The cover paths exported are the
/// local filesystem paths, not URLs, so they remain valid on the same machine.
pub fn write_library_json(conn: &Connection, output_path: &std::path::Path) -> Result<String, String> {
    let exported_at = chrono::Utc::now().to_rfc3339();

    let games = query_as_json(
        conn,
        "SELECT * FROM games ORDER BY title ASC",
        &[],
    )?;
    let collections = query_as_json(
        conn,
        "SELECT * FROM collections ORDER BY name ASC",
        &[],
    )?;
    let collection_memberships = query_as_json(
        conn,
        "SELECT * FROM collection_games ORDER BY collection_id, game_id",
        &[],
    )?;
    let milestones = query_as_json(
        conn,
        "SELECT m.*, g.title AS game_title \
         FROM milestones m \
         LEFT JOIN games g ON g.id = m.game_id \
         ORDER BY m.achievement_date DESC",
        &[],
    )?;
    let journal_entries = query_as_json(
        conn,
        "SELECT je.*, g.title AS game_title \
         FROM journal_entries je \
         LEFT JOIN games g ON g.id = je.game_id \
         ORDER BY je.created_at DESC",
        &[],
    )?;
    let sessions = query_as_json(
        conn,
        "SELECT s.*, g.title AS game_title \
         FROM sessions s \
         LEFT JOIN games g ON g.id = s.game_id \
         ORDER BY s.started_at DESC",
        &[],
    )?;
    let settings = query_as_json(
        conn,
        "SELECT key, value FROM settings ORDER BY key ASC",
        &[],
    )?;

    let export = LibraryExport {
        schema_version: 1,
        exported_at,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        games,
        collections,
        collection_memberships,
        milestones,
        journal_entries,
        sessions,
        settings,
    };

    let json = serde_json::to_string_pretty(&export)
        .map_err(|e| format!("JSON serialization failed: {e}"))?;

    // Ensure parent directories exist.
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {e}"))?;
    }

    std::fs::write(output_path, json.as_bytes())
        .map_err(|e| format!("Failed to write export file: {e}"))?;

    let size_kb = json.len() / 1024;
    Ok(format!(
        "Exported {} games to '{}' ({size_kb} KB)",
        export.games.len(),
        output_path.display(),
    ))
}

// ── Markdown export engine ────────────────────────────────────────────────────

/// Write a human-readable profile report as a Markdown document.
///
/// Sections: Library Overview, Top Games by Playtime, Completed Games,
/// Milestones (grouped by game), Journal Highlights.
pub fn write_profile_markdown(conn: &Connection, output_path: &std::path::Path) -> Result<String, String> {
    let mut md = String::with_capacity(16 * 1024);

    let exported_at = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // ── Header ────────────────────────────────────────────────────────────────
    md.push_str(&format!("# Pirate Harbor — Gaming Profile\n\n"));
    md.push_str(&format!("*Generated on {}*\n\n---\n\n", exported_at));

    // ── Library overview ──────────────────────────────────────────────────────
    let total_games: i64 = conn.query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0)).unwrap_or(0);
    let completed:   i64 = conn.query_row("SELECT COUNT(*) FROM games WHERE status='completed'", [], |r| r.get(0)).unwrap_or(0);
    let playing:     i64 = conn.query_row("SELECT COUNT(*) FROM games WHERE status='playing'",   [], |r| r.get(0)).unwrap_or(0);
    let unplayed:    i64 = conn.query_row("SELECT COUNT(*) FROM games WHERE status='unplayed'",  [], |r| r.get(0)).unwrap_or(0);
    let total_secs:  i64 = conn.query_row("SELECT COALESCE(SUM(total_playtime_secs), 0) FROM games", [], |r| r.get(0)).unwrap_or(0);
    let milestones:  i64 = conn.query_row("SELECT COUNT(*) FROM milestones", [], |r| r.get(0)).unwrap_or(0);

    let total_hours = total_secs / 3600;
    let completion_pct = if total_games > 0 {
        completed * 100 / total_games
    } else {
        0
    };

    md.push_str("## 📊 Library Overview\n\n");
    md.push_str(&format!("| Stat | Value |\n|------|-------|\n"));
    md.push_str(&format!("| Total Games    | {} |\n", total_games));
    md.push_str(&format!("| Completed      | {} ({completion_pct}%) |\n", completed));
    md.push_str(&format!("| Currently Playing | {} |\n", playing));
    md.push_str(&format!("| Unplayed       | {} |\n", unplayed));
    md.push_str(&format!("| Total Playtime | {}h |\n", total_hours));
    md.push_str(&format!("| Milestones     | {} |\n\n", milestones));

    // ── Top games by playtime ─────────────────────────────────────────────────
    md.push_str("## 🏆 Top 10 Games by Playtime\n\n");
    md.push_str("| # | Game | Playtime | Status |\n|---|------|----------|--------|\n");

    let mut stmt = conn.prepare(
        "SELECT title, total_playtime_secs, status \
         FROM games \
         WHERE total_playtime_secs > 0 \
         ORDER BY total_playtime_secs DESC \
         LIMIT 10",
    ).map_err(|e| e.to_string())?;

    let top_games: Vec<(String, i64, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for (i, (title, secs, status)) in top_games.iter().enumerate() {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        md.push_str(&format!("| {} | {} | {}h {}m | {} |\n", i + 1, title, h, m, status));
    }
    md.push('\n');

    // ── Completed games ───────────────────────────────────────────────────────
    md.push_str("## ✅ Completed Games\n\n");
    let mut stmt = conn.prepare(
        "SELECT title, genre, developer, total_playtime_secs \
         FROM games \
         WHERE status='completed' \
         ORDER BY title ASC",
    ).map_err(|e| e.to_string())?;

    let completed_games: Vec<(String, Option<String>, Option<String>, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    if completed_games.is_empty() {
        md.push_str("*No completed games yet — keep playing!*\n\n");
    } else {
        md.push_str("| Game | Genre | Developer | Playtime |\n|------|-------|-----------|----------|\n");
        for (title, genre, dev, secs) in &completed_games {
            let h = secs / 3600;
            md.push_str(&format!(
                "| {} | {} | {} | {}h |\n",
                title,
                genre.as_deref().unwrap_or("—"),
                dev.as_deref().unwrap_or("—"),
                h,
            ));
        }
        md.push('\n');
    }

    // ── Milestones ────────────────────────────────────────────────────────────
    md.push_str("## 🏅 Milestones\n\n");
    let mut stmt = conn.prepare(
        "SELECT m.title, m.category, m.achievement_date, g.title AS game_title, m.points \
         FROM milestones m \
         LEFT JOIN games g ON g.id = m.game_id \
         ORDER BY m.achievement_date DESC \
         LIMIT 50",
    ).map_err(|e| e.to_string())?;

    let milestone_rows: Vec<(String, String, String, Option<String>, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    if milestone_rows.is_empty() {
        md.push_str("*No milestones recorded yet.*\n\n");
    } else {
        md.push_str("| Milestone | Game | Category | Date | Points |\n|-----------|------|----------|------|--------|\n");
        for (title, cat, date, game, pts) in &milestone_rows {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                title,
                game.as_deref().unwrap_or("—"),
                cat,
                &date[..10.min(date.len())],
                pts,
            ));
        }
        md.push('\n');
    }

    // ── Journal highlights (last 10 entries) ───────────────────────────────────
    md.push_str("## 📓 Recent Journal Entries\n\n");
    let mut stmt = conn.prepare(
        "SELECT je.title, je.body, je.entry_type, je.created_at, g.title AS game_title \
         FROM journal_entries je \
         LEFT JOIN games g ON g.id = je.game_id \
         ORDER BY je.created_at DESC \
         LIMIT 10",
    ).map_err(|e| e.to_string())?;

    let journal_rows: Vec<(Option<String>, String, String, String, Option<String>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    if journal_rows.is_empty() {
        md.push_str("*No journal entries yet.*\n\n");
    } else {
        for (title, body, entry_type, created_at, game_title) in &journal_rows {
            let display_title = title.as_deref().unwrap_or("Untitled");
            let date = &created_at[..10.min(created_at.len())];
            let game = game_title.as_deref().unwrap_or("General");
            // Truncate long bodies to 200 chars for readability.
            let body_preview = if body.len() > 200 {
                format!("{}…", &body[..200])
            } else {
                body.clone()
            };
            md.push_str(&format!(
                "### {} `[{}]` — *{}* ({})  \n{}\n\n",
                display_title, entry_type, game, date, body_preview
            ));
        }
    }

    md.push_str("---\n\n*Exported by Pirate Harbor — Your Gaming Library Manager*\n");

    // ── Write file ────────────────────────────────────────────────────────────
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {e}"))?;
    }

    let mut file = std::fs::File::create(output_path)
        .map_err(|e| format!("Failed to create markdown file: {e}"))?;
    file.write_all(md.as_bytes())
        .map_err(|e| format!("Failed to write markdown file: {e}"))?;

    Ok(format!(
        "Profile exported to '{}' ({} bytes)",
        output_path.display(),
        md.len()
    ))
}

// ── Background job wrappers ───────────────────────────────────────────────────

/// Background job that writes the full JSON library dump.
pub struct ExportLibraryJsonJob {
    pub output_path: PathBuf,
}

impl Job for ExportLibraryJsonJob {
    fn name(&self) -> &str { "export_library_json" }

    fn execute(&self, ctx: JobContext) -> Result<JobResult, String> {
        let conn = ctx.db.lock().map_err(|_| "DB lock poisoned".to_string())?;
        let summary = write_library_json(&conn, &self.output_path)?;
        Ok(JobResult::with_payload(
            summary,
            serde_json::json!({ "output_path": self.output_path.to_string_lossy() }),
        ))
    }
}

/// Background job that writes the Markdown profile report.
pub struct ExportProfileMarkdownJob {
    pub output_path: PathBuf,
}

impl Job for ExportProfileMarkdownJob {
    fn name(&self) -> &str { "export_profile_markdown" }

    fn execute(&self, ctx: JobContext) -> Result<JobResult, String> {
        let conn = ctx.db.lock().map_err(|_| "DB lock poisoned".to_string())?;
        let summary = write_profile_markdown(&conn, &self.output_path)?;
        Ok(JobResult::with_payload(
            summary,
            serde_json::json!({ "output_path": self.output_path.to_string_lossy() }),
        ))
    }
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Return a lightweight size estimate for the export dialog.
///
/// Fast — does four COUNT queries.  No file I/O.
#[tauri::command]
pub fn get_export_preview(
    db: State<'_, DbState>,
) -> Result<ExportPreview, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    build_export_preview(&conn)
}

/// Export the full library to a JSON file at the given absolute path.
///
/// Returns a [`ExportQueued`] immediately.  The actual file write happens in
/// the background; poll `get_job_status(job_id)` for completion.
///
/// `path` must be a valid absolute filesystem path including filename, e.g.:
/// `C:\Users\name\Desktop\pirate_harbor_export.json`
#[tauri::command]
pub fn export_library_json(
    db: State<'_, DbState>,
    scheduler: State<'_, JobScheduler>,
    path: String,
) -> Result<ExportQueued, String> {
    // Validate the path before queuing so we fail fast.
    let output_path = PathBuf::from(&path);
    let ext = output_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "json" {
        return Err("Export file must have a .json extension (e.g. export.json)".to_string());
    }

    // Quick preview so the job can embed the count in its result.
    {
        let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
        let _ = build_export_preview(&conn)?; // validates DB is accessible
    }

    let job = ExportLibraryJsonJob { output_path: output_path.clone() };
    let job_id = scheduler.enqueue(job);

    Ok(ExportQueued {
        job_id,
        output_path: path,
    })
}

/// Export a human-readable profile report to a Markdown file.
///
/// Returns a [`ExportQueued`] immediately; the write is async.
///
/// `path` example: `C:\Users\name\Desktop\my_profile.md`
#[tauri::command]
pub fn export_profile_markdown(
    db: State<'_, DbState>,
    scheduler: State<'_, JobScheduler>,
    path: String,
) -> Result<ExportQueued, String> {
    let output_path = PathBuf::from(&path);
    let ext = output_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "md" {
        return Err("Profile export file must have a .md extension (e.g. profile.md)".to_string());
    }

    {
        let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
        let _ = build_export_preview(&conn)?;
    }

    let job = ExportProfileMarkdownJob { output_path: output_path.clone() };
    let job_id = scheduler.enqueue(job);

    Ok(ExportQueued {
        job_id,
        output_path: path,
    })
}

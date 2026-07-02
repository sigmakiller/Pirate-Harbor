//! Full-text search commands — T29.
//!
//! Wraps the FTS5 virtual tables (`games_fts`, `journal_fts`) added in
//! Migration 007.  All queries use prefix-search (`term*`) so partial words
//! like "witc" match "Witcher".  Special FTS5 characters in user input are
//! escaped before building the MATCH expression.
//!
//! # Commands
//! - `search_global`   — unified search across games, journal entries, milestones
//! - `rebuild_search_index` — drops and re-populates the FTS tables (background-safe)

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;

// ── Result types ──────────────────────────────────────────────────────────────

/// A single game hit from the FTS index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSearchHit {
    pub id:         String,
    pub title:      String,
    pub developer:  Option<String>,
    pub genre:      Option<String>,
    pub status:     String,
    pub cover_path: Option<String>,
}

/// A single journal entry hit from the FTS index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalSearchHit {
    pub id:         String,
    pub title:      Option<String>,
    pub body:       String,
    pub entry_type: String,
    pub game_id:    Option<String>,
    pub game_title: Option<String>,
    pub created_at: String,
}

/// A single milestone hit (plain LIKE search).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneSearchHit {
    pub id:         String,
    pub title:      String,
    pub game_id:    String,
    pub game_title: Option<String>,
    pub category:   String,
}

/// Aggregated result set returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub games:           Vec<GameSearchHit>,
    pub journal_entries: Vec<JournalSearchHit>,
    pub milestones:      Vec<MilestoneSearchHit>,
    /// Total hits across all categories.
    pub total:           usize,
}

/// Result of an index rebuild operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebuildResult {
    pub games_indexed:   usize,
    pub journal_indexed: usize,
}

// ── FTS5 input sanitisation ───────────────────────────────────────────────────

/// Escape user input for use in an FTS5 `MATCH` expression.
///
/// Wraps the query in double-quotes (phrase mode) and appends `*` for prefix
/// matching.  Embedded double-quotes are doubled per the FTS5 spec.
/// Returns `None` for empty / whitespace-only queries.
fn fts_escape(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let escaped = trimmed.replace('"', "\"\"");
    Some(format!("\"{}\"*", escaped))
}

// ── Private query helpers ─────────────────────────────────────────────────────
//
// Each helper owns its `Statement` for the duration of the query, avoiding the
// E0597 lifetime error that arises when stmt is dropped before the iterator
// is fully consumed.

fn query_games(
    conn: &Connection,
    fts_query: &str,
    cap: i64,
) -> Result<Vec<GameSearchHit>, String> {
    const SQL: &str = r#"
        SELECT g.id, g.title, g.developer, g.genre, g.status, g.cover_path_local
        FROM games_fts
        JOIN games g ON games_fts.rowid = g.rowid
        WHERE games_fts MATCH ?1
        ORDER BY rank
        LIMIT ?2
    "#;
    let mut stmt = conn.prepare(SQL).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![fts_query, cap], |row| {
            Ok(GameSearchHit {
                id:         row.get(0)?,
                title:      row.get(1)?,
                developer:  row.get(2)?,
                genre:      row.get(3)?,
                status:     row.get::<_, String>(4).unwrap_or_default(),
                cover_path: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

fn query_journal(
    conn: &Connection,
    fts_query: &str,
    cap: i64,
) -> Result<Vec<JournalSearchHit>, String> {
    const SQL: &str = r#"
        SELECT je.id, je.title, je.body, je.entry_type,
               je.game_id, g.title AS game_title, je.created_at
        FROM journal_fts
        JOIN journal_entries je ON journal_fts.rowid = je.rowid
        LEFT JOIN games g ON je.game_id = g.id
        WHERE journal_fts MATCH ?1
        ORDER BY rank
        LIMIT ?2
    "#;
    let mut stmt = conn.prepare(SQL).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![fts_query, cap], |row| {
            Ok(JournalSearchHit {
                id:         row.get(0)?,
                title:      row.get(1)?,
                body:       row.get::<_, String>(2).unwrap_or_default(),
                entry_type: row.get::<_, String>(3).unwrap_or_default(),
                game_id:    row.get(4)?,
                game_title: row.get(5)?,
                created_at: row.get::<_, String>(6).unwrap_or_default(),
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

fn query_milestones(
    conn: &Connection,
    pattern: &str,
    cap: i64,
) -> Result<Vec<MilestoneSearchHit>, String> {
    const SQL: &str = r#"
        SELECT m.id, m.title, m.game_id, g.title AS game_title, m.category
        FROM milestones m
        LEFT JOIN games g ON m.game_id = g.id
        WHERE m.title LIKE ?1
        ORDER BY m.created_at DESC
        LIMIT ?2
    "#;
    let mut stmt = conn.prepare(SQL).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![pattern, cap], |row| {
            Ok(MilestoneSearchHit {
                id:         row.get(0)?,
                title:      row.get::<_, String>(1).unwrap_or_default(),
                game_id:    row.get::<_, String>(2).unwrap_or_default(),
                game_title: row.get(3)?,
                category:   row.get::<_, String>(4).unwrap_or_default(),
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Global search across games, journal entries, and milestones.
///
/// Returns up to `limit` results per category (default 20, max 100).
/// Game and journal results are ordered by FTS5 bm25 relevance rank.
#[tauri::command]
pub fn search_global(
    db: State<'_, DbState>,
    query: String,
    limit: Option<usize>,
) -> Result<SearchResults, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    let cap = limit.unwrap_or(20).min(100) as i64;

    let fts_query = match fts_escape(&query) {
        Some(q) => q,
        None => {
            return Ok(SearchResults {
                games: vec![],
                journal_entries: vec![],
                milestones: vec![],
                total: 0,
            });
        }
    };

    let games           = query_games(&conn, &fts_query, cap)?;
    let journal_entries = query_journal(&conn, &fts_query, cap)?;
    let milestones      = query_milestones(&conn, &format!("%{}%", query.trim()), cap)?;
    let total           = games.len() + journal_entries.len() + milestones.len();

    Ok(SearchResults { games, journal_entries, milestones, total })
}

/// Rebuild the FTS5 indexes from scratch.
///
/// Uses the FTS5 built-in `'rebuild'` command which re-indexes all rows from
/// the content tables (`games` and `journal_entries`).  Safe to call at any
/// time; the operation is synchronous — callers may wrap it in a background job
/// for large libraries.
#[tauri::command]
pub fn rebuild_search_index(db: State<'_, DbState>) -> Result<RebuildResult, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;

    conn.execute_batch("INSERT INTO games_fts(games_fts) VALUES('rebuild');")
        .map_err(|e| format!("Failed to rebuild games_fts: {e}"))?;

    conn.execute_batch("INSERT INTO journal_fts(journal_fts) VALUES('rebuild');")
        .map_err(|e| format!("Failed to rebuild journal_fts: {e}"))?;

    let games_indexed = conn
        .query_row("SELECT COUNT(*) FROM games", [], |row| row.get::<_, i64>(0))
        .map(|n| n as usize)
        .unwrap_or(0);

    let journal_indexed = conn
        .query_row("SELECT COUNT(*) FROM journal_entries", [], |row| row.get::<_, i64>(0))
        .map(|n| n as usize)
        .unwrap_or(0);

    Ok(RebuildResult { games_indexed, journal_indexed })
}

//! Journal commands — Phase 2.
//!
//! Chronological log of play sessions, notes, and milestones.
//! Entries can be linked to a specific game or be library-wide.

use std::str::FromStr;

use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::db::DbState;
use crate::models::{EntryType, JournalEntry, NewJournalEntry, UpdateJournalEntry};

// ── Row mapper ────────────────────────────────────────────────────────────────

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JournalEntry> {
    let entry_type_str: String = row.get(5)?;
    let entry_type = EntryType::from_str(&entry_type_str).unwrap_or_default();

    Ok(JournalEntry {
        id:         row.get(0)?,
        game_id:    row.get(1)?,
        game_title: row.get(2)?,
        title:      row.get(3)?,
        body:       row.get(4)?,
        entry_type,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Internal helper — loads entries for a specific game_id (avoids lifetime issues).
fn load_entries_by_game(
    conn: &rusqlite::Connection,
    game_id: &str,
    max: i64,
) -> Result<Vec<JournalEntry>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, game_id, game_title, title, body, entry_type, created_at, updated_at
             FROM journal_entries WHERE game_id = ?1
             ORDER BY created_at DESC LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let result = stmt
        .query_map(rusqlite::params![game_id, max], map_row)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(result)
}

/// Internal helper — loads all entries (avoids lifetime issues).
fn load_all_entries(
    conn: &rusqlite::Connection,
    max: i64,
) -> Result<Vec<JournalEntry>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, game_id, game_title, title, body, entry_type, created_at, updated_at
             FROM journal_entries
             ORDER BY created_at DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let result = stmt
        .query_map(rusqlite::params![max], map_row)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(result)
}

/// Return journal entries, newest first.
///
/// Optionally filter by `game_id`. Returns at most `limit` entries (default 100).
#[tauri::command]
pub fn get_journal_entries(
    db_state: State<'_, DbState>,
    game_id:  Option<String>,
    limit:    Option<i64>,
) -> Result<Vec<JournalEntry>, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let max  = limit.unwrap_or(100);

    if let Some(ref gid) = game_id {
        load_entries_by_game(&conn, gid, max)
    } else {
        load_all_entries(&conn, max)
    }
}

/// Create a new journal entry.
#[tauri::command]
pub fn create_journal_entry(
    db_state: State<'_, DbState>,
    payload:  NewJournalEntry,
) -> Result<JournalEntry, String> {
    if payload.body.trim().is_empty() && payload.title.as_deref().unwrap_or("").trim().is_empty() {
        return Err("Entry must have a body or title.".to_string());
    }

    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let id   = Uuid::new_v4().to_string();
    let now  = Utc::now().to_rfc3339();
    let etype = payload.entry_type.unwrap_or_default();

    // Resolve game title from the games table if a game_id is provided
    let game_title: Option<String> = if let Some(ref gid) = payload.game_id {
        conn.query_row(
            "SELECT title FROM games WHERE id = ?1",
            rusqlite::params![gid],
            |row| row.get(0),
        )
        .ok()
    } else {
        None
    };

    conn.execute(
        "INSERT INTO journal_entries
         (id, game_id, game_title, title, body, entry_type, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        rusqlite::params![
            id,
            payload.game_id,
            game_title.as_deref(),
            payload.title.as_deref(),
            payload.body.trim(),
            etype.as_str(),
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(JournalEntry {
        id,
        game_id:    payload.game_id,
        game_title,
        title:      payload.title,
        body:       payload.body.trim().to_string(),
        entry_type: etype,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// Update an existing journal entry.
#[tauri::command]
pub fn update_journal_entry(
    db_state: State<'_, DbState>,
    id:       String,
    payload:  UpdateJournalEntry,
) -> Result<JournalEntry, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let now  = Utc::now().to_rfc3339();

    // Load current values
    let current = conn
        .query_row(
            "SELECT id, game_id, game_title, title, body, entry_type, created_at, updated_at
             FROM journal_entries WHERE id = ?1",
            rusqlite::params![id],
            map_row,
        )
        .map_err(|e| format!("Entry not found: {}", e))?;

    let new_title = payload.title.or(current.title.clone());
    let new_body  = payload.body.unwrap_or(current.body.clone());
    let new_etype = payload.entry_type.unwrap_or(current.entry_type.clone());

    conn.execute(
        "UPDATE journal_entries SET title = ?1, body = ?2, entry_type = ?3,
         updated_at = ?4 WHERE id = ?5",
        rusqlite::params![new_title, new_body, new_etype.as_str(), now, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(JournalEntry {
        id:         current.id,
        game_id:    current.game_id,
        game_title: current.game_title,
        title:      new_title,
        body:       new_body,
        entry_type: new_etype,
        created_at: current.created_at,
        updated_at: now,
    })
}

/// Delete a journal entry permanently.
#[tauri::command]
pub fn delete_journal_entry(
    db_state: State<'_, DbState>,
    id:       String,
) -> Result<(), String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM journal_entries WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

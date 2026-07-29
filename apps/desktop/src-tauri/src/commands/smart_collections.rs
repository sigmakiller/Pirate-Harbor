//! Smart collections — T60.
//!
//! A *smart collection* stores a JSON array of `SmartRule` values and
//! automatically re-evaluates its member set against the current game
//! library whenever `refresh_all_smart_collections` is called (startup
//! + after every game add/update).
//!
//! Supported rule fields and operators:
//!
//! | Field       | Operators              | Value format        |
//! |-------------|------------------------|---------------------|
//! | status      | eq                     | "playing" etc.      |
//! | genre       | contains               | e.g. "RPG"          |
//! | playtime    | gt, lt                 | minutes (integer)   |
//! | developer   | eq, contains           | string              |
//! | is_favorite | is_true                | "true"              |
//!
//! All rules in a collection are AND-combined.

use serde::Serialize;
use tauri::State;

use crate::db::DbState;
use crate::models::{SmartField, SmartOp, SmartRule};

// ── Internal helpers ──────────────────────────────────────────────────────────

/// A lightweight game record used only during smart-collection evaluation.
struct GameRow {
    id:                  String,
    status:              String,
    genre:               Option<String>,
    total_playtime_secs: i64,
    developer:           Option<String>,
    is_favorite:         bool,
}

fn load_all_games(conn: &rusqlite::Connection) -> Vec<GameRow> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT id, status, genre, total_playtime_secs, developer, is_favorite FROM games",
    ) else {
        return vec![];
    };

    let Ok(rows) = stmt.query_map([], |row| {
        Ok(GameRow {
            id:                  row.get::<_, String>(0)?,
            status:              row.get::<_, String>(1)?,
            genre:               row.get::<_, Option<String>>(2)?,
            total_playtime_secs: row.get::<_, i64>(3).unwrap_or(0),
            developer:           row.get::<_, Option<String>>(4)?,
            is_favorite:         row.get::<_, i64>(5).map(|v| v != 0).unwrap_or(false),
        })
    }) else {
        return vec![];
    };

    rows.filter_map(|r| r.ok()).collect()
}

/// Evaluate a single rule against a game row.
fn matches_rule(game: &GameRow, rule: &SmartRule) -> bool {
    match (&rule.field, &rule.operator) {
        (SmartField::Status, SmartOp::Eq) => game.status == rule.value,

        (SmartField::Genre, SmartOp::Contains) => game
            .genre
            .as_deref()
            .unwrap_or("")
            .split(',')
            .map(|s| s.trim())
            .any(|g| g.eq_ignore_ascii_case(&rule.value)),

        (SmartField::Playtime, SmartOp::Gt) => {
            let threshold_secs = rule.value.parse::<i64>().unwrap_or(0) * 60;
            game.total_playtime_secs > threshold_secs
        }
        (SmartField::Playtime, SmartOp::Lt) => {
            let threshold_secs = rule.value.parse::<i64>().unwrap_or(0) * 60;
            game.total_playtime_secs < threshold_secs
        }

        (SmartField::Developer, SmartOp::Eq) => game
            .developer
            .as_deref()
            .unwrap_or("")
            .eq_ignore_ascii_case(&rule.value),

        (SmartField::Developer, SmartOp::Contains) => game
            .developer
            .as_deref()
            .unwrap_or("")
            .to_ascii_lowercase()
            .contains(&rule.value.to_ascii_lowercase()),

        (SmartField::IsFavorite, SmartOp::IsTrue) => game.is_favorite,

        _ => false,
    }
}

/// Return the IDs of all games that satisfy ALL rules (AND logic).
fn evaluate_rules(games: &[GameRow], rules: &[SmartRule]) -> Vec<String> {
    games
        .iter()
        .filter(|g| rules.iter().all(|r| matches_rule(g, r)))
        .map(|g| g.id.clone())
        .collect()
}

/// Sync the `collection_games` table for a smart collection.
///
/// Adds newly matching games and removes games that no longer match.
/// This is idempotent and order-preserving (existing rows keep their `added_at`).
fn sync_collection_games(
    conn:          &rusqlite::Connection,
    collection_id: &str,
    matching_ids:  &[String],
    now:           &str,
) -> rusqlite::Result<()> {
    // Collect existing members before modifying
    let existing: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT game_id FROM collection_games WHERE collection_id = ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![collection_id], |r| r.get::<_, String>(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    // Remove members that no longer match
    for old_id in &existing {
        if !matching_ids.contains(old_id) {
            conn.execute(
                "DELETE FROM collection_games WHERE collection_id = ?1 AND game_id = ?2",
                rusqlite::params![collection_id, old_id],
            )?;
        }
    }

    // Add new members
    for new_id in matching_ids {
        if !existing.contains(new_id) {
            conn.execute(
                "INSERT OR IGNORE INTO collection_games (collection_id, game_id, added_at)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![collection_id, new_id, now],
            )?;
        }
    }

    Ok(())
}

// ── Public commands ───────────────────────────────────────────────────────────

/// Result returned from `create_smart_collection`.
#[derive(Debug, Serialize)]
pub struct SmartCollectionCreated {
    pub id:          String,
    pub name:        String,
    pub match_count: usize,
}

/// Create a smart collection from a rule set.
///
/// `rule_json` — JSON-encoded `Vec<SmartRule>`.  The collection is evaluated
/// immediately so the caller sees `game_ids` populated on the first fetch.
#[tauri::command]
pub fn create_smart_collection(
    state:     State<'_, DbState>,
    name:      String,
    rule_json: String,
) -> Result<SmartCollectionCreated, String> {
    if name.trim().is_empty() {
        return Err("Collection name cannot be empty.".to_string());
    }
    // Validate JSON before storing
    let rules: Vec<SmartRule> =
        serde_json::from_str(&rule_json).map_err(|e| format!("Invalid rule_json: {}", e))?;

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id   = uuid::Uuid::new_v4().to_string();
    let now  = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO collections
             (id, name, description, cover_path, cover_mode, cover_game_id,
              is_smart, rule_json, created_at, updated_at)
         VALUES (?1, ?2, NULL, NULL, 'auto', NULL, 1, ?3, ?4, ?4)",
        rusqlite::params![id, name.trim(), rule_json, now],
    )
    .map_err(|e| e.to_string())?;

    // Evaluate and populate immediately
    let games        = load_all_games(&conn);
    let matching_ids = evaluate_rules(&games, &rules);
    let match_count  = matching_ids.len();

    sync_collection_games(&conn, &id, &matching_ids, &now)
        .map_err(|e| e.to_string())?;

    Ok(SmartCollectionCreated {
        id,
        name: name.trim().to_string(),
        match_count,
    })
}

/// Re-evaluate a single smart collection and return the updated game count.
///
/// Returns an error if the collection is not a smart collection.
#[tauri::command]
pub fn evaluate_smart_collection(
    state: State<'_, DbState>,
    id:    String,
) -> Result<usize, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now  = chrono::Utc::now().to_rfc3339();

    let (is_smart, rule_json): (bool, Option<String>) = conn
        .query_row(
            "SELECT is_smart, rule_json FROM collections WHERE id = ?1",
            rusqlite::params![id],
            |row| Ok((row.get::<_, i64>(0).map(|v| v != 0)?, row.get::<_, Option<String>>(1)?)),
        )
        .map_err(|e| format!("Collection not found: {}", e))?;

    if !is_smart {
        return Err("Not a smart collection.".to_string());
    }

    let rule_str = rule_json.ok_or("Smart collection has no rule_json.")?;
    let rules: Vec<SmartRule> =
        serde_json::from_str(&rule_str).map_err(|e| format!("Invalid rule_json: {}", e))?;

    let games        = load_all_games(&conn);
    let matching_ids = evaluate_rules(&games, &rules);
    let count        = matching_ids.len();

    sync_collection_games(&conn, &id, &matching_ids, &now)
        .map_err(|e| e.to_string())?;

    // Bump updated_at
    let _ = conn.execute(
        "UPDATE collections SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, id],
    );

    Ok(count)
}

/// Re-evaluate ALL smart collections and sync their members.
///
/// Returns the number of smart collections refreshed.
/// Call this at startup and after any game add/update.
#[tauri::command]
pub fn refresh_all_smart_collections(state: State<'_, DbState>) -> Result<usize, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now  = chrono::Utc::now().to_rfc3339();

    // Collect smart collection IDs + rule_json before releasing stmt
    let smart_cols: Vec<(String, String)> = {
        let mut stmt = conn
            .prepare(
                "SELECT id, rule_json FROM collections WHERE is_smart = 1 AND rule_json IS NOT NULL",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;

        rows.filter_map(|r| r.ok()).collect()
    }; // stmt dropped here — conn lock still held

    let games = load_all_games(&conn);
    let count = smart_cols.len();

    for (col_id, rule_str) in &smart_cols {
        let rules: Vec<SmartRule> = match serde_json::from_str(rule_str) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let matching_ids = evaluate_rules(&games, &rules);
        let _ = sync_collection_games(&conn, col_id, &matching_ids, &now);
        let _ = conn.execute(
            "UPDATE collections SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, col_id],
        );
    }

    Ok(count)
}

//! Session query commands.

use tauri::State;

use crate::db::DbState;
use crate::models::Session;

/// Get all play sessions for a game, ordered newest first.
#[tauri::command]
pub fn get_sessions(
    db_state: State<'_, DbState>,
    game_id: String,
) -> Result<Vec<Session>, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, game_id, started_at, ended_at, duration_secs
             FROM sessions
             WHERE game_id = ?1
             ORDER BY started_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let sessions = stmt
        .query_map([&game_id], |row| {
            Ok(Session {
                id:            row.get(0)?,
                game_id:       row.get(1)?,
                started_at:    row.get(2)?,
                ended_at:      row.get(3)?,
                duration_secs: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(sessions)
}

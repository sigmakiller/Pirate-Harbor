//! Settings CRUD Tauri commands.
//!
//! The `settings` table is a simple key-value store.
//! Keys are well-known strings (e.g. "default_view", "scan_directories").

use std::collections::HashMap;
use tauri::State;

use crate::db::DbState;

/// Get the value of a single setting by key.
#[tauri::command]
pub fn get_setting(state: State<DbState>, key: String) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let result = conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [&key],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(val)                         => Ok(Some(val)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e)                          => Err(e.to_string()),
    }
}

/// Set (insert or replace) a setting key-value pair.
#[tauri::command]
pub fn set_setting(state: State<DbState>, key: String, value: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Get all settings as a key-value map.
#[tauri::command]
pub fn get_all_settings(state: State<DbState>) -> Result<HashMap<String, String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;

    let map = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(map)
}

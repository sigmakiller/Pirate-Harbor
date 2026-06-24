//! Collections commands — Phase 2.
//!
//! Curated galleries of games. Each collection has a name, optional
//! description, an optional hero cover (derived from a linked game), and a
//! many-to-many relationship with `games` via `collection_games`.

use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::db::DbState;
use crate::models::{Collection, NewCollection, UpdateCollection};

// ── Read helpers ──────────────────────────────────────────────────────────────

/// Fetch the game_ids and game_count for a single collection.
fn load_game_ids(conn: &rusqlite::Connection, collection_id: &str) -> Vec<String> {
    conn.prepare(
        "SELECT game_id FROM collection_games WHERE collection_id = ?1 ORDER BY added_at",
    )
    .ok()
    .and_then(|mut stmt| {
        stmt.query_map(rusqlite::params![collection_id], |row| {
            row.get::<_, String>(0)
        })
        .ok()
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
    })
    .unwrap_or_default()
}

fn row_to_collection(
    conn: &rusqlite::Connection,
    id: String,
    name: String,
    description: Option<String>,
    cover_game_id: Option<String>,
    created_at: String,
    updated_at: String,
) -> Collection {
    let game_ids = load_game_ids(conn, &id);
    let game_count = game_ids.len() as i64;
    Collection {
        id,
        name,
        description,
        cover_game_id,
        created_at,
        updated_at,
        game_ids,
        game_count,
    }
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Return all collections ordered by creation date (newest first).
#[tauri::command]
pub fn get_collections(db_state: State<'_, DbState>) -> Result<Vec<Collection>, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, cover_game_id, created_at, updated_at
             FROM collections ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let collections: Vec<Collection> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .map(|(id, name, description, cover_game_id, created_at, updated_at)| {
            row_to_collection(&conn, id, name, description, cover_game_id, created_at, updated_at)
        })
        .collect();

    Ok(collections)
}

/// Return a single collection by ID.
#[tauri::command]
pub fn get_collection(
    db_state: State<'_, DbState>,
    id: String,
) -> Result<Collection, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    let result = conn
        .query_row(
            "SELECT id, name, description, cover_game_id, created_at, updated_at
             FROM collections WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .map_err(|e| format!("Collection not found: {}", e))?;

    let (cid, name, description, cover_game_id, created_at, updated_at) = result;
    Ok(row_to_collection(&conn, cid, name, description, cover_game_id, created_at, updated_at))
}

/// Create a new empty collection.
#[tauri::command]
pub fn create_collection(
    db_state: State<'_, DbState>,
    payload: NewCollection,
) -> Result<Collection, String> {
    if payload.name.trim().is_empty() {
        return Err("Collection name cannot be empty.".to_string());
    }

    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let id  = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO collections (id, name, description, cover_game_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        rusqlite::params![id, payload.name.trim(), payload.description, payload.cover_game_id, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(row_to_collection(
        &conn,
        id,
        payload.name.trim().to_string(),
        payload.description,
        payload.cover_game_id,
        now.clone(),
        now,
    ))
}

/// Update a collection's name, description, or hero cover game.
#[tauri::command]
pub fn update_collection(
    db_state: State<'_, DbState>,
    id: String,
    payload: UpdateCollection,
) -> Result<Collection, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let now  = Utc::now().to_rfc3339();

    // Fetch current values
    let (cur_name, cur_desc, cur_cover) = conn
        .query_row(
            "SELECT name, description, cover_game_id FROM collections WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .map_err(|e| format!("Collection not found: {}", e))?;

    let new_name  = payload.name.as_deref().unwrap_or(&cur_name).trim().to_string();
    let new_desc  = payload.description.or(cur_desc);
    let new_cover = if payload.cover_game_id.is_some() {
        payload.cover_game_id
    } else {
        cur_cover
    };

    conn.execute(
        "UPDATE collections SET name = ?1, description = ?2, cover_game_id = ?3,
         updated_at = ?4 WHERE id = ?5",
        rusqlite::params![new_name, new_desc, new_cover, now, id],
    )
    .map_err(|e| e.to_string())?;

    let created_at = conn
        .query_row(
            "SELECT created_at FROM collections WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(row_to_collection(&conn, id, new_name, new_desc, new_cover, created_at, now))
}

/// Delete a collection (games themselves are NOT deleted — only the collection record).
#[tauri::command]
pub fn delete_collection(
    db_state: State<'_, DbState>,
    id: String,
) -> Result<(), String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM collections WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Add a game to a collection (idempotent).
#[tauri::command]
pub fn add_game_to_collection(
    db_state: State<'_, DbState>,
    collection_id: String,
    game_id: String,
) -> Result<Collection, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let now  = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR IGNORE INTO collection_games (collection_id, game_id, added_at)
         VALUES (?1, ?2, ?3)",
        rusqlite::params![collection_id, game_id, now],
    )
    .map_err(|e| e.to_string())?;

    let (name, desc, cover, created_at, updated_at) = conn
        .query_row(
            "SELECT name, description, cover_game_id, created_at, updated_at
             FROM collections WHERE id = ?1",
            rusqlite::params![collection_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .map_err(|e| format!("Collection not found: {}", e))?;

    Ok(row_to_collection(&conn, collection_id, name, desc, cover, created_at, updated_at))
}

/// Remove a game from a collection.
#[tauri::command]
pub fn remove_game_from_collection(
    db_state: State<'_, DbState>,
    collection_id: String,
    game_id: String,
) -> Result<Collection, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM collection_games WHERE collection_id = ?1 AND game_id = ?2",
        rusqlite::params![collection_id, game_id],
    )
    .map_err(|e| e.to_string())?;

    let (name, desc, cover, created_at, updated_at) = conn
        .query_row(
            "SELECT name, description, cover_game_id, created_at, updated_at
             FROM collections WHERE id = ?1",
            rusqlite::params![collection_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .map_err(|e| format!("Collection not found: {}", e))?;

    Ok(row_to_collection(&conn, collection_id, name, desc, cover, created_at, updated_at))
}

/// Return the IDs of all collections a specific game belongs to.
#[tauri::command]
pub fn get_game_collections(
    db_state: State<'_, DbState>,
    game_id: String,
) -> Result<Vec<String>, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT collection_id FROM collection_games WHERE game_id = ?1")
        .map_err(|e| e.to_string())?;

    let ids: Vec<String> = stmt
        .query_map(rusqlite::params![game_id], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(ids)
}

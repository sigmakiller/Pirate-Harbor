//! Game CRUD Tauri commands.
//!
//! All commands receive `State<DbState>` and execute SQLite operations
//! behind the mutex. Errors are returned as `String` so Tauri can
//! propagate them to the frontend as rejected promises.

use tauri::State;
use uuid::Uuid;
use chrono::Utc;

use crate::db::DbState;
use crate::models::{Game, GameFilters, NewGame, UpdateGame};

// ── Helper ────────────────────────────────────────────────────────────────────

/// Map a rusqlite Row into a `Game`. Column order must match the SELECT below.
fn row_to_game(row: &rusqlite::Row<'_>) -> rusqlite::Result<Game> {
    let status_str: String = row.get(13)?;
    let status = status_str.parse().unwrap_or_default();

    Ok(Game {
        id:                  row.get(0)?,
        title:               row.get(1)?,
        exe_path:            row.get(2)?,
        cover_path:          row.get(3)?,
        banner_path:         row.get(4)?,
        developer:           row.get(5)?,
        publisher:           row.get(6)?,
        genre:               row.get(7)?,
        is_favorite:         row.get::<_, i64>(8)? != 0,
        added_at:            row.get(9)?,
        last_played:         row.get(10)?,
        total_playtime_secs: row.get(11)?,
        launch_count:        row.get(12)?,
        status,
    })
}

const SELECT_GAME: &str =
    "SELECT id, title, exe_path, cover_path, banner_path, developer, publisher,
            genre, is_favorite, added_at, last_played, total_playtime_secs,
            launch_count, status
     FROM games";

// ── Commands ──────────────────────────────────────────────────────────────────

/// Get all games, optionally filtered and searched.
#[tauri::command]
pub fn get_all_games(
    state: State<DbState>,
    filters: Option<GameFilters>,
) -> Result<Vec<Game>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let filters = filters.unwrap_or_default();

    let mut sql = format!("{} WHERE 1=1", SELECT_GAME);
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref q) = filters.query {
        sql.push_str(" AND (LOWER(title) LIKE LOWER(?1))");
        params.push(Box::new(format!("%{}%", q)));
    }
    if let Some(ref status) = filters.status {
        sql.push_str(&format!(" AND status = ?{}", params.len() + 1));
        params.push(Box::new(status.as_str().to_string()));
    }
    if let Some(ref genre) = filters.genre {
        sql.push_str(&format!(" AND genre LIKE ?{}", params.len() + 1));
        params.push(Box::new(format!("%{}%", genre)));
    }
    if filters.favorites_only.unwrap_or(false) {
        sql.push_str(" AND is_favorite = 1");
    }

    sql.push_str(" ORDER BY LOWER(title) ASC");

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let games = stmt
        .query_map(param_refs.as_slice(), row_to_game)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(games)
}

/// Get a single game by ID.
#[tauri::command]
pub fn get_game(state: State<DbState>, id: String) -> Result<Game, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let sql = format!("{} WHERE id = ?1", SELECT_GAME);
    conn.query_row(&sql, [&id], row_to_game)
        .map_err(|e| format!("Game not found: {}", e))
}

/// Add a new game to the library.
#[tauri::command]
pub fn add_game(state: State<DbState>, new_game: NewGame) -> Result<Game, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let id       = Uuid::new_v4().to_string();
    let added_at = Utc::now().to_rfc3339();
    let status   = new_game.status.unwrap_or_default();

    conn.execute(
        "INSERT INTO games
            (id, title, exe_path, cover_path, banner_path, developer,
             publisher, genre, is_favorite, added_at, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, ?10)",
        rusqlite::params![
            id,
            new_game.title,
            new_game.exe_path,
            new_game.cover_path,
            new_game.banner_path,
            new_game.developer,
            new_game.publisher,
            new_game.genre,
            added_at,
            status.as_str(),
        ],
    )
    .map_err(|e| e.to_string())?;

    let sql = format!("{} WHERE id = ?1", SELECT_GAME);
    conn.query_row(&sql, [&id], row_to_game)
        .map_err(|e| e.to_string())
}

/// Update fields on an existing game.
#[tauri::command]
pub fn update_game(
    state: State<DbState>,
    id: String,
    updates: UpdateGame,
) -> Result<Game, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    // Build SET clause dynamically — only update provided fields
    let mut sets: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    macro_rules! push_field {
        ($field:expr, $val:expr) => {
            if let Some(v) = $val {
                params.push(Box::new(v));
                sets.push(format!("{} = ?{}", $field, params.len()));
            }
        };
    }

    push_field!("title",       updates.title);
    push_field!("exe_path",    updates.exe_path);
    push_field!("cover_path",  updates.cover_path);
    push_field!("banner_path", updates.banner_path);
    push_field!("developer",   updates.developer);
    push_field!("publisher",   updates.publisher);
    push_field!("genre",       updates.genre);

    if let Some(status) = updates.status {
        params.push(Box::new(status.as_str().to_string()));
        sets.push(format!("status = ?{}", params.len()));
    }
    if let Some(fav) = updates.is_favorite {
        params.push(Box::new(if fav { 1i64 } else { 0i64 }));
        sets.push(format!("is_favorite = ?{}", params.len()));
    }

    if sets.is_empty() {
        // Nothing to update — just return the current state
        let sql = format!("{} WHERE id = ?1", SELECT_GAME);
        return conn
            .query_row(&sql, [&id], row_to_game)
            .map_err(|e| e.to_string());
    }

    params.push(Box::new(id.clone()));
    let where_idx = params.len();
    let sql = format!("UPDATE games SET {} WHERE id = ?{}", sets.join(", "), where_idx);

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, param_refs.as_slice())
        .map_err(|e| e.to_string())?;

    let select = format!("{} WHERE id = ?1", SELECT_GAME);
    conn.query_row(&select, [&id], row_to_game)
        .map_err(|e| e.to_string())
}

/// Delete a game and all its sessions (CASCADE).
#[tauri::command]
pub fn delete_game(state: State<DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM games WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Toggle a game's favorite status.
#[tauri::command]
pub fn toggle_favorite(state: State<DbState>, id: String) -> Result<Game, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE games SET is_favorite = CASE WHEN is_favorite = 1 THEN 0 ELSE 1 END WHERE id = ?1",
        [&id],
    )
    .map_err(|e| e.to_string())?;

    let sql = format!("{} WHERE id = ?1", SELECT_GAME);
    conn.query_row(&sql, [&id], row_to_game)
        .map_err(|e| e.to_string())
}

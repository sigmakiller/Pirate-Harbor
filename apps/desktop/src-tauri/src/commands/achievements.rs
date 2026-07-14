//! Achievement tracking Tauri commands — T42.
//!
//! # Commands
//!
//! | Command                         | Description                                       |
//! |---------------------------------|---------------------------------------------------|
//! | `enable_achievement_tracking`   | Inject Goldberg DLL + start file watcher          |
//! | `disable_achievement_tracking`  | Stop watcher + restore original DLL               |
//! | `get_achievement_tracking_status` | Query per-game tracking state from DB           |
//! | `add_achievement_mapping`       | Manually add a Steam ID → display_name mapping    |
//! | `remove_achievement_mapping`    | Delete a mapping row by ID                        |
//! | `get_achievement_mappings`      | List all mappings for a game                      |
//! | `import_achievements_from_steam`| Bulk-import from Steam GetSchemaForGame API       |

use std::path::PathBuf;
use std::sync::Mutex;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::db::DbState;
use crate::steam_bridge::{
    achievement_watcher::{start_watcher, stop_watcher, WatcherRegistry},
    dll_swap, steam_api,
};

// ── Domain types exposed to the frontend ─────────────────────────────────────

/// A single Steam achievement mapping stored in `steam_achievement_mappings`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementMapping {
    pub id:           String,
    pub game_id:      String,
    pub steam_id:     String,
    pub display_name: String,
    pub description:  Option<String>,
    pub points:       i32,
    pub created_at:   String,
}

/// Per-game achievement tracking status.
#[derive(Debug, Serialize)]
pub struct TrackingStatus {
    pub enabled:       bool,
    pub steam_app_id:  Option<String>,
    pub mapping_count: usize,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Read the `app_data_dir` path from the settings table and return the
/// absolute path to `pirate_harbor.db`.
fn resolve_db_path(conn: &rusqlite::Connection) -> Result<PathBuf, String> {
    let dir: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'app_data_dir'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("Could not read app_data_dir from settings: {e}"))?;
    Ok(PathBuf::from(dir).join("pirate_harbor.db"))
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Install the Goldberg DLL into the game directory and start watching
/// `achievements.json` for unlock events.
///
/// On each file change the router diffs old vs new state, creates milestones
/// for newly earned achievements, and emits `"achievement-unlocked"` events.
#[tauri::command]
pub async fn enable_achievement_tracking(
    db:           State<'_, DbState>,
    registry:     State<'_, WatcherRegistry>,
    app_handle:   tauri::AppHandle,
    game_id:      String,
    exe_path:     String,
    steam_app_id: String,
) -> Result<(), String> {
    let game_dir = dll_swap::game_dir_from_exe(&exe_path)?;
    dll_swap::inject_dll(&game_dir, &steam_app_id, &app_handle)?;

    // Persist tracking state to the DB.
    let db_path = {
        let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
        conn.execute(
            "UPDATE games SET achievement_tracking_enabled = 1, steam_app_id = ?1 WHERE id = ?2",
            rusqlite::params![steam_app_id, game_id],
        )
        .map_err(|e| e.to_string())?;
        resolve_db_path(&conn)?
    };

    // The watcher closure runs on a background thread owned by notify.
    // We pass the DB path so the closure can open its own connection,
    // avoiding the "can't clone a Mutex<Connection>" problem.
    let gid       = game_id.clone();
    let app_clone = app_handle.clone();

    start_watcher(&registry, game_id, steam_app_id, move |json| {
        use crate::steam_bridge::achievement_router;

        // Each watcher instance keeps its own state snapshot.
        // OnceLock initialises once per closure instance created by this call;
        // a new OnceLock is created each time enable_achievement_tracking is
        // called because the closure captures a fresh stack frame.
        static STATE: std::sync::OnceLock<Mutex<achievement_router::AchievementState>> =
            std::sync::OnceLock::new();
        let state_mutex = STATE.get_or_init(|| Mutex::new(Default::default()));
        let mut old = state_mutex.lock().unwrap();

        // Open a per-call connection on this background thread.
        let Ok(conn) = rusqlite::Connection::open(&db_path) else { return };
        if let Ok(new_state) = achievement_router::process_changes(
            &old, &json, &gid, &conn, &app_clone,
        ) {
            *old = new_state;
        }
    })?;

    Ok(())
}

/// Stop the file watcher and restore the original `steam_api64.dll`.
#[tauri::command]
pub fn disable_achievement_tracking(
    db:       State<'_, DbState>,
    registry: State<'_, WatcherRegistry>,
    game_id:  String,
    exe_path: String,
) -> Result<(), String> {
    stop_watcher(&registry, &game_id);

    let game_dir = dll_swap::game_dir_from_exe(&exe_path)?;
    dll_swap::restore_dll(&game_dir)?;

    let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
    conn.execute(
        "UPDATE games SET achievement_tracking_enabled = 0 WHERE id = ?1",
        rusqlite::params![game_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Return the current achievement tracking state for a game.
#[tauri::command]
pub fn get_achievement_tracking_status(
    db:      State<'_, DbState>,
    game_id: String,
) -> Result<TrackingStatus, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;

    let (enabled, steam_app_id): (bool, Option<String>) = conn
        .query_row(
            "SELECT achievement_tracking_enabled, steam_app_id FROM games WHERE id = ?1",
            rusqlite::params![game_id],
            |r| Ok((r.get::<_, i32>(0)? != 0, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let mapping_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM steam_achievement_mappings WHERE game_id = ?1",
            rusqlite::params![game_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(TrackingStatus {
        enabled,
        steam_app_id,
        mapping_count: mapping_count as usize,
    })
}

/// Manually add (or replace) an achievement mapping for a game.
#[tauri::command]
pub fn add_achievement_mapping(
    db:           State<'_, DbState>,
    game_id:      String,
    steam_id:     String,
    display_name: String,
    description:  Option<String>,
    points:       i32,
) -> Result<AchievementMapping, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
    let id  = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO steam_achievement_mappings
         (id, game_id, steam_id, display_name, description, points, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, game_id, steam_id, display_name, description, points, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(AchievementMapping {
        id,
        game_id,
        steam_id,
        display_name,
        description,
        points,
        created_at: now,
    })
}

/// Delete a single achievement mapping by its UUID.
#[tauri::command]
pub fn remove_achievement_mapping(
    db:         State<'_, DbState>,
    mapping_id: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
    conn.execute(
        "DELETE FROM steam_achievement_mappings WHERE id = ?1",
        rusqlite::params![mapping_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Return all achievement mappings for a game, ordered by `steam_id`.
#[tauri::command]
pub fn get_achievement_mappings(
    db:      State<'_, DbState>,
    game_id: String,
) -> Result<Vec<AchievementMapping>, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
    let mut stmt = conn
        .prepare(
            "SELECT id, game_id, steam_id, display_name, description, points, created_at
             FROM steam_achievement_mappings
             WHERE game_id = ?1
             ORDER BY steam_id",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![game_id], |r| {
            Ok(AchievementMapping {
                id:           r.get(0)?,
                game_id:      r.get(1)?,
                steam_id:     r.get(2)?,
                display_name: r.get(3)?,
                description:  r.get(4)?,
                points:       r.get(5)?,
                created_at:   r.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    Ok(rows)
}

/// Fetch achievement definitions from Steam's public schema API and bulk-insert
/// them as mappings for the given game. Existing entries are not overwritten
/// (`INSERT OR IGNORE`).
///
/// Returns the full list of mappings that were successfully inserted.
#[tauri::command]
pub async fn import_achievements_from_steam(
    db:           State<'_, DbState>,
    game_id:      String,
    steam_app_id: String,
) -> Result<Vec<AchievementMapping>, String> {
    let client = reqwest::Client::new();
    let defs   = steam_api::fetch_achievement_defs(&client, &steam_app_id).await?;

    let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
    let now  = Utc::now().to_rfc3339();
    let mut inserted = Vec::new();

    for def in defs {
        let id = Uuid::new_v4().to_string();
        let rows_changed = conn
            .execute(
                "INSERT OR IGNORE INTO steam_achievement_mappings
                 (id, game_id, steam_id, display_name, description, points, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 10, ?6)",
                rusqlite::params![id, game_id, def.name, def.display_name, def.description, now],
            )
            .unwrap_or(0);

        if rows_changed > 0 {
            inserted.push(AchievementMapping {
                id,
                game_id:      game_id.clone(),
                steam_id:     def.name,
                display_name: def.display_name,
                description:  def.description,
                points:       10,
                created_at:   now.clone(),
            });
        }
    }

    Ok(inserted)
}

//! Game launcher and playtime tracking commands.
//!
//! Architecture:
//! - `launch_game` spawns the game process, creates a DB session, and starts
//!   a background Tokio task to monitor the process.
//! - The monitor polls `sysinfo` every 5 seconds. When the process exits, it
//!   finalises the session, updates total playtime, and emits `game-stopped`.
//! - `LauncherState` (Mutex<Option<RunningGame>>) is stored in Tauri managed
//!   state so any command can read the currently running game.

use std::sync::Mutex;

use chrono::Utc;
use sysinfo::{Pid, System};
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

use crate::db::DbState;

// ── State ─────────────────────────────────────────────────────────────────────

/// Details of the currently running game.
#[allow(dead_code)]
pub struct RunningGame {
    pub game_id:    String,
    pub pid:        u32,
    pub session_id: String,
    /// Unix timestamp (seconds) — used to compute duration without re-parsing ISO strings.
    pub started_at: i64,
}

/// Tauri managed state for the launcher engine.
/// Wrapped in `Mutex` for safe access from both commands and the background task.
pub struct LauncherState(pub Mutex<Option<RunningGame>>);

// ── Commands ──────────────────────────────────────────────────────────────────

/// Launch a game by its library ID.
///
/// 1. Verifies no other game is currently running.
/// 2. Reads `exe_path` from the database.
/// 3. Spawns the OS process.
/// 4. Creates a `sessions` row and increments `launch_count`.
/// 5. Stores PID in `LauncherState`.
/// 6. Spawns a background monitor task.
#[tauri::command]
pub async fn launch_game(
    id: String,
    db_state: State<'_, DbState>,
    launcher_state: State<'_, LauncherState>,
    app: AppHandle,
) -> Result<(), String> {
    // ── Guard: only one game at a time ────────────────────────────────────────
    {
        let guard = launcher_state.0.lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Err("A game is already running. Close it before launching another.".into());
        }
    }

    // ── Fetch exe_path ────────────────────────────────────────────────────────
    let exe_path: String = {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT exe_path FROM games WHERE id = ?1",
            [&id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Game not found: {}", e))?
    };

    // ── Spawn game process ────────────────────────────────────────────────────
    let child = std::process::Command::new(&exe_path)
        .spawn()
        .map_err(|e| format!("Failed to launch '{}': {}", exe_path, e))?;

    let pid          = child.id();
    let session_id   = Uuid::new_v4().to_string();
    let now          = Utc::now();
    let started_iso  = now.to_rfc3339();
    let started_unix = now.timestamp();

    // ── Persist session + update game stats ───────────────────────────────────
    {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO sessions (id, game_id, started_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![session_id, id, started_iso],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE games
             SET launch_count = launch_count + 1,
                 last_played  = ?1
             WHERE id = ?2",
            rusqlite::params![started_iso, id],
        )
        .map_err(|e| e.to_string())?;
    }

    // ── Store running state ───────────────────────────────────────────────────
    {
        let mut guard = launcher_state.0.lock().map_err(|e| e.to_string())?;
        *guard = Some(RunningGame {
            game_id:    id.clone(),
            pid,
            session_id: session_id.clone(),
            started_at: started_unix,
        });
    }

    // ── Background monitor ────────────────────────────────────────────────────
    tokio::spawn(monitor_process(app, id, pid, session_id, started_unix));

    Ok(())
}

/// Returns the ID of the currently running game, or `null` if none.
#[tauri::command]
pub fn get_running_game(launcher_state: State<'_, LauncherState>) -> Option<String> {
    launcher_state
        .0
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|r| r.game_id.clone()))
}

// ── Background monitor ────────────────────────────────────────────────────────

/// Polls every 5 seconds until the process exits, then finalises the session.
///
/// Uses `app.state()` instead of `State<>` so it can live inside `tokio::spawn`.
async fn monitor_process(
    app: AppHandle,
    game_id: String,
    pid: u32,
    session_id: String,
    started_unix: i64,
) {
    // Give the process a short window to fully start before monitoring.
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let sysinfo_pid = Pid::from(pid as usize);

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Check if the process is still alive
        let mut sys = System::new();
        sys.refresh_process(sysinfo_pid);

        if sys.process(sysinfo_pid).is_none() {
            // ── Process exited ────────────────────────────────────────────────
            let ended_now    = Utc::now();
            let ended_iso    = ended_now.to_rfc3339();
            let duration_sec = (ended_now.timestamp() - started_unix).max(0);

            // Finalise session + update playtime
            let db = app.state::<DbState>();
            if let Ok(conn) = db.0.lock() {
                let _ = conn.execute(
                    "UPDATE sessions
                     SET ended_at = ?1, duration_secs = ?2
                     WHERE id = ?3",
                    rusqlite::params![ended_iso, duration_sec, session_id],
                );
                let _ = conn.execute(
                    "UPDATE games
                     SET total_playtime_secs = total_playtime_secs + ?1
                     WHERE id = ?2",
                    rusqlite::params![duration_sec, game_id],
                );
            }

            // Clear launcher state
            let launcher = app.state::<LauncherState>();
            if let Ok(mut guard) = launcher.0.lock() {
                *guard = None;
            }

            // Notify the frontend so it can refresh the game detail view
            let _ = app.emit("game-stopped", game_id.clone());

            break;
        }
    }
}

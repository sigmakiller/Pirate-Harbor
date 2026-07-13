//! File watcher for Goldberg's achievements.json — T40.
//!
//! # How Goldberg writes achievements
//!
//! Goldberg Steam Emu saves unlock state to:
//! ```text
//! %APPDATA%\Goldberg SteamEmu Saves\{steam_app_id}\achievements.json
//! ```
//! The file is rewritten by Goldberg on every unlock event. We watch the
//! **parent directory** (non-recursive) because `notify` is more reliable
//! on directories than on individual files for cross-platform compatibility.
//!
//! # Architecture
//!
//! * [`start_watcher`] creates a `notify` watcher, stores it in
//!   [`WatcherRegistry`], and returns immediately (non-blocking).
//! * The watcher runs on its own background thread managed by `notify`.
//! * The `on_change` callback is called with the raw JSON string on every
//!   `Modify` or `Create` event targeting `achievements.json`.
//! * Dropping a [`WatcherHandle`] (via [`stop_watcher`] or app exit) stops
//!   the watcher automatically — no explicit cleanup is needed.
//!
//! # Tauri state
//!
//! Register with `app.manage(WatcherRegistry::new())` in `lib.rs`. T41
//! will use the registry through Tauri `State<'_, WatcherRegistry>`.

// All public items become used in T41 Tauri commands.
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

// ── WatcherHandle ─────────────────────────────────────────────────────────────

/// Opaque handle representing one active file watcher.
///
/// Dropping this value stops the underlying `notify` watcher thread and
/// releases all associated OS resources. The drop happens automatically
/// when the entry is removed from [`WatcherRegistry`].
pub struct WatcherHandle {
    // The underscore prefix signals intentional non-use — the watcher is
    // driven purely by its background thread; we only hold it to keep it
    // alive. Dropping it cancels the watch.
    _watcher: RecommendedWatcher,
}

// ── WatcherRegistry ───────────────────────────────────────────────────────────

/// Tauri managed state: map of `game_id → active WatcherHandle`.
///
/// Registered once with `app.manage(WatcherRegistry::new())` in `lib.rs`.
/// Accessible from Tauri commands as `State<'_, WatcherRegistry>`.
pub struct WatcherRegistry(pub Mutex<HashMap<String, WatcherHandle>>);

impl WatcherRegistry {
    /// Create an empty registry — call this in `lib.rs` setup.
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

// ── Path helpers ──────────────────────────────────────────────────────────────

/// Resolve Goldberg's `achievements.json` path for a given Steam App ID.
///
/// Windows path:
/// `%APPDATA%\Goldberg SteamEmu Saves\{steam_app_id}\achievements.json`
///
/// Returns `None` if the system `APPDATA` directory cannot be determined
/// (unlikely on Windows, possible in minimal CI environments).
pub fn goldberg_achievements_path(steam_app_id: &str) -> Option<PathBuf> {
    dirs::data_dir().map(|base| {
        base.join("Goldberg SteamEmu Saves")
            .join(steam_app_id)
            .join("achievements.json")
    })
}

// ── Watcher lifecycle ─────────────────────────────────────────────────────────

/// Start watching `achievements.json` for the given `game_id`.
///
/// If a watcher is already registered for this `game_id`, it is stopped and
/// replaced — calling `start_watcher` twice for the same game is safe.
///
/// `on_change` is called with the raw JSON string of `achievements.json`
/// on every `Modify` or `Create` event. It executes on `notify`'s internal
/// background thread — keep it fast and non-blocking.
///
/// Returns `Err` if:
/// * The Goldberg save directory cannot be determined (see [`goldberg_achievements_path`]).
/// * `notify` cannot set up the OS-level file watch (e.g., permission error).
pub fn start_watcher<F>(
    registry:     &WatcherRegistry,
    game_id:      String,
    steam_app_id: String,
    on_change:    F,
) -> Result<(), String>
where
    F: Fn(String) + Send + Sync + 'static,
{
    let watch_path = goldberg_achievements_path(&steam_app_id)
        .ok_or_else(|| "Cannot resolve Goldberg save directory (APPDATA unavailable)".to_string())?;

    // Ensure the parent directory exists so notify can attach immediately.
    // Goldberg creates it on first launch, but we create it now so the watch
    // is ready before the game runs.
    if let Some(parent) = watch_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let watch_path_for_callback = watch_path.clone();
    let on_change = Arc::new(on_change);

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        let Ok(event) = res else { return };

        // Only react to content-change events.
        let is_modify_or_create = matches!(
            event.kind,
            EventKind::Modify(_) | EventKind::Create(_)
        );
        if !is_modify_or_create {
            return;
        }

        // Filter: only fire when achievements.json itself changed.
        let targets_our_file = event.paths.iter().any(|p| p == &watch_path_for_callback);
        if !targets_our_file {
            return;
        }

        // Read the file and hand the raw JSON to the caller.
        match std::fs::read_to_string(&watch_path_for_callback) {
            Ok(json) => on_change(json),
            Err(e)   => eprintln!("[achievement_watcher] Failed to read achievements.json: {e}"),
        }
    })
    .map_err(|e| format!("Failed to create file watcher: {e}"))?;

    // Watch the parent directory (non-recursive) — more reliable than
    // watching a single file across platforms and editor save strategies.
    let watch_dir = watch_path
        .parent()
        .ok_or_else(|| "achievements.json has no parent directory".to_string())?;

    watcher
        .watch(watch_dir, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Failed to start watching {}: {e}", watch_dir.display()))?;

    let handle = WatcherHandle { _watcher: watcher };

    let mut map = registry.0.lock()
        .map_err(|_| "WatcherRegistry lock is poisoned".to_string())?;
    // Replacing an existing entry drops the old WatcherHandle → stops old watcher.
    map.insert(game_id, handle);

    Ok(())
}

/// Stop watching `achievements.json` for the given `game_id`.
///
/// Drops the [`WatcherHandle`], which stops the background watcher thread.
/// Safe to call when no watcher is registered (no-op).
pub fn stop_watcher(registry: &WatcherRegistry, game_id: &str) {
    if let Ok(mut map) = registry.0.lock() {
        // Dropping the WatcherHandle stops the notify watcher automatically.
        map.remove(game_id);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// The resolved path must contain the App ID and end with `achievements.json`.
    #[test]
    fn goldberg_path_contains_app_id() {
        // This may return None in minimal CI environments without APPDATA.
        // We skip gracefully in that case rather than failing the build.
        if let Some(path) = goldberg_achievements_path("570") {
            assert!(
                path.to_string_lossy().contains("570"),
                "Path must contain the Steam App ID"
            );
            assert!(
                path.ends_with("achievements.json"),
                "Path must end with achievements.json"
            );
        }
    }

    /// Inserting a WatcherHandle into the registry and removing it must not panic.
    /// Verifies the stop_watcher → empty registry flow.
    #[test]
    fn start_and_stop_watcher_does_not_panic() {
        let registry = WatcherRegistry::new();

        // Use a real temp dir so the notify watcher can attach.
        let dir  = tempdir().unwrap();
        let path = dir.path().join("achievements.json");
        fs::write(&path, "{}").unwrap();

        // Construct a watcher manually (bypassing goldberg_path resolution)
        // so the test is not coupled to the APPDATA environment variable.
        let mut watcher = notify::recommended_watcher(|_: notify::Result<Event>| {})
            .expect("Failed to create test watcher");
        watcher
            .watch(dir.path(), RecursiveMode::NonRecursive)
            .expect("Failed to start test watcher");

        let handle = WatcherHandle { _watcher: watcher };
        registry.0.lock().unwrap().insert("test_game".to_string(), handle);

        // Registry must contain one entry.
        assert_eq!(
            registry.0.lock().unwrap().len(),
            1,
            "Registry must contain the inserted watcher"
        );

        // Stopping the watcher must leave the registry empty.
        stop_watcher(&registry, "test_game");
        assert!(
            registry.0.lock().unwrap().is_empty(),
            "Registry must be empty after stop_watcher"
        );
    }

    /// stop_watcher on an unknown game_id must be a silent no-op.
    #[test]
    fn stop_unknown_game_is_noop() {
        let registry = WatcherRegistry::new();
        // Must not panic or return an error.
        stop_watcher(&registry, "nonexistent_game_id");
        assert!(registry.0.lock().unwrap().is_empty());
    }
}

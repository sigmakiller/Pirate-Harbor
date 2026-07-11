# T38–T40 — Phase 5 Achievement Tracking (Foundation)
## Engineer Implementation Plan

**Phase:** 5 — Automated Achievement Tracking  
**Tasks:** T38 (DB + Types), T39 (DLL Swap), T40 (File Watcher)  
**Depends on:** T26–T31 complete ✅  
**Approach:** File-watching via `notify` crate on Goldberg's `achievements.json`

> Implement one task at a time in order. Do not start T39 until T38 passes
> `cargo test`. Do not start T40 until T39 unit tests pass.

---

## T38 — Database Migration + TypeScript Types

### 1. `db/migrations.rs`

Bump `CURRENT_SCHEMA_VERSION` from `7` to `8`:

```rust
pub const CURRENT_SCHEMA_VERSION: i32 = 8;
```

Add to the `MIGRATIONS` array:

```rust
Migration { version: 8, description: "Achievement tracking", sql: MIGRATION_008 },
```

Add the SQL constant:

```rust
/// 008 — Steam achievement tracking: mapping table + games columns
const MIGRATION_008: &str = r#"
CREATE TABLE IF NOT EXISTS steam_achievement_mappings (
    id           TEXT PRIMARY KEY,
    game_id      TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    steam_id     TEXT NOT NULL,
    display_name TEXT NOT NULL,
    description  TEXT,
    points       INTEGER NOT NULL DEFAULT 10,
    created_at   TEXT NOT NULL,
    UNIQUE(game_id, steam_id)
);

CREATE INDEX IF NOT EXISTS idx_ach_mappings_game
    ON steam_achievement_mappings(game_id);
"#;
```

Add to `MIGRATION_008_ALTER` (same pattern as `MIGRATION_005_ALTER`):

```rust
/// Columns added to `games` by MIGRATION_008.
const MIGRATION_008_ALTER: &[(&str, &str, &str)] = &[
    ("games", "achievement_tracking_enabled", "INTEGER NOT NULL DEFAULT 0"),
    ("games", "steam_app_id",                 "TEXT"),
];
```

Wire the ALTER into `run_migrations()` following the same pattern used for
Migration 005 — apply each ALTER and ignore "duplicate column" errors.

### 2. Add unit tests to `db/migrations.rs`

```rust
#[test]
fn test_migration_008_creates_achievement_table() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'
             AND name='steam_achievement_mappings'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_migration_008_games_columns_exist() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    // INSERT with new columns must not error
    conn.execute(
        "INSERT INTO games (id, title, exe_path, added_at, status,
                            achievement_tracking_enabled, steam_app_id)
         VALUES ('g1','Test','C:/test.exe',datetime('now'),'unplayed',0,NULL)",
        [],
    ).unwrap();
}
```

### 3. `src/lib/api.ts` — new TypeScript types

Add to the existing types section:

```typescript
// ── Achievement Tracking (Phase 5) ──────────────────────────────────────────

export interface AchievementMapping {
  id: string;
  game_id: string;
  steam_id: string;         // e.g. "ACH_WIN_ONE_GAME"
  display_name: string;
  description: string | null;
  points: number;
  created_at: string;
}

export interface TrackingStatus {
  enabled: boolean;
  steam_app_id: string | null;
  mapping_count: number;
}

export interface SteamAchievementDef {
  steam_id: string;
  display_name: string;
  description: string | null;
}

export type AppIdSource = "rawg" | "local_file" | "not_found";

export interface AppIdDetectionResult {
  app_id: string | null;
  source: AppIdSource;
}
```

### T38 Acceptance Criteria

- [ ] `cargo test` — all tests pass including the 2 new migration tests
- [ ] Schema version reports `8` on a fresh DB
- [ ] `steam_achievement_mappings` table exists with correct columns
- [ ] `games` table has `achievement_tracking_enabled` and `steam_app_id`
- [ ] TypeScript types added with no `tsc --noEmit` errors

---

## T39 — DLL Swap Module

### Overview

This module handles the filesystem operations for installing/removing the
Goldberg emulator DLL into a game's directory. It performs **atomic backups**
(rename, not copy) and provides crash-recovery via `SwapState` detection.

### Resource Bundle Setup

Place the pre-compiled Goldberg `steam_api64.dll` into:

```
apps/desktop/src-tauri/resources/plugins/steam_api64.dll
```

Add to `apps/desktop/src-tauri/tauri.conf.json` under `"bundle"`:

```json
{
  "bundle": {
    "resources": ["resources/plugins/*"]
  }
}
```

### New file: `src/steam_bridge/dll_swap.rs`

```rust
//! DLL swap — backup / inject Goldberg / restore — T39.
//!
//! Atomic strategy: rename (not copy) so a crash mid-operation leaves the
//! game in a detectable `BackupOnlyNoActive` state that can be recovered.

use std::path::{Path, PathBuf};
use tauri::Manager;

const BACKUP_SUFFIX:   &str = ".ph_backup";
const APPID_FILE:      &str = "steam_appid.txt";
const SETTINGS_DIR:    &str = "steam_settings";

/// Current state of the DLL in a game directory.
#[derive(Debug, PartialEq)]
pub enum SwapState {
    /// No steam_api64.dll present — tracking not supported for this game.
    NoSteamDll,
    /// Original DLL present, our DLL not installed.
    OriginalPresent,
    /// Goldberg DLL active, backup exists — normal active state.
    OurDllInstalled,
    /// Backup exists but active DLL is missing — app crashed mid-swap.
    /// Call `restore_dll` to recover.
    BackupOnlyNoActive,
}

pub fn verify_swap_integrity(game_dir: &Path) -> Result<SwapState, String> {
    let dll    = game_dir.join("steam_api64.dll");
    let backup = game_dir.join(format!("steam_api64.dll{}", BACKUP_SUFFIX));

    match (dll.exists(), backup.exists()) {
        (false, false) => Ok(SwapState::NoSteamDll),
        (true,  false) => Ok(SwapState::OriginalPresent),
        (true,  true)  => Ok(SwapState::OurDllInstalled),
        (false, true)  => Ok(SwapState::BackupOnlyNoActive),
    }
}

/// Install Goldberg DLL into the game directory.
///
/// Steps:
///   1. Verify no swap is already active.
///   2. Rename original → .ph_backup  (atomic on same volume)
///   3. Copy our Goldberg DLL from the resource bundle.
///   4. Write steam_settings/steam_appid.txt (required by Goldberg).
pub fn inject_dll(
    game_dir:    &Path,
    steam_app_id: &str,
    app_handle:  &tauri::AppHandle,
) -> Result<(), String> {
    // Guard: already installed or no DLL present.
    match verify_swap_integrity(game_dir)? {
        SwapState::OurDllInstalled  => return Ok(()), // idempotent
        SwapState::NoSteamDll       => return Err("Game has no steam_api64.dll — achievement tracking not supported.".into()),
        SwapState::BackupOnlyNoActive => return Err("Previous swap is in a broken state. Restore the original DLL first.".into()),
        SwapState::OriginalPresent  => {}
    }

    let dll_dest   = game_dir.join("steam_api64.dll");
    let backup     = game_dir.join(format!("steam_api64.dll{}", BACKUP_SUFFIX));

    // Step 1: atomic rename original → backup
    std::fs::rename(&dll_dest, &backup)
        .map_err(|e| format!("Failed to backup original DLL: {e}"))?;

    // Step 2: copy our Goldberg DLL from resource bundle
    let dll_src = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Resource dir error: {e}"))?
        .join("plugins")
        .join("steam_api64.dll");

    std::fs::copy(&dll_src, &dll_dest)
        .map_err(|e| format!("Failed to copy Goldberg DLL: {e}"))?;

    // Step 3: write steam_settings/steam_appid.txt
    let settings_dir = game_dir.join(SETTINGS_DIR);
    std::fs::create_dir_all(&settings_dir)
        .map_err(|e| format!("Failed to create steam_settings/: {e}"))?;
    std::fs::write(settings_dir.join(APPID_FILE), steam_app_id)
        .map_err(|e| format!("Failed to write steam_appid.txt: {e}"))?;

    Ok(())
}

/// Restore the original DLL, removing the Goldberg installation.
///
/// Safe to call even if swap is not active (idempotent).
pub fn restore_dll(game_dir: &Path) -> Result<(), String> {
    let dll_dest = game_dir.join("steam_api64.dll");
    let backup   = game_dir.join(format!("steam_api64.dll{}", BACKUP_SUFFIX));

    if !backup.exists() {
        // Nothing to restore.
        return Ok(());
    }

    // Remove active DLL if present (may be missing after crash).
    if dll_dest.exists() {
        std::fs::remove_file(&dll_dest)
            .map_err(|e| format!("Failed to remove Goldberg DLL: {e}"))?;
    }

    // Restore backup.
    std::fs::rename(&backup, &dll_dest)
        .map_err(|e| format!("Failed to restore original DLL: {e}"))?;

    // Clean up steam_settings/ directory we created.
    let settings_dir = game_dir.join(SETTINGS_DIR);
    if settings_dir.exists() {
        std::fs::remove_dir_all(&settings_dir)
            .map_err(|e| format!("Failed to remove steam_settings/: {e}"))?;
    }

    Ok(())
}

/// Derive the game directory from an exe path.
pub fn game_dir_from_exe(exe_path: &str) -> Result<PathBuf, String> {
    Path::new(exe_path)
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| format!("Cannot determine game directory from: {exe_path}"))
}
```

### New file: `src/steam_bridge/mod.rs`

```rust
//! Steam Bridge — achievement tracking infrastructure — Phase 5.

pub mod dll_swap;
// T40 additions:
// pub mod achievement_watcher;
// pub mod achievement_router;
// pub mod steam_api;
```

### Wire into `src/lib.rs`

```rust
mod steam_bridge;
```

### T39 Unit Tests (in `dll_swap.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;  // add tempfile = "3" to dev-dependencies

    fn make_game_dir_with_dll() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("steam_api64.dll"), b"original").unwrap();
        dir
    }

    #[test]
    fn state_no_dll() {
        let dir = tempdir().unwrap();
        assert_eq!(verify_swap_integrity(dir.path()).unwrap(), SwapState::NoSteamDll);
    }

    #[test]
    fn state_original_present() {
        let dir = make_game_dir_with_dll();
        assert_eq!(verify_swap_integrity(dir.path()).unwrap(), SwapState::OriginalPresent);
    }

    #[test]
    fn state_our_dll_installed() {
        let dir = make_game_dir_with_dll();
        // Simulate installed state manually.
        fs::rename(
            dir.path().join("steam_api64.dll"),
            dir.path().join("steam_api64.dll.ph_backup"),
        ).unwrap();
        fs::write(dir.path().join("steam_api64.dll"), b"goldberg").unwrap();
        assert_eq!(verify_swap_integrity(dir.path()).unwrap(), SwapState::OurDllInstalled);
    }

    #[test]
    fn state_backup_only_no_active() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("steam_api64.dll.ph_backup"), b"original").unwrap();
        assert_eq!(verify_swap_integrity(dir.path()).unwrap(), SwapState::BackupOnlyNoActive);
    }

    #[test]
    fn restore_is_idempotent_when_nothing_installed() {
        let dir = make_game_dir_with_dll();
        // Should succeed without doing anything.
        restore_dll(dir.path()).unwrap();
        assert!(dir.path().join("steam_api64.dll").exists());
    }
}
```

Add to `Cargo.toml` `[dev-dependencies]`:
```toml
tempfile = "3"
```

### T39 Acceptance Criteria

- [ ] All 5 unit tests pass
- [ ] `inject_dll` on a dir with no `steam_api64.dll` returns a clear error
- [ ] `inject_dll` is idempotent (calling twice is safe)
- [ ] `restore_dll` is idempotent (calling on non-swapped game is safe)
- [ ] `steam_settings/steam_appid.txt` is created/deleted correctly
- [ ] `cargo check` — no new warnings

---

## T40 — File Watcher Module

### Overview

Starts a `notify` watcher on Goldberg's `achievements.json` for a given
Steam App ID. When the file changes, hands off to `achievement_router`
(implemented in T41). Manages active watchers in a `WatcherRegistry` Tauri
state object so watchers survive for the app lifetime.

### New Cargo dependency

```toml
notify = "6"
dirs   = "5"
```

### New file: `src/steam_bridge/achievement_watcher.rs`

```rust
//! File watcher for Goldberg's achievements.json — T40.
//!
//! Goldberg writes achievement unlock state to:
//!   %APPDATA%\Goldberg SteamEmu Saves\{steam_app_id}\achievements.json
//!
//! We watch this file with the `notify` crate. On any Modify/Create event
//! we hand off to `achievement_router::process_changes`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// Opaque handle representing one active watcher.
/// Dropping this stops the watcher.
pub struct WatcherHandle {
    _watcher: RecommendedWatcher,
}

/// Tauri state: map of game_id → active watcher handle.
///
/// Managed as `app.manage(WatcherRegistry::new())` in lib.rs.
pub struct WatcherRegistry(pub Mutex<HashMap<String, WatcherHandle>>);

impl WatcherRegistry {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

/// Resolve Goldberg's achievements.json path for a given Steam App ID.
///
/// Windows: %APPDATA%\Goldberg SteamEmu Saves\{app_id}\achievements.json
pub fn goldberg_achievements_path(steam_app_id: &str) -> Option<PathBuf> {
    dirs::data_dir().map(|base| {
        base.join("Goldberg SteamEmu Saves")
            .join(steam_app_id)
            .join("achievements.json")
    })
}

/// Start watching achievements.json for `game_id`.
///
/// If a watcher is already running for this game, it is replaced.
/// Returns an error if the Goldberg saves directory cannot be determined.
///
/// `on_change` is called with the raw JSON string on every file change.
/// It runs on a background thread — keep it fast and non-blocking.
pub fn start_watcher<F>(
    registry:    &WatcherRegistry,
    game_id:     String,
    steam_app_id: String,
    on_change:   F,
) -> Result<(), String>
where
    F: Fn(String) + Send + 'static,
{
    let watch_path = goldberg_achievements_path(&steam_app_id)
        .ok_or("Cannot resolve Goldberg save directory")?;

    // Ensure parent directory exists (Goldberg creates it on first launch,
    // but we create it now so notify can set up the watch immediately).
    if let Some(parent) = watch_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let watch_path_clone = watch_path.clone();
    let on_change = Arc::new(on_change);

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        let Ok(event) = res else { return };

        // Only react to file content changes.
        let is_modify = matches!(
            event.kind,
            EventKind::Modify(_) | EventKind::Create(_)
        );
        if !is_modify { return; }

        // Only fire when the achievements.json itself changed.
        let targets_our_file = event.paths.iter().any(|p| p == &watch_path_clone);
        if !targets_our_file { return; }

        // Read file content and pass to handler.
        match std::fs::read_to_string(&watch_path_clone) {
            Ok(json) => on_change(json),
            Err(e) => eprintln!("[watcher] Failed to read achievements.json: {e}"),
        }
    })
    .map_err(|e| format!("Failed to create watcher: {e}"))?;

    // Watch the parent directory (notify works better on dirs than single files).
    let watch_dir = watch_path
        .parent()
        .ok_or("achievements.json has no parent directory")?;

    watcher
        .watch(watch_dir, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Failed to start watching: {e}"))?;

    let handle = WatcherHandle { _watcher: watcher };

    let mut map = registry.0.lock().map_err(|_| "WatcherRegistry lock poisoned")?;
    map.insert(game_id, handle);

    Ok(())
}

/// Stop watching achievements.json for `game_id`.
///
/// Safe to call if no watcher is running (no-op).
pub fn stop_watcher(registry: &WatcherRegistry, game_id: &str) {
    if let Ok(mut map) = registry.0.lock() {
        map.remove(game_id); // dropping WatcherHandle stops the watcher
    }
}
```

### Update `src/steam_bridge/mod.rs`

```rust
//! Steam Bridge — achievement tracking infrastructure — Phase 5.

pub mod dll_swap;
pub mod achievement_watcher;
// T41 additions:
// pub mod achievement_router;
// pub mod steam_api;
```

### Wire `WatcherRegistry` into `src/lib.rs`

```rust
use crate::steam_bridge::achievement_watcher::WatcherRegistry;

// Inside the builder, after other .manage() calls:
.manage(WatcherRegistry::new())
```

### T40 Unit Tests (in `achievement_watcher.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    #[test]
    fn goldberg_path_contains_app_id() {
        let path = goldberg_achievements_path("570").unwrap();
        assert!(path.to_string_lossy().contains("570"));
        assert!(path.ends_with("achievements.json"));
    }

    #[test]
    fn start_and_stop_watcher_does_not_panic() {
        let registry = WatcherRegistry::new();
        let fired    = Arc::new(Mutex::new(false));
        let fired2   = fired.clone();

        // Use a temp dir as the "Goldberg saves" directory.
        let dir  = tempdir().unwrap();
        let path = dir.path().join("achievements.json");
        fs::write(&path, "{}").unwrap();

        // Manually start watcher pointing at the temp path.
        // (We bypass goldberg_path resolution here for testability.)
        let mut watcher = notify::recommended_watcher(move |_| {
            *fired2.lock().unwrap() = true;
        }).unwrap();
        watcher.watch(dir.path(), RecursiveMode::NonRecursive).unwrap();

        let handle = WatcherHandle { _watcher: watcher };
        registry.0.lock().unwrap().insert("test_game".into(), handle);

        stop_watcher(&registry, "test_game");
        assert!(registry.0.lock().unwrap().is_empty());
    }
}
```

### T40 Acceptance Criteria

- [ ] `cargo test` — all tests pass (including T38 and T39 tests)
- [ ] `WatcherRegistry` is registered as Tauri state in `lib.rs`
- [ ] `start_watcher` replaces an existing watcher if called twice for same `game_id`
- [ ] `stop_watcher` is a no-op for unknown `game_id`
- [ ] `goldberg_achievements_path` returns a path containing the App ID and ending in `achievements.json`
- [ ] `cargo check` — no new warnings

---

## Summary Checklist for Engineer

### T38
- [ ] Bump `CURRENT_SCHEMA_VERSION` to `8`
- [ ] Add `MIGRATION_008` SQL + `MIGRATION_008_ALTER`
- [ ] Wire migration into `run_migrations()`
- [ ] Add 2 migration unit tests
- [ ] Add TypeScript types to `api.ts`

### T39
- [ ] Create `src/steam_bridge/dll_swap.rs`
- [ ] Create `src/steam_bridge/mod.rs`
- [ ] Add `mod steam_bridge;` to `lib.rs`
- [ ] Place Goldberg `steam_api64.dll` in `resources/plugins/`
- [ ] Update `tauri.conf.json` bundle resources
- [ ] Add `tempfile = "3"` to `[dev-dependencies]`
- [ ] 5 unit tests pass

### T40
- [ ] Add `notify = "6"` and `dirs = "5"` to `Cargo.toml`
- [ ] Create `src/steam_bridge/achievement_watcher.rs`
- [ ] Update `steam_bridge/mod.rs` to pub `achievement_watcher`
- [ ] Add `WatcherRegistry::new()` to `lib.rs` managed state
- [ ] 2 unit tests pass
- [ ] Hand off to T41 (router + commands)

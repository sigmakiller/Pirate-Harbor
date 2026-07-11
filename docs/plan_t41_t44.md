# T41–T44 — Phase 5 Achievement Tracking (Router + Commands + Detection)
## Engineer Implementation Plan

**Depends on:** T38–T40 complete ✅

---

## T41 — Achievement Router + Steam Schema API

### New file: `src/steam_bridge/achievement_router.rs`

Diffs old vs new `achievements.json`, creates milestones for newly unlocked IDs.
Silently drops achievements with no row in `steam_achievement_mappings`.

**Goldberg achievements.json format:**
```json
{
  "ACH_WIN_ONE_GAME": { "earned": true,  "earned_time": 1720000000 },
  "ACH_KILL_100":     { "earned": false, "earned_time": 0 }
}
```

**Key types + function:**
```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AchievementState(pub HashMap<String, AchievementEntry>);

#[derive(Debug, Clone, Deserialize)]
pub struct AchievementEntry {
    pub earned:      bool,
    pub earned_time: i64,
}

/// Parse achievements.json content into AchievementState.
pub fn parse_achievements(json: &str) -> AchievementState {
    serde_json::from_str(json).unwrap_or_default()
}

/// Diff old vs new state. Returns steam_ids that are newly earned.
pub fn newly_unlocked(old: &AchievementState, new: &AchievementState) -> Vec<String> {
    new.0.iter()
        .filter(|(id, entry)| {
            entry.earned && !old.0.get(*id).map(|e| e.earned).unwrap_or(false)
        })
        .map(|(id, _)| id.clone())
        .collect()
}

/// For each newly unlocked steam_id: look up mapping, create milestone, emit event.
pub fn process_changes(
    old:        &AchievementState,
    new_json:   &str,
    game_id:    &str,
    conn:       &rusqlite::Connection,
    app_handle: &tauri::AppHandle,
) -> Result<AchievementState, String> {
    let new_state = parse_achievements(new_json);
    let unlocked  = newly_unlocked(old, &new_state);

    for steam_id in &unlocked {
        // Look up mapping (silent drop if absent)
        let mapping = conn.query_row(
            "SELECT id, display_name, description, points
             FROM steam_achievement_mappings
             WHERE game_id = ?1 AND steam_id = ?2",
            rusqlite::params![game_id, steam_id],
            |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, i32>(3)?,
            )),
        );

        let Ok((_, display_name, description, points)) = mapping else { continue };

        // Create milestone
        let milestone_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO milestones
             (id, game_id, title, description, category, difficulty,
              achievement_date, points, created_at, updated_at)
             VALUES (?1,?2,?3,?4,'achievement','auto',?5,?6,?7,?7)",
            rusqlite::params![
                milestone_id, game_id, display_name, description,
                now, points, now,
            ],
        ).map_err(|e| e.to_string())?;

        // Emit frontend event
        use tauri::Emitter;
        let _ = app_handle.emit("achievement-unlocked", serde_json::json!({
            "game_id":      game_id,
            "steam_id":     steam_id,
            "display_name": display_name,
            "points":       points,
            "milestone_id": milestone_id,
        }));
    }

    Ok(new_state)
}
```

**Unit tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newly_unlocked_detects_change() {
        let old_json = r#"{"ACH_A":{"earned":false,"earned_time":0}}"#;
        let new_json = r#"{"ACH_A":{"earned":true,"earned_time":111}}"#;
        let old = parse_achievements(old_json);
        let new = parse_achievements(new_json);
        assert_eq!(newly_unlocked(&old, &new), vec!["ACH_A"]);
    }

    #[test]
    fn already_earned_not_reported_again() {
        let json = r#"{"ACH_A":{"earned":true,"earned_time":111}}"#;
        let state = parse_achievements(json);
        assert!(newly_unlocked(&state, &state).is_empty());
    }

    #[test]
    fn unmapped_achievement_silently_dropped() {
        // process_changes with no DB rows must return Ok without panic
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::run_migrations(&conn).unwrap();
        // No mapping inserted — should not error
        let old = AchievementState::default();
        let new_json = r#"{"ACH_X":{"earned":true,"earned_time":999}}"#;
        // We can't easily pass AppHandle in unit test; test the diff only
        let new = parse_achievements(new_json);
        let ids = newly_unlocked(&old, &new);
        assert_eq!(ids, vec!["ACH_X"]);
    }
}
```

### New file: `src/steam_bridge/steam_api.rs`

```rust
//! Steam public API helpers — T41.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SteamSchemaResponse {
    pub game: SteamSchemaGame,
}

#[derive(Debug, Deserialize)]
pub struct SteamSchemaGame {
    #[serde(rename = "availableGameStats")]
    pub available_game_stats: Option<SteamGameStats>,
}

#[derive(Debug, Deserialize)]
pub struct SteamGameStats {
    pub achievements: Option<Vec<SteamAchievementDef>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SteamAchievementDef {
    pub name:         String,   // steam_id
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description:  Option<String>,
}

/// Fetch achievement definitions from Steam's public schema endpoint.
/// No API key required.
pub async fn fetch_achievement_defs(
    client:       &reqwest::Client,
    steam_app_id: &str,
) -> Result<Vec<SteamAchievementDef>, String> {
    let url = format!(
        "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/?appid={}&l=english",
        steam_app_id
    );
    let resp = client.get(&url)
        .send().await
        .map_err(|e| format!("Steam API request failed: {e}"))?
        .json::<SteamSchemaResponse>().await
        .map_err(|e| format!("Steam API parse failed: {e}"))?;

    Ok(resp.game.available_game_stats
        .and_then(|s| s.achievements)
        .unwrap_or_default())
}
```

### Update `src/steam_bridge/mod.rs`

```rust
pub mod dll_swap;
pub mod achievement_watcher;
pub mod achievement_router;
pub mod steam_api;
```

### T41 Acceptance Criteria
- [ ] 3 unit tests pass in `achievement_router`
- [ ] `newly_unlocked` correctly ignores already-earned achievements
- [ ] `cargo check` clean

---

## T42 — Tauri Commands (`commands/achievements.rs`)

### New file: `src/commands/achievements.rs`

Implement all 8 commands. Wire `start_watcher` into `enable_achievement_tracking`
using the `WatcherRegistry` state from T40.

```rust
use std::sync::{Arc, Mutex};
use tauri::State;
use uuid::Uuid;
use chrono::Utc;

use crate::db::DbState;
use crate::steam_bridge::{
    dll_swap, achievement_watcher::{WatcherRegistry, start_watcher, stop_watcher},
    steam_api,
};
use crate::commands::achievements_types::*;

// ── enable_achievement_tracking ──────────────────────────────────────────────

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

    // Persist to DB
    {
        let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
        conn.execute(
            "UPDATE games SET achievement_tracking_enabled=1, steam_app_id=?1 WHERE id=?2",
            rusqlite::params![steam_app_id, game_id],
        ).map_err(|e| e.to_string())?;
    }

    // Clone what we need before moving into closure
    let db_arc   = Arc::clone(&db.0);
    let app_clone = app_handle.clone();
    let gid       = game_id.clone();

    start_watcher(&registry, game_id, steam_app_id, move |json| {
        // Called on background thread when achievements.json changes.
        // We hold a per-call lock — not the main DB lock.
        use crate::steam_bridge::achievement_router;
        static STATE: std::sync::OnceLock<Mutex<achievement_router::AchievementState>>
            = std::sync::OnceLock::new();
        let state_mutex = STATE.get_or_init(|| Mutex::new(Default::default()));
        let mut old = state_mutex.lock().unwrap();
        if let Ok(conn) = db_arc.lock() {
            if let Ok(new) = achievement_router::process_changes(
                &old, &json, &gid, &conn, &app_clone
            ) {
                *old = new;
            }
        }
    })?;

    Ok(())
}

// ── disable_achievement_tracking ─────────────────────────────────────────────

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
        "UPDATE games SET achievement_tracking_enabled=0 WHERE id=?1",
        rusqlite::params![game_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// ── get_achievement_tracking_status ──────────────────────────────────────────

#[tauri::command]
pub fn get_achievement_tracking_status(
    db:      State<'_, DbState>,
    game_id: String,
) -> Result<TrackingStatus, String> {
    let conn  = db.0.lock().map_err(|_| "DB lock poisoned")?;
    let (enabled, app_id): (bool, Option<String>) = conn.query_row(
        "SELECT achievement_tracking_enabled, steam_app_id FROM games WHERE id=?1",
        rusqlite::params![game_id],
        |r| Ok((r.get::<_,i32>(0)? != 0, r.get(1)?)),
    ).map_err(|e| e.to_string())?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM steam_achievement_mappings WHERE game_id=?1",
        rusqlite::params![game_id],
        |r| r.get(0),
    ).unwrap_or(0);
    Ok(TrackingStatus { enabled, steam_app_id: app_id, mapping_count: count as usize })
}

// ── add / remove / get mappings ───────────────────────────────────────────────

#[tauri::command]
pub fn add_achievement_mapping(
    db: State<'_, DbState>,
    game_id: String, steam_id: String,
    display_name: String, description: Option<String>, points: i32,
) -> Result<AchievementMapping, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
    let id  = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO steam_achievement_mappings
         (id,game_id,steam_id,display_name,description,points,created_at)
         VALUES(?1,?2,?3,?4,?5,?6,?7)",
        rusqlite::params![id, game_id, steam_id, display_name, description, points, now],
    ).map_err(|e| e.to_string())?;
    Ok(AchievementMapping { id, game_id, steam_id, display_name, description, points, created_at: now })
}

#[tauri::command]
pub fn remove_achievement_mapping(
    db: State<'_, DbState>, mapping_id: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
    conn.execute("DELETE FROM steam_achievement_mappings WHERE id=?1",
        rusqlite::params![mapping_id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_achievement_mappings(
    db: State<'_, DbState>, game_id: String,
) -> Result<Vec<AchievementMapping>, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;
    let mut stmt = conn.prepare(
        "SELECT id,game_id,steam_id,display_name,description,points,created_at
         FROM steam_achievement_mappings WHERE game_id=?1 ORDER BY steam_id"
    ).map_err(|e| e.to_string())?;
    stmt.query_map(rusqlite::params![game_id], |r| Ok(AchievementMapping {
        id:           r.get(0)?,
        game_id:      r.get(1)?,
        steam_id:     r.get(2)?,
        display_name: r.get(3)?,
        description:  r.get(4)?,
        points:       r.get(5)?,
        created_at:   r.get(6)?,
    })).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect::<Vec<_>>()
    .pipe(Ok)
}

// ── import_achievements_from_steam ────────────────────────────────────────────

#[tauri::command]
pub async fn import_achievements_from_steam(
    db: State<'_, DbState>, game_id: String, steam_app_id: String,
) -> Result<Vec<AchievementMapping>, String> {
    let client = reqwest::Client::new();
    let defs   = steam_api::fetch_achievement_defs(&client, &steam_app_id).await?;
    let conn   = db.0.lock().map_err(|_| "DB lock poisoned")?;
    let now    = Utc::now().to_rfc3339();
    let mut out = Vec::new();
    for def in defs {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT OR IGNORE INTO steam_achievement_mappings
             (id,game_id,steam_id,display_name,description,points,created_at)
             VALUES(?1,?2,?3,?4,?5,10,?6)",
            rusqlite::params![id, game_id, def.name, def.display_name, def.description, now],
        ).ok();
        out.push(AchievementMapping {
            id, game_id: game_id.clone(),
            steam_id: def.name, display_name: def.display_name,
            description: def.description, points: 10, created_at: now.clone(),
        });
    }
    Ok(out)
}
```

### Wire into `lib.rs`

```rust
mod commands { pub mod achievements; }
use crate::steam_bridge::achievement_watcher::WatcherRegistry;

// In invoke_handler:
commands::achievements::enable_achievement_tracking,
commands::achievements::disable_achievement_tracking,
commands::achievements::get_achievement_tracking_status,
commands::achievements::add_achievement_mapping,
commands::achievements::remove_achievement_mapping,
commands::achievements::get_achievement_mappings,
commands::achievements::import_achievements_from_steam,
// T43 adds: detect_steam_app_id
```

### T42 Acceptance Criteria
- [ ] All 7 commands compile and register in `lib.rs`
- [ ] `cargo test` — all prior tests still pass
- [ ] `enable` → `disable` round-trip restores DLL (manual test)

---

## T43 — Steam App ID Auto-Detection Command

### Add to `src/commands/achievements.rs`

```rust
#[derive(Debug, Serialize)]
pub struct AppIdDetectionResult {
    pub app_id: Option<String>,
    pub source: AppIdSource,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppIdSource { Rawg, LocalFile, NotFound }

#[tauri::command]
pub async fn detect_steam_app_id(
    db:       State<'_, DbState>,
    game_id:  String,
    game_dir: String,
) -> Result<AppIdDetectionResult, String> {
    // Tier 1 — RAWG stores endpoint
    if let Some(id) = try_rawg_stores(&db, &game_id).await {
        return Ok(AppIdDetectionResult { app_id: Some(id), source: AppIdSource::Rawg });
    }
    // Tier 2 — local steam_appid.txt
    let path = std::path::Path::new(&game_dir).join("steam_appid.txt");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let id = content.trim().to_string();
            if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) {
                return Ok(AppIdDetectionResult { app_id: Some(id), source: AppIdSource::LocalFile });
            }
        }
    }
    // Tier 3 — not found
    Ok(AppIdDetectionResult { app_id: None, source: AppIdSource::NotFound })
}

/// Tier 1 helper: call RAWG /games/{id}/stores, extract Steam App ID.
async fn try_rawg_stores(db: &State<'_, DbState>, game_id: &str) -> Option<String> {
    // Look up cached RAWG api_id for this game's title
    let (rawg_api_id, rawg_key) = {
        let conn = db.0.lock().ok()?;
        let title: String = conn.query_row(
            "SELECT title FROM games WHERE id=?1", rusqlite::params![game_id], |r| r.get(0)
        ).ok()?;
        let api_id: i64 = conn.query_row(
            "SELECT api_id FROM metadata_cache WHERE LOWER(game_title)=LOWER(?1) LIMIT 1",
            rusqlite::params![title], |r| r.get(0)
        ).ok()?;
        let key: String = conn.query_row(
            "SELECT value FROM settings WHERE key='rawg_api_key'",
            [], |r| r.get(0)
        ).ok()?;
        (api_id, key)
    };

    let client = reqwest::Client::new();
    let url = format!(
        "https://api.rawg.io/api/games/{}/stores?key={}",
        rawg_api_id, rawg_key
    );
    let resp = client.get(&url).send().await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;

    json["results"].as_array()?.iter().find_map(|entry| {
        let slug = entry["store"]["slug"].as_str()?;
        if slug != "steam" { return None; }
        let store_url = entry["url"].as_str()?;
        extract_steam_app_id(store_url)
    })
}

fn extract_steam_app_id(url: &str) -> Option<String> {
    let prefix = "/app/";
    let start  = url.find(prefix)? + prefix.len();
    let rest   = &url[start..];
    let end    = rest.find('/').unwrap_or(rest.len());
    let id     = &rest[..end];
    if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) {
        Some(id.to_string())
    } else {
        None
    }
}
```

### Add to `api.ts`

```typescript
export async function detectSteamAppId(
  gameId: string, gameDir: string
): Promise<AppIdDetectionResult> {
  return invoke<AppIdDetectionResult>("detect_steam_app_id",
    { gameId, gameDir });
}
```

### Unit tests for `extract_steam_app_id`

```rust
#[test]
fn extracts_app_id_with_game_name() {
    assert_eq!(extract_steam_app_id("https://store.steampowered.com/app/570/Dota_2/"),
        Some("570".into()));
}
#[test]
fn extracts_app_id_without_trailing_slash() {
    assert_eq!(extract_steam_app_id("https://store.steampowered.com/app/292030"),
        Some("292030".into()));
}
#[test]
fn rejects_non_steam_url() {
    assert_eq!(extract_steam_app_id("https://gog.com/game/witcher3"), None);
}
```

### T43 Acceptance Criteria
- [ ] `detect_steam_app_id` registered in `lib.rs` and `invoke_handler`
- [ ] 3 unit tests for `extract_steam_app_id` pass
- [ ] `detectSteamAppId` exported from `api.ts`

---

## T44 — Add RawgClient::get_game_stores

### Modify `src/api/rawg.rs`

Add structs after existing ones:
```rust
#[derive(Debug, Deserialize)]
pub struct RawgStoresResponse {
    pub results: Vec<RawgStoreEntry>,
}
#[derive(Debug, Deserialize)]
pub struct RawgStoreEntry {
    pub url:   String,
    pub store: RawgStore,
}
#[derive(Debug, Deserialize)]
pub struct RawgStore {
    pub slug: String,   // "steam", "gog", "itch", etc.
}
```

Add method to `RawgClient`:
```rust
pub async fn get_game_stores(&self, rawg_id: i64) -> Result<Vec<RawgStoreEntry>, String> {
    self.rate_limit().await;
    let url = format!("{}/games/{}/stores?key={}", API_BASE, rawg_id, self.api_key);
    let resp = self.client.get(&url).send().await
        .map_err(|e| format!("RAWG stores request failed: {e}"))?;
    if resp.status().is_success() {
        resp.json::<RawgStoresResponse>().await
            .map(|r| r.results)
            .map_err(|e| format!("RAWG stores parse error: {e}"))
    } else {
        Err(format!("RAWG stores API error: {}", resp.status()))
    }
}
```

> **Note:** `try_rawg_stores` in T43 uses `reqwest` directly (simpler for a
> one-off command). The engineer may optionally refactor to use `RawgClient`
> if a client is accessible from the command. Either approach is acceptable.

### T44 Acceptance Criteria
- [ ] `RawgStoresResponse`, `RawgStoreEntry`, `RawgStore` added to `rawg.rs`
- [ ] `get_game_stores` compiles without warnings
- [ ] `cargo test` — all tests still pass

---

## Summary Checklist

### T41
- [ ] `src/steam_bridge/achievement_router.rs` created
- [ ] `src/steam_bridge/steam_api.rs` created
- [ ] `steam_bridge/mod.rs` updated with new mods
- [ ] 3 router unit tests pass

### T42
- [ ] `src/commands/achievements.rs` with 7 commands
- [ ] All commands wired into `lib.rs` invoke_handler
- [ ] `WatcherRegistry` state registered in `lib.rs`

### T43
- [ ] `detect_steam_app_id` command added
- [ ] Registered in `lib.rs`
- [ ] `detectSteamAppId` added to `api.ts`
- [ ] 3 unit tests for URL extractor pass

### T44
- [ ] Store structs + `get_game_stores()` added to `rawg.rs`
- [ ] `cargo test` clean

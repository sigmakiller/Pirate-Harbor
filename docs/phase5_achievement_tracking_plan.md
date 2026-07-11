# Phase 5 — Automated Achievement Tracking
## Implementation Plan (APPROVED + Updated)

**Author:** Architect  
**Status:** ✅ APPROVED  
**Scope:** T38–T42  
**Updated:** Steam App ID auto-detection added

---

## Approved Decisions

| Decision | Resolution |
|----------|-----------|
| IPC approach | **File-watching** (`notify` crate on `achievements.json`) |
| DLL type | **Stock pre-compiled Goldberg** 64-bit binary |
| 32-bit support | **Out of scope** |
| Unmapped achievements | **Silently dropped** |
| Achievement display names | **Steam public Web API** (`GetSchemaForGame`) |
| Steam App ID entry | **Auto-detected** (3-tier cascade, manual as last fallback) |

---

## Steam App ID Auto-Detection (New)

When the user opens achievement tracking for a game, the backend runs a
**3-tier detection cascade** and returns the App ID with its source label.
The frontend auto-fills the input field; the user only types manually if
all automated methods fail.

### Detection Cascade

```
detect_steam_app_id(game_id, game_dir)
        │
        ▼ Tier 1 — RAWG Stores Endpoint
        │  Does metadata_cache have an api_id for this game?
        │    YES → GET /games/{api_id}/stores (RAWG API)
        │          Parse response for store.slug == "steam"
        │          Regex: /app/(\d+)/ on the store URL
        │          → return { app_id, source: "rawg" }
        │    NO  → fall through
        │
        ▼ Tier 2 — Local File Scan
        │  Does game_dir/steam_appid.txt exist?
        │    YES → read file, trim, validate all-digits
        │          → return { app_id, source: "local_file" }
        │    NO  → fall through
        │
        ▼ Tier 3 — Manual Input
           → return { app_id: null, source: "not_found" }
           UI shows input field, user types manually
```

### Architectural Note — RAWG Stores Endpoint

> [!IMPORTANT]
> The existing `RawgGame` struct in `api/rawg.rs` does **not** include store
> data. The RAWG `/games/{id}/stores` endpoint is a **separate API call**
> from the game detail endpoint. The engineer must:
>
> 1. Add a new `get_game_stores(api_id: i64)` method to `RawgClient`
> 2. Add new response structs `RawgStoresResponse` and `RawgStoreEntry`
> 3. The `api_id` to use is stored in `metadata_cache.api_id` — look it
>    up by joining on `games.title`

### New Structs in `api/rawg.rs`

```rust
/// Response from GET /games/{id}/stores
#[derive(Debug, Deserialize)]
pub struct RawgStoresResponse {
    pub results: Vec<RawgStoreEntry>,
}

#[derive(Debug, Deserialize)]
pub struct RawgStoreEntry {
    pub store_id: i64,
    pub url: String,      // full store page URL
    pub store: RawgStore,
}

#[derive(Debug, Deserialize)]
pub struct RawgStore {
    pub id:   i64,
    pub name: String,
    pub slug: String,     // "steam", "gog", "itch", etc.
}
```

### New Method in `RawgClient`

```rust
/// Fetch store URLs for a game. Used to extract Steam App ID.
pub async fn get_game_stores(&self, rawg_id: i64) 
    -> Result<Vec<RawgStoreEntry>, String>;
```

### Steam App ID Extraction (Regex)

Steam store URLs follow the pattern:
```
https://store.steampowered.com/app/570/Dota_2/
                                   ^^^
```

Regex: `r"/app/(\d+)"` — capture group 1 is the App ID.

No additional crate needed — use Rust's standard `std::str::find` + slice,
or add the lightweight `regex` crate if the engineer prefers:

```toml
# Cargo.toml — only if engineer chooses regex crate
regex = { version = "1", default-features = false, features = ["std"] }
```

Simple string extraction (no crate):
```rust
fn extract_steam_app_id(url: &str) -> Option<String> {
    let prefix = "/app/";
    let start = url.find(prefix)? + prefix.len();
    let rest = &url[start..];
    let end = rest.find('/').unwrap_or(rest.len());
    let id = &rest[..end];
    if id.chars().all(|c| c.is_ascii_digit()) && !id.is_empty() {
        Some(id.to_string())
    } else {
        None
    }
}
```

---

## New Tauri Command: `detect_steam_app_id`

**File:** `src/commands/achievements.rs`

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AppIdDetectionResult {
    /// The detected App ID, or None if not found.
    pub app_id: Option<String>,
    /// Where the ID came from.
    pub source: AppIdSource,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppIdSource {
    Rawg,       // extracted from RAWG stores endpoint
    LocalFile,  // found steam_appid.txt in game directory
    NotFound,   // user must enter manually
}

/// Auto-detect the Steam App ID for a game using a 3-tier cascade.
///
/// Called automatically when the user opens the achievement tracking
/// section in EditGamePage. The frontend uses the result to pre-fill
/// the Steam App ID input.
#[tauri::command]
pub async fn detect_steam_app_id(
    db:       State<'_, DbState>,
    game_id:  String,
    game_dir: String,        // parent directory of the game's exe
) -> Result<AppIdDetectionResult, String>;
```

---

## Updated Command Table

| Command | Params | Returns |
|---------|--------|---------|
| `detect_steam_app_id` | `game_id, game_dir` | `AppIdDetectionResult` ← **NEW** |
| `enable_achievement_tracking` | `game_id, exe_path, steam_app_id` | `Result<(), String>` |
| `disable_achievement_tracking` | `game_id, exe_path` | `Result<(), String>` |
| `get_achievement_tracking_status` | `game_id` | `Result<TrackingStatus, String>` |
| `add_achievement_mapping` | `game_id, steam_id, display_name, description, points` | `Result<AchievementMapping, String>` |
| `remove_achievement_mapping` | `mapping_id` | `Result<(), String>` |
| `get_achievement_mappings` | `game_id` | `Result<Vec<AchievementMapping>, String>` |
| `import_achievements_from_steam` | `game_id, steam_app_id` | `Result<Vec<AchievementMapping>, String>` |

---

## Updated Frontend Spec — EditGamePage.tsx

The Achievement Tracking section now auto-runs detection on mount and
shows a source label next to the auto-filled field.

```
┌─ Achievement Tracking ─────────────────────────────────────┐
│  ℹ Only for games with steam_api64.dll.                    │
│                                                            │
│  [●] Enable Automated Achievement Tracking                 │
│                                                            │
│  Steam App ID                                              │
│  [  570         ] ✓ Auto-detected via RAWG                 │
│                  ↑ label changes per source:               │
│                    "✓ Auto-detected via RAWG"              │
│                    "✓ Found in game folder"                │
│                    (no label — user types manually)        │
│                                                            │
│  [Import Achievement Names from Steam ↓]                  │
│   (button enabled only when App ID is present)            │
│                                                            │
│  Mappings (3)                                             │
│  ┌────────────────────────────┬──────────────────┬──────┐  │
│  │ ACH_WIN_ONE_GAME           │ Win Your First   │  10  │  │
│  │ ACH_KILL_100               │ Slayer           │  25  │  │
│  └────────────────────────────┴──────────────────┴──────┘  │
│  [+ Add Mapping]                                           │
└────────────────────────────────────────────────────────────┘
```

**Frontend logic on section open:**
```typescript
// On mount of achievement section:
const result = await detectSteamAppId(gameId, gameDir);
if (result.app_id) {
  setSteamAppId(result.app_id);
  setDetectionSource(result.source); // shows label
}
// else: input stays empty, user types manually
```

---

## Database Schema — Migration 008 (unchanged)

```sql
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

ALTER TABLE games ADD COLUMN achievement_tracking_enabled INTEGER NOT NULL DEFAULT 0;
ALTER TABLE games ADD COLUMN steam_app_id TEXT;
```

---

## Module Tree (Final)

```
src/steam_bridge/
├── mod.rs                  — public API: enable/disable/detect/start_watcher
├── dll_swap.rs             — backup / inject Goldberg / restore
├── achievement_watcher.rs  — notify-crate file watcher
├── achievement_router.rs   — diff achievements.json → create milestones
└── steam_api.rs            — GetSchemaForGame HTTP call

src/commands/
└── achievements.rs         — all 8 Tauri commands including detect_steam_app_id

src/api/
└── rawg.rs                 — add get_game_stores() + new store structs
```

---

## New Cargo Dependencies

```toml
notify = { version = "6", features = ["serde"] }
dirs   = "5"
# regex = "1"   ← optional; plain string parsing is sufficient
```

---

## Task Breakdown (Final)

| Task | Title | Key deliverables |
|------|-------|-----------------|
| **T38** | DB Migration + Types | `MIGRATION_008`, bump schema to v8, TS types incl. `AppIdDetectionResult` |
| **T39** | DLL Swap Module | `dll_swap.rs`, bundle Goldberg in `resources/plugins/`, `SwapState` enum |
| **T40** | File Watcher | `achievement_watcher.rs`, `WatcherRegistry` Tauri state, `notify` dep |
| **T41** | Router + Commands | `achievement_router.rs`, all 8 commands, `get_game_stores()` in `rawg.rs`, `detect_steam_app_id` |
| **T42** | Frontend UI + Toast | Achievement section in `EditGamePage.tsx`, auto-fill UX, `achievement-unlocked` toast |

---

## Verification Plan

### Automated Tests
- `dll_swap::verify_swap_integrity()` — all 4 `SwapState` branches (mock dirs)
- `achievement_router::process_changes()` — mapped unlocked → milestone, unmapped → silently dropped
- `extract_steam_app_id()` — various URL formats including trailing slash and without game name
- `detect_steam_app_id` — mock: RAWG returns store URL (Tier 1 hit), no cache + file present (Tier 2), neither (Tier 3)
- Migration 008 idempotency

### Manual Verification
1. Game with RAWG metadata → open achievement section → App ID auto-fills with "✓ Auto-detected via RAWG"
2. Game without RAWG but with `steam_appid.txt` → auto-fills with "✓ Found in game folder"
3. Game with neither → input blank, user types manually
4. Import from Steam → mappings populate with display names
5. Enable tracking → DLL swapped, watcher starts
6. Trigger in-game achievement → toast appears, milestone in DB
7. Disable tracking → original DLL restored

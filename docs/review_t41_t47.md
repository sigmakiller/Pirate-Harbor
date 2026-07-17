# Review — T41–T47 Phase 5 Achievement Tracking
**Reviewer:** Architect  
**Date:** 2026-07-17  
**Build:** ✅ `cargo test` — 79/79 pass | `cargo check` — clean

---

## Overall Verdict

| Area | Score | Notes |
|------|-------|-------|
| Correctness | ⭐⭐⭐⭐ | 1 real bug (OnceLock), rest solid |
| Architecture | ⭐⭐⭐⭐⭐ | Follows plan exactly |
| Test coverage | ⭐⭐⭐⭐ | 6 new tests; router process_changes not directly tested |
| Frontend | ⭐⭐⭐⭐ | 1 missing CSS rule |
| Wiring | ⭐⭐⭐⭐⭐ | All 8 commands registered in lib.rs ✅ |

**Decision: APPROVED with 1 must-fix bug and 2 moderate issues.**

---

## ✅ What Was Implemented Correctly

- **T41 router** — `newly_unlocked()` diff is correct, sorts output for determinism, silently skips unmapped achievements ✅
- **T41 steam_api** — `SteamSchemaResponse` struct and `fetch_achievement_defs()` match Steam API format ✅
- **T42 commands** — All 7 commands implemented correctly; `INSERT OR REPLACE` on `add_achievement_mapping`, `INSERT OR IGNORE` on `import_achievements_from_steam` ✅
- **T42 wiring** — All 8 commands (including T43 `detect_steam_app_id`) registered in `lib.rs` invoke_handler ✅
- **T43 detection** — 3-tier cascade correct; `extract_steam_app_id` handles both URL formats; 3 unit tests pass ✅
- **T43 RAWG tier** — Reads `api_id` from `metadata_cache`, looks up `rawg_api_key` from `settings` ✅
- **T44 rawg.rs** — `RawgStoresResponse`, `RawgStoreEntry`, `RawgStore` structs added; `get_game_stores()` method added ✅
- **T45 EditGamePage** — Section renders correctly; toggle disabled when no exe_path; auto-detect fires on mount; add/remove mappings work ✅
- **T46 App.tsx** — `listen("achievement-unlocked")` registered at App root, cleans up on unmount; `ToastType` extended with `"achievement"` ✅
- **T47 GameDetailPage** — `getMilestones(id, "achievement")` called in `Promise.allSettled`; panel auto-hides when empty ✅

---

## 🔴 M1 — Bug: `OnceLock` is `static`, shared across all watcher closures

**File:** `commands/achievements.rs` lines 109–111  
**Severity:** Moderate-High — causes **duplicate milestone suppression across games**

**Problem:**  
```rust
static STATE: std::sync::OnceLock<Mutex<AchievementState>> = std::sync::OnceLock::new();
```
`static` items are shared **globally** across the entire process. If the user enables tracking on Game A and then Game B, both closures share the same `OnceLock` cell. The second call to `enable_achievement_tracking` does **not** create a new `OnceLock` — it reuses the first one initialised by Game A's closure. This means Game B's achievements are diffed against Game A's last-known state.

**Fix:** Replace `OnceLock` with a `Mutex<AchievementState>` captured directly in the closure:

```rust
// In enable_achievement_tracking, replace the OnceLock block with:
let state_mutex = std::sync::Arc::new(std::sync::Mutex::new(
    crate::steam_bridge::achievement_router::AchievementState::default()
));

start_watcher(&registry, game_id, steam_app_id, move |json| {
    use crate::steam_bridge::achievement_router;
    let mut old = state_mutex.lock().unwrap();
    let Ok(conn) = rusqlite::Connection::open(&db_path) else { return };
    if let Ok(new_state) = achievement_router::process_changes(
        &old, &json, &gid, &conn, &app_clone,
    ) {
        *old = new_state;
    }
})?;
```

The `Arc<Mutex<AchievementState>>` is captured by value into the closure. Each call to `enable_achievement_tracking` creates a new `Arc`, so each game watcher gets its own independent state snapshot.

---

## 🟡 M2 — Achievement Toast Variant Missing CSS

**File:** `src/index.css` — no `achievement` class rule found  
**Severity:** Moderate — toast renders but with no visual distinction from `info`

The `ToastContainer.tsx` sets `color: "#fbbf24"` on the icon span (line 24) but the overall toast box (`background`, `border`) is not styled differently. The T46 plan called for:

```css
/* Add to index.css toast section */
.toast-achievement {
  background: linear-gradient(135deg, #1c1917 0%, #2d1a4a 100%);
  border-color: #7c3aed;
}
```

**Determine** how the toast component applies variant styles (inline or CSS class) and add the achievement background gradient accordingly.

---

## 🟡 M3 — `import_achievements_from_steam` Returns Only Newly Inserted Rows

**File:** `commands/achievements.rs` lines 292–302  
**Severity:** Moderate — UX issue, not a data correctness bug

The command returns only rows where `rows_changed > 0` (i.e., new inserts). If the user clicks "Import from Steam" a second time, they get an **empty list** back, which the frontend uses to call `setMappings(maps)` — wiping the display of all previously imported mappings.

**Fix:** After bulk-insert, fetch and return the full list:
```rust
// Replace the Ok(inserted) line with:
drop(conn); // release lock before re-querying
get_achievement_mappings(db, game_id).await
// Or inline the SELECT query
```

Alternatively, the frontend can handle this: don't call `setMappings(maps)` if `maps` is empty — but fixing it at the command level is cleaner.

---

## 🔵 m4 — Minor: `resolve_db_path` helper is fragile

**File:** `commands/achievements.rs` lines 55–64  
**Severity:** Minor

`resolve_db_path` reads `app_data_dir` from the `settings` table. This is the same C1 issue fixed in T32–T37 (the key must be written at startup). If the app startup fix is in place this is fine, but the helper should at least document this precondition in a comment.

**Fix:** Add a doc comment:
```rust
/// Reads `app_data_dir` from the settings table.
///
/// **Precondition:** `lib.rs` setup() must have written this key at startup.
/// See T32 fix in backup.rs.
fn resolve_db_path(conn: &rusqlite::Connection) -> Result<PathBuf, String> {
```

---

## 🔵 m5 — Minor: `#[allow(dead_code)]` module-level suppressor in T41 modules

**Files:** `achievement_router.rs` line 17, `steam_api.rs` line 7

Both files suppress all dead code warnings with a module-level `#![allow(dead_code)]`. This was the T26–T31 pattern for stubs, but these modules are now **fully wired** — their public items are called from `achievements.rs`. The suppressors should be removed so the compiler can catch any new dead code going forward.

**Fix:**
```rust
// Delete these lines from both files:
#![allow(dead_code)]
```

---

## 🔵 m6 — Minor: `import_achievements_from_steam` is async but re-locks DB after fetch

**File:** `commands/achievements.rs` line 277  
The `db.0.lock()` is held across the entire `for def in defs` loop while inserting rows. This is fine for correctness (SQLite is single-writer) but the lock is held longer than necessary.

No fix required — this is acceptable for Phase 5. Flag for future optimization if import lists grow large.

---

## Summary

| Issue | Severity | File | Action |
|-------|----------|------|--------|
| M1 — `OnceLock` shared across games | **Must Fix** | `commands/achievements.rs:109` | Replace with per-closure `Arc<Mutex<>>` |
| M2 — Achievement toast missing CSS | Moderate | `src/index.css` | Add gradient background rule |
| M3 — Import returns empty on second call | Moderate | `commands/achievements.rs:305` | Return full list after insert |
| m4 — `resolve_db_path` missing precondition doc | Minor | `commands/achievements.rs:55` | Add comment |
| m5 — Stale `#![allow(dead_code)]` in T41 modules | Minor | router + steam_api | Remove the suppressors |
| m6 — DB lock held during bulk insert loop | Informational | `commands/achievements.rs:277` | No action needed |

# Review — T49–T58 Phase 6 (Live Intelligence, Identity, Distribution, Hardening)
**Reviewer:** Architect  
**Date:** 2026-07-25  
**Build:** ✅ `cargo test` — 86/86 pass | `cargo check` — 1 warning (see M1)

---

## Overall Verdict

| Area | Score | Notes |
|------|-------|-------|
| Correctness | ⭐⭐⭐⭐ | 1 real bug (M1 dead field), 1 URL inconsistency (M2) |
| Architecture | ⭐⭐⭐⭐⭐ | All 10 tasks implemented, structure matches plan |
| Test coverage | ⭐⭐⭐⭐⭐ | 86 tests; T54 streak tests robust; T58 performance tests meet all 3 targets |
| Frontend | ⭐⭐⭐⭐⭐ | Year-in-Review, heatmap, timeline, streak card all wired |
| Distribution | ⭐⭐⭐⭐ | Updater config correct; signing keypair present; NSIS configured |

**Decision: APPROVED with 1 must-fix and 2 moderate issues.**

---

## ✅ What Was Implemented Correctly

- **T49** — `AutoBackupJob` and `MetadataRefreshJob` queued at startup; `auto_backup_enabled` setting respected ✅
- **T50** — `BulkEnrichmentJob` moves bulk enrichment off the main thread; `start_bulk_enrichment_job` command returns immediately; 5-game-per-tick rate limiting present ✅
- **T51** — `get_stale_games_count` queries games with no valid `metadata_cache` entry; 24-hour dismiss window correctly implemented using `Date.now()` arithmetic ✅
- **T52** — `YearInReviewPage.tsx` created; `/identity/year-in-review` route registered in `App.tsx`; `getSessionYears()` → `availableYears[0] ?? currentYear` default logic matches approved spec ✅
- **T53** — `IdentityHeatmap` (52×7 grid) and `IdentityMilestoneTimeline` components implemented; both called from `IdentityPage.tsx`; graceful empty state ✅
- **T54** — `build_milestone_streak_stats` algorithm is correct: anchors to today/yesterday, uses `HashSet` for O(1) lookup, sliding window for longest streak; 4 unit tests pass ✅
- **T55** — `tauri_plugin_updater` registered; `check_for_updates` command in `commands/updater.rs` never throws (always returns `available: false` on any error) ✅
- **T55** — Signing pubkey present in `tauri.conf.json` (base64-encoded minisign key); GitHub Actions release workflow committed ✅
- **T56** — `targets: ["nsis", "msi"]` set; NSIS `installMode: "currentUser"`, `startMenuFolder` set; custom icons replaced ✅
- **T57** — 5 s delayed update check in `App.tsx`; `clearTimeout` cleanup on unmount; `duration: 10000` gives user time to act ✅
- **T57** — SettingsPage `Updates` section: version display, "Check Now" button, inline changelog from `releases/latest.json` ✅
- **T58** — Three performance tests added and passing: FTS5 at 5000 games (<100ms), heatmap at 5000 sessions (<200ms), year-in-review at 5000 sessions (<500ms) ✅

---

## 🔴 M1 — Dead field warning: `AchievementEntry::earned_time`

**File:** `steam_bridge/achievement_router.rs` line 37  
**Severity:** Moderate — compiler emits `warning: field 'earned_time' is never read`

The `earned_time` field is deserialized from Goldberg JSON but never accessed in application code. It was kept because it was part of the original schema spec, but the milestone creation uses `chrono::Utc::now()` instead of the Goldberg-reported timestamp.

**Decision required:** Pick one:

**Option A — Use it** (preferred): Pass `earned_time` to the milestone `achievement_date` field. This gives more accurate timestamps (the game's actual achievement trigger time rather than when the file watcher fired).

```rust
// In process_changes(), replace:
let now = chrono::Utc::now().to_rfc3339();
// With:
let earned_ts = new_state.0.get(steam_id)
    .map(|e| e.earned_time)
    .unwrap_or(0);
let ach_date = if earned_ts > 0 {
    chrono::DateTime::from_timestamp(earned_ts, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
} else {
    chrono::Utc::now().to_rfc3339()
};
```

**Option B — Suppress** (minor): Add `#[allow(dead_code)]` to the field only (not the whole struct).

```rust
pub struct AchievementEntry {
    pub earned: bool,
    #[allow(dead_code)]
    pub earned_time: i64,
}
```

**Recommendation: Option A.** Using the Goldberg timestamp gives users a correct unlock history that survives watcher delays.

---

## 🟡 M2 — Changelog fetch URL in SettingsPage differs from plan

**File:** `src/pages/SettingsPage.tsx` line 113  
**Severity:** Moderate — will fail to fetch the changelog in production

The changelog fetch in `handleCheckForUpdates` uses:
```
https://raw.githubusercontent.com/sigmakiller/Pirate-Harbor/main/releases/latest.json
```

But the approved plan (and `tauri.conf.json` updater endpoint) uses the **GitHub Releases** URL:
```
https://github.com/sigmakiller/Pirate-Harbor/releases/latest/download/latest.json
```

`raw.githubusercontent.com/main/releases/latest.json` points to whatever is committed to the `main` branch, not the actual release asset. On GitHub Releases, the file is attached to a specific tag — not to `main`. These are two different files with potentially different content.

**Fix:** Use the same URL as the updater:
```typescript
const resp = await fetch(
  "https://github.com/sigmakiller/Pirate-Harbor/releases/latest/download/latest.json"
);
```

Or better — reuse the `notes` field already returned by `checkForUpdates()` (it comes from the same manifest) rather than making a second HTTP call:
```typescript
const result = await checkForUpdates();
setUpdateResult(result);
if (result.notes) setChangelog(result.notes);  // no second fetch needed
```

---

## 🟡 M3 — `STALE_DAYS` constant declared after its first use

**File:** `commands/metadata.rs` — `STALE_DAYS` declared at line 556, first used at line 423  
**Severity:** Minor — Rust resolves module-level constants regardless of declaration order, so this **compiles correctly**. However it violates the codebase convention of declaring constants before use and makes the code hard to read. A reviewer looking at `get_stale_games_count` on line 421 has to scroll 130 lines down to find the value.

**Fix:** Move `const STALE_DAYS: i64 = 30;` to the top of the file, just after the `use` imports.

---

## 🔵 m4 — Minor: `get_stale_games_count` SQL JOIN condition is asymmetric with BulkEnrichmentJob

**File:** `commands/metadata.rs` lines 427–434

`get_stale_games_count` uses:
```sql
LEFT JOIN metadata_cache mc ON LOWER(mc.game_title) = LOWER(g.title)
  AND mc.expires_at > ?1
WHERE mc.id IS NULL
```

This counts a game as stale if its cache entry **is expired** OR **doesn't exist**. This is correct.

However, `BulkEnrichmentJob` (T50) uses a slightly different query with `STALE_DAYS` at line 574 to find games to enrich. Verify both queries produce the same result set — if they diverge, the banner count will not match the actual number of games refreshed. (No code change required if they match; this is an audit note for the engineer to verify.)

---

## 🔵 m5 — Minor: `tauri.conf.json` missing `active: true` in updater section

**File:** `apps/desktop/src-tauri/tauri.conf.json`

Per Tauri v2 docs, the `updater` plugin section should include `"active": true` to be explicit about enabling updates. The plan spec included it; the implementation omitted it.

```json
"updater": {
  "active": true,          // ← missing
  "endpoints": [...],
  "dialog": false,
  "pubkey": "..."
}
```

In Tauri v2 the plugin is activated by registering it in `lib.rs` (`tauri_plugin_updater::Builder::new().build()`) regardless of the config key, so this is non-breaking — but the config should match the documented schema to avoid confusion.

---

## 🔵 m6 — Security: Private key commit removed but history not purged

**Commit:** `3ed669b chore: remove accidentally committed private key from staging`

The private key was committed and then removed in a follow-up commit. The key is still present in git history (accessible via `git show 3ed669b~1`). If the repository is public or becomes public, the key should be considered compromised.

**Action required (outside code review):**
1. Generate a new keypair: `tauri signer generate -w pirate_harbor.key`
2. Update `tauri.conf.json` with the new public key
3. Store the new private key **only** in GitHub Actions secrets
4. Optionally rewrite history with `git filter-repo` to remove the old key from all commits

---

## Summary Table

| Issue | Severity | File | Action |
|-------|----------|------|--------|
| M1 — `earned_time` never used (dead field warning) | **Must Fix** | `achievement_router.rs:37` | Use for milestone date (Option A recommended) |
| M2 — Changelog URL differs from updater endpoint | Moderate | `SettingsPage.tsx:113` | Use same URL or reuse `result.notes` |
| M3 — `STALE_DAYS` declared after first use | Minor | `commands/metadata.rs:556` | Move const to top of file |
| m4 — Stale query vs enrich query asymmetry | Informational | `commands/metadata.rs` | Verify both queries match |
| m5 — `updater.active` missing from tauri.conf.json | Minor | `tauri.conf.json` | Add `"active": true` |
| m6 — Private key in git history | Security | Repository | Rotate keypair; purge history |

**Target before shipping v0.1:** M1 + M2 + m6 must be resolved.

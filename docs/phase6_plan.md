# Pirate Harbor — Phase 6 Implementation Plan

> **Phases 1–5:** ✅ Complete (79 tests, 0 warnings, 0 TSC errors)  
> **Status:** ✅ APPROVED — 2026-07-17  
> **Design System:** Atlas OS monochrome | **Convention:** `feat: T<N> - <desc>`

### Approved Decisions

| Question | Decision |
|----------|----------|
| Code signing | **Unsigned** — SmartScreen warning acceptable for v0.1 beta |
| App icon | **Sleek monochrome typographic icon** — generated via `generate_image`, then processed with `tauri icon` |
| Update endpoint | **GitHub Releases** — static `releases/latest.json` committed to repo root |
| Year-in-Review default | **Most-recent year containing session data**, falling back to current calendar year |

---

## Where We Stand After Phase 5

| Layer | Status |
|-------|--------|
| Core data model (games, sessions, collections, milestones, journal) | ✅ Complete |
| Scanner, launcher, metadata enrichment (RAWG) | ✅ Complete |
| Asset manager (covers, backgrounds, gallery, dedup) | ✅ Complete |
| Background job scheduler (priority queue, worker loop) | ✅ Complete |
| FTS5 full-text search (Cmd+K global overlay) | ✅ Complete |
| Analytics & metadata engines (stats, heatmap, year-in-review) | ✅ **Backend only** — no UI surfaces wired |
| Recommendation engine (4 strategies, weighted combiner) | ✅ Complete |
| Data export (JSON + Markdown) | ✅ Complete |
| Local backup/restore (.phb ZIP format) | ✅ Complete |
| Achievement tracking (Goldberg DLL + file-watcher) | ✅ Complete |
| Settings (storage stats, diagnostics, integrity check) | ✅ Complete |
| UX polish (skeletons, a11y, keyboard nav, animations) | ✅ Complete |

**What is still missing (Phase 6 scope):**

1. **Auto-backup never triggers** — `AutoBackupJob` exists but startup never schedules it.
2. **Metadata auto-refresh** — Library enrichment is 100% manual. Stale covers never refresh.
3. **Year-in-Review page** — `build_year_in_review()` backend complete, zero frontend.
4. **Activity Heatmap on Identity** — `get_activity_heatmap` command registered, never called in UI.
5. **Distribution** — No code signing, no auto-updater, no installer config. Cannot ship.
6. **Scale hardening** — No performance test above 1000 games.

---

## Phase 6 Overview (T49–T58)

### Pillar 1 — Live Platform Intelligence (T49–T51)
Wire the scheduler to auto-backup and auto-refresh metadata on startup, and surface stale data to the user.

### Pillar 2 — Identity & Analytics Surfaces (T52–T54)
Wire the analytics backends that exist to rich, data-dense UI screens.

### Pillar 3 — Distribution Readiness (T55–T57)
Package and ship as a real desktop application with auto-updates.

### Pillar 4 — Performance & Hardening (T58)
Prove correctness and query performance at scale (5000+ games).

---

## Task Overview (T49–T58)

| Task | Title | Pillar | Effort |
|------|-------|--------|--------|
| **T49** | Startup auto-backup + scheduled metadata refresh | Intelligence | 1.5d |
| **T50** | Background enrichment queue (batch RAWG refresh) | Intelligence | 1d |
| **T51** | Stale-data detection + notification banner | Intelligence | 1d |
| **T52** | Year-in-Review page | Identity | 2d |
| **T53** | Enhanced Identity dashboard (heatmap + milestone timeline) | Identity | 1.5d |
| **T54** | Milestone streak engine + deep-dive panel | Identity | 1d |
| **T55** | Tauri updater + release signing config | Distribution | 1d |
| **T56** | Windows installer (NSIS) + app icon polish | Distribution | 1d |
| **T57** | Update notification UI + changelog viewer | Distribution | 0.5d |
| **T58** | Scale testing (5000 games) + memory profiling | Hardening | 1.5d |

**Total estimate: ~13 days**

---

## Dependency Graph

```
T49 ──► T50 ──► T51 ──► T58
T49 ──► T52
T52 ──► T53 ──► T54
T55 ──► T56 ──► T57
```

---

## Pillar 1 — Live Platform Intelligence

### T49 — Startup Auto-Backup + Scheduled Metadata Refresh

**Problem:** `AutoBackupJob` in `background/jobs.rs` is fully implemented but never queued at startup. The user gets no automatic protection of their data.

**Backend changes:**

In `lib.rs` `setup()`, after the DB is initialised, queue two recurring jobs:

```rust
// Auto-backup every 24 hours
scheduler.push(Job {
    id:       "auto_backup_daily".into(),
    kind:     JobKind::AutoBackup,
    priority: Priority::Low,
    interval: Some(Duration::from_secs(86_400)),
});

// Metadata refresh check every 7 days
scheduler.push(Job {
    id:       "metadata_refresh_weekly".into(),
    kind:     JobKind::MetadataRefresh,
    priority: Priority::Low,
    interval: Some(Duration::from_secs(604_800)),
});
```

**New `JobKind` variant:** `MetadataRefresh` — queues all games whose `metadata_cache` entry is older than 30 days into the enrichment queue.

**New setting:** `auto_backup_enabled` (default `true`) — checked by `AutoBackupJob` before running. Exposed in SettingsPage.

**Acceptance:**
- [ ] `AutoBackupJob` runs within 30 s of first startup (testable by setting interval to 30 s in dev)
- [ ] Auto-backup file appears in backup directory
- [ ] `auto_backup_enabled = false` in settings prevents the job from writing

---

### T50 — Background Enrichment Queue (Batch RAWG Refresh)

**Problem:** `bulk_enrich_library` is a synchronous blocking command that locks the UI. Large libraries stall the app.

**Changes:**
- Move bulk enrichment into the background job scheduler as `JobKind::BulkEnrichment`
- Job processes 5 games per tick (respects RAWG's 10 req/min rate limiter)
- Emits `"enrichment-progress"` Tauri events so the frontend can show progress without blocking
- Add `EnrichmentProgressBar` (already exists in components) to the LibraryPage header when a bulk job is running

**New Tauri command:** `start_bulk_enrichment_job` — queues the job and returns immediately.

**Frontend:** Replace the existing blocking `bulk_enrich_library` call in SettingsPage with `start_bulk_enrichment_job`. Show live progress via event listener.

**Acceptance:**
- [ ] Bulk enrichment runs in background; UI remains responsive
- [ ] Progress bar updates every 5 games
- [ ] Job can be cancelled via `cancel_job` command

---

### T51 — Stale-Data Detection + Notification Banner

**Problem:** Users don't know their metadata is out-of-date.

**New Tauri command:** `get_stale_games_count` — returns count of games with no metadata or metadata older than 30 days.

**Frontend:** Show a dismissible banner at the top of LibraryPage:

```
┌──────────────────────────────────────────────────────────────┐
│ ℹ  12 games have stale metadata (>30 days old).              │
│    [Refresh Now]   [Dismiss]                                 │
└──────────────────────────────────────────────────────────────┘
```

Banner is shown once per session and respects a `stale_banner_dismissed_at` setting.

**Acceptance:**
- [ ] Banner appears when stale count > 0
- [ ] "Refresh Now" triggers `start_bulk_enrichment_job`
- [ ] "Dismiss" hides banner for 24 hours
- [ ] Banner hidden if stale count = 0

---

## Pillar 2 — Identity & Analytics Surfaces

### T52 — Year-in-Review Page

**Backend:** `build_year_in_review()` in `analytics/year_in_review.rs` is complete and registered as `get_year_in_review`. Wire it to a new route.

**New route:** `/identity/year-in-review` — add to `App.tsx`.

**New page:** `YearInReviewPage.tsx`

Layout:

```
┌─────────────────────────────────────────────────────────────┐
│  2025 IN REVIEW                                              │
│  ─────────────────────────────────────────────────────────  │
│                                                             │
│  [ Total Playtime ]   [ Games Played ]   [ Milestones ]     │
│       847h                  34                  91          │
│                                                             │
│  ── TOP GAMES ─────────────────────────────────────────── ──│
│  1. The Witcher 3      312h  ████████████████████           │
│  2. Hollow Knight       98h  ██████                         │
│  3. Celeste             61h  ████                           │
│                                                             │
│  ── GENRE BREAKDOWN ───────────────────────────────────── ──│
│  RPG ████████ 42%   Action ██████ 31%   Indie ████ 19%      │
│                                                             │
│  ── MONTHLY PLAYTIME ──────────────────────────────────── ──│
│  [bar chart of playtime per month]                          │
│                                                             │
│  ── ACHIEVEMENTS EARNED ───────────────────────────────── ──│
│  91 milestones · 12 achievements · 3 completion badges      │
└─────────────────────────────────────────────────────────────┘
```

**Year selector:** Dropdown to switch between years (use `DISTINCT strftime('%Y', ...)` on sessions).

**Data source:** `get_year_in_review(year: i32)`.

**Default year logic** (approved):
```typescript
// On mount: find the most-recent year that has ≥1 session,
// fall back to current calendar year if library is empty.
const availableYears = await getSessionYears();   // new command
const defaultYear = availableYears[0] ?? new Date().getFullYear();
```

**New Tauri command:** `get_session_years` — returns `Vec<i32>` of years with ≥1 session, sorted descending.

```sql
SELECT DISTINCT CAST(strftime('%Y', started_at) AS INTEGER) AS yr
FROM sessions ORDER BY yr DESC
```

**Acceptance:**
- [ ] Page defaults to the most-recent year with session data
- [ ] Falls back to current calendar year when no sessions exist
- [ ] Year selector changes data
- [ ] Graceful empty state when no data for selected year

---

### T53 — Enhanced Identity Dashboard (Heatmap + Timeline)

**Backend exists:** `get_activity_heatmap` returns 365 days of session activity — never called in UI.

**Changes to `IdentityPage.tsx`:**

Add two new sections below the existing content:

**Section A — Activity Heatmap (GitHub-style)**

52×7 grid of cells. Cell colour intensity proportional to minutes played that day.

```
Jan    Feb    Mar    Apr ──────────────────── Dec
□□□□□  □□■□□  □□□□□  □□■■□  ...  □■■□□  □□□□□  ← Mon
□□□■□  □□□□□  □□□□■  □□□□□  ...  □□□□□  □□■□□  ← Tue
...
```

- Hover → tooltip: `"2h 14min · Jul 3"`
- Colour scale: 4 intensity levels using `--color-text-*` CSS vars

**Section B — Milestone Timeline**

Chronological list of the last 20 milestones, grouped by month.

```
July 2025
  🏆  Win Your First Game      · The Witcher 3     · Jul 3
  ★   Completed                · Hollow Knight     · Jul 9

June 2025
  🏆  Speedrunner               · Celeste           · Jun 28
```

**New Tauri command:** `get_recent_milestones(limit: usize)` — returns milestones sorted by `achievement_date DESC`.

**Acceptance:**
- [ ] Heatmap renders 365 cells; empty days render as dim squares
- [ ] Milestone timeline shows correct dates and game names
- [ ] Both sections render gracefully on first-run (empty state)

---

### T54 — Milestone Streak Engine + Deep-Dive Panel

**Backend:** Add streak calculation to `analytics/milestones.rs`:

```rust
pub struct MilestoneStreakStats {
    pub current_streak_days: i32,   // consecutive days with ≥1 milestone
    pub longest_streak_days: i32,
    pub total_milestones: i32,
    pub this_month: i32,
    pub this_week: i32,
}

pub fn build_milestone_streak_stats(conn: &Connection) -> Result<MilestoneStreakStats, String>;
```

**New Tauri command:** `get_milestone_streak_stats` — registered in `lib.rs`.

**Frontend:** Add a "Milestone Activity" card to IdentityPage:

```
┌─ Milestone Activity ──────────────┐
│  Current Streak   3 days          │
│  Longest Streak   14 days         │
│  This Week        7               │
│  This Month       23              │
│  All Time         312             │
└───────────────────────────────────┘
```

**Acceptance:**
- [ ] Streak counts are correct (unit-tested)
- [ ] Streak resets to 0 if no milestone earned yesterday
- [ ] Card hidden if `total_milestones === 0`

---

## Pillar 3 — Distribution Readiness

### T55 — Tauri Updater + Release Signing Config

**Current state:** `tauri.conf.json` has no `plugins.updater` section. Version is `0.1.0`.

**Changes to `tauri.conf.json`:**

```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/sigmakiller/Pirate-Harbor/releases/latest/download/latest.json"
      ],
      "dialog": false,
      "pubkey": "<<GENERATE WITH tauri signer generate>>"
    }
  }
}
```

> **Approved:** GitHub Releases is the update endpoint. The engineer must commit `releases/latest.json` to the repo root AND attach it as a release asset on every GitHub Release.

**Add to `Cargo.toml`:**
```toml
tauri-plugin-updater = "2"
```

**New command:** `check_for_updates` — calls the updater plugin, returns `{ available: bool, version: string | null, notes: string | null }`.

**Signing:** Generate a keypair with `tauri signer generate`. Store private key in GitHub Actions secret `TAURI_PRIVATE_KEY`. Public key committed to `tauri.conf.json`.

**Releases JSON format** — committed to repo root as `releases/latest.json` AND attached to every GitHub Release:
```json
{
  "version": "0.1.0",
  "notes": "Initial release",
  "pub_date": "2026-07-17T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "...",
      "url": "https://github.com/sigmakiller/Pirate-Harbor/releases/download/v0.1.0/Pirate-Harbor_0.1.0_x64-setup.exe"
    }
  }
}
```

> **No code signing cert.** Windows SmartScreen will show an "Unknown publisher" warning on first run — acceptable for v0.1 beta.

**Acceptance:**
- [ ] `check_for_updates` returns `available: false` when on latest version
- [ ] Signing keypair generated and public key committed
- [ ] `cargo tauri build` succeeds with updater plugin enabled

---

### T56 — Windows Installer (NSIS) + App Icon Polish

**Current state:** `tauri.conf.json` targets `"all"`. Icons exist but are Tauri defaults.

**Changes:**

1. **App icon** — Generate a **sleek, monochrome typographic icon** using `generate_image` tool (black background, white "PH" logotype, clean sans-serif geometry). Run `tauri icon <master.png>` to auto-derive all required sizes (32×32 through 512×512, `.ico`, `.icns`).

2. **NSIS installer config** in `tauri.conf.json`:
```json
{
  "bundle": {
    "targets": ["nsis", "msi"],
    "nsis": {
      "displayLanguageSelector": false,
      "installMode": "currentUser",
      "shortcutName": "Pirate Harbor"
    }
  }
}
```

3. **Windows metadata:**
```json
{
  "bundle": {
    "windows": {
      "wix": null,
      "allowDowngrades": false,
      "certificateThumbprint": null
    }
  }
}
```

4. **README update** — Add installation instructions.

**Acceptance:**
- [ ] `cargo tauri build` produces `.exe` installer and `.msi`
- [ ] App installs to `%LOCALAPPDATA%\Programs\Pirate Harbor` (currentUser mode)
- [ ] Start Menu shortcut created
- [ ] Custom icon shows in taskbar and About dialog

---

### T57 — Update Notification UI + Changelog Viewer

**Frontend:** Add update check to `App.tsx` on startup (non-blocking):

```typescript
// On app mount, after 5s delay (don't block startup)
setTimeout(async () => {
  const update = await checkForUpdates();
  if (update.available) {
    addToast({
      message: `Update available: v${update.version}`,
      type: "info",
      action: { label: "View", onClick: () => navigate("/settings#updates") }
    });
  }
}, 5000);
```

**SettingsPage — Updates section (already has placeholder):**

```
┌─ Updates ──────────────────────────────────────────────────┐
│  Current version: 0.2.0                                    │
│  Status: ✓ Up to date  [Check Now]                         │
│                                                            │
│  ── Changelog ─────────────────────────────────────────── │
│  v0.2.0 — 2026-07-17                                       │
│  • Achievement tracking via Goldberg emulator              │
│  • Import achievements from Steam public API               │
│  • Auto-backup on startup                                  │
└────────────────────────────────────────────────────────────┘
```

**Changelog** is fetched from the same `releases/latest.json` and displayed in-app.

**Acceptance:**
- [ ] Update toast appears if a newer version exists
- [ ] "Check Now" button in Settings works
- [ ] Changelog section renders (or shows "Could not fetch" on network failure)
- [ ] Update check never blocks or crashes the app

---

## Pillar 4 — Performance & Hardening

### T58 — Scale Testing (5000 Games) + Memory Profiling

**Test scenarios:**

| Scenario | Target | Pass Condition |
|----------|--------|----------------|
| FTS5 search on 5000 games | < 100 ms p99 | Already tested at 1000 — rerun at 5000 |
| `get_all_games` paginated | < 50 ms | Must paginate; no full table scan |
| `build_year_in_review` | < 500 ms | Acceptable for on-demand call |
| `get_activity_heatmap` | < 200 ms | 365-day window query |
| App startup time (cold) | < 2 s | From launch to first render |
| Memory usage at rest (5000 games) | < 150 MB | Measure with Windows Task Manager |

**Code changes if tests fail:**

- `get_all_games` — add `LIMIT`/`OFFSET` pagination parameters if missing
- `build_year_in_review` — add a composite index on `sessions(game_id, started_at)`
- `get_activity_heatmap` — verify the `idx_sessions_started` index is used (check `EXPLAIN QUERY PLAN`)

**New integration test fixture:**
```rust
fn seed_5000_games(conn: &Connection) { ... }

#[test]
fn t58_fts5_search_sub_100ms_on_5000_games() { ... }

#[test]
fn t58_year_in_review_sub_500ms_on_5000_sessions() { ... }
```

**Acceptance:**
- [ ] All 6 performance targets met
- [ ] 2 new integration tests pass
- [ ] `cargo test` total: ≥ 85 tests
- [ ] No new Rust warnings

---

## Resolved Decisions

| # | Question | Resolution |
|---|----------|------------|
| Q1 | Code signing | **Unsigned** — SmartScreen warning acceptable for v0.1 beta |
| Q2 | App icon | **Monochrome typographic icon** generated via `generate_image` → `tauri icon` |
| Q3 | Update server | **GitHub Releases** — static `releases/latest.json` in repo + as release asset |
| Q4 | Year-in-Review default | **Most-recent year with session data**, fallback to current year |

---

## Summary Checklist for Engineer

### Pillar 1 — Intelligence
- [ ] T49: Auto-backup job scheduled at startup; `MetadataRefresh` JobKind added
- [ ] T50: Bulk enrichment runs in background; progress events emitted
- [ ] T51: Stale-data banner on LibraryPage

### Pillar 2 — Identity
- [ ] T52: `YearInReviewPage.tsx` + `/identity/year-in-review` route
- [ ] T53: Heatmap + milestone timeline on IdentityPage
- [ ] T54: `MilestoneStreakStats` backend + Identity card

### Pillar 3 — Distribution
- [ ] T55: Tauri updater plugin + signing keypair
- [ ] T56: NSIS installer config + custom icons
- [ ] T57: Update toast + Settings changelog section

### Pillar 4 — Hardening
- [ ] T58: 5000-game performance tests; all targets met

**Target: ≥ 85 tests passing at Phase 6 completion.**

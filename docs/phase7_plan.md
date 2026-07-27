# Pirate Harbor — Phase 7 Implementation Plan

> **Phases 1–6:** ✅ Complete (86 tests, 0 warnings, 0 TSC errors)
> **Status:** ✅ APPROVED — 2026-07-27

### Approved Decisions

| # | Question | Decision |
|---|----------|----------|
| Q1 | Smart collection rule logic | **AND-only** — all rules must match |
| Q2 | Journal screenshot storage | **Shared `AssetManager`** — deduplication benefits |
| Q3 | GOG detection priority | **Registry first**, Galaxy DB as fallback |
> **Design System:** Atlas OS monochrome | **Convention:** `feat: T<N> - <desc>`

---

## Where We Stand After Phase 6

| Layer | Status |
|-------|--------|
| Core data model + CRUD | ✅ Complete |
| Scanner, launcher, RAWG metadata enrichment | ✅ Complete |
| Asset manager, gallery, dedup | ✅ Complete |
| Background job scheduler (auto-backup, bulk enrich) | ✅ Complete |
| FTS5 search, recommendations, analytics engines | ✅ Complete |
| Achievement tracking (Goldberg DLL + file watcher) | ✅ Complete |
| Identity dashboard, Year-in-Review, heatmap, streak | ✅ Complete |
| Distribution: NSIS installer, updater, CI/CD | ✅ Complete |
| Scale-tested at 5000 games | ✅ Complete |

**What is still missing before v1.0:**

1. **Library filters are shallow** — only status + favorites. No genre, no playtime range, no developer/publisher filter.
2. **Onboarding is a 3-step placeholder** — no RAWG API key setup step, no initial scan prompt. New users can't discover features.
3. **Collections are manual-only** — no smart/rule-based collections ("All RPGs", "Backlog > 10h").
4. **Scanner is DRM-folder-only** — no GOG / Epic Games / itch.io store detection.
5. **CI/CD is build-only** — no automated tests in CI, no lint, no TypeScript check.
6. **README is a design doc** — no installation guide, no feature list, no screenshots.
7. **Journal lacks screenshots** — design spec says "screenshots and milestones" but attach is not implemented.

---

## Phase 7 Overview (T59–T66)

Phase 7 is the **v1.0 Polish & Ship** phase. Every task makes the product more complete, discoverable, or maintainable — nothing experimental.

### Pillar 1 — Library Power-User Features (T59–T60)
Give users real control over their library with multi-criteria filtering and smart auto-collections.

### Pillar 2 — Onboarding & Discovery (T61–T62)
A first-run experience that actually works: RAWG setup, initial scan, and store auto-detection.

### Pillar 3 — Journal Screenshots (T63)
Wire the existing gallery asset pipeline to journal entries.

### Pillar 4 — CI/CD Hardening + Documentation (T64–T65)
Make every PR verifiable and the project presentable to the public.

### Pillar 5 — v1.0 Release (T66)
Final version bump, CHANGELOG, and tagging.

---

## Task Overview (T59–T66)

| Task | Title | Pillar | Effort |
|------|-------|--------|--------|
| **T59** | Advanced library filters (genre, playtime range, developer) | Power Features | 1.5d |
| **T60** | Smart collections (rule-based, auto-updating) | Power Features | 2d |
| **T61** | Onboarding v2 (RAWG key setup + initial scan prompt) | Onboarding | 1.5d |
| **T62** | GOG + Epic Games library auto-detection in scanner | Onboarding | 1.5d |
| **T63** | Journal screenshot attachments | Journal | 1d |
| **T64** | CI/CD: tests + TypeScript check + lint on every PR | CI/CD | 1d |
| **T65** | README + CHANGELOG rewrite for public release | Docs | 0.5d |
| **T66** | v1.0 release: version bump, tag, GitHub Release | Release | 0.5d |

**Total estimate: ~10 days**

---

## Dependency Graph

```
T59 ──► T60 ──► T66
T61 ──► T62 ──► T66
T63 ──► T66
T64 ──► T66
T65 ──► T66
```

All pillars are independent and can be worked in parallel. T66 gates on all others.

---

## Pillar 1 — Library Power-User Features

### T59 — Advanced Library Filters

**Current state:** LibraryPage filters by `status` and `is_favorite` only. No genre, no playtime, no developer.

**Changes to `useLibraryStore` (Zustand):**

Add new filter fields:
```typescript
interface LibraryFilters {
  status:       GameStatus | null;
  favoritesOnly: boolean;
  // NEW:
  genre:        string | null;      // exact match on comma-split genres
  developer:    string | null;      // substring match
  minPlaytime:  number | null;      // minutes
  maxPlaytime:  number | null;      // minutes; null = no upper bound
  neverPlayed:  boolean;            // playtime === 0
}
```

**New backend command:** `get_library_facets` — returns available genres and developers derived from the game library for populating filter dropdowns:
```rust
pub struct LibraryFacets {
    pub genres:     Vec<String>,   // distinct, sorted, split from comma-joined values
    pub developers: Vec<String>,   // distinct, sorted
}
```

**LibraryPage UI — Filter Drawer:**

Add a collapsible filter panel (toggle button beside the sort dropdown):

```
┌─ Filters ──────────────────────────────────────────────────┐
│  Genre        [All ▾]   [RPG] [Action] [Strategy]         │
│  Developer    [______________________]                      │
│  Playtime     [Any ▾] → min [___] h   max [___] h          │
│  ☐ Never played                                            │
│                                      [Clear Filters]       │
└────────────────────────────────────────────────────────────┘
```

Filtering is client-side (all games already in memory). Genre chips are populated from `get_library_facets`.

**Active filter count badge** appears on the filter toggle button when filters are active.

**Acceptance:**
- [ ] Genre filter correctly handles comma-split genre strings (e.g. "RPG, Action" matches "RPG" filter)
- [ ] Playtime range filters work correctly with min/max = null
- [ ] `get_library_facets` returns unique sorted values
- [ ] Active filter count badge shows/hides correctly
- [ ] "Clear Filters" resets all new fields
- [ ] `cargo test` still passes

---

### T60 — Smart Collections (Rule-Based, Auto-Updating)

**Problem:** Collections are manually maintained. Users must add/remove games by hand. A "Completed games" collection breaks the moment the user marks a game completed.

**DB change:** Migration 009.

```sql
-- Add rule columns to existing collections table
ALTER TABLE collections ADD COLUMN is_smart INTEGER NOT NULL DEFAULT 0;
ALTER TABLE collections ADD COLUMN rule_json TEXT;      -- JSON-encoded SmartRule[]
```

**`SmartRule` type:**
```rust
#[derive(Serialize, Deserialize)]
pub struct SmartRule {
    pub field:    SmartField,   // Status | Genre | Playtime | Developer | IsFavorite
    pub operator: SmartOp,      // Eq | Contains | Gt | Lt | IsTrue
    pub value:    String,       // serialised as string regardless of type
}
```

**New Tauri commands:**
- `create_smart_collection(title, rules: Vec<SmartRule>)` → `Collection`
- `evaluate_smart_collection(collection_id)` → `Vec<String>` (game IDs)
- `refresh_all_smart_collections` — called at startup and after any game update

**Frontend — Smart Collection Creator:**

Add a "Smart Collection" tab to the new-collection modal in `CollectionsPage`:

```
┌─ New Smart Collection ─────────────────────────────────────┐
│  Name:  [Completed RPGs___________________________]        │
│                                                            │
│  Rules: (match ALL of the following)                       │
│  ┌─────────────────────────────────────────────────────┐  │
│  │  Status  [is] [completed ▾]           [×]           │  │
│  │  Genre   [contains] [RPG__________]   [×]           │  │
│  └─────────────────────────────────────────────────────┘  │
│  [+ Add Rule]                                              │
│                                              [Create]      │
└────────────────────────────────────────────────────────────┘
```

Smart collections show a ⚡ icon to distinguish them from manual ones. Games are **read-only** in smart collections — users cannot manually add/remove them.

**Auto-refresh triggers:** `refresh_all_smart_collections` called:
1. At app startup (in `lib.rs` setup, after jobs)
2. After any `update_game` command
3. After any `add_game` or `delete_game` command

**Acceptance:**
- [ ] Migration 009 applies cleanly; idempotency test passes
- [ ] Smart collection correctly evaluates all 5 rule types
- [ ] Multi-rule evaluation uses AND logic
- [ ] Smart collection game list updates when a game's status changes
- [ ] ⚡ icon visible on smart collections in sidebar
- [ ] Manual add/remove disabled for smart collections

---

## Pillar 2 — Onboarding & Discovery

### T61 — Onboarding v2 (RAWG Key Setup + Initial Scan)

**Current state:** `OnboardingPage.tsx` is a 3-step "Welcome / How it works / Finish" placeholder. No RAWG key input, no scanner integration.

**New 5-step flow:**

| Step | Title | Content |
|------|-------|---------|
| 0 | Welcome to Pirate Harbor | Tagline, hero visual, "Get Started" |
| 1 | Connect Metadata | RAWG API key input with validation ping, link to RAWG registration, "Skip for now" option |
| 2 | Find Your Games | Folder picker → runs scanner → shows preview count |
| 3 | Your Library is Ready | Stats: N games found, M enriched |
| 4 | Let's Go | Navigate to `/library` |

**Step 1 — RAWG key validation:**
```typescript
// Hit the RAWG search endpoint with the key to validate
const testResult = await testRawgKey(key); // new Tauri command
if (!testResult.valid) showError("Invalid key — check and try again");
```

**New backend command:** `test_rawg_key(api_key: String)` → `{ valid: bool }` — makes a minimal RAWG API call and returns whether it succeeds.

**Step 2 — Scan integration:**
Reuse the existing `ScanPage` logic (scan directory, show results) as an embedded flow rather than a separate route.

**Acceptance:**
- [ ] RAWG key step saves key to `settings` table and emits success state
- [ ] Skip button skips RAWG step cleanly (key remains empty)
- [ ] Scan step reuses existing `scan_directory` command
- [ ] Step 3 shows accurate game count
- [ ] Back navigation between steps works
- [ ] `onboarding_complete` setting is set before navigating away
- [ ] Returning users never see onboarding (setting already set)

---

### T62 — GOG + Epic Games Library Auto-Detection

**Current state:** Scanner walks user-selected directories looking for `.exe` files. It has no knowledge of store-specific layouts.

**New scanner tier: Store Detection**

Before the directory scan, check known store library locations:

**GOG Galaxy (Registry first — approved, Galaxy DB fallback):**
```
Primary:  HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\*  →  read "path" value
Fallback: %ProgramData%\GOG.com\Galaxy\storage\galaxy-2.0.db  (SQLite, only if registry empty)
```

**Epic Games Store:**
```
%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item  →  JSON files
Read: "InstallLocation", "DisplayName", "LaunchExecutable"
```

**itch.io (Butler):**
```
%APPDATA%\itch\db\butler.db  →  SQLite
SELECT path, verdict FROM caves JOIN baskets
```

**New Tauri command:** `detect_store_libraries` → `Vec<StoreGame>`:
```rust
pub struct StoreGame {
    pub title:    String,
    pub exe_path: String,
    pub store:    String,   // "gog" | "epic" | "itch"
}
```

**Frontend — Store Import in ScanPage:**

Add a "Detect from Stores" button above the folder picker:
```
[Detect from GOG / Epic / itch.io]   or   [Browse Folder…]
```

Clicking it runs `detect_store_libraries` and shows the same confirmation list as the folder scanner. The user can select which games to import.

**Acceptance:**
- [ ] GOG detection reads registry or Galaxy DB
- [ ] Epic detection reads `.item` manifest files
- [ ] itch.io detection reads butler SQLite
- [ ] Graceful empty result when store not installed (no panic)
- [ ] Detected games flow through the same `add_game` pipeline as scanned games
- [ ] `cargo test` includes a unit test for each parser (using fixture data)

---

## Pillar 3 — Journal Screenshots

### T63 — Journal Screenshot Attachments

**Design spec (from JournalPage header comment):** "screenshots and milestones" — but the journal entry form has no attachment support.

**DB change:** Add `image_path` column to `journal_entries` (migration 009 alongside T60 smart collections):
```sql
ALTER TABLE journal_entries ADD COLUMN image_path TEXT;
```

**Backend changes:**
- `create_journal_entry` and `update_journal_entry` accept `image_path: Option<String>`
- New command: `attach_journal_screenshot(entry_id, source_path)` — uses the existing `AssetManager` to copy + dedup the image, returns the stored asset path

**Frontend — JournalPage compose form:**

Add an optional image attachment button below the text area:
```
┌─────────────────────────────────────────────────────┐
│  [textarea for journal text]                        │
│                                                     │
│  📎 [Attach Screenshot]                             │
│     preview thumbnail if selected                   │
└─────────────────────────────────────────────────────┘
```

In the timeline feed, entries with images show an inline full-width image below the text.

**Acceptance:**
- [ ] Attach button opens file dialog (image formats only)
- [ ] Selected image shows as thumbnail in compose form
- [ ] Saved entry shows inline image in feed
- [ ] Image stored via `AssetManager` (dedup, not raw copy)
- [ ] Entry without image renders identically to current behaviour

---

## Pillar 4 — CI/CD Hardening + Documentation

### T64 — CI/CD: Tests + TypeScript + Lint on Every PR

**Current state:** `.github/workflows/release.yml` only runs `pnpm tauri build` on tag pushes. There is no PR check workflow.

**New file:** `.github/workflows/ci.yml`

```yaml
name: CI
on:
  pull_request:
  push:
    branches: [main]

jobs:
  rust-checks:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy, rustfmt }
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: apps/desktop/src-tauri }
      - name: Clippy (deny warnings)
        working-directory: apps/desktop/src-tauri
        run: cargo clippy -- -D warnings
      - name: Tests
        working-directory: apps/desktop/src-tauri
        run: cargo test
      - name: Format check
        working-directory: apps/desktop/src-tauri
        run: cargo fmt --check

  frontend-checks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v3
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - name: Install deps
        working-directory: apps/desktop
        run: pnpm install
      - name: TypeScript check
        working-directory: apps/desktop
        run: pnpm exec tsc --noEmit
      - name: ESLint
        working-directory: apps/desktop
        run: pnpm exec eslint src --max-warnings 0
```

**Any PR that fails Clippy, tests, tsc, or lint is blocked from merge.**

**Acceptance:**
- [ ] `ci.yml` runs on push to `main` and on all PRs
- [ ] Clippy `-D warnings` passes against current codebase (fix any existing clippy warnings first)
- [ ] `tsc --noEmit` passes (fix any existing type errors)
- [ ] ESLint passes with `--max-warnings 0`
- [ ] CI green on a test PR

---

### T65 — README + CHANGELOG for Public Release

**Current state:** `README.md` is the design system doc. There is no `CHANGELOG.md`.

**Rewrite `README.md`:**

```markdown
# Pirate Harbor 🏴‍☠️

A premium offline game library manager for Windows.
Track your gaming history, achievements, and milestones — no Steam required.

## Features
- 🎮 Game library with automatic metadata (covers, genres, playtime)
- 📊 Analytics: activity heatmap, year-in-review, genre breakdown
- 🏆 Achievement tracking via Goldberg Steam emulator
- 📔 Play journal with screenshot attachments
- 🔍 Full-text search across your entire library
- 📦 Local backup & restore (.phb format)
- 🔄 Auto-updates via GitHub Releases

## Installation
Download the latest installer from [GitHub Releases](https://github.com/sigmakiller/Pirate-Harbor/releases).

## Building from Source
...

## Tech Stack
- Frontend: React 19 + TypeScript + Tauri v2
- Backend: Rust (rusqlite, notify, reqwest, tauri-plugin-updater)
- Database: SQLite with FTS5 full-text search
```

**Create `CHANGELOG.md`:**

```markdown
# Changelog

All notable changes to Pirate Harbor are documented here.

## [1.0.0] — 2026-07-27

### Added
- Automated achievement tracking via Goldberg Steam Emulator
- Activity heatmap and Year-in-Review analytics
- Smart rule-based collections
- GOG, Epic Games, and itch.io library auto-detection
- Journal screenshot attachments
- Background metadata refresh and auto-backup
- Auto-updater via GitHub Releases (signed, no SmartScreen bypass required)
- NSIS installer (per-user, no admin required)
...
```

**Acceptance:**
- [ ] README has Installation, Features, Building from Source, Tech Stack sections
- [ ] CHANGELOG covers all phases (1–7) in conventional changelog format
- [ ] No design-system-only content in the public README (move to `docs/design-system.md`)

---

## Pillar 5 — v1.0 Release

### T66 — Version Bump, Tag, GitHub Release

**Changes:**
1. Bump version in `tauri.conf.json`: `"version": "1.0.0"`
2. Bump version in `apps/desktop/package.json`: `"version": "1.0.0"`
3. Commit: `chore: bump version to 1.0.0`
4. Tag: `git tag v1.0.0 && git push --tags`
5. CI release workflow triggers automatically on the tag push
6. GitHub Release created with NSIS installer and signed `.exe`

**Acceptance:**
- [ ] `tauri.conf.json` and `package.json` both at `1.0.0`
- [ ] Tag `v1.0.0` pushed to origin
- [ ] GitHub Actions release workflow completes without errors
- [ ] GitHub Release page shows installer download and `releases/latest.json`
- [ ] `check_for_updates` on a v0.1.0 build returns `available: true, version: "1.0.0"`

---

## Resolved Decisions

| # | Question | Resolution |
|---|----------|------------|
| Q1 | Smart collection rule logic | **AND-only** across all rules — no OR toggle in v1.0 |
| Q2 | Journal screenshot storage | **Shared `AssetManager`** — same pipeline as game gallery, dedup included |
| Q3 | GOG detection priority | **Registry first** (`HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\*`), Galaxy DB SQLite fallback |

---

## Summary Checklist for Engineer

### Pillar 1 — Power Features
- [ ] T59: `get_library_facets` command + filter drawer UI (genre, developer, playtime, never-played)
- [ ] T60: Migration 009 smart collections + `create_smart_collection` + `evaluate_smart_collection` + ⚡ UI

### Pillar 2 — Onboarding
- [ ] T61: 5-step onboarding v2 (RAWG validation + scan step + `test_rawg_key` command)
- [ ] T62: `detect_store_libraries` for GOG/Epic/itch + "Detect from Stores" button in ScanPage

### Pillar 3 — Journal
- [ ] T63: `image_path` on `journal_entries` + `attach_journal_screenshot` + inline image in feed

### Pillar 4 — CI/CD + Docs
- [ ] T64: `.github/workflows/ci.yml` (Clippy -D warnings + cargo test + tsc + ESLint)
- [ ] T65: README rewrite + `CHANGELOG.md` creation

### Pillar 5 — Release
- [ ] T66: Version bump to 1.0.0 + tag + GitHub Release

**Target: ≥ 95 tests passing at Phase 7 completion.**

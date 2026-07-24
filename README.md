# Pirate Harbor 🏴‍☠️

> A premium, monochrome desktop game launcher — your personal OS for preserving gaming history.

Built with **Tauri v2**, **React 19**, and **Rust**. Designed around the **Atlas OS** design system: typography-driven, strictly monochrome, no bounce, no noise — a museum, not a toy.

---

## Design System — Atlas OS

All visual decisions are governed by the `Design/` folder, which is the **single source of truth**.

| Principle | Rule |
|-----------|------|
| Color | Monochrome only. Game artwork is the only color source. |
| Typography | Space Grotesk (display) · Inter (body) · JetBrains Mono (code/meta) |
| Motion | Fade, opacity, subtle translate only. No bounce, no elastic. `prefers-reduced-motion` respected globally. |
| Layout | 12-column grid · 72px margins · 128px vertical sections |
| Ambient | Game Detail pages ONLY — desaturated, darkened, blurred at 8–15% opacity |

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 19 + TypeScript + Vite 7 |
| Styling | Vanilla CSS (design tokens via CSS custom properties) |
| Routing | React Router v7 |
| State | Zustand v5 |
| Desktop Shell | Tauri v2 |
| Database | SQLite via `rusqlite` (bundled, FTS5 enabled) |
| Image Pipeline | `image` crate (WebP encode, Lanczos3 resize) |
| IPC | Typed `invoke()` wrappers (`src/lib/api.ts`) |
| Package Manager | pnpm (monorepo) |

---

## Project Structure

```
pirate-harbor/
├── apps/
│   └── desktop/
│       ├── src/
│       │   ├── layouts/
│       │   │   └── AppLayout.tsx        # Sidebar + TopBar + skip-nav + Cmd+K shortcut
│       │   ├── pages/                   # 12 route pages
│       │   │   ├── LauncherPage.tsx
│       │   │   ├── LibraryPage.tsx
│       │   │   ├── GameDetailPage.tsx
│       │   │   ├── AddGamePage.tsx
│       │   │   ├── EditGamePage.tsx
│       │   │   ├── CollectionsPage.tsx
│       │   │   ├── JournalPage.tsx
│       │   │   ├── MilestonesPage.tsx
│       │   │   ├── IdentityPage.tsx
│       │   │   ├── ScanPage.tsx
│       │   │   ├── SettingsPage.tsx     # Storage stats, diagnostics, backup/restore
│       │   │   └── OnboardingPage.tsx
│       │   ├── components/
│       │   │   ├── TopBar.tsx           # Global search (Ctrl+K) + job indicator
│       │   │   ├── SearchOverlay.tsx    # FTS5 search modal
│       │   │   ├── GameCard.tsx         # Keyboard-navigable (tabIndex, Enter/Space)
│       │   │   ├── SkeletonLoader.tsx   # GhostCard / GhostGrid / GhostList
│       │   │   ├── PageWrapper.tsx      # atlas-enter page transition wrapper
│       │   │   ├── EnrichmentProgressBar.tsx
│       │   │   └── ToastContainer.tsx   # aria-live polite toast stack
│       │   ├── hooks/
│       │   │   ├── useGridArrowNav.ts   # Arrow-key keyboard nav for CSS grids
│       │   │   └── useEnrichmentProgress.ts
│       │   ├── lib/
│       │   │   ├── api.ts               # Typed Tauri invoke() wrappers (all modules)
│       │   │   └── utils.ts             # cn(), formatPlaytime(), formatDate()
│       │   ├── stores/                  # Zustand state stores
│       │   ├── engine/                  # Ambient Engine (contextual immersion)
│       │   └── types/                   # TypeScript domain types
│       └── src-tauri/
│           └── src/
│               ├── db/                  # SQLite init + migration runner (v7)
│               ├── models.rs            # Rust domain structs
│               ├── assets/              # Asset management pipeline
│               │   ├── asset_manager.rs # Orchestrator (covers/bg/gallery/dedup/stats)
│               │   ├── cover_cache.rs   # 512×512 WebP resize
│               │   ├── background_cache.rs # 1920×1080 WebP resize
│               │   ├── thumbnail_gen.rs # 256×256 thumbnails
│               │   └── dedup.rs         # Hash-based deduplication
│               ├── background/          # Job scheduler + worker thread
│               │   ├── scheduler.rs     # JobScheduler (enqueue/cancel/status)
│               │   ├── job.rs           # Job trait + JobStatus + JobContext
│               │   └── queue.rs         # Priority queue
│               ├── analytics/           # Playtime + identity analytics
│               │   ├── recommendations/ # 4-strategy recommendation engine
│               │   ├── year_in_review.rs
│               │   ├── identity.rs
│               │   ├── milestones.rs
│               │   └── completion_stats.rs
│               ├── metadata/            # RAWG client + normalizer
│               ├── api/                 # External API clients
│               ├── images/              # Legacy image utilities
│               ├── tests/               # Integration test suite (T37)
│               │   └── integration.rs   # 60 acceptance tests
│               └── commands/            # 16 Tauri command modules
│                   ├── games.rs
│                   ├── launcher.rs
│                   ├── sessions.rs
│                   ├── scanner.rs
│                   ├── metadata.rs
│                   ├── collections.rs
│                   ├── journal.rs
│                   ├── milestones.rs
│                   ├── identity.rs
│                   ├── assets.rs
│                   ├── search.rs
│                   ├── background.rs
│                   ├── export.rs        # JSON + Markdown export
│                   ├── backup.rs        # .phb create/restore/list
│                   ├── diagnostics.rs   # Schema version, integrity check, table stats
│                   └── settings.rs
├── packages/
│   └── shared/                          # Canonical TypeScript types
├── Design/                              # Atlas OS design system (source of truth)
│   └── Pages/                           # Per-page design specs
└── docs/                                # Architect plans & phase reviews
    ├── phase4_plan.md
    ├── review_t26_t31.md
    └── review_t32_t37.md
```

---

## Installation

> **Windows only** — macOS / Linux builds are not yet packaged.

### Pre-built Installer (recommended)

1. Go to the [**Releases page**](https://github.com/sigmakiller/Pirate-Harbor/releases/latest).
2. Download **`Pirate-Harbor_x.x.x_x64-setup.exe`**.
3. Run the installer — it installs to `%LOCALAPPDATA%\Programs\Pirate Harbor` (no admin required).
4. A **Start Menu** shortcut ("Pirate Harbor") is created automatically.

> **Windows SmartScreen** may show an "Unknown publisher" warning on first run (v0.1 beta is unsigned). Click **"More info → Run anyway"** to proceed.

### Automatic Updates

Pirate Harbor checks for updates on startup. When a new version is available, a notification will appear in the app. Updates are signed with an Ed25519 keypair and verified before installation.

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/) >= 9
- [Rust](https://rustup.rs/) (stable toolchain)
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) (WebView2 on Windows)

### Development

```bash
# Install all dependencies
pnpm install

# Run the desktop app in Tauri dev mode
pnpm --filter desktop tauri dev

# Type-check the frontend only
pnpm --filter desktop exec tsc --noEmit

# Run Rust unit + integration tests (60 tests)
cd apps/desktop/src-tauri && cargo test
```

### Build

```bash
# Production bundle (Tauri app installer)
pnpm --filter desktop tauri build
```

---

## Implementation Progress

### Phase 1 — Foundation

| Task | Description | Status |
|------|-------------|--------|
| T1 | Frontend dependencies, design tokens | ✅ Done |
| T2 | App shell, routing, sidebar, navigation | ✅ Done |
| T3 | SQLite database schema & Rust migrations | ✅ Done |
| T4 | Rust CRUD commands (games + settings) | ✅ Done |
| T5 | TypeScript types & Tauri API bindings | ✅ Done |
| T6 | Game launcher & playtime tracking | ✅ Done |
| T7 | Ambient Engine (contextual immersion layer) | ✅ Done |
| T8 | Phase 1 pages — full implementation | ✅ Done |
| T9 | Settings page & accessibility polish | ✅ Done |

### Phase 2 — Library Management

| Task | Description | Status |
|------|-------------|--------|
| T10 | Collections system (CRUD + game membership) | ✅ Done |
| T11 | Game scanner (directory walk + batch import) | ✅ Done |
| T12 | Scan UI page | ✅ Done |
| T13 | Edit Game page | ✅ Done |
| T14 | Onboarding flow | ✅ Done |

### Phase 3 — Metadata & Enrichment

| Task | Description | Status |
|------|-------------|--------|
| T15 | RAWG API client (Rust) | ✅ Done |
| T16 | Metadata enrichment engine | ✅ Done |
| T17 | Bulk enrichment background job | ✅ Done |
| T18 | Image downloading pipeline | ✅ Done |
| T19 | Journal system (entries + game linking) | ✅ Done |
| T20 | Journal UI page | ✅ Done |
| T21 | Milestone system (CRUD + templates) | ✅ Done |
| T22 | Milestones UI page | ✅ Done |
| T23 | Analytics engine (playtime stats) | ✅ Done |
| T24 | Identity dashboard | ✅ Done |
| T25 | Game Detail page (full immersive view) | ✅ Done |

### Phase 4 — Performance, Asset Pipeline & Polish

| Task | Description | Status |
|------|-------------|--------|
| T26 | Schema versioning & migration runner (v7, idempotent) | ✅ Done |
| T27 | Background job scheduler (thread pool + priority queue) | ✅ Done |
| T28 | Asset management (covers, gallery, dedup, thumbnails, stats) | ✅ Done |
| T29 | SQLite FTS5 search index (global Ctrl+K, <100ms on 1000+ games) | ✅ Done |
| T30 | Year-in-Review & heatmap analytics | ✅ Done |
| T31 | Recommendation engine (4 strategies: genre, playtime, recency, content) | ✅ Done |
| T32 | Data export — JSON library dump + Markdown profile report | ✅ Done |
| T33 | Local backup / restore — `.phb` ZIP archive with FTS5 rebuild | ✅ Done |
| T34 | Game Detail enrichment — gallery, related games, recommendations | ✅ Done |
| T35 | Settings page — storage stats, diagnostics, schema integrity check | ✅ Done |
| T36 | UX polish — skeleton screens, arrow-key grid nav, skip-nav, page animations | ✅ Done |
| T37 | Integration testing & acceptance — 60 tests, 0 warnings, 0 TS errors | ✅ Done |

---

## Feature Overview

### 🎮 Game Library
Browse, search, and manage your entire game collection. Filter by title, status, genre, or favorites. Sort by playtime, recently added, or alphabetically. Arrow-key navigation moves focus through the game grid without a mouse.

### 🚀 Game Launcher
Launch any game executable directly from the app. Automatic playtime tracking starts when a game is launched and records the session on exit.

### 🔍 Global Search (Ctrl+K / Cmd+K)
FTS5-powered full-text search across your entire library — finds games by title, developer, publisher, and genre; journal entries by title and body; milestones by title. Results ranked by relevance. Runs in <100ms on libraries of 1,000+ games.

### 📁 Directory Scanner
Point the scanner at any folder and it will detect game executables, deduplicate against existing entries, and batch-import to your library.

### 🌐 Metadata Enrichment
Fetches game metadata (description, developer, publisher, genre, release date, screenshots) from the RAWG API. Supports single-game and bulk library enrichment as background jobs with live progress events.

### 🗂️ Collections
Organize your library into named collections (e.g., "Currently Playing", "Completed", "Wishlist"). Games can belong to multiple collections.

### 📔 Journal
Write rich notes tied to specific games. Each entry has a type (note, review, session log) and is linked to your library entry. Indexed for full-text search.

### 🏆 Milestones
Track personal achievements per game — completions, records, events. Milestone templates let you reuse common achievement structures. Full statistics dashboard.

### 🖼️ Asset Manager
Centralized image pipeline: game covers resized to 512×512 WebP, backgrounds to 1920×1080, gallery images converted to WebP with auto-generated 256×256 thumbnails. Hash-based deduplication prevents duplicate storage. Orphan cleanup cross-references the game library.

### 📊 Identity Dashboard
Aggregate gaming identity stats: total playtime, genre spread, most-played games, milestone count, library growth over time. Powered by the Year-in-Review analytics engine.

### 🤖 Recommendation Engine
Four scoring strategies — genre affinity, playtime patterns, recency bias, and content-based matching — combine to surface the best unplayed games from your own library.

### 📦 Export
Export your library as a structured **JSON** file or a human-readable **Markdown** profile report. Both formats are validated at the command level (`.json` / `.md` extension required).

### 💾 Backup & Restore
Create point-in-time `.phb` backup archives (ZIP-based) containing your database, settings, and all image assets. Restore from any backup with a single command — FTS5 indexes are rebuilt automatically after restore.

### ⚙️ Settings & Diagnostics
Manage storage, run SQLite `PRAGMA integrity_check`, view schema version, inspect table row counts, and trigger asset orphan cleanup — all from the Settings page.

### ⚙️ Background Jobs
Thread-safe priority queue for long-running operations (bulk enrichment, export, backup, index rebuild, orphan cleanup). Jobs are observable from the UI via the TopBar indicator with live progress.

### ♿ Accessibility & UX
- **Skip-navigation** link for keyboard users
- **Arrow-key grid navigation** — move through library cards without a mouse
- **Skeleton loading screens** (`GhostCard`, `GhostGrid`, `GhostList`) replace spinner text
- **Page-entry animations** via `atlas-enter` (220ms fade + 4px translate)
- All motion respects `prefers-reduced-motion`
- `aria-live="polite"` on dynamic counters and toast notifications

---

## Database Schema

| Table | Purpose |
|-------|---------|
| `games` | Library entries — title, exe, cover, playtime, status, metadata |
| `sessions` | Play sessions — start/end timestamps, duration |
| `settings` | Key-value store — user preferences + `app_data_dir` (set at startup) |
| `search_cache` | Cached RAWG search results |
| `metadata_cache` | Enriched metadata per game |
| `collections` | Named game collections |
| `collection_games` | Many-to-many: games ↔ collections |
| `journal_entries` | Freeform notes linked to games |
| `milestones` | Personal achievements per game |
| `milestone_templates` | Reusable milestone structures |
| `games_fts` | FTS5 virtual table — indexes games (title, developer, publisher, genre) |
| `journal_fts` | FTS5 virtual table — indexes journal entries (title, body) |

Schema is versioned (currently **v7**) and managed by an incremental, idempotent migration runner in `src-tauri/src/db/migrations.rs`. Existing databases from earlier versions are auto-detected and fast-forwarded without data loss.

---

## Tauri Command API

The IPC surface is fully typed — all commands have corresponding wrappers in `src/lib/api.ts`.

| Module | Commands |
|--------|----------|
| `games` | `get_all_games`, `get_game`, `add_game`, `update_game`, `delete_game`, `toggle_favorite` |
| `launcher` | `launch_game`, `get_running_game` |
| `sessions` | `get_sessions` |
| `scanner` | `scan_directory`, `scan_all_directories`, `batch_add_games`, `get/add/remove_scan_directory` |
| `metadata` | `search_game_metadata`, `enrich_game_metadata`, `bulk_enrich_library`, `download_game_images`, … |
| `collections` | `get/create/update/delete_collection`, `add/remove_game_from_collection`, … |
| `journal` | `get/create/update/delete_journal_entry` |
| `milestones` | `create/get/delete_milestone`, `milestone_templates`, `milestone_statistics`, … |
| `identity` | `get_gaming_identity` |
| `assets` | `store_cover`, `store_background`, `store_gallery_image`, `get_cover_path`, `get_storage_stats`, `cleanup_orphan_assets`, `check_duplicate` |
| `search` | `search_global`, `rebuild_search_index` |
| `background` | `get_job_status`, `cancel_job`, `list_active_jobs`, `queue_depth` |
| `export` | `export_library_json`, `export_profile_markdown`, `get_export_preview` |
| `backup` | `create_backup`, `restore_backup`, `list_auto_backups` |
| `diagnostics` | `get_diagnostics`, `run_integrity_check`, `get_db_path` |
| `settings` | `get_setting`, `set_setting`, `get_all_settings` |

---

## Testing

```bash
cd apps/desktop/src-tauri
cargo test
# → 60 tests: 37 unit + 23 integration
```

The integration suite (`src/tests/integration.rs`) covers every Phase 4 acceptance criterion:

| Area | Coverage |
|------|----------|
| Migration versioning | Fresh DB → v7, idempotent double-run, all 9 tables |
| Scheduler state machine | `is_terminal` all variants, cancel unknown id, empty depth |
| Asset manager | Directory creation, `get_storage_stats`, `cleanup_orphans` |
| FTS5 search | Partial prefix match, <100ms on 1,000 games, no false positives |
| Export | Valid JSON structure, Markdown headings, empty-library edge case |
| Backup / restore | ZIP archive integrity, data-intact round-trip (title, count) |
| Diagnostics | `integrity_check = "ok"`, table counts, schema version constant |

---

## Commit Convention

```
feat: T<N> - <Description>   # New task implementation
fix:  T<N> - <Description>   # Review fixes
```

---

## License

MIT

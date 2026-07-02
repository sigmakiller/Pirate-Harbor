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
| Motion | Fade, opacity, subtle translate only. No bounce, no elastic. |
| Layout | 12-column grid · 72px margins · 128px vertical sections |
| Ambient | Game Detail pages ONLY — desaturated, darkened, blurred at 8–15% opacity |

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 19 + TypeScript + Vite 7 |
| Styling | Tailwind CSS v4 (CSS-first) |
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
│       │   ├── layouts/          # AppLayout (sidebar + topbar)
│       │   ├── pages/            # 12 route pages
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
│       │   │   ├── SettingsPage.tsx
│       │   │   └── OnboardingPage.tsx
│       │   ├── components/       # Shared UI components
│       │   │   ├── TopBar.tsx    # Global search (Ctrl+K) + job indicator
│       │   │   ├── SearchOverlay.tsx  # FTS5 search modal (T29)
│       │   │   └── ...
│       │   ├── lib/
│       │   │   ├── api.ts        # Typed Tauri invoke() wrappers
│       │   │   └── utils.ts      # cn(), formatPlaytime(), formatDate()
│       │   ├── hooks/            # Custom React hooks
│       │   ├── stores/           # Zustand state stores
│       │   ├── engine/           # Ambient Engine (contextual immersion)
│       │   └── types/            # TypeScript domain types
│       └── src-tauri/
│           └── src/
│               ├── db/           # SQLite init, migrations (v7)
│               ├── models.rs     # Rust domain structs
│               ├── assets/       # Asset management pipeline (T28)
│               │   ├── asset_manager.rs   # Orchestrator (covers/bg/gallery)
│               │   ├── cover_cache.rs     # 512×512 WebP resize
│               │   ├── background_cache.rs # 1920×1080 WebP resize
│               │   ├── thumbnail_gen.rs   # 256×256 thumbnails
│               │   └── dedup.rs           # Hash-based deduplication
│               ├── background/   # Job scheduler + worker thread (T27)
│               ├── api/          # External API clients (RAWG)
│               ├── analytics/    # Playtime analytics
│               ├── images/       # Legacy image utilities
│               └── commands/     # 14 Tauri command modules
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
│                   ├── settings.rs
│                   └── settings.rs
├── packages/
│   └── shared/                   # Canonical TypeScript types
├── Design/                       # Atlas OS design system (source of truth)
│   └── Pages/                    # Per-page design specs
└── docs/                         # Architect plans & phase reviews
```

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

# Run Rust unit tests
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
| T1 | Frontend dependencies, Tailwind v4, design tokens | ✅ Done |
| T2 | App shell, routing, sidebar, navigation | ✅ Done |
| T3 | SQLite database schema & Rust migrations | ✅ Done |
| T4 | Rust CRUD commands (games + settings) | ✅ Done |
| T5 | TypeScript types & Tauri API bindings | ✅ Done |
| T6 | Game launcher & playtime tracking (Rust) | ✅ Done |
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

### Phase 4 — Performance & Asset Pipeline

| Task | Description | Status |
|------|-------------|--------|
| T26 | Schema versioning & migration runner | ✅ Done |
| T27 | Background job scheduler (thread pool + queue) | ✅ Done |
| T28 | Asset management system (covers, gallery, dedup, thumbnails) | ✅ Done |
| T29 | SQLite FTS5 search index (global Ctrl+K search) | ✅ Done |
| T30–T37 | Remaining Phase 4 tasks | 🔲 Pending |

---

## Feature Overview

### 🎮 Game Library
Browse, search, and manage your entire game collection. Filter by title, status, genre, or favorites. Sort by playtime, recently added, or alphabetically.

### 🚀 Game Launcher
Launch any game executable directly from the app. Automatic playtime tracking starts when a game is launched and records the session on exit.

### 🔍 Global Search (Ctrl+K)
FTS5-powered full-text search across your entire library — finds games by title, developer, publisher, and genre; journal entries by title and body; milestones by title. Results ranked by relevance.

### 📁 Directory Scanner
Point the scanner at any folder and it will detect game executables, deduplicate against existing entries, and batch-import to your library.

### 🌐 Metadata Enrichment
Fetches game metadata (description, developer, publisher, genre, release date, screenshots) from the RAWG API. Supports single-game and bulk library enrichment as background jobs.

### 🗂️ Collections
Organize your library into named collections (e.g., "Currently Playing", "Completed", "Wishlist"). Games can belong to multiple collections.

### 📔 Journal
Write rich notes tied to specific games. Each entry has a type (note, review, session log) and is linked to your library entry.

### 🏆 Milestones
Track personal achievements per game — completions, records, events. Milestone templates let you reuse common achievement structures. Full statistics dashboard.

### 🖼️ Asset Manager
Centralized image pipeline: game covers resized to 512×512 WebP, backgrounds to 1920×1080, gallery images converted to WebP with auto-generated 256×256 thumbnails. Hash-based deduplication prevents duplicate storage. Orphan cleanup cross-references the game library.

### 📊 Identity Dashboard
Aggregate gaming identity stats: total playtime, genre spread, most-played games, milestone count, library growth over time.

### ⚙️ Background Jobs
Thread-safe job queue for long-running operations (bulk enrichment, index rebuild, orphan cleanup). Jobs are observable from the UI via the TopBar indicator.

---

## Database Schema

| Table | Purpose |
|-------|---------|
| `games` | Library entries — title, exe, cover, playtime, status, metadata |
| `sessions` | Play sessions — start/end timestamps, duration |
| `settings` | Key-value store — user preferences |
| `search_cache` | Cached RAWG search results |
| `metadata_cache` | Enriched metadata per game |
| `collections` | Named game collections |
| `collection_games` | Many-to-many: games ↔ collections |
| `journal_entries` | Freeform notes linked to games |
| `milestones` | Personal achievements per game |
| `milestone_templates` | Reusable milestone structures |
| `games_fts` | FTS5 virtual table — indexes games (title, developer, publisher, genre) |
| `journal_fts` | FTS5 virtual table — indexes journal entries (title, body) |

Schema is versioned (currently **v7**) and managed by an incremental migration runner in `src-tauri/src/db/migrations.rs`.

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
| `assets` | `store_cover`, `store_background`, `store_gallery_image`, `get_cover_path`, `get_storage_stats`, `cleanup_orphan_assets`, `check_duplicate`, … |
| `search` | `search_global`, `rebuild_search_index` |
| `background` | `get_job_status`, `cancel_job`, `list_active_jobs`, `queue_depth` |
| `settings` | `get_setting`, `set_setting`, `get_all_settings` |

---

## Commit Convention

```
feat: T<N> - <Description>   # New task
fix:  T<N> - <Description>   # Review fixes
```

---

## License

MIT

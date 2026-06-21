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
| Database | SQLite via `rusqlite` (bundled) |
| IPC | Typed `invoke()` wrappers (`src/lib/api.ts`) |
| Package Manager | pnpm (monorepo) |

---

## Project Structure

```
pirate-harbor/
├── apps/
│   └── desktop/
│       ├── src/
│       │   ├── layouts/        # AppLayout (sidebar + topbar)
│       │   ├── pages/          # All route pages
│       │   ├── components/     # Shared UI components
│       │   ├── lib/
│       │   │   ├── api.ts      # Typed Tauri invoke() wrappers
│       │   │   └── utils.ts    # cn(), formatPlaytime(), formatDate()
│       │   ├── types/          # TypeScript domain types
│       │   ├── stores/         # Zustand stores (T8)
│       │   └── engine/         # Ambient Engine (T7)
│       └── src-tauri/
│           └── src/
│               ├── db/         # SQLite init, migrations
│               ├── commands/   # Tauri IPC commands
│               └── models.rs   # Rust domain structs
├── packages/
│   └── shared/                 # Canonical TypeScript types
├── Design/                     # Atlas OS design system (source of truth)
│   └── Pages/                  # Per-page design specs
└── docs/                       # Architect plans & reviews
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

| Task | Description | Status |
|------|-------------|--------|
| T1 | Frontend dependencies, Tailwind v4, design tokens | ✅ Done |
| T2 | App shell, routing, sidebar, navigation | ✅ Done |
| T3 | SQLite database schema & Rust migrations | ✅ Done |
| T4 | Rust CRUD commands (games + settings) | ✅ Done |
| T5 | TypeScript types & Tauri API bindings | ✅ Done |
| T6 | Game launcher & playtime tracking (Rust) | 🔲 Pending |
| T7 | Ambient Engine (contextual immersion layer) | 🔲 Pending |
| T8 | Phase 1 pages — full implementation | 🔲 Pending |
| T9 | Settings page & accessibility polish | 🔲 Pending |

---

## Phase 1 Features (MVP)

- 📚 **Game Library** — Add and manage your entire game collection
- 🚀 **Game Launcher** — Launch executables directly from the app
- ⏱️ **Playtime Tracking** — Automatic session recording with stats
- ⭐ **Favorites** — Star your most-played games
- 🔍 **Search & Filter** — Filter by title, status, genre, or favorites
- ⚙️ **Settings** — Persistent key-value preferences via SQLite
- 🎨 **Ambient Layer** — Game Detail pages breathe with contextual color (from artwork)

---

## Database Schema

Three tables, Phase 1:

| Table | Purpose |
|-------|---------|
| `games` | Library entries — title, exe path, cover, playtime, status |
| `sessions` | Play sessions — start/end timestamps, duration |
| `settings` | Key-value store — user preferences |

---

## Commit Convention

```
feat: T<N> - <Description>   # New task
fix:  T<N> - <Description>   # Review fixes
```

---

## License

MIT

# Pirate Harbor 🏴‍☠️

A blazing-fast, open-source desktop game launcher built with **Tauri v2**, **React 19**, and **Rust**.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 19 + TypeScript + Vite |
| Styling | Tailwind CSS + shadcn/ui |
| State | Zustand |
| Desktop Shell | Tauri v2 |
| Database | SQLite (via rusqlite) |
| Package Manager | pnpm (monorepo) |

## Project Structure

```
pirate-harbor/
├── apps/
│   └── desktop/         # Tauri + React app
│       ├── src/         # React frontend
│       └── src-tauri/   # Rust backend
├── packages/
│   └── shared/          # Shared types (future)
└── docs/                # Documentation
```

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/) >= 9
- [Rust](https://rustup.rs/) (stable)
- [Tauri prerequisites](https://tauri.app/start/prerequisites/)

### Development

```bash
# Install dependencies
pnpm install

# Run the desktop app in dev mode
pnpm dev
```

### Build

```bash
pnpm build
```

## Features (Phase 1 MVP)

- 📚 **Game Library** — Add and manage your game collection
- 🚀 **Game Launcher** — Launch games directly from the app
- ⏱️ **Playtime Tracking** — Automatic session recording
- ⭐ **Favorites** — Star your most-played games
- 🔍 **Search & Filter** — Find games instantly
- ⚙️ **Settings** — Persistent preferences

## License

MIT

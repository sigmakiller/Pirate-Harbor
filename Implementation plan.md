# Product Requirements Document (PRD)

# Steam of Pirates

### Universal PC Game Library & Achievement Platform

**Version:** 1.0
**Status:** Planning
**Author:** Blesson T. Abraham

---

# 1. Executive Summary

Atlas Launcher is a modern desktop application that automatically discovers PC games installed across all local drives, regardless of their source. It provides a unified game library, playtime tracking, achievement management, completion statistics, cloud synchronization, and social profiles.

Unlike traditional launchers that only support games purchased through their own platform, Atlas Launcher focuses on managing locally installed games while preserving user progress and achievements independently of the original distribution platform.

The project is designed around a modular architecture that allows new games to be supported through plugins without modifying the core application.

---

# 2. Problem Statement

PC gamers often have games installed from multiple sources:

* Standalone installers
* DRM-free games
* Portable games
* Older physical releases
* Community-modified games
* Multiple launchers

There is currently no unified platform that:

* Automatically detects all installed games
* Tracks playtime consistently
* Preserves achievements independently
* Stores progress in one profile
* Allows community-created achievement support

Atlas Launcher solves this problem by acting as a universal game management platform.

---

# 3. Product Vision

Create the best universal PC game launcher that offers:

* Automatic game discovery
* Beautiful modern library
* Steam-like experience
* Independent achievement system
* Cloud synchronization
* Community extensibility

---

# 4. Goals

## Primary Goals

* Automatically discover installed games
* Create a centralized library
* Launch games
* Track playtime
* Track completion percentage
* Store achievements locally
* Synchronize achievements to cloud
* Preserve achievements after reinstall
* Support plugins

## Secondary Goals

* Social profiles
* Achievement showcase
* Friends system
* Statistics dashboard
* Collections
* Community plugins

---

# 5. Target Users

### Casual Players

Need a single launcher for all games.

### Collectors

Want to preserve achievements and completion.

### Offline Players

Need progress without relying on online services.

### Power Users

Want advanced statistics and game management.

---

# 6. Functional Requirements

## Game Discovery

The application shall:

* Scan all available drives
* Detect installed games
* Detect executable files
* Identify games using known signatures
* Cache scan results
* Allow manual additions

---

## Game Library

The application shall display:

* Game cover
* Banner
* Name
* Developer
* Publisher
* Genre
* Last played
* Total playtime
* Completion percentage
* Achievement count

Users shall be able to:

* Search
* Sort
* Filter
* Favorite
* Create collections

---

## Game Launcher

The launcher shall:

* Launch games
* Monitor game process
* Detect crashes
* Record session duration
* Update statistics

---

## Playtime Tracking

Track:

* Total hours
* Daily playtime
* Weekly playtime
* Monthly playtime
* Longest session
* Average session
* Launch count

---

## Achievement Engine

The system shall:

* Support custom achievement definitions
* Unlock achievements automatically
* Store unlock timestamps
* Sync achievements
* Display rarity
* Award XP
* Maintain achievement history

---

## Completion Tracking

Track:

* Story completion
* Side missions
* Collectibles
* Achievement completion
* Overall completion

---

## Save File Parsing

Each supported game shall include a parser capable of:

* Reading save files
* Extracting player statistics
* Determining progress
* Detecting completed objectives

The parser shall never modify save files.

---

## User Accounts

Users can:

* Create profile
* Upload avatar
* Customize profile
* View statistics
* Sync progress

---

## Cloud Synchronization

Synchronize:

* Library
* Playtime
* Achievements
* User settings
* Collections
* Statistics

---

## Plugin System

Support:

* New games
* Achievement packs
* Save parsers
* Metadata providers
* Completion trackers

---

# 7. Non-Functional Requirements

## Performance

* Startup under 3 seconds
* Game launch delay under 1 second
* Drive scanning in background
* Minimal memory usage
* Lightweight executable

---

## Security

* Secure authentication
* Encrypted cloud communication
* Read-only save parsing
* Secure local storage

---

## Reliability

* Crash recovery
* Automatic backups
* Database integrity
* Offline support

---

## Scalability

Support:

* Thousands of games
* Multiple drives
* Plugin ecosystem
* Millions of cloud profiles

---

# 8. Core Modules

## Desktop Client

Responsible for:

* UI
* Navigation
* Settings
* Local database
* Library management

---

## Scanner Engine

Responsible for:

* Drive scanning
* Executable detection
* Game identification

---

## Launcher Engine

Responsible for:

* Launching games
* Process monitoring
* Session recording

---

## Achievement Engine

Responsible for:

* Unlock detection
* Achievement storage
* XP calculation

---

## Completion Engine

Responsible for:

* Progress calculation
* Statistics generation

---

## Save Parser Engine

Responsible for:

* Reading save files
* Progress extraction

---

## Cloud Service

Responsible for:

* Authentication
* Synchronization
* Profile storage

---

## Plugin Manager

Responsible for:

* Loading plugins
* Version management
* Validation

---

# 9. Database Overview

Tables:

* users
* games
* sessions
* achievements
* achievement_unlocks
* collections
* plugins
* settings

---

# 10. Future Features

* Steam integration
* Emulator support
* Screenshot manager
* Save backups
* Game recording
* Discord Rich Presence
* Achievement marketplace
* Public APIs
* Mobile companion app

---

# 11. Development Roadmap

## Phase 1

* Desktop application
* Game scanner
* Local library
* SQLite
* Playtime tracking

## Phase 2

* Metadata
* Search
* Collections
* Better UI

## Phase 3

* Achievement framework
* Plugin architecture
* Save parsing

## Phase 4

* Cloud backend
* Authentication
* User profiles

## Phase 5

* Community plugins
* Social features
* Leaderboards

---

# 12. Success Metrics

* Detect 95% of installed games automatically
* Startup under 3 seconds
* Accurate playtime tracking
* Reliable achievement synchronization
* Plugin support for at least 100 games within the first major release

---

# Recommended Technology Stack

## Desktop Application

* Tauri v2
* React 19
* TypeScript
* Vite
* Tailwind CSS
* shadcn/ui
* React Router
* TanStack Query
* Zustand
* React Hook Form
* Zod

## Native Backend

* Rust
* Tokio
* Serde
* SQLx
* rusqlite (or SQLx with SQLite)
* sysinfo
* walkdir
* notify
* image
* reqwest

## Local Database

* SQLite

## Backend API

* FastAPI
* Python 3.13+
* SQLAlchemy 2.x
* Alembic
* Pydantic
* JWT Authentication

## Cloud Database

* PostgreSQL

## Cache

* Redis

## Object Storage

* S3-compatible storage (Cloudflare R2, MinIO, or AWS S3)

## Authentication

* JWT
* OAuth (future)

## Search

* SQLite FTS5 initially
* Elasticsearch/OpenSearch (future)

## Plugin SDK

* Rust trait interfaces
* JSON manifests
* Versioned plugin API

## Game Metadata

* Community-maintained metadata database
* Cached locally for offline use

## DevOps

* Git
* GitHub
* GitHub Actions
* Docker
* Docker Compose

## Testing

* Rust: cargo test
* Frontend: Vitest + Playwright
* Backend: pytest

## Code Quality

* ESLint
* Prettier
* Clippy
* rustfmt
* Ruff
* mypy

## Repository Structure

```
atlas-launcher/
├── apps/
│   ├── desktop/
│   ├── backend/
│   └── plugin-sdk/
├── packages/
│   ├── shared-types/
│   ├── ui/
│   └── common/
├── plugins/
├── docs/
└── infrastructure/
```

## Architectural Principles

* Clean Architecture
* SOLID Principles
* Feature-first organization
* Repository Pattern
* Dependency Injection
* Event-driven communication
* Offline-first design
* Plugin-first extensibility
* Strong typing throughout the stack
* Separation of UI, business logic, and platform-specific code

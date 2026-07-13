//! Steam Bridge — achievement tracking infrastructure — Phase 5.
//!
//! # Architecture
//!
//! ```text
//! dll_swap            ← T39: inject / restore Goldberg steam_api64.dll
//! achievement_watcher ← T40: watch Goldberg's achievements.json for changes
//! achievement_router  ← T41: parse JSON and persist unlocks to DB
//! steam_api           ← T41: RAWG-sourced achievement definitions
//! ```

pub mod dll_swap;
pub mod achievement_watcher;
// T41 additions — uncomment when implemented:
// pub mod achievement_router;
// pub mod steam_api;

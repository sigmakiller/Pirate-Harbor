//! Steam Bridge -- achievement tracking infrastructure -- Phase 5.
//!
//! # Architecture
//!
//! `	ext
//! dll_swap            <- T39: inject / restore Goldberg steam_api64.dll
//! achievement_watcher <- T40: watch Goldberg's achievements.json for changes
//! achievement_router  <- T41: parse JSON, diff, persist milestones to DB
//! steam_api           <- T41: Steam public Web API schema fetcher
//! `

pub mod dll_swap;
pub mod achievement_watcher;
pub mod achievement_router;
pub mod steam_api;

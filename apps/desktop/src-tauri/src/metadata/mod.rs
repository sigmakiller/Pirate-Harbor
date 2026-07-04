//! Metadata engine — T30.
//!
//! Shared service layer for game metadata operations.  Provides:
//! - `normalizer` — title/genre normalization
//! - `game_lookup` — related-game queries (shared by Recommendations + Game Detail)
//! - `resolver`    — unified metadata source precedence (cache → RAWG)
//! - `cover_provider` — cover URL resolution and download via AssetManager
//!
//! Note: `#[allow(dead_code)]` is set at the module level because several
//! public items are called only through the Tauri `commands::analytics` layer,
//! which the Rust linter cannot track through macro expansion.
#![allow(dead_code)]

pub mod cover_provider;
pub mod game_lookup;
pub mod normalizer;
pub mod resolver;

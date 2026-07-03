//! Metadata engine — T30.
//!
//! Shared service layer for game metadata operations.  Provides:
//! - `normalizer` — title/genre normalization
//! - `game_lookup` — related-game queries (shared by Recommendations + Game Detail)
//! - `resolver`    — unified metadata source precedence (cache → RAWG)
//! - `cover_provider` — cover URL resolution and download via AssetManager

pub mod cover_provider;
pub mod game_lookup;
pub mod normalizer;
pub mod resolver;

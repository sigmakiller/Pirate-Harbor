//! Asset management system — public API module — T28.
//!
//! # Architecture
//!
//! ```text
//! Tauri command (store_cover / store_gallery_image / get_storage_stats …)
//!       │
//!       ▼
//!  AssetManager  (Tauri state — Send + Sync)
//!       │
//!       ├── cover_cache       → assets/covers/{game_id}.webp
//!       ├── background_cache  → assets/backgrounds/{game_id}.webp
//!       ├── gallery           → assets/gallery/{game_id}/{uuid}.webp
//!       ├── thumbnail_gen     → assets/thumbnails/{hash}_thumb.webp
//!       └── dedup             → assets/thumbnails/{hash}_dedup (sentinel)
//! ```
//!
//! # Usage from commands
//!
//! ```text
//! #[tauri::command]
//! fn my_command(assets: State<'_, AssetManager>, …) {
//!     let asset_ref = assets.store_cover(game_id, &source_path)?;
//! }
//! ```

pub mod asset_manager;
pub mod background_cache;
pub mod cover_cache;
pub mod dedup;
pub mod thumbnail_gen;

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use asset_manager::{AssetManager, AssetRef, AssetType, CleanupResult, StorageStats};

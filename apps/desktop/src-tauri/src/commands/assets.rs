//! Tauri commands for the Asset Management System — T28.
//!
//! Exposes cover, background, gallery, and storage operations to the frontend.

use std::path::Path;

use tauri::State;

use crate::assets::{AssetManager, AssetRef, CleanupResult, StorageStats};
use crate::db::DbState;

// ── Cover ─────────────────────────────────────────────────────────────────────

/// Store a cover image for a game.
///
/// The source image at `source_path` is resized to 512×512 WebP and stored in
/// `assets/covers/{game_id}.webp`.  A thumbnail is generated automatically.
///
/// Returns an `AssetRef` with the final path.
#[tauri::command]
pub fn store_cover(
    assets: State<'_, AssetManager>,
    game_id: String,
    source_path: String,
) -> Result<AssetRef, String> {
    let source = Path::new(&source_path);
    assets.store_cover(&game_id, source)
}

/// Store a background image for a game.
///
/// Resized to 1920×1080 WebP and stored in `assets/backgrounds/{game_id}.webp`.
#[tauri::command]
pub fn store_background(
    assets: State<'_, AssetManager>,
    game_id: String,
    source_path: String,
) -> Result<AssetRef, String> {
    let source = Path::new(&source_path);
    assets.store_background(&game_id, source)
}

/// Get the cover path for a game, if one is stored.
#[tauri::command]
pub fn get_cover_path(
    assets: State<'_, AssetManager>,
    game_id: String,
) -> Option<String> {
    assets.get_cover(&game_id)
        .map(|p| p.to_string_lossy().into_owned())
}

/// Delete the cover image for a game.
#[tauri::command]
pub fn delete_cover(
    assets: State<'_, AssetManager>,
    game_id: String,
) -> Result<(), String> {
    assets.delete_cover(&game_id)
}

// ── Gallery ───────────────────────────────────────────────────────────────────

/// Store a gallery image for a game.
///
/// The image is converted to WebP (no resize) and stored at
/// `assets/gallery/{game_id}/{uuid}.webp`.  A thumbnail is generated.
#[tauri::command]
pub fn store_gallery_image(
    assets: State<'_, AssetManager>,
    game_id: String,
    source_path: String,
) -> Result<AssetRef, String> {
    let source = Path::new(&source_path);
    assets.store_gallery_image(&game_id, source)
}

/// List all gallery images for a game.
///
/// Returns a list of `AssetRef` objects, sorted by filename.
#[tauri::command]
pub fn get_gallery_images(
    assets: State<'_, AssetManager>,
    game_id: String,
) -> Result<Vec<AssetRef>, String> {
    assets.get_gallery_images(&game_id)
}

/// Delete a single gallery image.
#[tauri::command]
pub fn delete_gallery_image(
    assets: State<'_, AssetManager>,
    path: String,
) -> Result<(), String> {
    let asset_ref = AssetRef {
        path: std::path::PathBuf::from(&path),
        asset_type: crate::assets::AssetType::Gallery,
        content_hash: String::new(),
    };
    assets.delete_asset(&asset_ref)
}

/// Delete all gallery images for a game.
///
/// Returns the number of images deleted.
#[tauri::command]
pub fn delete_game_gallery(
    assets: State<'_, AssetManager>,
    game_id: String,
) -> Result<u64, String> {
    assets.delete_game_gallery(&game_id)
}

// ── Storage stats ─────────────────────────────────────────────────────────────

/// Return disk usage statistics for the asset store.
#[tauri::command]
pub fn get_storage_stats(
    assets: State<'_, AssetManager>,
) -> Result<StorageStats, String> {
    assets.get_storage_stats()
}

// ── Orphan cleanup ────────────────────────────────────────────────────────────

/// Delete asset files for games that no longer exist in the database.
///
/// Returns the number of files deleted and bytes freed.
#[tauri::command]
pub fn cleanup_orphan_assets(
    assets: State<'_, AssetManager>,
    db: State<'_, DbState>,
) -> Result<CleanupResult, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    assets.cleanup_orphans(&conn)
}

// ── Deduplication ─────────────────────────────────────────────────────────────

/// Check whether a file's content is already stored.
///
/// Returns an `AssetRef` if a duplicate is found (so the UI can reuse the
/// existing path), or `null` if the content is new.
#[tauri::command]
pub fn check_duplicate(
    assets: State<'_, AssetManager>,
    source_path: String,
) -> Result<Option<AssetRef>, String> {
    let source = Path::new(&source_path);
    assets.deduplicate(source)
}

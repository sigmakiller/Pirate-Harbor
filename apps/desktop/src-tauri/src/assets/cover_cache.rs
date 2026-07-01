//! Cover-specific asset processing — T28.
//!
//! Covers are stored at `assets/covers/{game_id}.webp`.
//! Standard dimensions: 512 × 512 (square, aspect-ratio preserved fit).

use std::path::{Path, PathBuf};

use image::imageops::FilterType;

/// Cover target dimensions.
pub const COVER_WIDTH:  u32 = 512;
pub const COVER_HEIGHT: u32 = 512;

/// Process a cover image: resize to fit within 512×512 preserving aspect
/// ratio, convert to WebP, and write to `dest`.
pub fn process_cover(source: &Path, dest: &Path) -> Result<(), String> {
    let img = image::open(source)
        .map_err(|e| format!("Failed to open cover source: {e}"))?;

    // Fit within 512×512 without stretching.
    let resized = img.resize(COVER_WIDTH, COVER_HEIGHT, FilterType::Lanczos3);

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create covers directory: {e}"))?;
    }

    resized
        .save_with_format(dest, image::ImageFormat::WebP)
        .map_err(|e| format!("Failed to save cover: {e}"))?;

    Ok(())
}

/// Canonical cover path for a game.
pub fn cover_path(covers_dir: &Path, game_id: &str) -> PathBuf {
    covers_dir.join(format!("{}.webp", game_id))
}

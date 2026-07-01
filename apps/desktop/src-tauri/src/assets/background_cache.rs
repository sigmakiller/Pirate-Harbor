//! Background-specific asset processing — T28.
//!
//! Backgrounds are stored at `assets/backgrounds/{game_id}.webp`.
//! Standard dimensions: 1920 × 1080 (16:9 widescreen, aspect-ratio preserved
//! fit — not cropped, so portrait images will have letterboxing on the sides).
//! Backgrounds are intended to be used with CSS `blur()` overlays, so
//! resolution matters more than tight cropping.

use std::path::{Path, PathBuf};

use image::imageops::FilterType;

/// Background target dimensions (1080p widescreen).
pub const BG_WIDTH:  u32 = 1920;
pub const BG_HEIGHT: u32 = 1080;

/// Process a background image: resize to fit within 1920×1080, convert to
/// WebP, and write to `dest`.
pub fn process_background(source: &Path, dest: &Path) -> Result<(), String> {
    let img = image::open(source)
        .map_err(|e| format!("Failed to open background source: {e}"))?;

    // Fit within 1920×1080 without stretching.
    let resized = img.resize(BG_WIDTH, BG_HEIGHT, FilterType::Lanczos3);

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create backgrounds directory: {e}"))?;
    }

    resized
        .save_with_format(dest, image::ImageFormat::WebP)
        .map_err(|e| format!("Failed to save background: {e}"))?;

    Ok(())
}

/// Canonical background path for a game.
pub fn background_path(backgrounds_dir: &Path, game_id: &str) -> PathBuf {
    backgrounds_dir.join(format!("{}.webp", game_id))
}

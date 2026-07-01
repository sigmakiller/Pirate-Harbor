//! Thumbnail generation for covers and gallery images — T28.
//!
//! Thumbnails are 256×256 WebP images stored in `assets/thumbnails/`.
//! They are named by the content hash of the source file, so the same
//! source image always produces the same thumbnail path.

use std::path::{Path, PathBuf};


/// Standard thumbnail dimensions.
pub const THUMB_WIDTH: u32  = 256;
pub const THUMB_HEIGHT: u32 = 256;

/// Generate a thumbnail for the image at `source` and write it to `dest`.
///
/// The thumbnail is square-cropped to fit within 256×256 using `Thumbnail`
/// resampling (fast, good quality for small sizes), then saved as WebP.
pub fn generate_thumbnail(source: &Path, dest: &Path) -> Result<(), String> {
    let img = image::open(source)
        .map_err(|e| format!("Failed to open source image for thumbnail: {e}"))?;

    let thumb = img.thumbnail(THUMB_WIDTH, THUMB_HEIGHT);

    // Ensure parent directory exists.
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create thumbnail directory: {e}"))?;
    }

    thumb
        .save_with_format(dest, image::ImageFormat::WebP)
        .map_err(|e| format!("Failed to save thumbnail: {e}"))?;

    Ok(())
}

/// Build the canonical thumbnail path for a given content hash.
pub fn thumbnail_path(thumbnails_dir: &Path, content_hash: u64) -> PathBuf {
    thumbnails_dir.join(format!("{:016x}_thumb.webp", content_hash))
}

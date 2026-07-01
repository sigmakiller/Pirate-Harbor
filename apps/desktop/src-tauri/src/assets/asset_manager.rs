//! Core asset manager — T28.
//!
//! `AssetManager` is the single entry-point for all image pipeline operations.
//! It is registered as Tauri state at app startup and injected into every
//! command that stores or retrieves game art.
//!
//! # Storage layout
//!
//! ```text
//! <app_data_dir>/assets/
//! ├── covers/          {game_id}.webp
//! ├── backgrounds/     {game_id}.webp
//! ├── gallery/         {game_id}/{uuid}.webp
//! └── thumbnails/      {hash}_thumb.webp   (auto-generated)
//!                      {hash}_dedup        (zero-byte sentinel for dedup)
//! ```
//!
//! # Thread safety
//!
//! `AssetManager` is `Send + Sync` — all operations use standard blocking
//! filesystem I/O.  Callers that need async behaviour should wrap calls in
//! `tokio::task::spawn_blocking`.

use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::background_cache;
use super::cover_cache;
use super::dedup;
use super::thumbnail_gen;

// ── Result types ──────────────────────────────────────────────────────────────

/// A handle to a stored asset — passed back to callers after `store_*` calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRef {
    /// Absolute filesystem path.
    pub path: PathBuf,
    /// Asset type tag.
    pub asset_type: AssetType,
    /// Content hash of the original source file (hex string).
    pub content_hash: String,
}

impl AssetRef {
    /// Return the path as a string suitable for storing in the database.
    // T34: used when writing cover_path_local back to the games table.
    #[allow(dead_code)]
    pub fn path_str(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}

/// Discriminator for the type of asset stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    Cover,
    Background,
    Gallery,
    Thumbnail,
}

/// Disk usage and count statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total bytes used by all assets.
    pub total_bytes: u64,
    /// Total bytes used by cover images.
    pub covers_bytes: u64,
    /// Total bytes used by background images.
    pub backgrounds_bytes: u64,
    /// Total bytes used by gallery images.
    pub gallery_bytes: u64,
    /// Total bytes used by thumbnails (includes dedup markers).
    pub thumbnails_bytes: u64,
    /// Total number of stored assets (not counting dedup markers).
    pub file_count: u64,
}

/// Result of an orphan cleanup run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    /// Number of asset files deleted.
    pub deleted_count: u64,
    /// Bytes reclaimed.
    pub bytes_freed: u64,
}

// ── AssetManager ──────────────────────────────────────────────────────────────

/// Centralized image pipeline for all asset types.
pub struct AssetManager {
    // T35: base_dir exposed for diagnostics (storage path display).
    #[allow(dead_code)]
    base_dir:        PathBuf,
    covers_dir:      PathBuf,
    backgrounds_dir: PathBuf,
    gallery_dir:     PathBuf,
    thumbnails_dir:  PathBuf,
}

impl AssetManager {
    /// Create a new `AssetManager` rooted at `<app_data_dir>/assets/`.
    /// Creates all subdirectories if they don't exist.
    pub fn new(app_data_dir: &Path) -> Result<Self, String> {
        let base_dir = app_data_dir.join("assets");

        let covers_dir      = base_dir.join("covers");
        let backgrounds_dir = base_dir.join("backgrounds");
        let gallery_dir     = base_dir.join("gallery");
        let thumbnails_dir  = base_dir.join("thumbnails");

        std::fs::create_dir_all(&covers_dir)
            .map_err(|e| format!("Failed to create covers dir: {e}"))?;
        std::fs::create_dir_all(&backgrounds_dir)
            .map_err(|e| format!("Failed to create backgrounds dir: {e}"))?;
        std::fs::create_dir_all(&gallery_dir)
            .map_err(|e| format!("Failed to create gallery dir: {e}"))?;
        std::fs::create_dir_all(&thumbnails_dir)
            .map_err(|e| format!("Failed to create thumbnails dir: {e}"))?;

        Ok(Self {
            base_dir,
            covers_dir,
            backgrounds_dir,
            gallery_dir,
            thumbnails_dir,
        })
    }

    // ── Deduplication ─────────────────────────────────────────────────────────

    /// Check if a file with the same content already exists.
    ///
    /// Returns `Some(AssetRef)` if a duplicate is found, `None` if the
    /// content is new.  The returned `AssetRef` points to the *existing* file
    /// so the caller can reuse it without copying.
    pub fn deduplicate(&self, source: &Path) -> Result<Option<AssetRef>, String> {
        let hash = dedup::hash_file(source)
            .map_err(|e| format!("Failed to hash file: {e}"))?;

        let marker = dedup::dedup_marker_path(&self.thumbnails_dir, hash);

        if marker.exists() {
            // Duplicate detected — try to find the corresponding cover or
            // background (the caller's responsibility to check the right dir).
            // We return an AssetRef pointing to the thumbnail so the caller
            // knows the hash; they can reconstruct the cover path if needed.
            let thumb = thumbnail_gen::thumbnail_path(&self.thumbnails_dir, hash);
            if thumb.exists() {
                return Ok(Some(AssetRef {
                    path:         thumb,
                    asset_type:   AssetType::Thumbnail,
                    content_hash: dedup::hash_to_hex(hash),
                }));
            }
        }

        Ok(None)
    }

    /// Write the dedup sentinel for a given file hash.
    fn write_dedup_marker(&self, hash: u64) -> Result<(), String> {
        let marker = dedup::dedup_marker_path(&self.thumbnails_dir, hash);
        if !marker.exists() {
            std::fs::write(&marker, b"")
                .map_err(|e| format!("Failed to write dedup marker: {e}"))?;
        }
        Ok(())
    }

    // ── Store operations ──────────────────────────────────────────────────────

    /// Store a cover image for `game_id`.
    ///
    /// The image is resized to 512×512 and converted to WebP.
    /// An auto-generated thumbnail is created in `thumbnails/`.
    ///
    /// Returns an `AssetRef` pointing to the stored cover.
    pub fn store_cover(&self, game_id: &str, source: &Path) -> Result<AssetRef, String> {
        let hash = dedup::hash_file(source)
            .map_err(|e| format!("Failed to hash cover source: {e}"))?;

        let dest = cover_cache::cover_path(&self.covers_dir, game_id);
        cover_cache::process_cover(source, &dest)?;

        // Generate thumbnail.
        let thumb_dest = thumbnail_gen::thumbnail_path(&self.thumbnails_dir, hash);
        if !thumb_dest.exists() {
            thumbnail_gen::generate_thumbnail(&dest, &thumb_dest)?;
        }

        self.write_dedup_marker(hash)?;

        Ok(AssetRef {
            path:         dest,
            asset_type:   AssetType::Cover,
            content_hash: dedup::hash_to_hex(hash),
        })
    }

    /// Store a background image for `game_id`.
    ///
    /// Resized to 1920×1080, converted to WebP.
    pub fn store_background(&self, game_id: &str, source: &Path) -> Result<AssetRef, String> {
        let hash = dedup::hash_file(source)
            .map_err(|e| format!("Failed to hash background source: {e}"))?;

        let dest = background_cache::background_path(&self.backgrounds_dir, game_id);
        background_cache::process_background(source, &dest)?;

        self.write_dedup_marker(hash)?;

        Ok(AssetRef {
            path:         dest,
            asset_type:   AssetType::Background,
            content_hash: dedup::hash_to_hex(hash),
        })
    }

    /// Store a gallery image for `game_id`.
    ///
    /// Gallery images are stored at `gallery/{game_id}/{uuid}.webp`.
    /// No resize is applied — images are stored as-is but converted to WebP.
    /// The gallery limit (default 50, soft cap 100) is enforced by the
    /// `gallery` command, not here.
    pub fn store_gallery_image(
        &self,
        game_id: &str,
        source: &Path,
    ) -> Result<AssetRef, String> {
        let hash = dedup::hash_file(source)
            .map_err(|e| format!("Failed to hash gallery source: {e}"))?;

        let game_gallery_dir = self.gallery_dir.join(game_id);
        std::fs::create_dir_all(&game_gallery_dir)
            .map_err(|e| format!("Failed to create gallery dir for game: {e}"))?;

        let filename = format!("{}.webp", Uuid::new_v4());
        let dest = game_gallery_dir.join(&filename);

        // Convert to WebP without resizing.
        let img = image::open(source)
            .map_err(|e| format!("Failed to open gallery image: {e}"))?;
        img.save_with_format(&dest, image::ImageFormat::WebP)
            .map_err(|e| format!("Failed to save gallery image: {e}"))?;

        // Generate thumbnail.
        let thumb_dest = thumbnail_gen::thumbnail_path(&self.thumbnails_dir, hash);
        if !thumb_dest.exists() {
            thumbnail_gen::generate_thumbnail(&dest, &thumb_dest)?;
        }

        self.write_dedup_marker(hash)?;

        Ok(AssetRef {
            path:         dest,
            asset_type:   AssetType::Gallery,
            content_hash: dedup::hash_to_hex(hash),
        })
    }

    // ── Retrieve operations ───────────────────────────────────────────────────

    /// Return all gallery images for `game_id`, sorted by filename (which is
    /// UUID-based, so roughly chronological).
    pub fn get_gallery_images(&self, game_id: &str) -> Result<Vec<AssetRef>, String> {
        let game_gallery_dir = self.gallery_dir.join(game_id);

        if !game_gallery_dir.exists() {
            return Ok(Vec::new());
        }

        let mut refs = Vec::new();
        let entries = std::fs::read_dir(&game_gallery_dir)
            .map_err(|e| format!("Failed to read gallery dir: {e}"))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("webp") {
                refs.push(AssetRef {
                    path,
                    asset_type:   AssetType::Gallery,
                    content_hash: String::new(), // Not computed on read for performance.
                });
            }
        }

        refs.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(refs)
    }

    /// Return the cover path for a game if it exists.
    pub fn get_cover(&self, game_id: &str) -> Option<PathBuf> {
        let p = cover_cache::cover_path(&self.covers_dir, game_id);
        if p.exists() { Some(p) } else { None }
    }

    /// Return the background path for a game if it exists.
    // T34: used in GameDetailPage to load the background image.
    #[allow(dead_code)]
    pub fn get_background(&self, game_id: &str) -> Option<PathBuf> {
        let p = background_cache::background_path(&self.backgrounds_dir, game_id);
        if p.exists() { Some(p) } else { None }
    }

    // ── Delete operations ─────────────────────────────────────────────────────

    /// Delete a single asset file.  Returns an error if the file doesn't exist
    /// or the deletion fails; silently succeeds if the file is already gone.
    pub fn delete_asset(&self, asset_ref: &AssetRef) -> Result<(), String> {
        if asset_ref.path.exists() {
            std::fs::remove_file(&asset_ref.path)
                .map_err(|e| format!("Failed to delete asset {}: {e}", asset_ref.path.display()))?;
        }
        Ok(())
    }

    /// Delete all gallery images for `game_id` (entire directory).
    pub fn delete_game_gallery(&self, game_id: &str) -> Result<u64, String> {
        let game_gallery_dir = self.gallery_dir.join(game_id);
        if !game_gallery_dir.exists() {
            return Ok(0);
        }

        let count = count_files(&game_gallery_dir);
        std::fs::remove_dir_all(&game_gallery_dir)
            .map_err(|e| format!("Failed to delete gallery for game {game_id}: {e}"))?;

        Ok(count)
    }

    /// Delete a cover by game ID.
    pub fn delete_cover(&self, game_id: &str) -> Result<(), String> {
        let p = cover_cache::cover_path(&self.covers_dir, game_id);
        if p.exists() {
            std::fs::remove_file(&p)
                .map_err(|e| format!("Failed to delete cover for {game_id}: {e}"))?;
        }
        Ok(())
    }

    // ── Storage stats ─────────────────────────────────────────────────────────

    /// Return disk usage statistics for all asset directories.
    pub fn get_storage_stats(&self) -> Result<StorageStats, String> {
        let covers_bytes      = dir_size(&self.covers_dir)?;
        let backgrounds_bytes = dir_size(&self.backgrounds_dir)?;
        let gallery_bytes     = dir_size_recursive(&self.gallery_dir)?;
        let thumbnails_bytes  = dir_size(&self.thumbnails_dir)?;

        let total_bytes = covers_bytes + backgrounds_bytes + gallery_bytes + thumbnails_bytes;
        let file_count  = count_files(&self.covers_dir)
            + count_files(&self.backgrounds_dir)
            + count_files_recursive(&self.gallery_dir)
            + count_webp_files(&self.thumbnails_dir); // excludes dedup markers

        Ok(StorageStats {
            total_bytes,
            covers_bytes,
            backgrounds_bytes,
            gallery_bytes,
            thumbnails_bytes,
            file_count,
        })
    }

    // ── Orphan cleanup ────────────────────────────────────────────────────────

    /// Delete asset files that are no longer referenced by any game in the DB.
    ///
    /// An asset is considered orphaned when its `game_id` does not appear in
    /// the `games` table.  Gallery images are checked against `game_id`
    /// subdirectory names.
    pub fn cleanup_orphans(&self, conn: &Connection) -> Result<CleanupResult, String> {
        // Fetch all known game IDs.
        let game_ids: std::collections::HashSet<String> = conn
            .prepare("SELECT id FROM games")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| row.get::<_, String>(0))
                    .map(|iter| iter.flatten().collect())
            })
            .map_err(|e| format!("Failed to query game IDs: {e}"))?;

        let mut deleted_count: u64 = 0;
        let mut bytes_freed: u64 = 0;

        // Check covers.
        if let Ok(entries) = std::fs::read_dir(&self.covers_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if !game_ids.contains(stem) {
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        if std::fs::remove_file(&path).is_ok() {
                            deleted_count += 1;
                            bytes_freed   += size;
                        }
                    }
                }
            }
        }

        // Check backgrounds.
        if let Ok(entries) = std::fs::read_dir(&self.backgrounds_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if !game_ids.contains(stem) {
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        if std::fs::remove_file(&path).is_ok() {
                            deleted_count += 1;
                            bytes_freed   += size;
                        }
                    }
                }
            }
        }

        // Check gallery subdirectories.
        if let Ok(entries) = std::fs::read_dir(&self.gallery_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let game_id = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    if !game_ids.contains(game_id) {
                        let size = dir_size_recursive(&path).unwrap_or(0);
                        if std::fs::remove_dir_all(&path).is_ok() {
                            bytes_freed   += size;
                            // Count approximate number of deleted files.
                            deleted_count += 1; // (directory as one unit for simplicity)
                        }
                    }
                }
            }
        }

        Ok(CleanupResult { deleted_count, bytes_freed })
    }

    // ── Accessors for sub-directories (T34/T35) ───────────────────────────────
    // Suppressed until consumed by gallery command (T34) and diagnostics (T35).
    #[allow(dead_code)] pub fn base_dir(&self) -> &Path { &self.base_dir }
    #[allow(dead_code)] pub fn covers_dir(&self) -> &Path { &self.covers_dir }
    #[allow(dead_code)] pub fn backgrounds_dir(&self) -> &Path { &self.backgrounds_dir }
    #[allow(dead_code)] pub fn gallery_dir(&self) -> &Path { &self.gallery_dir }
    #[allow(dead_code)] pub fn thumbnails_dir(&self) -> &Path { &self.thumbnails_dir }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Sum of sizes of all *direct* children (non-recursive).
fn dir_size(dir: &Path) -> Result<u64, String> {
    if !dir.exists() { return Ok(0); }
    std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read dir {}: {e}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum::<u64>()
        .pipe(Ok)
}

/// Recursive sum of sizes.
fn dir_size_recursive(dir: &Path) -> Result<u64, String> {
    if !dir.exists() { return Ok(0); }
    let mut total = 0u64;
    for entry in walkdir::WalkDir::new(dir).min_depth(1) {
        if let Ok(e) = entry {
            if e.file_type().is_file() {
                total += e.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    Ok(total)
}

/// Count non-dedup-marker files directly in `dir`.
fn count_files(dir: &Path) -> u64 {
    if !dir.exists() { return 0; }
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .count() as u64
        })
        .unwrap_or(0)
}

/// Count `.webp` files in `dir` (excluding `_dedup` sentinels).
fn count_webp_files(dir: &Path) -> u64 {
    if !dir.exists() { return 0; }
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().and_then(|s| s.to_str()) == Some("webp")
                })
                .count() as u64
        })
        .unwrap_or(0)
}

/// Recursively count files.
fn count_files_recursive(dir: &Path) -> u64 {
    if !dir.exists() { return 0; }
    walkdir::WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count() as u64
}

// ── Pipe helper ───────────────────────────────────────────────────────────────

trait Pipe: Sized {
    fn pipe<F: FnOnce(Self) -> R, R>(self, f: F) -> R { f(self) }
}

impl<T> Pipe for T {}

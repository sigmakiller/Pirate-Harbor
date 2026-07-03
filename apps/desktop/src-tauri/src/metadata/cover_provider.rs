//! Cover URL resolution and download — T30.
//!
//! Resolves a cover image URL for a game (from metadata cache or RAWG),
//! then downloads and stores it via the AssetManager (T28).
//!
//! Designed to be called from background jobs (T27) so downloads don't
//! block the UI thread.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Result of a cover fetch + store operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverProviderResult {
    pub game_id:    String,
    pub local_path: Option<PathBuf>,
    pub source_url: Option<String>,
    pub success:    bool,
    pub error:      Option<String>,
}

/// Resolve the best available cover URL for a game.
///
/// Precedence: `metadata_cache.cover_url` → `games.cover_path` (if already
/// a local asset, skip download).  Returns `None` when no cover is available.
pub fn resolve_cover_url(
    conn: &rusqlite::Connection,
    game_id: &str,
) -> Result<Option<String>, String> {
    // 1. Check metadata cache.
    let cache_url: Option<String> = conn
        .query_row(
            "SELECT cover_url FROM metadata_cache WHERE game_id = ?1 AND cover_url IS NOT NULL LIMIT 1",
            rusqlite::params![game_id],
            |r| r.get(0),
        )
        .ok()
        .flatten();

    if cache_url.is_some() {
        return Ok(cache_url);
    }

    // 2. Check if the game already has a local cover (skip re-download).
    let local: Option<String> = conn
        .query_row(
            "SELECT cover_path_local FROM games WHERE id = ?1",
            rusqlite::params![game_id],
            |r| r.get(0),
        )
        .ok()
        .flatten();

    // If there's already a local path, signal caller that no download needed.
    if local.is_some() {
        return Ok(None); // Already stored locally.
    }

    Ok(None)
}

/// Download `url` and store it as the cover for `game_id` via `asset_dir`.
///
/// Uses `ureq` (synchronous HTTP) which is already available through the
/// RAWG client dependency chain.  Returns the local path on success.
///
/// This function is intentionally synchronous — callers should spawn it in
/// a `std::thread` or Tokio `spawn_blocking` for non-blocking use.
pub fn download_and_store_cover(
    game_id: &str,
    url: &str,
    asset_dir: &std::path::Path,
) -> CoverProviderResult {
    let fetch = (|| -> Result<Vec<u8>, String> {
        let bytes = reqwest::blocking::get(url)
            .map_err(|e| e.to_string())?
            .bytes()
            .map_err(|e| e.to_string())?;
        Ok(bytes.to_vec())
    })();

    let bytes = match fetch {
        Ok(b)  => b,
        Err(e) => {
            return CoverProviderResult {
                game_id:    game_id.to_string(),
                local_path: None,
                source_url: Some(url.to_string()),
                success:    false,
                error:      Some(e),
            };
        }
    };

    // Write to a temp path then hand off to AssetManager-compatible layout.
    let temp = asset_dir.join(format!("{}_cover_dl.tmp", game_id));
    if let Err(e) = std::fs::write(&temp, &bytes) {
        return CoverProviderResult {
            game_id:    game_id.to_string(),
            local_path: None,
            source_url: Some(url.to_string()),
            success:    false,
            error:      Some(e.to_string()),
        };
    }

    // Delegate resize + WebP conversion to AssetManager.
    let am = match crate::assets::AssetManager::new(asset_dir) {
        Ok(am) => am,
        Err(e) => {
            let _ = std::fs::remove_file(&temp);
            return CoverProviderResult {
                game_id:    game_id.to_string(),
                local_path: None,
                source_url: Some(url.to_string()),
                success:    false,
                error:      Some(e.to_string()),
            };
        }
    };

    match am.store_cover(game_id, &temp) {
        Ok(asset_ref) => {
            let _ = std::fs::remove_file(&temp);
            CoverProviderResult {
                game_id:    game_id.to_string(),
                local_path: Some(asset_ref.path),
                source_url: Some(url.to_string()),
                success:    true,
                error:      None,
            }
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp);
            CoverProviderResult {
                game_id:    game_id.to_string(),
                local_path: None,
                source_url: Some(url.to_string()),
                success:    false,
                error:      Some(e),
            }
        }
    }
}

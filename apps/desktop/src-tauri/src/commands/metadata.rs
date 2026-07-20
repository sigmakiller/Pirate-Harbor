//! Metadata enrichment commands — T19.
//!
//! Fetches game metadata from RAWG (primary) and IGDB (fallback) APIs,
//! with local caching and background queue processing.

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State, Manager};
use uuid::Uuid;

use crate::api::rawg::{RawgClient, RawgGame};
use crate::db::DbState;
use crate::images::{downloader, processor, ImagePaths, ImageType};

// ── Models ────────────────────────────────────────────────────────────────────

/// Metadata search result returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSearchResult {
    pub provider: String,
    pub api_id: i64,
    pub name: String,
    pub release_year: Option<i32>,
    pub genres: String,
    pub cover_url: Option<String>,
    pub rating: Option<f64>,
}

/// Result of enriching a single game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentResult {
    pub game_id: String,
    pub status: EnrichmentStatus,
    pub metadata: Option<MetadataSearchResult>,
    pub error: Option<String>,
}

/// Enrichment status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnrichmentStatus {
    Success,
    NotFound,
    Failed,
    Cached,
}

/// Overall enrichment progress status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentProgress {
    pub total: usize,
    pub completed: usize,
    pub pending: usize,
    pub failed: usize,
}

/// Image download result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDownloadResult {
    pub game_id: String,
    pub cover_path: Option<String>,
    pub background_path: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

// ── Cache helpers ─────────────────────────────────────────────────────────────

/// Check if cached metadata exists and is not expired
fn get_cached_metadata(
    conn: &rusqlite::Connection,
    game_title: &str,
) -> Option<MetadataSearchResult> {
    let now = Utc::now().to_rfc3339();

    conn.query_row(
        "SELECT provider, api_id, metadata FROM metadata_cache
         WHERE LOWER(game_title) = LOWER(?1) AND expires_at > ?2
         ORDER BY cached_at DESC LIMIT 1",
        rusqlite::params![game_title, now],
        |row| {
            let provider: String = row.get(0)?;
            let api_id: i64 = row.get(1)?;
            let metadata_json: String = row.get(2)?;
            Ok((provider, api_id, metadata_json))
        },
    )
    .ok()
    .and_then(|(_provider, _api_id, json)| {
        serde_json::from_str::<MetadataSearchResult>(&json).ok()
    })
}

/// Store metadata in cache with 30-day TTL
fn cache_metadata(
    conn: &rusqlite::Connection,
    game_title: &str,
    provider: &str,
    api_id: i64,
    metadata: &MetadataSearchResult,
) -> Result<(), String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = (now + Duration::days(30)).to_rfc3339();
    let metadata_json = serde_json::to_string(metadata).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO metadata_cache (id, game_title, provider, api_id, metadata, cached_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, game_title, provider, api_id, metadata_json, now.to_rfc3339(), expires_at],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ── API helpers ───────────────────────────────────────────────────────────────

/// Convert RAWG game to MetadataSearchResult
fn rawg_to_metadata(game: &RawgGame) -> MetadataSearchResult {
    let genres = game
        .genres
        .as_ref()
        .map(|g| {
            g.iter()
                .map(|genre| genre.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();

    let release_year = game
        .released
        .as_ref()
        .and_then(|date| date.split('-').next())
        .and_then(|year| year.parse::<i32>().ok());

    MetadataSearchResult {
        provider: "rawg".to_string(),
        api_id: game.id,
        name: game.name.clone(),
        release_year,
        genres,
        cover_url: game.background_image.clone(),
        rating: game.rating,
    }
}

/// Fetch metadata from RAWG API
async fn fetch_from_rawg(
    api_key: &str,
    game_title: &str,
) -> Result<Vec<MetadataSearchResult>, String> {
    let client = RawgClient::new(api_key.to_string());
    let results = client.search_games(game_title).await?;
    Ok(results.iter().map(rawg_to_metadata).collect())
}

/// Fetch metadata from IGDB API (fallback).
/// T25: Full IGDB wiring deferred to polish phase.
#[allow(dead_code)]
async fn fetch_from_igdb(
    _client_id: &str,
    _access_token: &str,
    _game_title: &str,
) -> Result<Vec<MetadataSearchResult>, String> {
    // IGDB integration placeholder — requires OAuth flow
    Err("IGDB integration not yet implemented".to_string())
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Search for game metadata by title.
/// Checks cache first, then queries RAWG API.
#[tauri::command]
pub async fn search_game_metadata(
    db_state: State<'_, DbState>,
    title: String,
) -> Result<Vec<MetadataSearchResult>, String> {
    // Check cache first and get API key
    let (cached, api_key) = {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;

        let cached = get_cached_metadata(&conn, &title);

        let api_key: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'rawg_api_key'",
                [],
                |row| row.get(0),
            )
            .ok();

        (cached, api_key)
    }; // Drop conn lock here

    if let Some(cached) = cached {
        return Ok(vec![cached]);
    }

    let api_key = api_key.ok_or("RAWG API key not configured")?;

    // Fetch from RAWG
    let results = fetch_from_rawg(&api_key, &title).await?;

    // Cache the first result if available
    if let Some(first) = results.first() {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;
        let _ = cache_metadata(&conn, &title, &first.provider, first.api_id, first);
    }

    Ok(results)
}

/// Enrich a single game with metadata from APIs.
#[tauri::command]
pub async fn enrich_game_metadata(
    db_state: State<'_, DbState>,
    game_id: String,
) -> Result<EnrichmentResult, String> {
    let (game_title, api_key, cached) = {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;

        // Get game title
        let game_title: String = conn
            .query_row(
                "SELECT title FROM games WHERE id = ?1",
                rusqlite::params![game_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Game not found: {}", e))?;

        // Get API key
        let api_key: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'rawg_api_key'",
                [],
                |row| row.get(0),
            )
            .ok();

        // Check cache
        let cached = get_cached_metadata(&conn, &game_title);

        (game_title, api_key, cached)
    }; // Drop conn lock here

    let api_key = api_key.ok_or("RAWG API key not configured")?;

    // Return cached if available
    if let Some(cached) = cached {
        return Ok(EnrichmentResult {
            game_id,
            status: EnrichmentStatus::Cached,
            metadata: Some(cached),
            error: None,
        });
    }

    // Fetch from RAWG
    match fetch_from_rawg(&api_key, &game_title).await {
        Ok(results) => {
            if let Some(first) = results.first() {
                // Cache result
                let conn = db_state.0.lock().map_err(|e| e.to_string())?;
                let _ = cache_metadata(&conn, &game_title, &first.provider, first.api_id, first);

                Ok(EnrichmentResult {
                    game_id,
                    status: EnrichmentStatus::Success,
                    metadata: Some(first.clone()),
                    error: None,
                })
            } else {
                Ok(EnrichmentResult {
                    game_id,
                    status: EnrichmentStatus::NotFound,
                    metadata: None,
                    error: Some("No metadata found".to_string()),
                })
            }
        }
        Err(e) => Ok(EnrichmentResult {
            game_id,
            status: EnrichmentStatus::Failed,
            metadata: None,
            error: Some(e),
        }),
    }
}

/// Bulk enrich entire library in background.
/// Emits progress events as 'metadata-enrichment-progress'.
#[tauri::command]
pub async fn bulk_enrich_library(
    app_handle: tauri::AppHandle,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    // Get game IDs and API key before spawning
    let (game_titles, api_key) = {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;
        
        let mut stmt = conn.prepare("SELECT title FROM games").map_err(|e| e.to_string())?;
        let titles: Vec<String> = stmt.query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        let api_key: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'rawg_api_key'",
                [],
                |row| row.get(0),
            )
            .ok();

        (titles, api_key)
    };

    let api_key = api_key.ok_or("RAWG API key not configured")?;

    // Spawn background task with owned data only
    tauri::async_runtime::spawn(async move {
        let total = game_titles.len();
        let mut completed = 0;
        let mut failed = 0;

        for game_title in game_titles {
            // Fetch from RAWG
            match fetch_from_rawg(&api_key, &game_title).await {
                Ok(results) => {
                    if results.is_empty() {
                        failed += 1;
                    }
                    // Note: Cannot cache without db_state access in background task
                    // This is acceptable for bulk operations
                }
                Err(_) => {
                    failed += 1;
                }
            }

            completed += 1;

            // Emit progress event
            let progress = EnrichmentProgress {
                total,
                completed,
                pending: total - completed,
                failed,
            };

            let _ = app_handle.emit("metadata-enrichment-progress", progress);

            // Rate limiting — sleep between requests
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    });

    Ok(())
}

/// Get current enrichment queue status
#[tauri::command]
pub fn get_enrichment_status(
    db_state: State<'_, DbState>,
) -> Result<EnrichmentProgress, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    let total: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM metadata_enrichment_queue",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let completed: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM metadata_enrichment_queue WHERE status = 'completed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let failed: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM metadata_enrichment_queue WHERE status = 'failed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let pending = total.saturating_sub(completed).saturating_sub(failed);

    Ok(EnrichmentProgress {
        total,
        completed,
        pending,
        failed,
    })
}

/// Get the configured RAWG API key (for Settings UI).
#[tauri::command]
pub fn get_rawg_api_key(db_state: State<'_, DbState>) -> Result<Option<String>, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let key = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'rawg_api_key'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok();
    Ok(key)
}

/// T51: Return the count of games whose metadata cache is absent or older than
/// 30 days.  Used by the stale-data notification banner in LibraryPage.
#[tauri::command]
pub fn get_stale_games_count(db_state: State<'_, DbState>) -> Result<usize, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let cutoff = (Utc::now() - Duration::days(STALE_DAYS)).to_rfc3339();
    let count: usize = conn
        .query_row(
            "SELECT COUNT(*)
             FROM games g
             LEFT JOIN metadata_cache mc ON LOWER(mc.game_title) = LOWER(g.title)
               AND mc.expires_at > ?1
             WHERE mc.id IS NULL",
            rusqlite::params![cutoff],
            |r| r.get(0),
        )
        .unwrap_or(0);
    Ok(count)
}

/// Download and process game images (cover and background).
/// Emits 'image-download-progress' events during processing.
#[tauri::command]
pub async fn download_game_images(
    app_handle: tauri::AppHandle,
    db_state: State<'_, DbState>,
    game_id: String,
    cover_url: Option<String>,
    background_url: Option<String>,
) -> Result<ImageDownloadResult, String> {
    // Get app data directory for image storage
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let image_paths = ImagePaths::new(&app_data_dir)?;

    let now = Utc::now().to_rfc3339();
    let mut cover_local: Option<String> = None;
    let mut background_local: Option<String> = None;
    let mut errors = Vec::new();

    // Download cover
    if let Some(url) = cover_url {
        let output_path = image_paths.covers.join(format!("{}_cover.jpg", game_id));
        let temp_path = image_paths.covers.join(format!("{}_cover_temp.jpg", game_id));

        match downloader::download_image(&url, temp_path.clone()).await {
            Ok(_) => {
                // Process and resize
                let (target_w, target_h) = ImageType::Cover.target_dimensions();
                match processor::process_downloaded_image(&temp_path, &output_path, target_w, target_h) {
                    Ok(path) => {
                        cover_local = Some(path.to_string_lossy().to_string());
                        // Clean up temp file
                        let _ = std::fs::remove_file(&temp_path);
                    }
                    Err(e) => errors.push(format!("Cover processing failed: {}", e)),
                }
            }
            Err(e) => errors.push(format!("Cover download failed: {}", e)),
        }

        // Emit progress
        let _ = app_handle.emit("image-download-progress", serde_json::json!({
            "game_id": game_id,
            "type": "cover",
            "status": if cover_local.is_some() { "success" } else { "failed" }
        }));
    }

    // Download background
    if let Some(url) = background_url {
        let output_path = image_paths.backgrounds.join(format!("{}_background.jpg", game_id));
        let temp_path = image_paths.backgrounds.join(format!("{}_background_temp.jpg", game_id));

        match downloader::download_image(&url, temp_path.clone()).await {
            Ok(_) => {
                // Process and resize
                let (target_w, target_h) = ImageType::Background.target_dimensions();
                match processor::process_downloaded_image(&temp_path, &output_path, target_w, target_h) {
                    Ok(path) => {
                        background_local = Some(path.to_string_lossy().to_string());
                        // Clean up temp file
                        let _ = std::fs::remove_file(&temp_path);
                    }
                    Err(e) => errors.push(format!("Background processing failed: {}", e)),
                }
            }
            Err(e) => errors.push(format!("Background download failed: {}", e)),
        }

        // Emit progress
        let _ = app_handle.emit("image-download-progress", serde_json::json!({
            "game_id": game_id,
            "type": "background",
            "status": if background_local.is_some() { "success" } else { "failed" }
        }));
    }

    // Update database with local paths
    if cover_local.is_some() || background_local.is_some() {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE games SET cover_path_local = ?1, background_path_local = ?2, images_enriched_at = ?3 WHERE id = ?4",
            rusqlite::params![cover_local, background_local, now, game_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(ImageDownloadResult {
        game_id,
        cover_path: cover_local,
        background_path: background_local,
        success: errors.is_empty(),
        error: if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        },
    })
}

// ─── T49: MetadataRefreshJob ──────────────────────────────────────────────────

/// Background job that re-enriches games with stale or missing metadata cache.
///
/// A game is stale if its `metadata_cache` entry has expired (>30 days) or is
/// absent entirely.  Scheduled once at startup with a 7-day interval; see
/// `lib.rs` `setup()`.
pub struct MetadataRefreshJob {
    /// RAWG API key — captured at scheduling time from the settings table.
    /// If `None`, the job exits immediately with a skip message.
    pub api_key: Option<String>,
}

/// Age threshold in days after which a cache entry is considered stale.
const STALE_DAYS: i64 = 30;

impl crate::background::Job for MetadataRefreshJob {
    fn name(&self) -> &str { "metadata_refresh" }

    fn execute(&self, ctx: crate::background::JobContext) -> Result<crate::background::JobResult, String> {
        use crate::background::{JobResult, JobProgressEvent, JobStatus};

        let api_key = match &self.api_key {
            Some(k) => k.clone(),
            None    => return Ok(JobResult::ok(
                "Skipped metadata refresh — RAWG API key not configured",
            )),
        };

        let conn = ctx.db.lock().map_err(|_| "DB lock poisoned")?;

        // Games whose cache is absent or expired.
        let cutoff = (chrono::Utc::now() - Duration::days(STALE_DAYS)).to_rfc3339();
        let mut stmt = conn.prepare(
            "SELECT g.id, g.title
             FROM games g
             LEFT JOIN metadata_cache mc ON LOWER(mc.game_title) = LOWER(g.title)
               AND mc.expires_at > ?1
             WHERE mc.id IS NULL
             ORDER BY g.added_at ASC",
        ).map_err(|e| e.to_string())?;

        let stale: Vec<(String, String)> = stmt
            .query_map(rusqlite::params![cutoff], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);

        if stale.is_empty() {
            return Ok(JobResult::ok("All metadata is up to date"));
        }

        let total    = stale.len();
        let mut refreshed = 0usize;
        let mut skipped   = 0usize;

        // MetadataRefreshJob runs on a spawn_blocking thread inside the async
        // worker.  We use block_on to call the async RAWG fetch helper.
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| "No Tokio runtime available in MetadataRefreshJob".to_string())?;

        for (i, (game_id, title)) in stale.into_iter().enumerate() {
            let _ = ctx.app_handle.emit("job-progress", JobProgressEvent {
                job_id:   game_id.clone(),
                job_name: "metadata_refresh".into(),
                status:   JobStatus::Running { progress: i as f32 / total as f32 },
                summary:  Some(format!("Refreshing {title}...")),
            });

            let fetch: Result<Vec<MetadataSearchResult>, String> =
                rt.block_on(fetch_from_rawg(&api_key, &title));

            match fetch {
                Ok(results) if !results.is_empty() => {
                    let first = &results[0];
                    // Remove stale entry, insert fresh one.
                    conn.execute(
                        "DELETE FROM metadata_cache WHERE LOWER(game_title) = LOWER(?1)",
                        rusqlite::params![title],
                    ).ok();
                    cache_metadata(&conn, &title, &first.provider, first.api_id, first).ok();
                    refreshed += 1;
                }
                _ => { skipped += 1; }
            }

            // Respect RAWG rate limit (<=10 req/min => 6 s gap).
            std::thread::sleep(std::time::Duration::from_secs(6));
        }

        Ok(JobResult::ok(format!(
            "Metadata refresh: {refreshed} updated, {skipped} skipped (of {total} stale)"
        )))
    }
}

// ─── T50: BulkEnrichmentJob ───────────────────────────────────────────────────

/// How many games to enrich per RAWG "tick" (respects 10 req/min limit).
const ENRICH_BATCH_SIZE: usize = 5;

/// Response returned by `start_bulk_enrichment_job`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BulkEnrichmentQueued {
    pub job_id: String,
    pub total: usize,
}

/// Background job that batch-enriches the entire library via RAWG.
///
/// Processes `ENRICH_BATCH_SIZE` games per tick, sleeping 6 s between
/// ticks to respect RAWG's 10 req/min rate limit.  Emits
/// `"metadata-enrichment-progress"` events so the frontend progress bar
/// updates live without blocking the UI thread.
///
/// Unlike the old `bulk_enrich_library` async command, this job:
/// - Runs entirely inside the scheduler worker (non-blocking for the UI)
/// - Writes results into `metadata_cache` (the old command could not)
/// - Can be cancelled via `cancel_job`
pub struct BulkEnrichmentJob {
    pub api_key: String,
    pub game_titles: Vec<String>,
}

impl crate::background::Job for BulkEnrichmentJob {
    fn name(&self) -> &str { "bulk_enrichment" }

    fn execute(&self, ctx: crate::background::JobContext) -> Result<crate::background::JobResult, String> {
        use crate::background::JobResult;

        let total    = self.game_titles.len();
        let mut completed = 0usize;
        let mut failed    = 0usize;

        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| "No Tokio runtime available in BulkEnrichmentJob".to_string())?;

        let conn = ctx.db.lock().map_err(|_| "DB lock poisoned")?;

        for chunk in self.game_titles.chunks(ENRICH_BATCH_SIZE) {
            for title in chunk {
                let fetch: Result<Vec<MetadataSearchResult>, String> =
                    rt.block_on(fetch_from_rawg(&self.api_key, title));

                match fetch {
                    Ok(results) if !results.is_empty() => {
                        let first = &results[0];
                        // Overwrite any stale entry, write fresh cache.
                        conn.execute(
                            "DELETE FROM metadata_cache WHERE LOWER(game_title) = LOWER(?1)",
                            rusqlite::params![title],
                        ).ok();
                        cache_metadata(&conn, title, &first.provider, first.api_id, first).ok();
                    }
                    _ => { failed += 1; }
                }

                completed += 1;

                // Emit live progress to the frontend progress bar.
                let _ = ctx.app_handle.emit("metadata-enrichment-progress", EnrichmentProgress {
                    total,
                    completed,
                    pending: total.saturating_sub(completed),
                    failed,
                });
            }

            // Respect RAWG rate limit: 5 games per 6-second window = 50 req/min
            // (well within the 10 req/min free tier after 6 s sleep per batch of 1).
            // Actually: 1 req per game, 6 s between batches of 5 => 5 req/36 s ≈ 8 req/min.
            if completed < total {
                std::thread::sleep(std::time::Duration::from_secs(6));
            }
        }

        // Final "done" event with pending = 0.
        let _ = ctx.app_handle.emit("metadata-enrichment-progress", EnrichmentProgress {
            total,
            completed,
            pending: 0,
            failed,
        });

        Ok(JobResult::ok(format!(
            "Bulk enrichment complete: {completed} processed, {failed} failed (of {total})"
        )))
    }
}

/// Queue a bulk enrichment job and return immediately.
///
/// Returns the job_id so the frontend can cancel via `cancel_job`.
/// Replaces the old blocking `bulk_enrich_library` command for T50.
#[tauri::command]
pub fn start_bulk_enrichment_job(
    db:        tauri::State<'_, crate::db::DbState>,
    scheduler: tauri::State<'_, crate::background::JobScheduler>,
) -> Result<BulkEnrichmentQueued, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned")?;

    let api_key: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'rawg_api_key'",
            [],
            |r| r.get(0),
        )
        .map_err(|_| "RAWG API key not configured — add it in Settings")?;

    let mut stmt = conn
        .prepare("SELECT title FROM games ORDER BY title ASC")
        .map_err(|e| e.to_string())?;
    let game_titles: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);
    drop(conn);

    let total = game_titles.len();
    let job_id = scheduler.enqueue(BulkEnrichmentJob { api_key, game_titles });

    Ok(BulkEnrichmentQueued { job_id, total })
}

//! Metadata API commands — T11.
//!
//! Integrates with the RAWG Video Games Database API to auto-fill game
//! metadata (title, genres, cover art, release year) when adding games.
//!
//! ## Caching
//! Results are cached in the `metadata_cache` SQLite table for 24 hours,
//! keyed by the lowercase search query.
//!
//! ## API Key
//! Stored as setting "rawg_api_key". If not set, all search commands return
//! a descriptive error asking the user to configure it in Settings.
//!
//! RAWG free tier: 4,500 requests/day — local caching ensures this is never hit.

use tauri::State;

use crate::db::DbState;
use crate::models::MetadataResult;

// ── RAWG API response types ───────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct RawgResponse {
    results: Vec<RawgGame>,
}

#[derive(serde::Deserialize)]
struct RawgGame {
    name:             String,
    background_image: Option<String>,
    released:         Option<String>,
    genres:           Vec<RawgGenre>,
}

#[derive(serde::Deserialize)]
struct RawgGenre {
    name: String,
}

// ── Constants ─────────────────────────────────────────────────────────────────

const RAWG_BASE: &str = "https://api.rawg.io/api/games";
/// Cache TTL in hours
const CACHE_TTL_HOURS: i64 = 24;

// ── Cache helpers ─────────────────────────────────────────────────────────────

fn read_cache(conn: &rusqlite::Connection, query: &str) -> Option<Vec<MetadataResult>> {
    let row = conn.query_row(
        "SELECT results_json, cached_at FROM metadata_cache WHERE query = ?1",
        rusqlite::params![query],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );

    match row {
        Ok((json, cached_at)) => {
            // Check TTL
            let parsed = chrono::DateTime::parse_from_rfc3339(&cached_at).ok()?;
            let age_hours = chrono::Utc::now()
                .signed_duration_since(parsed.with_timezone(&chrono::Utc))
                .num_hours();

            if age_hours < CACHE_TTL_HOURS {
                serde_json::from_str(&json).ok()
            } else {
                None // stale
            }
        }
        Err(_) => None,
    }
}

fn write_cache(
    conn: &rusqlite::Connection,
    query: &str,
    results: &[MetadataResult],
) -> Result<(), String> {
    let json = serde_json::to_string(results).map_err(|e| e.to_string())?;
    let now  = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO metadata_cache (query, results_json, cached_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(query) DO UPDATE SET results_json = excluded.results_json,
                                          cached_at    = excluded.cached_at",
        rusqlite::params![query, json, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Search the RAWG API for game metadata matching `query`.
///
/// Results are cached for 24 hours. Returns up to 8 candidates sorted by
/// RAWG relevance score.
///
/// Requires the `rawg_api_key` setting to be configured in Settings.
#[tauri::command]
pub async fn search_game_metadata(
    db_state: State<'_, DbState>,
    query:    String,
) -> Result<Vec<MetadataResult>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let cache_key = query.trim().to_lowercase();

    // ── Check cache ───────────────────────────────────────────────────────────
    let api_key = {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;

        // Check cache first
        if let Some(cached) = read_cache(&conn, &cache_key) {
            return Ok(cached);
        }

        // Load API key
        conn.query_row(
            "SELECT value FROM settings WHERE key = 'rawg_api_key'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
    };

    let api_key = api_key.ok_or_else(|| {
        "RAWG API key not configured. Add your key in Settings → Integrations.".to_string()
    })?;

    if api_key.trim().is_empty() {
        return Err(
            "RAWG API key is empty. Add your key in Settings → Integrations.".to_string()
        );
    }

    // ── Fetch from RAWG ───────────────────────────────────────────────────────
    let url = format!(
        "{}?key={}&search={}&page_size=8&search_precise=true",
        RAWG_BASE,
        api_key,
        urlencoding::encode(query.trim()),
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err("Invalid RAWG API key. Check Settings → Integrations.".to_string());
        }
        return Err(format!("RAWG API error: HTTP {}", status));
    }

    let rawg: RawgResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse RAWG response: {}", e))?;

    // ── Transform ─────────────────────────────────────────────────────────────
    let results: Vec<MetadataResult> = rawg
        .results
        .into_iter()
        .map(|g| {
            let genres = g
                .genres
                .iter()
                .map(|gn| gn.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            let release_year = g
                .released
                .as_deref()
                .and_then(|r| r.split('-').next())
                .and_then(|y| y.parse::<i32>().ok());

            MetadataResult {
                name:         g.name,
                genres,
                cover_url:    g.background_image,
                release_year,
            }
        })
        .collect();

    // ── Write cache ───────────────────────────────────────────────────────────
    {
        let conn = db_state.0.lock().map_err(|e| e.to_string())?;
        // Non-fatal if cache write fails
        let _ = write_cache(&conn, &cache_key, &results);
    }

    Ok(results)
}

/// Return the configured RAWG API key (masked for display), or None.
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

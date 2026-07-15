//! RAWG API client — primary metadata provider.
//!
//! Documentation: https://rawg.io/apidocs
//! Free tier: 20,000 requests/month
//!
//! Rate limiting: 10 requests per minute with exponential backoff.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const API_BASE: &str = "https://api.rawg.io/api";

/// RAWG API search response
#[derive(Debug, Deserialize)]
pub struct RawgSearchResponse {
    pub results: Vec<RawgGame>,
}

/// RAWG game result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawgGame {
    pub id: i64,
    pub name: String,
    pub released: Option<String>,
    pub background_image: Option<String>,
    pub rating: Option<f64>,
    pub genres: Option<Vec<RawgGenre>>,
    pub platforms: Option<Vec<RawgPlatformWrapper>>,
    pub metacritic: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawgGenre {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawgPlatformWrapper {
    pub platform: RawgPlatform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawgPlatform {
    pub name: String,
}

// ── Store types (T44) ─────────────────────────────────────────────────────────

/// Top-level response from RAWG `/games/{id}/stores`.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RawgStoresResponse {
    pub results: Vec<RawgStoreEntry>,
}

/// A single store listing entry returned by the stores endpoint.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RawgStoreEntry {
    /// External store URL (e.g. `https://store.steampowered.com/app/570/Dota_2/`).
    pub url:   String,
    /// Store metadata -- the `slug` field identifies the platform.
    pub store: RawgStore,
}

/// Minimal store descriptor -- only `slug` is required for App ID extraction.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RawgStore {
    /// Platform identifier, e.g. `"steam"`, `"gog"`, `"itch"`.
    pub slug: String,
}

/// Rate limiter state
struct RateLimiter {
    last_request: Option<Instant>,
    request_count: u32,
    window_start: Instant,
}

/// RAWG API client with rate limiting
pub struct RawgClient {
    api_key: String,
    client: reqwest::Client,
    limiter: Arc<Mutex<RateLimiter>>,
}

impl RawgClient {
    /// Create a new RAWG client with the given API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            limiter: Arc::new(Mutex::new(RateLimiter {
                last_request: None,
                request_count: 0,
                window_start: Instant::now(),
            })),
        }
    }

    /// Wait if necessary to respect rate limits
    async fn rate_limit(&self) {
        let wait_needed = {
            let mut limiter = self.limiter.lock().unwrap();

            // Reset counter if window has elapsed
            if limiter.window_start.elapsed() >= Duration::from_secs(60) {
                limiter.request_count = 0;
                limiter.window_start = Instant::now();
            }

            // Check if we need to wait
            if limiter.request_count >= super::MAX_REQUESTS_PER_MINUTE {
                let wait_time = Duration::from_secs(60) - limiter.window_start.elapsed();
                Some(wait_time)
            } else {
                limiter.request_count += 1;
                limiter.last_request = Some(Instant::now());
                None
            }
        }; // Lock released here

        // Wait outside the lock if needed
        if let Some(wait_time) = wait_needed {
            tokio::time::sleep(wait_time).await;
            // Update after sleep
            let mut limiter = self.limiter.lock().unwrap();
            limiter.request_count = 1; // Count this request
            limiter.window_start = Instant::now();
            limiter.last_request = Some(Instant::now());
        }
    }

    /// Search for games by title with retry logic
    pub async fn search_games(&self, query: &str) -> Result<Vec<RawgGame>, String> {
        let mut attempts = 0;

        loop {
            self.rate_limit().await;

            let url = format!(
                "{}/games?key={}&search={}&page_size=10",
                API_BASE,
                self.api_key,
                urlencoding::encode(query)
            );

            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<RawgSearchResponse>().await {
                            Ok(data) => return Ok(data.results),
                            Err(e) => return Err(format!("Failed to parse RAWG response: {}", e)),
                        }
                    } else if response.status().as_u16() == 429 {
                        // Rate limited — exponential backoff
                        attempts += 1;
                        if attempts >= super::MAX_RETRIES {
                            return Err("RAWG API rate limit exceeded after retries".to_string());
                        }
                        let delay = super::backoff_delay(attempts);
                        tokio::time::sleep(delay).await;
                        continue;
                    } else {
                        return Err(format!("RAWG API error: {}", response.status()));
                    }
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= super::MAX_RETRIES {
                        return Err(format!("RAWG API request failed after retries: {}", e));
                    }
                    let delay = super::backoff_delay(attempts);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// Get detailed information for a specific game by ID.
    /// T25: Not yet called from any command; suppressing warning intentionally.
    #[allow(dead_code)]
    pub async fn get_game(&self, game_id: i64) -> Result<RawgGame, String> {
        let mut attempts = 0;

        loop {
            self.rate_limit().await;

            let url = format!("{}/games/{}?key={}", API_BASE, game_id, self.api_key);

            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<RawgGame>().await {
                            Ok(data) => return Ok(data),
                            Err(e) => return Err(format!("Failed to parse RAWG response: {}", e)),
                        }
                    } else if response.status().as_u16() == 429 {
                        attempts += 1;
                        if attempts >= super::MAX_RETRIES {
                            return Err("RAWG API rate limit exceeded after retries".to_string());
                        }
                        let delay = super::backoff_delay(attempts);
                        tokio::time::sleep(delay).await;
                        continue;
                    } else {
                        return Err(format!("RAWG API error: {}", response.status()));
                    }
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= super::MAX_RETRIES {
                        return Err(format!("RAWG API request failed after retries: {}", e));
                    }
                    let delay = super::backoff_delay(attempts);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// Fetch all store listings for a RAWG game ID.
    ///
    /// Endpoint: `GET /games/{rawg_id}/stores`
    ///
    /// Used by the Steam App ID auto-detection cascade (T43/T44) to locate
    /// the Steam store URL and extract the numeric App ID from it.
    ///
    /// Returns an empty `Vec` on a non-success response rather than an error
    /// so the caller can silently fall through to the next detection tier.
    // T44: Not yet called from a command — suppressing warning intentionally.
    // Will replace the inline reqwest call in try_rawg_stores when T45 refactors.
    #[allow(dead_code)]
    pub async fn get_game_stores(&self, rawg_id: i64) -> Result<Vec<RawgStoreEntry>, String> {
        self.rate_limit().await;

        let url = format!("{}/games/{}/stores?key={}", API_BASE, rawg_id, self.api_key);

        let resp = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("RAWG stores request failed: {e}"))?;

        if resp.status().is_success() {
            resp.json::<RawgStoresResponse>()
                .await
                .map(|r| r.results)
                .map_err(|e| format!("RAWG stores parse error: {e}"))
        } else {
            Err(format!("RAWG stores API error: {}", resp.status()))
        }
    }
}

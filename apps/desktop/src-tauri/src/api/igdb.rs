//! IGDB API client — fallback metadata provider.
//!
//! Documentation: https://api-docs.igdb.com/
//! Free tier: 4 requests per second
//!
//! Note: IGDB requires Twitch authentication. This is a simplified
//! implementation — full OAuth flow should be added in production.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const API_BASE: &str = "https://api.igdb.com/v4";

/// IGDB game result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgdbGame {
    pub id: i64,
    pub name: String,
    pub summary: Option<String>,
    pub cover: Option<IgdbCover>,
    pub genres: Option<Vec<IgdbGenre>>,
    pub release_dates: Option<Vec<IgdbReleaseDate>>,
    pub rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgdbCover {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgdbGenre {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgdbReleaseDate {
    pub date: Option<i64>,
    pub human: Option<String>,
}

/// Rate limiter state
struct RateLimiter {
    last_request: Option<Instant>,
}

/// IGDB API client with rate limiting
pub struct IgdbClient {
    client_id: String,
    access_token: String,
    client: reqwest::Client,
    limiter: Arc<Mutex<RateLimiter>>,
}

impl IgdbClient {
    /// Create a new IGDB client with Twitch credentials
    pub fn new(client_id: String, access_token: String) -> Self {
        Self {
            client_id,
            access_token,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            limiter: Arc::new(Mutex::new(RateLimiter {
                last_request: None,
            })),
        }
    }

    /// Wait if necessary to respect rate limits (4 requests per second)
    async fn rate_limit(&self) {
        let mut limiter = self.limiter.lock().unwrap();

        if let Some(last) = limiter.last_request {
            let elapsed = last.elapsed();
            let min_interval = Duration::from_millis(250); // 4 req/sec = 250ms between requests
            if elapsed < min_interval {
                let wait_time = min_interval - elapsed;
                drop(limiter); // Release lock before sleeping
                tokio::time::sleep(wait_time).await;
                let mut limiter = self.limiter.lock().unwrap();
                limiter.last_request = Some(Instant::now());
            } else {
                limiter.last_request = Some(Instant::now());
            }
        } else {
            limiter.last_request = Some(Instant::now());
        }
    }

    /// Search for games by title
    pub async fn search_games(&self, query: &str) -> Result<Vec<IgdbGame>, String> {
        let mut attempts = 0;

        loop {
            self.rate_limit().await;

            let url = format!("{}/games", API_BASE);
            let body = format!(
                r#"search "{}"; fields name,summary,cover.url,genres.name,release_dates.date,release_dates.human,rating; limit 10;"#,
                query.replace('"', "")
            );

            match self
                .client
                .post(&url)
                .header("Client-ID", &self.client_id)
                .header("Authorization", format!("Bearer {}", self.access_token))
                .header("Content-Type", "text/plain")
                .body(body)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<Vec<IgdbGame>>().await {
                            Ok(data) => return Ok(data),
                            Err(e) => return Err(format!("Failed to parse IGDB response: {}", e)),
                        }
                    } else if response.status().as_u16() == 429 {
                        attempts += 1;
                        if attempts >= super::MAX_RETRIES {
                            return Err("IGDB API rate limit exceeded after retries".to_string());
                        }
                        let delay = super::backoff_delay(attempts);
                        tokio::time::sleep(delay).await;
                        continue;
                    } else {
                        return Err(format!("IGDB API error: {}", response.status()));
                    }
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= super::MAX_RETRIES {
                        return Err(format!("IGDB API request failed after retries: {}", e));
                    }
                    let delay = super::backoff_delay(attempts);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

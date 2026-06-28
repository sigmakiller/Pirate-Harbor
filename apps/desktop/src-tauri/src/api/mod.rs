//! API client infrastructure for external metadata providers.
//!
//! Provides rate-limited HTTP clients for RAWG and IGDB APIs with
//! automatic retry logic and exponential backoff.

pub mod rawg;
pub mod igdb;

use std::time::Duration;

/// Rate limiter configuration
pub const MAX_REQUESTS_PER_MINUTE: u32 = 10;
pub const INITIAL_BACKOFF_MS: u64 = 1000;
pub const MAX_BACKOFF_MS: u64 = 16000;
pub const MAX_RETRIES: u32 = 3;

/// Calculate exponential backoff delay
pub fn backoff_delay(attempt: u32) -> Duration {
    let delay_ms = INITIAL_BACKOFF_MS * 2_u64.pow(attempt.min(4));
    Duration::from_millis(delay_ms.min(MAX_BACKOFF_MS))
}

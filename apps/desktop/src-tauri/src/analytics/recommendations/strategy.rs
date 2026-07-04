//! Strategy trait definition — T31.
//!
//! Every recommendation strategy implements this trait.  The combiner calls
//! `score()` and `explain()` on each strategy and aggregates the results.

use rusqlite::Connection;

use super::Candidate;

// ── UserContext ───────────────────────────────────────────────────────────────

/// Pre-computed user preferences derived from play history.
///
/// Passed to every strategy so they can make context-aware decisions without
/// hitting the database again.
#[derive(Debug, Clone)]
pub struct UserContext {
    /// Total playtime per genre (seconds).  Keys are genre strings from the DB.
    pub genre_playtime: std::collections::HashMap<String, i64>,
    /// Total lifetime playtime across all games (seconds).
    pub total_playtime_secs: i64,
    /// Fraction of library that is `completed` (0.0–1.0).
    pub completion_rate: f64,
}

// ── Strategy trait ────────────────────────────────────────────────────────────

/// A pluggable recommendation strategy.
///
/// - `score()` returns a normalized 0.0–1.0 relevance score.
/// - `explain()` returns a short, human-readable reason for the UI.
///
/// Strategies must be `Send + Sync` so they can be stored in `StrategyCombiner`
/// and called from async Tauri command handlers via `spawn_blocking`.
pub trait RecommendationStrategy: Send + Sync {
    /// Short stable name used for strategy contribution labels.
    fn name(&self) -> &str;

    /// Score this candidate (0.0 = irrelevant, 1.0 = perfect match).
    fn score(&self, conn: &Connection, candidate: &Candidate, ctx: &UserContext) -> f64;

    /// Human-readable explanation for why this game is recommended.
    fn explain(&self, conn: &Connection, candidate: &Candidate, ctx: &UserContext) -> String;
}

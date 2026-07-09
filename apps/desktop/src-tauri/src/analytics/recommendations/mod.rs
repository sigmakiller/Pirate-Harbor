//! Recommendation engine — T31.
//!
//! Strategy-pattern recommendations: "Which unplayed games in your library
//! should you try next?"
//!
//! # Architecture
//!
//! ```text
//! StrategyCombiner
//!   ├── ContentBasedStrategy  (genre affinity × playtime weight)
//!   ├── GenreStrategy         (pure genre match)
//!   ├── PlaytimeStrategy      (owned but never played)
//!   └── RecencyStrategy       (recently added + unplayed)
//! ```
//!
//! Each strategy scores a candidate game 0.0–1.0.  The combiner applies
//! configurable weights and sums the results.  The top-N candidates are
//! returned with a human-readable explanation string.

pub mod combiner;
pub mod content_based;
pub mod genre_strategy;
pub mod playtime_strategy;
pub mod recency_strategy;
pub mod strategy;

#[cfg(test)]
mod tests;

pub use combiner::StrategyCombiner;
pub use strategy::UserContext;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// ── Public output types ───────────────────────────────────────────────────────

/// A recommended game with its composite score and explanation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationResult {
    pub game_id:     String,
    pub title:       String,
    pub cover_path:  Option<String>,
    pub genre:       Option<String>,
    pub developer:   Option<String>,
    pub status:      String,
    /// 0.0–1.0 composite score across all strategies.
    pub score:       f64,
    /// Human-readable reason shown in the UI.
    pub reason:      String,
    /// Which strategies contributed (for debugging / transparency).
    pub strategy_contributions: Vec<StrategyContribution>,
}

/// Individual strategy contribution to the final score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyContribution {
    pub strategy: String,
    pub score:    f64,
    pub weight:   f64,
}

// ── Candidate struct (internal) ───────────────────────────────────────────────

/// A candidate unplayed game for scoring.
#[derive(Debug, Clone)]
pub(crate) struct Candidate {
    pub id:              String,
    pub title:           String,
    pub cover_path:      Option<String>,
    pub genre:           Option<String>,
    pub developer:       Option<String>,
    #[allow(dead_code)] // Reserved for publisher-based filtering (Phase 5)
    pub publisher:       Option<String>,
    pub status:          String,
    pub total_playtime:  i64,
    pub added_at:        String,
    pub launch_count:    i64,
}

// ── Context builder (public helper used by commands) ─────────────────────────

/// Build a `UserContext` from the current database state.
///
/// This derives the user's preferences from their actual play history:
/// - Favourite genres from most-played games (by playtime)
/// - Completion rate
/// - Total playtime
pub fn build_user_context(conn: &Connection) -> Result<UserContext, String> {
    // Gather genre playtime.
    let mut genre_playtime: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

    // Fetch per-game genre + playtime so we can split comma-separated genres
    // in Rust.  GROUP BY genre would treat "Action, RPG" as a single opaque key.
    let mut stmt = conn
        .prepare(
            "SELECT genre, total_playtime_secs
             FROM games
             WHERE genre IS NOT NULL AND total_playtime_secs > 0",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, i64)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for (genre_str, pt) in rows {
        // Split "Action, RPG" → ["Action", "RPG"], add playtime to each.
        for g in genre_str.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            *genre_playtime.entry(g.to_string()).or_insert(0) += pt;
        }
    }

    let total_playtime: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_playtime_secs), 0) FROM games",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let completed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE status = 'completed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_games: i64 = conn
        .query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))
        .unwrap_or(0);

    let completion_rate = if total_games > 0 {
        completed as f64 / total_games as f64
    } else {
        0.0
    };

    Ok(UserContext {
        genre_playtime,
        total_playtime_secs: total_playtime,
        completion_rate,
    })
}

/// Fetch unplayed candidate games from the database.
///
/// Candidates are games with status `'unplayed'` or zero playtime.
/// Completed/dropped games are excluded.
pub(crate) fn fetch_candidates(conn: &Connection) -> Result<Vec<Candidate>, String> {
    let mut stmt = conn
        .prepare(
            r#"SELECT id, title, COALESCE(cover_path_local, cover_path), genre,
                      developer, publisher, status, total_playtime_secs, added_at,
                      launch_count
               FROM games
               WHERE status = 'unplayed' OR (status = 'playing' AND total_playtime_secs = 0)
               ORDER BY added_at DESC"#,
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Candidate {
                id:             row.get(0)?,
                title:          row.get(1)?,
                cover_path:     row.get(2)?,
                genre:          row.get(3)?,
                developer:      row.get(4)?,
                publisher:      row.get(5)?,
                status:         row.get::<_, String>(6).unwrap_or_default(),
                total_playtime: row.get::<_, i64>(7).unwrap_or(0),
                added_at:       row.get::<_, String>(8).unwrap_or_default(),
                launch_count:   row.get::<_, i64>(9).unwrap_or(0),
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

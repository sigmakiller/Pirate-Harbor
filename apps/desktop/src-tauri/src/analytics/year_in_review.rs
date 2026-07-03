//! Year-in-Review generator — T30.
//!
//! Aggregates all analytics engines into a single annual summary structure.
//! Used by the Identity page and (in Phase 5) a shareable Year-in-Review card.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::{
    completion_stats,
    gaming_stats::{self, GamePlaytime},
    genre_stats::{self, GenreStat},
};

// ── Output type ───────────────────────────────────────────────────────────────

/// Annual gaming summary for a given `year` (4-digit string, e.g. `"2025"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearInReview {
    pub year:                  String,
    pub total_playtime_secs:   i64,
    /// Number of distinct games played this year.
    pub games_played:          i64,
    /// Games completed this year.
    pub games_completed:       i64,
    /// New games added to the library this year.
    pub games_added:           i64,
    /// Total sessions recorded this year.
    pub sessions:              i64,
    /// Top 5 most-played games for the year.
    pub top_games:             Vec<GamePlaytime>,
    /// Genre distribution for the year.
    pub top_genres:            Vec<GenreStat>,
    /// Most active month (1–12) by session count.
    pub most_active_month:     Option<i64>,
    /// Longest single session recorded this year (seconds).
    pub longest_session_secs:  i64,
    /// Completion rate for games touched this year (completed / engaged).
    pub completion_rate:       f64,
}

// ── Generator ─────────────────────────────────────────────────────────────────

/// Build a Year-in-Review for `year` (e.g. `2025`).
///
/// Uses the library-wide genre engine for genre stats (scoped to touched
/// games is a future enhancement).  All time-scoped queries use
/// `started_at` on `sessions` or `added_at`/`last_played` on `games`.
pub fn year_in_review(conn: &Connection, year: i32) -> Result<YearInReview, String> {
    let year_str = year.to_string();
    let year_start = format!("{}-01-01", year);
    let year_end   = format!("{}-12-31 23:59:59", year);

    // ── Sessions this year ─────────────────────────────────────────────────
    let sessions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE started_at BETWEEN ?1 AND ?2",
            rusqlite::params![year_start, year_end],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let total_playtime_secs: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM sessions WHERE started_at BETWEEN ?1 AND ?2",
            rusqlite::params![year_start, year_end],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let longest_session_secs: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(duration_secs), 0) FROM sessions WHERE started_at BETWEEN ?1 AND ?2",
            rusqlite::params![year_start, year_end],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // ── Games touched this year ─────────────────────────────────────────────
    let games_played: i64 = conn
        .query_row(
            r#"SELECT COUNT(DISTINCT game_id) FROM sessions
               WHERE started_at BETWEEN ?1 AND ?2"#,
            rusqlite::params![year_start, year_end],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let games_completed: i64 = conn
        .query_row(
            r#"SELECT COUNT(*) FROM games
               WHERE status='completed'
               AND last_played BETWEEN ?1 AND ?2"#,
            rusqlite::params![year_start, year_end],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let games_added: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE added_at BETWEEN ?1 AND ?2",
            rusqlite::params![year_start, year_end],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // ── Most active month ──────────────────────────────────────────────────
    let most_active_month: Option<i64> = {
        let result = conn.query_row(
            r#"SELECT CAST(strftime('%m', started_at) AS INTEGER) AS month, COUNT(*) AS cnt
               FROM sessions
               WHERE started_at BETWEEN ?1 AND ?2
               GROUP BY month
               ORDER BY cnt DESC
               LIMIT 1"#,
            rusqlite::params![year_start, year_end],
            |r| r.get::<_, i64>(0),
        );
        result.ok()
    };

    // ── Completion rate (year-scoped engaged games) ─────────────────────────
    let stats = completion_stats::completion_stats(conn)?;
    // Use the library-wide rate as a proxy (year-scoped would need a
    // dedicated completion_date column — tracked for T34+).
    let completion_rate = stats.completion_rate;

    // ── Top 5 games (lifetime playtime, year filter is session-based) ───────
    let top_games = gaming_stats::most_played_games(conn, 5)?;

    // ── Top genres (library-wide) ────────────────────────────────────────────
    let top_genres = genre_stats::top_genres(conn, 5)?;

    Ok(YearInReview {
        year:                 year_str,
        total_playtime_secs,
        games_played,
        games_completed,
        games_added,
        sessions,
        top_games,
        top_genres,
        most_active_month,
        longest_session_secs,
        completion_rate,
    })
}

//! Year-in-Review generator â€” T30.
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

// â”€â”€ Output type â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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
    /// Most active month (1â€“12) by session count.
    pub most_active_month:     Option<i64>,
    /// Longest single session recorded this year (seconds).
    pub longest_session_secs:  i64,
    /// Completion rate for games touched this year (completed / engaged).
    pub completion_rate:       f64,
}

// â”€â”€ Generator â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// Build a Year-in-Review for `year` (e.g. `2025`).
///
/// Uses the library-wide genre engine for genre stats (scoped to touched
/// games is a future enhancement).  All time-scoped queries use
/// `started_at` on `sessions` or `added_at`/`last_played` on `games`.
pub fn year_in_review(conn: &Connection, year: i32) -> Result<YearInReview, String> {
    let year_str = year.to_string();
    let year_start = format!("{}-01-01", year);
    let year_end   = format!("{}-12-31 23:59:59", year);

    // â”€â”€ Sessions this year â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
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

    // â”€â”€ Games touched this year â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
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

    // â”€â”€ Most active month â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
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

    // â”€â”€ Completion rate (year-scoped engaged games) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    let stats = completion_stats::completion_stats(conn)?;
    // Use the library-wide rate as a proxy (year-scoped would need a
    // dedicated completion_date column â€” tracked for T34+).
    let completion_rate = stats.completion_rate;

    // â”€â”€ Top 5 games (lifetime playtime, year filter is session-based) â”€â”€â”€â”€â”€â”€â”€
    let top_games = gaming_stats::most_played_games(conn, 5)?;

    // â”€â”€ Top genres (library-wide) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    /// An empty database must return all-zero / None fields rather than an error.
    #[test]
    fn year_in_review_empty_db_returns_zeroes() {
        let conn = setup();
        let result = year_in_review(&conn, 2025).unwrap();
        assert_eq!(result.total_playtime_secs, 0, "no sessions → 0 playtime");
        assert_eq!(result.games_played,         0, "no sessions → 0 games played");
        assert_eq!(result.games_completed,      0, "no completions");
        assert_eq!(result.longest_session_secs, 0, "no sessions → longest = 0");
        assert!(result.most_active_month.is_none(), "no sessions → no active month");
    }

    /// `most_active_month` must be the month with the highest session count.
    /// Jan has 3 sessions, Feb has 1 → expect month 1.
    #[test]
    fn most_active_month_detected_correctly() {
        let conn = setup();
        conn.execute(
            "INSERT INTO games (id, title, status, exe_path, added_at) VALUES ('g1', 'Game A', 'playing', '', '2025-01-01T00:00:00')",
            [],
        ).unwrap();
        for _ in 0..3 {
            conn.execute(
                "INSERT INTO sessions (id, game_id, started_at, duration_secs) VALUES (lower(hex(randomblob(16))), 'g1', '2025-01-15T10:00:00', 3600)",
                [],
            ).unwrap();
        }
        conn.execute(
            "INSERT INTO sessions (id, game_id, started_at, duration_secs) VALUES (lower(hex(randomblob(16))), 'g1', '2025-02-10T10:00:00', 3600)",
            [],
        ).unwrap();
        let result = year_in_review(&conn, 2025).unwrap();
        assert_eq!(result.most_active_month, Some(1), "January (1) should win");
    }

    /// `longest_session_secs` must return the single longest session, not the sum.
    #[test]
    fn longest_session_recorded_correctly() {
        let conn = setup();
        conn.execute(
            "INSERT INTO games (id, title, status, exe_path, added_at) VALUES ('g1', 'Game A', 'playing', '', '2025-01-01T00:00:00')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO sessions (id, game_id, started_at, duration_secs) VALUES (lower(hex(randomblob(16))), 'g1', '2025-06-01T10:00:00', 7200)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO sessions (id, game_id, started_at, duration_secs) VALUES (lower(hex(randomblob(16))), 'g1', '2025-06-02T10:00:00', 3600)",
            [],
        ).unwrap();
        let result = year_in_review(&conn, 2025).unwrap();
        assert_eq!(result.longest_session_secs, 7200, "2h session is the longest");
    }
}
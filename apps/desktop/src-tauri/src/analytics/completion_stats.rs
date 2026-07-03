//! Completion statistics — T30.
//!
//! Completion rate over time, completion trends, and time-to-complete
//! estimates.  These are building blocks for Year-in-Review and the
//! Identity dashboard.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// ── Output types ──────────────────────────────────────────────────────────────

/// Snapshot of library completion state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionStats {
    pub total_games:        i64,
    pub completed:          i64,
    pub playing:            i64,
    pub unplayed:           i64,
    pub dropped:            i64,
    /// Completed / (completed + dropped) — excludes unplayed backlog.
    pub completion_rate:    f64,
    /// Completions grouped by calendar year.
    pub completions_by_year: Vec<YearlyCompletions>,
    /// Average playtime-to-complete across completed games (seconds).
    pub avg_time_to_complete_secs: i64,
}

/// Number of games completed in a given year.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyCompletions {
    pub year:  String,
    pub count: i64,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Calculate completion statistics for the entire library.
pub fn completion_stats(conn: &Connection) -> Result<CompletionStats, String> {
    let total_games: i64 = conn
        .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
        .unwrap_or(0);

    let completed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE status='completed'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let playing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE status='playing'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let unplayed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE status='unplayed'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let dropped: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE status='dropped'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let engaged = completed + dropped;
    let completion_rate = if engaged > 0 {
        (completed as f64 / engaged as f64 * 1000.0).round() / 1000.0
    } else {
        0.0
    };

    // Completions per year — approximated via `last_played` on completed games
    // (we don't store a dedicated completion_date column yet).
    let completions_by_year = completions_per_year(conn)?;

    // Average playtime of completed games.
    let avg_time_to_complete_secs: i64 = conn
        .query_row(
            "SELECT COALESCE(AVG(total_playtime_secs), 0) FROM games WHERE status='completed'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(CompletionStats {
        total_games,
        completed,
        playing,
        unplayed,
        dropped,
        completion_rate,
        completions_by_year,
        avg_time_to_complete_secs,
    })
}

fn completions_per_year(conn: &Connection) -> Result<Vec<YearlyCompletions>, String> {
    // Use the year portion of `last_played` as a proxy for completion year.
    let mut stmt = conn
        .prepare(
            r#"SELECT strftime('%Y', last_played) AS year, COUNT(*) AS count
               FROM games
               WHERE status='completed' AND last_played IS NOT NULL
               GROUP BY year
               ORDER BY year DESC"#,
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |r| {
            Ok(YearlyCompletions {
                year:  r.get::<_, String>(0).unwrap_or_default(),
                count: r.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

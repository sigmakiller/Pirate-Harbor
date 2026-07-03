//! Core gaming statistics — T30.
//!
//! Canonical stat calculations shared by Identity, Recommendations,
//! Year-in-Review, and any future analytics consumers.  Every function
//! takes a `&Connection` (not `&DbState`) so it can be composed inside
//! larger transactions without re-locking.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Output types ──────────────────────────────────────────────────────────────

/// Top game by total playtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlaytime {
    pub game_id:             String,
    pub title:               String,
    pub total_playtime_secs: i64,
    pub cover_path:          Option<String>,
    pub status:              String,
}

/// A single day's aggregated playtime for trend charts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPlaytime {
    pub date:          String, // YYYY-MM-DD
    pub playtime_secs: i64,
}

/// A cell in the activity heatmap (day-of-week × hour-of-day).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapCell {
    /// 0 = Sunday … 6 = Saturday (ISO: 0 = Monday — we use SQLite's `%w`).
    pub day_of_week: i64,
    /// 0–23 hour in local time.
    pub hour:        i64,
    /// Total sessions that started in this slot.
    pub sessions:    i64,
}

/// High-level library summary returned by `library_summary()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibrarySummary {
    pub total_games:          i64,
    pub total_playtime_secs:  i64,
    pub completed_games:      i64,
    pub playing_games:        i64,
    pub unplayed_games:       i64,
    pub dropped_games:        i64,
    pub favorite_games:       i64,
    pub total_sessions:       i64,
    pub total_milestones:     i64,
    pub average_session_secs: f64,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Total lifetime playtime across all games.
pub fn total_playtime(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT COALESCE(SUM(total_playtime_secs), 0) FROM games",
        [],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

/// Average session length across all recorded sessions (seconds).
pub fn average_session_length(conn: &Connection) -> f64 {
    conn.query_row(
        "SELECT COALESCE(AVG(duration_secs), 0.0) FROM sessions WHERE duration_secs > 0",
        [],
        |r| r.get::<_, f64>(0),
    )
    .unwrap_or(0.0)
}

/// Game count grouped by status.
pub fn games_by_status(conn: &Connection) -> HashMap<String, i64> {
    let mut map = HashMap::new();
    let mut stmt = conn
        .prepare("SELECT status, COUNT(*) FROM games GROUP BY status")
        .unwrap();
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)));
    if let Ok(rows) = rows {
        for row in rows.flatten() {
            map.insert(row.0, row.1);
        }
    }
    map
}

/// Top `limit` games by total playtime.
pub fn most_played_games(conn: &Connection, limit: usize) -> Result<Vec<GamePlaytime>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, total_playtime_secs, cover_path_local, status
             FROM games
             WHERE total_playtime_secs > 0
             ORDER BY total_playtime_secs DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![limit as i64], |r| {
            Ok(GamePlaytime {
                game_id:             r.get(0)?,
                title:               r.get(1)?,
                total_playtime_secs: r.get(2)?,
                cover_path:          r.get(3)?,
                status:              r.get::<_, String>(4).unwrap_or_default(),
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Daily playtime trend for the last `days` days (filled with zeros for gaps).
pub fn playtime_trend(conn: &Connection, days: i32) -> Result<Vec<DailyPlaytime>, String> {
    // Sum session durations per calendar day.
    let mut stmt = conn
        .prepare(
            r#"SELECT DATE(started_at) as day, SUM(duration_secs) as total
               FROM sessions
               WHERE started_at >= DATE('now', ?1)
                 AND duration_secs > 0
               GROUP BY day
               ORDER BY day ASC"#,
        )
        .map_err(|e| e.to_string())?;

    let modifier = format!("-{} days", days);
    let db_rows: Vec<(String, i64)> = stmt
        .query_map(rusqlite::params![modifier], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Build a map for fast lookup, then fill every calendar day.
    let lookup: HashMap<String, i64> = db_rows.into_iter().collect();

    let mut result = Vec::with_capacity(days as usize);
    for offset in (0..days).rev() {
        let date = (chrono::Utc::now() - chrono::Duration::days(offset as i64))
            .format("%Y-%m-%d")
            .to_string();
        let playtime_secs = lookup.get(&date).copied().unwrap_or(0);
        result.push(DailyPlaytime { date, playtime_secs });
    }

    Ok(result)
}

/// Activity heatmap: sessions bucketed by (day_of_week, hour_of_day).
///
/// SQLite's `strftime('%w', ...)` returns 0=Sunday … 6=Saturday.
pub fn activity_heatmap(conn: &Connection) -> Result<Vec<HeatmapCell>, String> {
    let mut stmt = conn
        .prepare(
            r#"SELECT
                 CAST(strftime('%w', started_at) AS INTEGER) AS dow,
                 CAST(strftime('%H', started_at) AS INTEGER) AS hour,
                 COUNT(*) AS sessions
               FROM sessions
               WHERE started_at IS NOT NULL
               GROUP BY dow, hour
               ORDER BY dow, hour"#,
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |r| {
            Ok(HeatmapCell {
                day_of_week: r.get(0)?,
                hour:        r.get(1)?,
                sessions:    r.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Compact library summary — one-stop shop for dashboard widgets.
pub fn library_summary(conn: &Connection) -> Result<LibrarySummary, String> {
    let status_map = games_by_status(conn);

    Ok(LibrarySummary {
        total_games: conn
            .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
            .unwrap_or(0),
        total_playtime_secs: total_playtime(conn),
        completed_games:  status_map.get("completed").copied().unwrap_or(0),
        playing_games:    status_map.get("playing").copied().unwrap_or(0),
        unplayed_games:   status_map.get("unplayed").copied().unwrap_or(0),
        dropped_games:    status_map.get("dropped").copied().unwrap_or(0),
        favorite_games:   conn
            .query_row("SELECT COUNT(*) FROM games WHERE is_favorite=1", [], |r| r.get(0))
            .unwrap_or(0),
        total_sessions:   conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
            .unwrap_or(0),
        total_milestones: conn
            .query_row("SELECT COUNT(*) FROM milestones", [], |r| r.get(0))
            .unwrap_or(0),
        average_session_secs: average_session_length(conn),
    })
}

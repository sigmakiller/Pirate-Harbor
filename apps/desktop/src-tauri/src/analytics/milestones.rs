//! Milestone analytics and statistics — T22.
//!
//! Provides comprehensive statistics and trend analysis for milestone data.
//!
//! T25: Timeline and distribution analytics deferred to polish phase.
#![allow(dead_code)]


use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::db::DbState;

// ── Statistics Models ─────────────────────────────────────────────────────────

/// Comprehensive milestone statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneStatistics {
    pub total_count: i64,
    pub by_category: HashMap<String, i64>,
    pub by_difficulty: HashMap<String, i64>,
    pub recent_streak_days: i64,
    pub completion_rate: f64,
    pub average_per_week: f64,
    pub top_games: Vec<GameMilestoneCount>,
    pub timeline: Vec<MilestoneTimelineEntry>,
}

/// Game milestone count for leaderboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMilestoneCount {
    pub game_id: String,
    pub game_title: String,
    pub milestone_count: i64,
}

/// Timeline entry showing milestone activity over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneTimelineEntry {
    pub date: String,
    pub count: i64,
    pub category_breakdown: HashMap<String, i64>,
}

// ── Statistics Calculation ────────────────────────────────────────────────────

/// Calculate comprehensive milestone statistics
pub fn calculate_statistics(
    db_state: &DbState,
    game_id: Option<&str>,
) -> Result<MilestoneStatistics, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    // Build WHERE clause for optional game filter
    let where_clause = if game_id.is_some() {
        "WHERE game_id = ?1"
    } else {
        ""
    };

    // Total count
    let total_count: i64 = if let Some(gid) = game_id {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM milestones {}", where_clause),
            rusqlite::params![gid],
            |row| row.get(0),
        )
    } else {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM milestones {}", where_clause),
            [],
            |row| row.get(0),
        )
    }
    .unwrap_or(0);

    // By category
    let mut by_category: HashMap<String, i64> = HashMap::new();
    {
        let query = format!(
            "SELECT category, COUNT(*) as count FROM milestones {} GROUP BY category",
            where_clause
        );
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

        let mut rows = if let Some(gid) = game_id {
            stmt.query(rusqlite::params![gid])
        } else {
            stmt.query([])
        }
        .map_err(|e| e.to_string())?;

        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let category: String = row.get(0).unwrap_or_default();
            let count: i64 = row.get(1).unwrap_or(0);
            by_category.insert(category, count);
        }
    }

    // By difficulty
    let mut by_difficulty: HashMap<String, i64> = HashMap::new();
    {
        let query = format!(
            "SELECT difficulty, COUNT(*) as count FROM milestones {} AND difficulty IS NOT NULL GROUP BY difficulty",
            where_clause
        );
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

        let mut rows = if let Some(gid) = game_id {
            stmt.query(rusqlite::params![gid])
        } else {
            stmt.query([])
        }
        .map_err(|e| e.to_string())?;

        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let difficulty: String = row.get(0).unwrap_or_default();
            let count: i64 = row.get(1).unwrap_or(0);
            by_difficulty.insert(difficulty, count);
        }
    }

    // Recent streak days
    let recent_streak_days = calculate_streak(&conn, game_id)?;

    // Completion rate (milestones per game)
    let completion_rate = if game_id.is_some() {
        total_count as f64
    } else {
        let game_count: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT game_id) FROM milestones",
                [],
                |row| row.get(0),
            )
            .unwrap_or(1);
        if game_count > 0 {
            total_count as f64 / game_count as f64
        } else {
            0.0
        }
    };

    // Average per week
    let average_per_week = calculate_weekly_average(&conn, game_id)?;

    // Top games
    let top_games = calculate_top_games(&conn, game_id)?;

    // Timeline
    let timeline = calculate_timeline(&conn, game_id)?;

    Ok(MilestoneStatistics {
        total_count,
        by_category,
        by_difficulty,
        recent_streak_days,
        completion_rate,
        average_per_week,
        top_games,
        timeline,
    })
}

/// Calculate current milestone streak in days
fn calculate_streak(
    conn: &rusqlite::Connection,
    game_id: Option<&str>,
) -> Result<i64, String> {
    let query = if game_id.is_some() {
        "SELECT achievement_date FROM milestones WHERE game_id = ?1 ORDER BY achievement_date DESC"
    } else {
        "SELECT achievement_date FROM milestones ORDER BY achievement_date DESC"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let mut rows = if let Some(gid) = game_id {
        stmt.query(rusqlite::params![gid])
    } else {
        stmt.query([])
    }
    .map_err(|e| e.to_string())?;

    let mut dates = Vec::new();
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        if let Ok(date) = row.get::<_, String>(0) {
            dates.push(date);
        }
    }

    if dates.is_empty() {
        return Ok(0);
    }

    let now = Utc::now();
    let mut streak = 0i64;
    let mut current_date = now.date_naive();

    for date_str in &dates {
        if let Ok(date_time) = DateTime::parse_from_rfc3339(date_str) {
            let date = date_time.date_naive();
            let diff = current_date.signed_duration_since(date).num_days();

            if diff <= 1 {
                streak += 1;
                current_date = date - Duration::days(1);
            } else {
                break;
            }
        }
    }

    Ok(streak)
}

/// Calculate average milestones per week
fn calculate_weekly_average(
    conn: &rusqlite::Connection,
    game_id: Option<&str>,
) -> Result<f64, String> {
    let query = if game_id.is_some() {
        "SELECT MIN(achievement_date), MAX(achievement_date) FROM milestones WHERE game_id = ?1"
    } else {
        "SELECT MIN(achievement_date), MAX(achievement_date) FROM milestones"
    };

    let (min_date, max_date): (Option<String>, Option<String>) = if let Some(gid) = game_id {
        conn.query_row(query, rusqlite::params![gid], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
    } else {
        conn.query_row(query, [], |row| Ok((row.get(0)?, row.get(1)?)))
    }
    .unwrap_or((None, None));

    if let (Some(min), Some(max)) = (min_date, max_date) {
        if let (Ok(min_dt), Ok(max_dt)) = (
            DateTime::parse_from_rfc3339(&min),
            DateTime::parse_from_rfc3339(&max),
        ) {
            let days = max_dt.signed_duration_since(min_dt).num_days();
            let weeks = (days as f64 / 7.0).max(1.0);

            let count_query = if game_id.is_some() {
                "SELECT COUNT(*) FROM milestones WHERE game_id = ?1"
            } else {
                "SELECT COUNT(*) FROM milestones"
            };

            let count: i64 = if let Some(gid) = game_id {
                conn.query_row(count_query, rusqlite::params![gid], |row| row.get(0))
            } else {
                conn.query_row(count_query, [], |row| row.get(0))
            }
            .unwrap_or(0);

            return Ok(count as f64 / weeks);
        }
    }

    Ok(0.0)
}

/// Calculate top games by milestone count
fn calculate_top_games(
    conn: &rusqlite::Connection,
    game_id: Option<&str>,
) -> Result<Vec<GameMilestoneCount>, String> {
    if game_id.is_some() {
        // If filtering by game, return just that game
        return Ok(vec![]);
    }

    let query = "
        SELECT m.game_id, g.title, COUNT(*) as count
        FROM milestones m
        JOIN games g ON m.game_id = g.id
        GROUP BY m.game_id, g.title
        ORDER BY count DESC
        LIMIT 10
    ";

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let games: Vec<GameMilestoneCount> = stmt
        .query_map([], |row| {
            Ok(GameMilestoneCount {
                game_id: row.get(0)?,
                game_title: row.get(1)?,
                milestone_count: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(games)
}

/// Calculate milestone timeline (daily/weekly aggregation)
fn calculate_timeline(
    conn: &rusqlite::Connection,
    game_id: Option<&str>,
) -> Result<Vec<MilestoneTimelineEntry>, String> {
    let query = if game_id.is_some() {
        "SELECT DATE(achievement_date) as date, category, COUNT(*) as count
         FROM milestones
         WHERE game_id = ?1
         GROUP BY DATE(achievement_date), category
         ORDER BY date DESC
         LIMIT 90"
    } else {
        "SELECT DATE(achievement_date) as date, category, COUNT(*) as count
         FROM milestones
         GROUP BY DATE(achievement_date), category
         ORDER BY date DESC
         LIMIT 90"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let mut rows = if let Some(gid) = game_id {
        stmt.query(rusqlite::params![gid])
    } else {
        stmt.query([])
    }
    .map_err(|e| e.to_string())?;

    // Group by date
    let mut date_map: HashMap<String, HashMap<String, i64>> = HashMap::new();

    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let date: String = row.get(0).unwrap_or_default();
        let category: String = row.get(1).unwrap_or_default();
        let count: i64 = row.get(2).unwrap_or(0);
        
        date_map
            .entry(date)
            .or_insert_with(HashMap::new)
            .insert(category, count);
    }

    // Convert to timeline entries
    let mut timeline: Vec<MilestoneTimelineEntry> = date_map
        .into_iter()
        .map(|(date, category_breakdown)| {
            let count = category_breakdown.values().sum();
            MilestoneTimelineEntry {
                date,
                count,
                category_breakdown,
            }
        })
        .collect();

    timeline.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(timeline)
}

// ─── T54: Milestone Streak Engine ─────────────────────────────────────────────

/// Aggregated streak and activity stats for milestone achievements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneStreakStats {
    /// Consecutive days (ending today or yesterday) with ≥1 milestone.
    pub current_streak_days: i32,
    /// All-time longest streak in days.
    pub longest_streak_days: i32,
    /// Total milestones ever recorded.
    pub total_milestones: i32,
    /// Milestones earned in the current calendar month.
    pub this_month: i32,
    /// Milestones earned in the current calendar week (Mon–Sun).
    pub this_week: i32,
}

/// Build milestone streak statistics from the `milestones` table.
///
/// The current streak counts backward from today: if a milestone exists today
/// the streak starts today; otherwise it can still be alive if yesterday had
/// one. The streak breaks on the first calendar day with no milestone.
pub fn build_milestone_streak_stats(
    conn: &rusqlite::Connection,
) -> Result<MilestoneStreakStats, String> {
    use chrono::{Datelike, Local, Duration};

    // ── Total ─────────────────────────────────────────────────────────────────
    let total_milestones: i32 = conn
        .query_row("SELECT COUNT(*) FROM milestones", [], |r| r.get(0))
        .unwrap_or(0);

    let today = Local::now().date_naive();

    // ── This week (Mon–Sun) ────────────────────────────────────────────────────
    // weekday() is Mon=0..Sun=6 in chrono
    let days_since_mon = today.weekday().num_days_from_monday() as i64;
    let week_start = today - Duration::days(days_since_mon);
    let week_start_str = week_start.format("%Y-%m-%d").to_string();
    let week_end_str   = today.format("%Y-%m-%d").to_string();

    let this_week: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM milestones
             WHERE DATE(achievement_date) BETWEEN ?1 AND ?2",
            rusqlite::params![week_start_str, week_end_str],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // ── This month ─────────────────────────────────────────────────────────────
    let month_start_str = format!("{}-{:02}-01", today.year(), today.month());

    let this_month: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM milestones
             WHERE DATE(achievement_date) BETWEEN ?1 AND ?2",
            rusqlite::params![month_start_str, week_end_str],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // ── Streak calculation ─────────────────────────────────────────────────────
    // Fetch all distinct dates that have ≥1 milestone, newest first.
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT DATE(achievement_date) AS d
             FROM milestones
             ORDER BY d DESC",
        )
        .map_err(|e| e.to_string())?;

    let dates: Vec<chrono::NaiveDate> = stmt
        .query_map([], |r| {
            let s: String = r.get(0)?;
            Ok(s)
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter_map(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .collect();

    // Current streak: walk backward from today
    let mut current_streak_days: i32 = 0;
    if !dates.is_empty() {
        // Allow the streak to start from today OR yesterday
        let anchor = if dates[0] == today {
            today
        } else if dates[0] == today - Duration::days(1) {
            today - Duration::days(1)
        } else {
            // Most recent milestone was before yesterday → streak is 0
            chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
        };

        let date_set: std::collections::HashSet<chrono::NaiveDate> =
            dates.iter().cloned().collect();

        let mut cursor = anchor;
        while date_set.contains(&cursor) {
            current_streak_days += 1;
            match cursor.checked_sub_signed(Duration::days(1)) {
                Some(prev) => cursor = prev,
                None => break,
            }
        }
    }

    // Longest streak: sliding window over sorted dates
    let mut longest_streak_days: i32 = 0;
    if !dates.is_empty() {
        // dates is DESC; we need ASC for the window walk
        let mut asc: Vec<chrono::NaiveDate> = dates.clone();
        asc.sort();
        asc.dedup();

        let mut run = 1i32;
        for i in 1..asc.len() {
            if asc[i] == asc[i - 1] + Duration::days(1) {
                run += 1;
            } else {
                longest_streak_days = longest_streak_days.max(run);
                run = 1;
            }
        }
        longest_streak_days = longest_streak_days.max(run);
    }

    Ok(MilestoneStreakStats {
        current_streak_days,
        longest_streak_days,
        total_milestones,
        this_month,
        this_week,
    })
}

// ─── T54 unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod streak_tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;
    use chrono::{Local, Duration};

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        // Seed a game
        conn.execute(
            "INSERT INTO games (id, title, status, exe_path, added_at)
             VALUES ('g1', 'Test Game', 'playing', '', '2025-01-01T00:00:00')",
            [],
        ).unwrap();
        conn
    }

    fn insert_milestone(conn: &Connection, date: &str) {
        conn.execute(
            "INSERT INTO milestones (id, game_id, title, category, achievement_date, points, created_at, updated_at)
             VALUES (lower(hex(randomblob(16))), 'g1', 'Test', 'milestone', ?1, 10, ?1, ?1)",
            rusqlite::params![format!("{}T12:00:00", date)],
        ).unwrap();
    }

    #[test]
    fn empty_db_returns_zeroes() {
        let conn = setup();
        let stats = build_milestone_streak_stats(&conn).unwrap();
        assert_eq!(stats.total_milestones,     0);
        assert_eq!(stats.current_streak_days,  0);
        assert_eq!(stats.longest_streak_days,  0);
        assert_eq!(stats.this_week,            0);
        assert_eq!(stats.this_month,           0);
    }

    #[test]
    fn streak_resets_when_no_milestone_yesterday() {
        let conn = setup();
        // Insert a milestone 2 days ago (not yesterday, not today)
        let two_ago = (Local::now().date_naive() - Duration::days(2))
            .format("%Y-%m-%d").to_string();
        insert_milestone(&conn, &two_ago);
        let stats = build_milestone_streak_stats(&conn).unwrap();
        assert_eq!(stats.current_streak_days, 0, "streak must be 0 — gap before yesterday");
    }

    #[test]
    fn consecutive_days_produce_correct_streak() {
        let conn = setup();
        let today = Local::now().date_naive();
        for i in 0..5i64 {
            let d = (today - Duration::days(i)).format("%Y-%m-%d").to_string();
            insert_milestone(&conn, &d);
        }
        let stats = build_milestone_streak_stats(&conn).unwrap();
        assert_eq!(stats.current_streak_days, 5);
        assert_eq!(stats.longest_streak_days, 5);
    }

    #[test]
    fn longest_streak_detected_in_historical_data() {
        let conn = setup();
        let today = Local::now().date_naive();
        // Historical run of 7 (a month ago)
        for i in 0..7i64 {
            let d = (today - Duration::days(30 + i)).format("%Y-%m-%d").to_string();
            insert_milestone(&conn, &d);
        }
        // Current run of 2
        insert_milestone(&conn, &today.format("%Y-%m-%d").to_string());
        insert_milestone(&conn, &(today - Duration::days(1)).format("%Y-%m-%d").to_string());

        let stats = build_milestone_streak_stats(&conn).unwrap();
        assert_eq!(stats.longest_streak_days, 7, "7-day historical streak should win");
        assert_eq!(stats.current_streak_days, 2);
    }
}

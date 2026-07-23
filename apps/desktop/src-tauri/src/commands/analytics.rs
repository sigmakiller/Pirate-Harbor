//! Analytics commands — T30.
//!
//! Exposes the T30 shared analytics engines to the frontend via Tauri commands.
//!
//! | Command                   | Source module              | UI surface                |
//! |---------------------------|----------------------------|---------------------------|
//! | `get_library_summary`     | gaming_stats               | Dashboard widgets         |
//! | `get_most_played_games`   | gaming_stats               | Identity, Launcher        |
//! | `get_playtime_trend`      | gaming_stats               | Identity trend chart      |
//! | `get_activity_heatmap`    | heatmap                    | Identity heatmap          |
//! | `get_genre_distribution`  | genre_stats                | Identity genre chart      |
//! | `get_completion_stats`    | completion_stats           | Identity completion panel |
//! | `get_year_in_review`      | year_in_review             | Identity / Year-in-Review |
//! | `get_related_games`       | metadata::game_lookup      | Game Detail related panel |

use tauri::State;
use chrono::Datelike;

use crate::analytics::{
    completion_stats,
    gaming_stats,
    genre_stats,
    heatmap,
    year_in_review,
};
use crate::metadata::game_lookup;
use crate::db::DbState;

// ── Library overview ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_library_summary(
    db: State<'_, DbState>,
) -> Result<gaming_stats::LibrarySummary, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    gaming_stats::library_summary(&conn)
}

// ── Playtime ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_most_played_games(
    db:    State<'_, DbState>,
    limit: Option<usize>,
) -> Result<Vec<gaming_stats::GamePlaytime>, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    gaming_stats::most_played_games(&conn, limit.unwrap_or(10).min(50))
}

/// Daily playtime trend for the last `days` days (default 30).
#[tauri::command]
pub fn get_playtime_trend(
    db:   State<'_, DbState>,
    days: Option<i32>,
) -> Result<Vec<gaming_stats::DailyPlaytime>, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    gaming_stats::playtime_trend(&conn, days.unwrap_or(30).min(365))
}

// ── Heatmap ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_activity_heatmap(
    db: State<'_, DbState>,
) -> Result<heatmap::ActivityHeatmap, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    heatmap::build_heatmap(&conn)
}

// ── Genre ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_genre_distribution(
    db: State<'_, DbState>,
) -> Result<genre_stats::GenreDistribution, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    genre_stats::genre_distribution(&conn)
}

// ── Completion ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_completion_stats(
    db: State<'_, DbState>,
) -> Result<completion_stats::CompletionStats, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    completion_stats::completion_stats(&conn)
}

// ── Year-in-Review ────────────────────────────────────────────────────────────

/// Generate a Year-in-Review for `year` (e.g. 2025).
/// Defaults to the current year.
#[tauri::command]
pub fn get_year_in_review(
    db:   State<'_, DbState>,
    year: Option<i32>,
) -> Result<year_in_review::YearInReview, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    let y = year.unwrap_or_else(|| chrono::Utc::now().year());
    year_in_review::year_in_review(&conn, y)
}

// ── Related games ─────────────────────────────────────────────────────────────

/// Find games related to `game_id` by genre/developer/publisher.
/// Used by Game Detail page "Related Titles" section.
#[tauri::command]
pub fn get_related_games(
    db:      State<'_, DbState>,
    game_id: String,
    limit:   Option<usize>,
) -> Result<Vec<game_lookup::RelatedGame>, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    game_lookup::find_related_games(&conn, &game_id, limit.unwrap_or(8).min(20))
}

// ── T52: Year-in-Review support ───────────────────────────────────────────────

/// Returns the distinct years (desc) that have at least one session recorded.
/// Used by YearInReviewPage to build the year selector and pick the default.
#[tauri::command]
pub fn get_session_years(db: State<'_, DbState>) -> Result<Vec<i32>, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT CAST(strftime('%Y', started_at) AS INTEGER) AS yr
             FROM sessions
             ORDER BY yr DESC",
        )
        .map_err(|e| e.to_string())?;
    let years: Vec<i32> = stmt
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(years)
}

/// Monthly playtime breakdown for a given year.
/// Returns 12 entries (month 1-12) with total seconds played that month.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonthlyPlaytime {
    pub month: i32,   // 1-12
    pub secs:  i64,
}

#[tauri::command]
pub fn get_monthly_playtime(
    db:   State<'_, DbState>,
    year: i32,
) -> Result<Vec<MonthlyPlaytime>, String> {
    let conn      = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    let year_start = format!("{}-01-01", year);
    let year_end   = format!("{}-12-31 23:59:59", year);

    let mut stmt = conn
        .prepare(
            "SELECT CAST(strftime('%m', started_at) AS INTEGER) AS month,
                    COALESCE(SUM(duration_secs), 0) AS secs
             FROM sessions
             WHERE started_at BETWEEN ?1 AND ?2
             GROUP BY month
             ORDER BY month ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<MonthlyPlaytime> = stmt
        .query_map(rusqlite::params![year_start, year_end], |r| {
            Ok(MonthlyPlaytime { month: r.get(0)?, secs: r.get(1)? })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Fill in months with zero playtime so the frontend always gets 12 entries.
    let mut full: Vec<MonthlyPlaytime> = (1..=12)
        .map(|m| {
            rows.iter()
                .find(|r| r.month == m)
                .cloned()
                .unwrap_or(MonthlyPlaytime { month: m, secs: 0 })
        })
        .collect();
    full.sort_by_key(|r| r.month);
    Ok(full)
}

// ─── T53: Date-based heatmap (GitHub-style 365-day calendar) ─────────────────

/// A single day's session summary for the calendar heatmap.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DateHeatmapCell {
    pub date:   String,  // ISO-8601 date "YYYY-MM-DD"
    pub secs:   i64,     // total playtime seconds that day
    pub count:  i64,     // number of sessions that day
}

/// Return one `DateHeatmapCell` per day for the last 365 days.
/// Days with no sessions are omitted (frontend fills gaps with zero).
#[tauri::command]
pub fn get_date_heatmap(db: State<'_, DbState>) -> Result<Vec<DateHeatmapCell>, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT strftime('%Y-%m-%d', started_at) AS day,
                    COALESCE(SUM(duration_secs), 0)  AS secs,
                    COUNT(*)                          AS cnt
             FROM sessions
             WHERE started_at >= date('now', '-365 days')
             GROUP BY day
             ORDER BY day ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<DateHeatmapCell> = stmt
        .query_map([], |r| {
            Ok(DateHeatmapCell {
                date:  r.get(0)?,
                secs:  r.get(1)?,
                count: r.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

// ─── T54: Milestone streak stats ──────────────────────────────────────────────

/// T54: Return milestone streak statistics for the Identity page "Milestone
/// Activity" card.  Calls the streak engine in `analytics::milestones`.
#[tauri::command]
pub fn get_milestone_streak_stats(
    db: State<'_, DbState>,
) -> Result<crate::analytics::milestones::MilestoneStreakStats, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    crate::analytics::milestones::build_milestone_streak_stats(&conn)
}

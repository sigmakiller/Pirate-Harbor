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

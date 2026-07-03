//! Activity heatmap — T30.
//!
//! Produces a 7 × 24 grid of session counts (day_of_week × hour_of_day).
//! The data is ready to render as a GitHub-style contribution calendar or
//! a time-of-day heat grid in the Identity and Year-in-Review pages.
//!
//! This is a thin public wrapper around `gaming_stats::activity_heatmap` so
//! callers can import it from either location.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub use super::gaming_stats::HeatmapCell;

/// A fully-populated 7 × 24 heatmap (all cells, including zeros).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityHeatmap {
    /// All 168 cells (7 days × 24 hours) — guaranteed to include zeros.
    pub cells:            Vec<HeatmapCell>,
    /// Day of week with the most sessions (0=Sun … 6=Sat).
    pub most_active_day:  Option<i64>,
    /// Hour of day with the most sessions.
    pub most_active_hour: Option<i64>,
    /// Total sessions used to build the heatmap.
    pub total_sessions:   i64,
}

/// Build a fully-filled 7 × 24 activity heatmap from session data.
///
/// Missing (day, hour) pairs are filled with 0 so the frontend never has
/// to handle sparse data.
pub fn build_heatmap(conn: &Connection) -> Result<ActivityHeatmap, String> {
    let sparse = super::gaming_stats::activity_heatmap(conn)?;

    // Index sparse data for fast lookup.
    let mut lookup: std::collections::HashMap<(i64, i64), i64> =
        std::collections::HashMap::new();
    for cell in &sparse {
        lookup.insert((cell.day_of_week, cell.hour), cell.sessions);
    }

    // Build dense 7×24 grid.
    let mut cells = Vec::with_capacity(7 * 24);
    for day in 0i64..7 {
        for hour in 0i64..24 {
            cells.push(HeatmapCell {
                day_of_week: day,
                hour,
                sessions: lookup.get(&(day, hour)).copied().unwrap_or(0),
            });
        }
    }

    let total_sessions: i64 = cells.iter().map(|c| c.sessions).sum();

    // Most active day.
    let mut day_totals = [0i64; 7];
    for c in &cells {
        day_totals[c.day_of_week as usize] += c.sessions;
    }
    let most_active_day = day_totals
        .iter()
        .enumerate()
        .max_by_key(|(_, &v)| v)
        .filter(|(_, &v)| v > 0)
        .map(|(i, _)| i as i64);

    // Most active hour.
    let mut hour_totals = [0i64; 24];
    for c in &cells {
        hour_totals[c.hour as usize] += c.sessions;
    }
    let most_active_hour = hour_totals
        .iter()
        .enumerate()
        .max_by_key(|(_, &v)| v)
        .filter(|(_, &v)| v > 0)
        .map(|(i, _)| i as i64);

    Ok(ActivityHeatmap {
        cells,
        most_active_day,
        most_active_hour,
        total_sessions,
    })
}

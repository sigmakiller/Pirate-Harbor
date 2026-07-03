//! Genre analysis engine — T30.
//!
//! Calculates per-genre playtime, game counts, and weighted preference scores.
//! Used by Identity, Recommendations, and Year-in-Review.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// ── Output types ──────────────────────────────────────────────────────────────

/// Genre statistics entry with composite preference score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreStat {
    pub genre:               String,
    pub game_count:          i64,
    pub completed_count:     i64,
    pub total_playtime_secs: i64,
    pub milestone_count:     i64,
    /// Weighted preference score 0.0–1.0 (playtime-weighted fraction of
    /// total playtime, capped at 1.0).
    pub preference_score:    f64,
}

/// Genre breakdown for quick display widgets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreDistribution {
    pub genres:              Vec<GenreStat>,
    pub total_playtime_secs: i64,
    pub dominant_genre:      Option<String>,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Full genre breakdown ordered by preference score.
///
/// Games with `NULL` genre are aggregated as `"Unknown"`.
pub fn genre_distribution(conn: &Connection) -> Result<GenreDistribution, String> {
    let total_pt: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_playtime_secs), 0) FROM games",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Per-genre playtime + counts.
    let mut stmt = conn
        .prepare(
            r#"SELECT
                 COALESCE(genre, 'Unknown') AS genre,
                 COUNT(*) AS game_count,
                 SUM(CASE WHEN status='completed' THEN 1 ELSE 0 END) AS completed,
                 COALESCE(SUM(total_playtime_secs), 0) AS playtime
               FROM games
               GROUP BY genre
               ORDER BY playtime DESC"#,
        )
        .map_err(|e| e.to_string())?;

    // Per-genre milestone count (separate query to avoid cross-join blowup).
    let mut ms_stmt = conn
        .prepare(
            r#"SELECT COALESCE(g.genre, 'Unknown'), COUNT(m.id)
               FROM milestones m
               JOIN games g ON m.game_id = g.id
               GROUP BY g.genre"#,
        )
        .map_err(|e| e.to_string())?;

    let mut milestone_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let ms_rows = ms_stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .map_err(|e| e.to_string())?;
    for row in ms_rows.flatten() {
        milestone_map.insert(row.0, row.1);
    }

    let genres: Vec<GenreStat> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .map(|(genre, game_count, completed_count, playtime)| {
            let ms = milestone_map.get(&genre).copied().unwrap_or(0);
            let pref = if total_pt > 0 {
                (playtime as f64 / total_pt as f64).min(1.0)
            } else {
                0.0
            };
            GenreStat {
                genre,
                game_count,
                completed_count,
                total_playtime_secs: playtime,
                milestone_count: ms,
                preference_score: (pref * 1000.0).round() / 1000.0,
            }
        })
        .collect();

    let dominant_genre = genres.first().map(|g| g.genre.clone());

    Ok(GenreDistribution {
        genres,
        total_playtime_secs: total_pt,
        dominant_genre,
    })
}

/// Top `limit` genres by preference score (convenience wrapper).
pub fn top_genres(conn: &Connection, limit: usize) -> Result<Vec<GenreStat>, String> {
    genre_distribution(conn).map(|d| d.genres.into_iter().take(limit).collect())
}

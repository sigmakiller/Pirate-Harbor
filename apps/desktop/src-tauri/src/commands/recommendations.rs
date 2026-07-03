//! Recommendation commands — T31.
//!
//! Exposes the recommendation engine to the frontend via two Tauri commands:
//!
//! - `get_recommendations` — General "Suggested for You" list (LauncherPage,
//!   LibraryPage, IdentityPage).
//! - `get_game_recommendations` — "If you enjoyed this…" filtered by a
//!   specific game's genres/developer (GameDetailPage).

use tauri::State;

use crate::analytics::recommendations::{
    self, RecommendationResult, StrategyCombiner,
};
use crate::db::DbState;

// ── Commands ──────────────────────────────────────────────────────────────────

/// Return top `limit` recommended unplayed games from the library.
///
/// Uses the default weighted combiner (ContentBased + Genre + Playtime +
/// Recency).  Results include a composite score and a human-readable reason.
///
/// Used by: LauncherPage, LibraryPage sidebar, IdentityPage.
#[tauri::command]
pub fn get_recommendations(
    db: State<'_, DbState>,
    limit: Option<usize>,
) -> Result<Vec<RecommendationResult>, String> {
    let conn    = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    let cap     = limit.unwrap_or(10).min(50);
    let ctx     = recommendations::build_user_context(&conn)?;
    let candidates = recommendations::fetch_candidates(&conn)?;
    let combiner   = StrategyCombiner::default_combiner();
    Ok(combiner.rank(&conn, &candidates, &ctx, cap))
}

/// Return top `limit` recommended games related to a specific game.
///
/// Filters candidates by genre and/or developer overlap with the given
/// `game_id`, then applies the full scoring pipeline.
///
/// Used by: GameDetailPage "If you enjoyed this…" section.
#[tauri::command]
pub fn get_game_recommendations(
    db: State<'_, DbState>,
    game_id: String,
    limit: Option<usize>,
) -> Result<Vec<RecommendationResult>, String> {
    let conn = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
    let cap  = limit.unwrap_or(6).min(20);

    // Fetch the source game's genre and developer.
    let (genre, developer): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT genre, developer FROM games WHERE id = ?1",
            rusqlite::params![game_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Game not found: {e}"))?;

    let ctx        = recommendations::build_user_context(&conn)?;
    let candidates = recommendations::fetch_candidates(&conn)?;

    // Filter to genre/developer overlap, excluding the source game itself.
    let related: Vec<_> = candidates
        .into_iter()
        .filter(|c| c.id != game_id)
        .filter(|c| {
            let genre_match = genre.as_ref()
                .zip(c.genre.as_ref())
                .map(|(a, b)| a == b)
                .unwrap_or(false);
            let dev_match = developer.as_ref()
                .zip(c.developer.as_ref())
                .map(|(a, b)| a == b)
                .unwrap_or(false);
            genre_match || dev_match
        })
        .collect();

    let combiner = StrategyCombiner::default_combiner();
    Ok(combiner.rank(&conn, &related, &ctx, cap))
}

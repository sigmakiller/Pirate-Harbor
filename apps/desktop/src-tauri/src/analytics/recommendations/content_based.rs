//! Content-based strategy — T31.
//!
//! Scores a candidate by how much the user has historically played games in
//! the same genre(s), weighted by total playtime.
//!
//! Formula:
//! ```text
//! genre_pt = SUM of genre_playtime[g] for each genre g in candidate.genre.split(',')
//! score    = genre_pt / total_playtime_secs  (clamped 0..1)
//! ```
//!
//! Returns 0.0 when the candidate has no genre or the user has no playtime.
//! Handles comma-separated multi-genre strings (C2 / M1 review fix).

use rusqlite::Connection;

use super::{Candidate, strategy::{RecommendationStrategy, UserContext}};

pub struct ContentBasedStrategy;

impl RecommendationStrategy for ContentBasedStrategy {
    fn name(&self) -> &str { "content_based" }

    fn score(&self, _conn: &Connection, candidate: &Candidate, ctx: &UserContext) -> f64 {
        let genre_str = match &candidate.genre {
            Some(g) => g,
            None => return 0.0,
        };

        if ctx.total_playtime_secs == 0 {
            return 0.0;
        }

        // Sum playtime across all genres in a multi-genre string.
        let genre_pt: i64 = genre_str
            .split(',')
            .map(str::trim)
            .filter_map(|g| ctx.genre_playtime.get(g).copied())
            .sum();

        (genre_pt as f64 / ctx.total_playtime_secs as f64).min(1.0)
    }

    fn explain(&self, _conn: &Connection, candidate: &Candidate, ctx: &UserContext) -> String {
        let genre_str = match &candidate.genre {
            Some(g) => g.clone(),
            None => return "Matches your library style".to_string(),
        };

        // Find the genre within the candidate that the user has spent most time on.
        let best = genre_str
            .split(',')
            .map(str::trim)
            .max_by_key(|g| ctx.genre_playtime.get(*g).copied().unwrap_or(0));

        let best_genre = best.unwrap_or(genre_str.as_str());
        let genre_pt = ctx.genre_playtime.get(best_genre).copied().unwrap_or(0);

        if genre_pt == 0 {
            return format!("A {} game you haven't tried yet", best_genre);
        }

        let hours = genre_pt / 3600;
        format!("You've spent {}h in {} — give this one a go", hours, best_genre)
    }
}

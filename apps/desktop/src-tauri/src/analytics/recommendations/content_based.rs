//! Content-based strategy — T31.
//!
//! Scores a candidate by how much the user has historically played games in
//! the same genre, weighted by total playtime.
//!
//! Formula:
//! ```text
//! genre_pt = genre_playtime.get(candidate.genre) or 0
//! score    = genre_pt / total_playtime_secs  (clamped 0..1)
//! ```
//!
//! Returns 0.0 when the candidate has no genre or the user has no playtime.

use rusqlite::Connection;

use super::{Candidate, strategy::{RecommendationStrategy, UserContext}};

pub struct ContentBasedStrategy;

impl RecommendationStrategy for ContentBasedStrategy {
    fn name(&self) -> &str { "content_based" }

    fn score(&self, _conn: &Connection, candidate: &Candidate, ctx: &UserContext) -> f64 {
        let genre = match &candidate.genre {
            Some(g) => g,
            None => return 0.0,
        };

        if ctx.total_playtime_secs == 0 {
            return 0.0;
        }

        let genre_pt = ctx.genre_playtime.get(genre).copied().unwrap_or(0);
        (genre_pt as f64 / ctx.total_playtime_secs as f64).min(1.0)
    }

    fn explain(&self, _conn: &Connection, candidate: &Candidate, ctx: &UserContext) -> String {
        let genre = match &candidate.genre {
            Some(g) => g.clone(),
            None => return "Matches your library style".to_string(),
        };

        let genre_pt = ctx.genre_playtime.get(&genre).copied().unwrap_or(0);
        if genre_pt == 0 {
            return format!("A {} game you haven't tried yet", genre);
        }

        let hours = genre_pt / 3600;
        format!("You've spent {}h in {} — give this one a go", hours, genre)
    }
}

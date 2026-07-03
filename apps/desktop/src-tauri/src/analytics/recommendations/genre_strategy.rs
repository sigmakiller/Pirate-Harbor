//! Genre match strategy — T31.
//!
//! Binary genre match: 1.0 if the candidate's genre appears anywhere in the
//! user's playtime map (i.e. they have played *any* game in that genre),
//! 0.5 if the user has *no* genre data at all (fallback: boost everything),
//! 0.0 if the genre is known but the user hasn't played it.
//!
//! Lighter-weight than ContentBasedStrategy — ignores time amounts.

use rusqlite::Connection;

use super::{Candidate, strategy::{RecommendationStrategy, UserContext}};

pub struct GenreStrategy;

impl RecommendationStrategy for GenreStrategy {
    fn name(&self) -> &str { "genre_match" }

    fn score(&self, _conn: &Connection, candidate: &Candidate, ctx: &UserContext) -> f64 {
        let genre = match &candidate.genre {
            Some(g) => g,
            None => return 0.3, // No genre info — mild boost
        };

        if ctx.genre_playtime.is_empty() {
            // User has no playtime history yet — treat all genres equally.
            return 0.5;
        }

        if ctx.genre_playtime.contains_key(genre.as_str()) {
            1.0
        } else {
            0.0
        }
    }

    fn explain(&self, _conn: &Connection, candidate: &Candidate, _ctx: &UserContext) -> String {
        match &candidate.genre {
            Some(g) => format!("In your favourite genre: {}", g),
            None    => "Something different to explore".to_string(),
        }
    }
}

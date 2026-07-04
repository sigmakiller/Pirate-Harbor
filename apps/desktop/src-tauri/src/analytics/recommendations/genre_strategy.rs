//! Genre match strategy — T31.
//!
//! Binary genre match: 1.0 if **any** token in the candidate's genre string
//! appears in the user's playtime map (i.e. they have played a game in that
//! genre), 0.5 if the user has no genre history, 0.0 otherwise.
//!
//! Handles comma-separated multi-genre strings (C2 / review fix):
//! "Action, RPG" scores 1.0 if the user has played RPG games, even if they've
//! never played a pure "Action, RPG" tagged game.

use rusqlite::Connection;

use super::{Candidate, strategy::{RecommendationStrategy, UserContext}};

pub struct GenreStrategy;

impl RecommendationStrategy for GenreStrategy {
    fn name(&self) -> &str { "genre_match" }

    fn score(&self, _conn: &Connection, candidate: &Candidate, ctx: &UserContext) -> f64 {
        let genre_str = match &candidate.genre {
            Some(g) => g,
            None => return 0.3, // No genre info — mild boost
        };

        if ctx.genre_playtime.is_empty() {
            // User has no playtime history — treat all genres equally.
            return 0.5;
        }

        // Match if ANY genre token in the candidate is in the user's history.
        let any_match = genre_str
            .split(',')
            .map(str::trim)
            .any(|g| ctx.genre_playtime.contains_key(g));

        if any_match { 1.0 } else { 0.0 }
    }

    fn explain(&self, _conn: &Connection, candidate: &Candidate, _ctx: &UserContext) -> String {
        match &candidate.genre {
            Some(g) => format!("In your favourite genre: {}", g),
            None    => "Something different to explore".to_string(),
        }
    }
}

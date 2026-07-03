//! Playtime strategy — T31.
//!
//! "You own this but have never played it."
//!
//! Candidates with zero playtime AND zero launch count are highlighted.
//! Score is 1.0 for complete backlog items, 0.5 for games that were
//! launched once but have no recorded playtime (e.g. crash on start).
//!
//! This strategy intentionally ignores genre — its purpose is purely
//! backlog-guilt reduction.

use rusqlite::Connection;

use super::{Candidate, strategy::{RecommendationStrategy, UserContext}};

pub struct PlaytimeStrategy;

impl RecommendationStrategy for PlaytimeStrategy {
    fn name(&self) -> &str { "unplayed_backlog" }

    fn score(&self, _conn: &Connection, candidate: &Candidate, _ctx: &UserContext) -> f64 {
        if candidate.total_playtime == 0 && candidate.launch_count == 0 {
            1.0 // Never touched — strongest signal
        } else if candidate.total_playtime == 0 {
            0.5 // Launched but no playtime tracked
        } else {
            0.0 // Has playtime — not a backlog candidate
        }
    }

    fn explain(&self, _conn: &Connection, candidate: &Candidate, _ctx: &UserContext) -> String {
        if candidate.launch_count == 0 {
            format!("\"{}\" is sitting untouched in your library", candidate.title)
        } else {
            format!("You started \"{}\" but never logged any playtime", candidate.title)
        }
    }
}

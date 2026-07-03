//! Recency strategy — T31.
//!
//! "You just added this and haven't played it yet."
//!
//! Scores candidates by how recently they were added to the library.
//! A game added within the last 7 days scores 1.0; 30 days → 0.5;
//! older → decays linearly to 0.0 at 365 days.
//!
//! This surfaces fresh acquisitions so the user doesn't forget new purchases.

use rusqlite::Connection;

use super::{Candidate, strategy::{RecommendationStrategy, UserContext}};

pub struct RecencyStrategy;

impl RecommendationStrategy for RecencyStrategy {
    fn name(&self) -> &str { "recently_added" }

    fn score(&self, _conn: &Connection, candidate: &Candidate, _ctx: &UserContext) -> f64 {
        let days_old = days_since(&candidate.added_at);

        if days_old <= 7 {
            1.0
        } else if days_old <= 30 {
            // Linear decay from 1.0 at 7d to 0.5 at 30d.
            let t = (days_old - 7) as f64 / 23.0;
            1.0 - t * 0.5
        } else if days_old <= 365 {
            // Linear decay from 0.5 at 30d to 0.0 at 365d.
            let t = (days_old - 30) as f64 / 335.0;
            0.5 - t * 0.5
        } else {
            0.0
        }
    }

    fn explain(&self, _conn: &Connection, candidate: &Candidate, _ctx: &UserContext) -> String {
        let days_old = days_since(&candidate.added_at);
        if days_old <= 7 {
            format!("You added \"{}\" this week — give it a try!", candidate.title)
        } else if days_old <= 30 {
            "Recently added to your library".to_string()
        } else {
            "Been in your library for a while, still unplayed".to_string()
        }
    }
}

/// Parse an ISO-8601 `added_at` string and return how many days ago it was.
/// Returns `i64::MAX` on parse failure so old-format dates don't score high.
fn days_since(added_at: &str) -> i64 {
    // Accept both "YYYY-MM-DD..." and full ISO timestamps.
    let date_part = &added_at[..added_at.len().min(10)];
    match chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
        Ok(date) => {
            let today = chrono::Utc::now().date_naive();
            (today - date).num_days().max(0)
        }
        Err(_) => i64::MAX,
    }
}

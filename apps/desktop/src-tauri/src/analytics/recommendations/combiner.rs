//! Strategy combiner — T31.
//!
//! `StrategyCombiner` holds a weighted list of strategies and produces a
//! ranked list of `RecommendationResult` from a set of candidate games.
//!
//! # Score computation
//!
//! ```text
//! composite = Σ (strategy.score(candidate, ctx) × weight) / Σ weights
//! ```
//!
//! The final list is sorted descending by composite score.  The primary
//! explanation string is taken from the highest-contributing strategy.

use rusqlite::Connection;

use super::{
    strategy::{RecommendationStrategy, UserContext},
    Candidate, RecommendationResult, StrategyContribution,
};

/// Weighted combination of recommendation strategies.
pub struct StrategyCombiner {
    strategies: Vec<(Box<dyn RecommendationStrategy>, f64)>,
}

impl StrategyCombiner {
    /// Construct with an ordered list of (strategy, weight) pairs.
    pub fn new(strategies: Vec<(Box<dyn RecommendationStrategy>, f64)>) -> Self {
        Self { strategies }
    }

    /// Build the default combiner with production-tuned weights:
    ///
    /// | Strategy       | Weight | Rationale                                      |
    /// |----------------|--------|------------------------------------------------|
    /// | ContentBased   | 0.40   | Genre affinity is the strongest long-term signal |
    /// | GenreMatch     | 0.25   | Binary genre sanity check                       |
    /// | PlaytimeBacklog| 0.20   | Backlog guilt driver                            |
    /// | Recency        | 0.15   | Fresh additions deserve attention               |
    pub fn default_combiner() -> Self {
        use super::{
            content_based::ContentBasedStrategy,
            genre_strategy::GenreStrategy,
            playtime_strategy::PlaytimeStrategy,
            recency_strategy::RecencyStrategy,
        };
        Self::new(vec![
            (Box::new(ContentBasedStrategy), 0.40),
            (Box::new(GenreStrategy),        0.25),
            (Box::new(PlaytimeStrategy),     0.20),
            (Box::new(RecencyStrategy),      0.15),
        ])
    }

    /// Score all candidates and return the top `limit` results.
    pub fn rank(
        &self,
        conn: &Connection,
        candidates: &[Candidate],
        ctx: &UserContext,
        limit: usize,
    ) -> Vec<RecommendationResult> {
        let total_weight: f64 = self.strategies.iter().map(|(_, w)| w).sum();

        let mut scored: Vec<(f64, RecommendationResult)> = candidates
            .iter()
            .map(|c| {
                let mut composite = 0.0_f64;
                let mut contributions: Vec<StrategyContribution> = Vec::new();
                let mut best_contribution_score = 0.0_f64;
                let mut best_reason = String::new();

                for (strategy, weight) in &self.strategies {
                    let raw   = strategy.score(conn, c, ctx).clamp(0.0, 1.0);
                    let weighted = raw * weight;
                    composite += weighted;

                    // Track the highest weighted contribution for the primary reason.
                    if weighted > best_contribution_score {
                        best_contribution_score = weighted;
                        best_reason = strategy.explain(conn, c, ctx);
                    }

                    contributions.push(StrategyContribution {
                        strategy: strategy.name().to_string(),
                        score:    raw,
                        weight:   *weight,
                    });
                }

                let final_score = if total_weight > 0.0 {
                    composite / total_weight
                } else {
                    0.0
                };

                let result = RecommendationResult {
                    game_id:    c.id.clone(),
                    title:      c.title.clone(),
                    cover_path: c.cover_path.clone(),
                    genre:      c.genre.clone(),
                    developer:  c.developer.clone(),
                    status:     c.status.clone(),
                    score:      (final_score * 1000.0).round() / 1000.0,
                    reason:     if best_reason.is_empty() {
                        "Worth a look from your backlog".to_string()
                    } else {
                        best_reason
                    },
                    strategy_contributions: contributions,
                };

                (final_score, result)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();

        // Sort descending by score.
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter().take(limit).map(|(_, r)| r).collect()
    }
}

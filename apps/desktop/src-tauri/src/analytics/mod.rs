//! Analytics engine for Pirate Harbor.
//!
//! Provides statistics and insights for gaming behavior, milestones,
//! and progress tracking.

pub mod milestones;
pub mod identity;
pub mod recommendations;

// ── T30 shared engines ────────────────────────────────────────────────────────
pub mod gaming_stats;
pub mod genre_stats;
pub mod completion_stats;
pub mod heatmap;
pub mod year_in_review;

use serde::{Deserialize, Serialize};

/// Generic timeline entry for trend analysis
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub date: String,
    pub count: i64,
}

/// Generic distribution entry
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionEntry {
    pub label: String,
    pub count: i64,
    pub percentage: f64,
}

//! Analytics engine for Pirate Harbor.
//!
//! Provides statistics and insights for gaming behavior, milestones,
//! and progress tracking.

pub mod milestones;

use serde::{Deserialize, Serialize};

/// Generic timeline entry for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub date: String,
    pub count: i64,
}

/// Generic distribution entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionEntry {
    pub label: String,
    pub count: i64,
    pub percentage: f64,
}

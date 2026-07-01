//! Job trait and core types for the background job system — T27.
//!
//! Every schedulable unit of work implements [`Job`]. The scheduler and
//! worker use only this trait, so new job types can be added without
//! touching infrastructure code.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// ── Status ────────────────────────────────────────────────────────────────────

/// Lifecycle state of a background job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum JobStatus {
    /// Waiting in the queue for a worker to pick it up.
    Queued,
    /// Actively running; `progress` is 0.0–1.0.
    Running { progress: f32 },
    /// Completed successfully.
    Done,
    /// Terminal failure with a human-readable message.
    Failed { reason: String },
    /// Cancelled before it was picked up.
    Cancelled,
}

impl JobStatus {
    /// Returns true if no further state transitions are possible.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done | Self::Failed { .. } | Self::Cancelled)
    }
}

// ── Result ────────────────────────────────────────────────────────────────────

/// Outcome returned from a successful job execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Human-readable summary of what was done.
    pub summary: String,
    /// Optional structured payload (JSON-serialisable).
    pub payload: Option<serde_json::Value>,
}

impl JobResult {
    // T28–T33: These constructors are the primary API for job implementations.
    #[allow(dead_code)]
    pub fn ok(summary: impl Into<String>) -> Self {
        Self { summary: summary.into(), payload: None }
    }

    #[allow(dead_code)]
    pub fn with_payload(summary: impl Into<String>, payload: serde_json::Value) -> Self {
        Self { summary: summary.into(), payload: Some(payload) }
    }
}

// ── Context ───────────────────────────────────────────────────────────────────

/// Everything a job needs to do its work.
///
/// `db` is a *separate* SQLite connection opened exclusively for background
/// jobs, so jobs never contend with the main Tauri command connection.
///
/// `app_handle` lets jobs emit progress events to the frontend.
// T28–T33: `db` and `app_handle` are accessed in every job implementation.
#[allow(dead_code)]
#[derive(Clone)]
pub struct JobContext {
    /// Dedicated background SQLite connection.
    pub db: Arc<Mutex<rusqlite::Connection>>,
    /// Tauri app handle — clone freely, it is reference-counted internally.
    pub app_handle: tauri::AppHandle,
}

impl JobContext {
    /// Construct from already-created parts.
    pub(super) fn new(
        db: Arc<Mutex<rusqlite::Connection>>,
        app_handle: tauri::AppHandle,
    ) -> Self {
        Self { db, app_handle }
    }
}

// ── Job trait ─────────────────────────────────────────────────────────────────

/// A unit of schedulable background work.
///
/// Implement this trait for each job type.  The worker calls [`Job::execute`]
/// on a Tokio `spawn_blocking` thread so implementations may use synchronous
/// I/O freely.  Report progress by calling `ctx.app_handle.emit(…)` directly.
pub trait Job: Send + Sync + 'static {
    /// Short identifier used in events and logs.
    /// Example: `"metadata_enrich"`, `"library_scan"`.
    fn name(&self) -> &str;

    /// Execute the job.  Return `Ok(JobResult)` on success or `Err(String)`
    /// on failure.  Must not panic.
    fn execute(&self, ctx: JobContext) -> Result<JobResult, String>;
}

// ── Info (serialisable snapshot) ──────────────────────────────────────────────

/// Serialisable snapshot of a queued or active job — returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: String,
    pub name: String,
    pub status: JobStatus,
    pub queued_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

// ── Progress event payload ─────────────────────────────────────────────────────

/// Payload emitted as `"job-progress"` Tauri events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgressEvent {
    pub job_id: String,
    pub job_name: String,
    pub status: JobStatus,
    pub summary: Option<String>,
}

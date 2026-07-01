//! Job scheduler — state management and public enqueueing API — T27.
//!
//! [`JobScheduler`] is managed as Tauri state and is the single entry-point
//! for all job operations.  It wraps the shared mutable state in an `Arc<Mutex>`
//! so Tauri commands can safely enqueue and query jobs from multiple threads.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use uuid::Uuid;

use super::job::{Job, JobInfo, JobStatus};
use super::queue::{JobQueue, Priority, QueueEntry};

// ── Shared mutable state ──────────────────────────────────────────────────────

/// All mutable scheduler state, protected by a single `Mutex`.
pub(crate) struct SchedulerState {
    /// Jobs waiting to run.
    pub queue: JobQueue,
    /// Jobs that are currently running or have recently finished.
    /// Keyed by job ID.
    pub running: HashMap<String, JobInfo>,
}

impl SchedulerState {
    fn new() -> Self {
        Self {
            queue: JobQueue::new(),
            running: HashMap::new(),
        }
    }
}

// ── Scheduler ─────────────────────────────────────────────────────────────────

/// Tauri-managed background job scheduler.
///
/// Clone this freely — it is a cheap `Arc` clone.
#[derive(Clone)]
pub struct JobScheduler {
    /// Shared state — `pub(crate)` so `lib.rs` can pass it to `start_worker`.
    pub(crate) state: Arc<Mutex<SchedulerState>>,
}

impl JobScheduler {
    /// Create a new scheduler.  Call `start` after creating Tauri state.
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SchedulerState::new())),
        }
    }

    // ── Enqueueing ────────────────────────────────────────────────────────────

    /// Enqueue a job with `Normal` priority.
    /// Returns the assigned job ID.
    // T28–T33: Called by job-specific commands (export, backup, scan, etc.).
    #[allow(dead_code)]
    pub fn enqueue(&self, job: impl Job) -> String {
        self.enqueue_with_priority(job, Priority::Normal)
    }

    /// Enqueue a job with the given priority.
    /// Returns the assigned job ID.
    #[allow(dead_code)]
    pub fn enqueue_with_priority(&self, job: impl Job, priority: Priority) -> String {
        let id = Uuid::new_v4().to_string();
        let queued_at = Utc::now().to_rfc3339();

        let entry = QueueEntry {
            id: id.clone(),
            priority,
            job: Box::new(job),
            queued_at: queued_at.clone(),
        };

        let mut state = self.state.lock().unwrap();

        // Record in running map as Queued so it's visible immediately.
        state.running.insert(
            id.clone(),
            JobInfo {
                id: id.clone(),
                name: entry.job.name().to_string(),
                status: JobStatus::Queued,
                queued_at,
                started_at: None,
                finished_at: None,
            },
        );

        state.queue.push(entry);
        id
    }

    // ── Querying ──────────────────────────────────────────────────────────────

    /// Return a snapshot of a specific job, or `None` if unknown.
    pub fn get_job_status(&self, job_id: &str) -> Option<JobInfo> {
        let state = self.state.lock().unwrap();
        state.running.get(job_id).cloned()
    }

    /// Return all non-terminal (queued or running) jobs.
    pub fn list_active_jobs(&self) -> Vec<JobInfo> {
        let state = self.state.lock().unwrap();
        state
            .running
            .values()
            .filter(|info| !info.status.is_terminal())
            .cloned()
            .collect()
    }

    /// Return all jobs (active + recently finished).
    pub fn list_all_jobs(&self) -> Vec<JobInfo> {
        let state = self.state.lock().unwrap();
        let mut jobs: Vec<JobInfo> = state.running.values().cloned().collect();
        // Sort by queued_at descending so newest is first.
        jobs.sort_by(|a, b| b.queued_at.cmp(&a.queued_at));
        jobs
    }

    // ── Cancellation ──────────────────────────────────────────────────────────

    /// Cancel a queued job.
    ///
    /// Returns `true` if the job was found in the queue and removed.
    /// Returns `false` if the job is already running or finished — those
    /// cannot be cancelled via this API.
    pub fn cancel_job(&self, job_id: &str) -> bool {
        let mut state = self.state.lock().unwrap();
        if state.queue.cancel(job_id) {
            // Update the status snapshot.
            if let Some(info) = state.running.get_mut(job_id) {
                info.status = JobStatus::Cancelled;
            }
            true
        } else {
            false
        }
    }

    // ── Stats ─────────────────────────────────────────────────────────────────

    /// Number of jobs waiting in the queue.
    pub fn queue_depth(&self) -> usize {
        self.state.lock().unwrap().queue.len()
    }
}

impl Default for JobScheduler {
    fn default() -> Self {
        Self::new()
    }
}

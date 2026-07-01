//! Tauri commands for the background job system — T27.
//!
//! Exposed commands:
//! - `get_job_status`   — snapshot of a single job
//! - `cancel_job`       — remove a queued job before it starts
//! - `list_active_jobs` — all queued/running jobs
//! - `list_all_jobs`    — active + recently finished

use tauri::State;

use crate::background::{JobInfo, JobScheduler};

/// Return the status snapshot for a single job.
///
/// Returns `None` if the job ID is unknown (never queued, or pruned from history).
#[tauri::command]
pub fn get_job_status(
    scheduler: State<'_, JobScheduler>,
    job_id: String,
) -> Option<JobInfo> {
    scheduler.get_job_status(&job_id)
}

/// Attempt to cancel a queued job.
///
/// Returns `true` if the job was found in the queue and removed.
/// Returns `false` if the job was already running, finished, or unknown —
/// running jobs cannot be cancelled via this API.
#[tauri::command]
pub fn cancel_job(
    scheduler: State<'_, JobScheduler>,
    job_id: String,
) -> bool {
    scheduler.cancel_job(&job_id)
}

/// Return all currently queued or running jobs.
#[tauri::command]
pub fn list_active_jobs(
    scheduler: State<'_, JobScheduler>,
) -> Vec<JobInfo> {
    scheduler.list_active_jobs()
}

/// Return all jobs including recently finished ones (up to the last 20).
#[tauri::command]
pub fn list_all_jobs(
    scheduler: State<'_, JobScheduler>,
) -> Vec<JobInfo> {
    scheduler.list_all_jobs()
}

/// Return the number of jobs currently waiting in the queue.
#[tauri::command]
pub fn queue_depth(
    scheduler: State<'_, JobScheduler>,
) -> usize {
    scheduler.queue_depth()
}

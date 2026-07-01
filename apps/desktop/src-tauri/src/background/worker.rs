//! Async worker — pulls jobs from the queue and executes them — T27.
//!
//! The worker runs as a Tokio task started at app launch.  It polls the
//! queue every 500 ms and runs each job in a `spawn_blocking` call so
//! synchronous job code never blocks the async runtime.
//!
//! Progress is reported to the frontend via Tauri events:
//!   - `"job-progress"` — emitted on status changes (Running, Done, Failed)
//!   - `"job-started"`  — emitted when a job is picked up
//!   - `"job-finished"` — emitted when a job completes (any terminal state)

use std::sync::{Arc, Mutex};

use chrono::Utc;
use tauri::Emitter;

use super::job::{JobContext, JobInfo, JobProgressEvent, JobStatus};
use super::scheduler::SchedulerState;

/// Start the background worker loop.  Call once from `lib.rs` setup.
///
/// The worker runs indefinitely until the app exits.
pub fn start_worker(
    state: Arc<Mutex<SchedulerState>>,
    db_path: std::path::PathBuf,
    app_handle: tauri::AppHandle,
) {
    let app = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        loop {
            // Poll every 500ms.
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // Dequeue next job under the lock, then immediately release.
            let entry = {
                let mut sched = state.lock().unwrap();
                sched.queue.pop()
            };

            let Some(entry) = entry else { continue };

            let job_id = entry.id.clone();
            let job_name = entry.job.name().to_string();
            let queued_at = entry.queued_at.clone();
            let started_at = Utc::now().to_rfc3339();

            // Mark as Running.
            {
                let mut sched = state.lock().unwrap();
                sched.running.insert(
                    job_id.clone(),
                    JobInfo {
                        id: job_id.clone(),
                        name: job_name.clone(),
                        status: JobStatus::Running { progress: 0.0 },
                        queued_at: queued_at.clone(),
                        started_at: Some(started_at.clone()),
                        finished_at: None,
                    },
                );
            }

            let _ = app.emit("job-started", JobProgressEvent {
                job_id: job_id.clone(),
                job_name: job_name.clone(),
                status: JobStatus::Running { progress: 0.0 },
                summary: None,
            });

            // Open a dedicated background connection for this job.
            let bg_conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => {
                    let _ = c.pragma_update(None, "journal_mode", "WAL");
                    let _ = c.pragma_update(None, "foreign_keys", "ON");
                    c
                }
                Err(e) => {
                    let reason = format!("Failed to open background DB connection: {e}");
                    finish_job(&state, &app, &job_id, &job_name, &queued_at, &started_at,
                               JobStatus::Failed { reason: reason.clone() }, None);
                    continue;
                }
            };

            let ctx = JobContext::new(
                Arc::new(Mutex::new(bg_conn)),
                app.clone(),
            );

            // Run the job on a blocking thread so sync I/O doesn't stall the runtime.
            let job = entry.job;
            let result = tokio::task::spawn_blocking(move || job.execute(ctx)).await;

            let (final_status, summary) = match result {
                Ok(Ok(job_result)) => (JobStatus::Done, Some(job_result.summary)),
                Ok(Err(reason)) => (JobStatus::Failed { reason }, None),
                Err(join_err) => (
                    JobStatus::Failed { reason: format!("Worker panicked: {join_err}") },
                    None,
                ),
            };

            finish_job(&state, &app, &job_id, &job_name, &queued_at, &started_at,
                       final_status, summary);
        }
    });
}

/// Update the running map and emit `"job-finished"`.
fn finish_job(
    state: &Arc<Mutex<SchedulerState>>,
    app: &tauri::AppHandle,
    job_id: &str,
    job_name: &str,
    queued_at: &str,
    started_at: &str,
    status: JobStatus,
    summary: Option<String>,
) {
    let finished_at = Utc::now().to_rfc3339();

    {
        let mut sched = state.lock().unwrap();
        sched.running.insert(
            job_id.to_string(),
            JobInfo {
                id: job_id.to_string(),
                name: job_name.to_string(),
                status: status.clone(),
                queued_at: queued_at.to_string(),
                started_at: Some(started_at.to_string()),
                finished_at: Some(finished_at.clone()),
            },
        );
        // Prune terminal jobs older than the last 20 to avoid unbounded growth.
        let terminal_ids: Vec<_> = sched
            .running
            .iter()
            .filter(|(_, info)| info.status.is_terminal())
            .map(|(id, _)| id.clone())
            .collect();
        if terminal_ids.len() > 20 {
            for id in &terminal_ids[..terminal_ids.len() - 20] {
                sched.running.remove(id);
            }
        }
    }

    let _ = app.emit("job-finished", JobProgressEvent {
        job_id: job_id.to_string(),
        job_name: job_name.to_string(),
        status,
        summary,
    });
}

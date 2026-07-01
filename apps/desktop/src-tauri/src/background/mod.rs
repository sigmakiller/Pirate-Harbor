//! Background job system — public API module — T27.
//!
//! # Architecture
//!
//! ```text
//! Tauri command
//!      │  enqueue(job)
//!      ▼
//! JobScheduler ──► JobQueue (VecDeque, priority-ordered)
//!                       │
//!                       │  poll every 500ms
//!                       ▼
//!                   Worker (Tokio task)
//!                       │  spawn_blocking(job.execute(ctx))
//!                       ▼
//!                   Job impl
//!                       │  app_handle.emit("job-progress", …)
//!                       ▼
//!                   Frontend (useJobProgress hook)
//! ```
//!
//! # Usage
//!
//! 1. Create a [`JobScheduler`] and register it as Tauri state.
//! 2. Call [`start_worker`] once after setup.
//! 3. In any Tauri command, `State<'_, JobScheduler>` gives access to
//!    `enqueue`, `cancel_job`, `list_active_jobs`, etc.
//!
//! # Implementing a job
//!
//! ```text
//! struct MyJob { arg: String }
//!
//! impl Job for MyJob {
//!     fn name(&self) -> &str { "my_job" }
//!     fn execute(&self, ctx: JobContext) -> Result<JobResult, String> {
//!         // Do work, emit progress…
//!         Ok(JobResult::ok("Done"))
//!     }
//! }
//! ```

pub mod job;
pub mod queue;
pub mod scheduler;
pub mod worker;

// ── Re-exports ────────────────────────────────────────────────────────────────

// T28–T33 job implementations will consume these — suppress "unused" until then.
#[allow(unused_imports)]
pub use job::{Job, JobContext, JobInfo, JobProgressEvent, JobResult, JobStatus};
#[allow(unused_imports)]
pub use queue::Priority;
pub use scheduler::JobScheduler;
pub use worker::start_worker;

// ── Tauri state alias ─────────────────────────────────────────────────────────

/// Convenience alias — register this as Tauri managed state.
/// T28+: job-specific commands will request `State<'_, BackgroundState>`.
#[allow(dead_code)]
pub type BackgroundState = JobScheduler;

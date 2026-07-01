//! Priority queue for background jobs — T27.
//!
//! A simple `VecDeque`-based FIFO queue with two priorities: `High` and
//! `Normal`.  High-priority entries are placed at the front; normal entries
//! are appended to the back.  The queue is wrapped in a `Mutex` in
//! [`crate::background::mod`] so it can be shared across threads.
//!
//! T28–T33: All public items here are consumed by job implementations.
#![allow(dead_code)]

use std::collections::VecDeque;

use super::job::Job;

// ── Priority ──────────────────────────────────────────────────────────────────

/// Scheduling priority for a queued job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    /// Runs before all `Normal` jobs.
    High,
    /// Default priority — appended to the back of the queue.
    Normal,
}

// ── Entry ─────────────────────────────────────────────────────────────────────

/// A job waiting in the queue, paired with its generated ID.
pub struct QueueEntry {
    pub id: String,
    pub priority: Priority,
    pub job: Box<dyn Job>,
    pub queued_at: String,
}

// ── Queue ─────────────────────────────────────────────────────────────────────

/// Non-thread-safe job queue.  Always access through the `Mutex` in
/// [`super::JobScheduler`].
pub struct JobQueue {
    inner: VecDeque<QueueEntry>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self { inner: VecDeque::new() }
    }

    /// Enqueue a job.  High-priority jobs go to the front; normal jobs to
    /// the back.
    pub fn push(&mut self, entry: QueueEntry) {
        if entry.priority == Priority::High {
            self.inner.push_front(entry);
        } else {
            self.inner.push_back(entry);
        }
    }

    /// Dequeue the next job to run (front of queue).
    pub fn pop(&mut self) -> Option<QueueEntry> {
        self.inner.pop_front()
    }

    /// Number of jobs waiting.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove a queued (not yet started) job by ID.
    /// Returns `true` if the job was found and removed.
    pub fn cancel(&mut self, id: &str) -> bool {
        if let Some(pos) = self.inner.iter().position(|e| e.id == id) {
            self.inner.remove(pos);
            true
        } else {
            false
        }
    }

    /// IDs of all queued entries in order.
    pub fn queued_ids(&self) -> Vec<String> {
        self.inner.iter().map(|e| e.id.clone()).collect()
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}

/**
 * useJobProgress — T27
 *
 * Subscribes to the three Tauri background-job events:
 *   - `"job-started"`  — a job was picked up by the worker
 *   - `"job-finished"` — a job reached a terminal state
 *   - `"job-progress"` — in-flight progress updates (optional)
 *
 * Returns the list of currently active (queued/running) jobs and a boolean
 * `hasActiveJobs` for simple indicator rendering in TopBar.
 *
 * Usage:
 * ```tsx
 * const { activeJobs, hasActiveJobs } = useJobProgress();
 * ```
 */

import { useEffect, useState, useCallback } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

// ── Types (mirror Rust side) ──────────────────────────────────────────────────

export type JobStatusType =
  | { type: "queued" }
  | { type: "running"; progress: number }
  | { type: "done" }
  | { type: "failed"; reason: string }
  | { type: "cancelled" };

export interface JobInfo {
  id: string;
  name: string;
  status: JobStatusType;
  queued_at: string;
  started_at: string | null;
  finished_at: string | null;
}

export interface JobProgressEvent {
  job_id: string;
  job_name: string;
  status: JobStatusType;
  summary: string | null;
}

// ── Hook ──────────────────────────────────────────────────────────────────────

export function useJobProgress() {
  const [activeJobs, setActiveJobs] = useState<JobInfo[]>([]);

  /** Refresh the active-job list from the backend. */
  const refresh = useCallback(async () => {
    try {
      const jobs = await invoke<JobInfo[]>("list_active_jobs");
      setActiveJobs(jobs);
    } catch {
      // Silently ignore — app may still be booting.
    }
  }, []);

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    const setup = async () => {
      // Initial load.
      await refresh();

      // Re-fetch when any job starts.
      unlisteners.push(
        await listen<JobProgressEvent>("job-started", () => refresh())
      );

      // Re-fetch when any job finishes (Done / Failed / Cancelled).
      unlisteners.push(
        await listen<JobProgressEvent>("job-finished", () => refresh())
      );

      // Optionally track in-flight progress updates (e.g. for progress bars).
      unlisteners.push(
        await listen<JobProgressEvent>("job-progress", (event) => {
          setActiveJobs((prev) =>
            prev.map((job) =>
              job.id === event.payload.job_id
                ? { ...job, status: event.payload.status }
                : job
            )
          );
        })
      );
    };

    setup();

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  }, [refresh]);

  const hasActiveJobs = activeJobs.length > 0;

  return { activeJobs, hasActiveJobs, refresh };
}

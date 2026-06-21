/**
 * utils.ts — Shared frontend utilities.
 */

import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

// ── Tailwind class helper ─────────────────────────────────────────────────────

/**
 * Merge Tailwind classes safely — handles conditional classes and
 * eliminates class conflicts (e.g. `p-4` vs `p-2`).
 *
 * Usage:
 *   cn("base-class", condition && "conditional-class", "always-on")
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

// ── Playtime formatting ───────────────────────────────────────────────────────

/**
 * Format total playtime seconds into a human-readable string.
 *
 * Examples:
 *   0        → "0 min"
 *   45       → "0 min"
 *   3600     → "1 hr"
 *   5400     → "1 hr 30 min"
 *   86400    → "24 hr"
 */
export function formatPlaytime(totalSecs: number): string {
  if (totalSecs <= 0) return "0 min";

  const hours   = Math.floor(totalSecs / 3600);
  const minutes = Math.floor((totalSecs % 3600) / 60);

  if (hours === 0) return `${minutes} min`;
  if (minutes === 0) return `${hours} hr`;
  return `${hours} hr ${minutes} min`;
}

// ── Date formatting ───────────────────────────────────────────────────────────

/**
 * Format an ISO 8601 date string into a locale-friendly display string.
 *
 * Example: "2024-03-15T10:30:00Z" → "Mar 15, 2024"
 */
export function formatDate(iso: string): string {
  const date = new Date(iso);
  return date.toLocaleDateString("en-US", {
    year:  "numeric",
    month: "short",
    day:   "numeric",
  });
}

/**
 * Format an ISO 8601 date string as a relative time description.
 *
 * Examples:
 *   < 1 min ago   → "Just now"
 *   < 1 hour ago  → "X minutes ago"
 *   < 1 day ago   → "X hours ago"
 *   < 7 days ago  → "X days ago"
 *   >= 7 days ago → formatted date (e.g. "Mar 15, 2024")
 */
export function formatRelativeDate(iso: string): string {
  const date = new Date(iso);
  const now  = new Date();
  const diffMs  = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);

  if (diffSec < 60)                      return "Just now";
  if (diffSec < 3600)                    return `${Math.floor(diffSec / 60)} minutes ago`;
  if (diffSec < 86400)                   return `${Math.floor(diffSec / 3600)} hours ago`;
  if (diffSec < 7 * 86400)              return `${Math.floor(diffSec / 86400)} days ago`;
  return formatDate(iso);
}

// ── Game status display ───────────────────────────────────────────────────────

import type { GameStatus } from "@/types";

/** Human-readable label for each game status. */
export const STATUS_LABELS: Record<GameStatus, string> = {
  unplayed:  "Unplayed",
  playing:   "Playing",
  completed: "Completed",
  dropped:   "Dropped",
};

/** Monochrome CSS color for each status (uses Atlas OS tokens). */
export const STATUS_COLORS: Record<GameStatus, string> = {
  unplayed:  "var(--color-status-unplayed)",
  playing:   "var(--color-status-playing)",
  completed: "var(--color-status-completed)",
  dropped:   "var(--color-status-dropped)",
};

/**
 * EnrichmentProgressBar — Global progress indicator for metadata enrichment.
 *
 * Displays a fixed progress bar at the top of the screen during bulk enrichment.
 * Shows completion percentage, completed/total counts, and allows cancellation.
 */

import { X } from "lucide-react";
import type { EnrichmentProgress } from "@/hooks/useEnrichmentProgress";

interface EnrichmentProgressBarProps {
  progress: EnrichmentProgress;
  onDismiss?: () => void;
}

export function EnrichmentProgressBar({ progress, onDismiss }: EnrichmentProgressBarProps) {
  const percentage = progress.total > 0
    ? Math.round((progress.completed / progress.total) * 100)
    : 0;

  const isDone = progress.completed === progress.total;

  return (
    <div style={styles.container} role="progressbar" aria-valuenow={percentage} aria-valuemin={0} aria-valuemax={100}>
      <div style={styles.content}>
        <div style={styles.info}>
          <span style={styles.label}>
            {isDone ? "Enrichment complete" : "Enriching library..."}
          </span>
          <span style={styles.stats}>
            {progress.completed} / {progress.total}
            {progress.failed > 0 && <span style={styles.failed}> ({progress.failed} failed)</span>}
          </span>
        </div>

        <div style={styles.barTrack}>
          <div
            style={{ ...styles.barFill, width: `${percentage}%` }}
            aria-hidden="true"
          />
        </div>

        {onDismiss && (
          <button
            type="button"
            onClick={onDismiss}
            style={styles.dismissBtn}
            aria-label="Dismiss enrichment progress"
          >
            <X size={12} />
          </button>
        )}
      </div>
    </div>
  );
}

const styles = {
  container: {
    position: "fixed" as const,
    top: 0,
    left: 0,
    right: 0,
    zIndex: 1000,
    background: "var(--color-surface)",
    borderBottom: "1px solid var(--color-border)",
    padding: "12px 20px",
    boxShadow: "0 2px 8px rgba(0, 0, 0, 0.1)",
  },
  content: {
    display: "flex",
    alignItems: "center",
    gap: 16,
    maxWidth: "100%",
  },
  info: {
    display: "flex",
    flexDirection: "column" as const,
    gap: 4,
    minWidth: 200,
  },
  label: {
    fontFamily: "var(--font-body)",
    fontSize: 12,
    fontWeight: 600,
    color: "var(--color-text-primary)",
  },
  stats: {
    fontFamily: "var(--font-mono)",
    fontSize: 10,
    letterSpacing: "0.04em",
    color: "var(--color-text-disabled)",
  },
  failed: {
    color: "var(--color-text-muted)",
  },
  barTrack: {
    flex: 1,
    height: 3,
    background: "var(--color-border)",
    borderRadius: 2,
    overflow: "hidden",
  },
  barFill: {
    height: "100%",
    background: "var(--color-text-secondary)",
    transition: "width 300ms ease",
  },
  dismissBtn: {
    flexShrink: 0,
    background: "none",
    border: "1px solid var(--color-border)",
    borderRadius: 1,
    color: "var(--color-text-muted)",
    cursor: "pointer",
    padding: "4px",
    display: "flex",
    transition: "border-color 150ms, color 150ms",
  },
} satisfies Record<string, React.CSSProperties>;

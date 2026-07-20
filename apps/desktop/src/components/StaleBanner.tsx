/**
 * StaleBanner — T51: Stale-data notification banner.
 *
 * Shown at the top of LibraryPage when one or more games lack fresh
 * metadata (>30 days or never fetched).  "Refresh Now" queues the
 * BulkEnrichmentJob (T50); "Dismiss" hides the banner for 24 hours by
 * writing `stale_banner_dismissed_at` into the settings table.
 */

import { AlertCircle, RefreshCw, X } from "lucide-react";

interface StaleBannerProps {
  count:       number;
  /** Called when the user clicks "Refresh Now". */
  onRefresh:   () => void;
  /** Called when the user clicks "Dismiss". */
  onDismiss:   () => void;
  /** True while the refresh job is being queued (shows spinner). */
  refreshing?: boolean;
}

export function StaleBanner({
  count,
  onRefresh,
  onDismiss,
  refreshing = false,
}: StaleBannerProps) {
  return (
    <div
      id="stale-metadata-banner"
      role="alert"
      aria-live="polite"
      style={styles.banner}
    >
      {/* Icon + message */}
      <span style={styles.iconWrap} aria-hidden="true">
        <AlertCircle size={14} />
      </span>

      <span style={styles.message}>
        <strong>{count}</strong>{" "}
        {count === 1 ? "game has" : "games have"} stale metadata (&gt;30 days
        old).
      </span>

      {/* Actions */}
      <div style={styles.actions}>
        <button
          id="stale-banner-refresh-btn"
          onClick={onRefresh}
          disabled={refreshing}
          style={{
            ...styles.btn,
            ...styles.refreshBtn,
            opacity: refreshing ? 0.6 : 1,
            cursor: refreshing ? "not-allowed" : "pointer",
          }}
          aria-label="Refresh stale metadata now"
        >
          <RefreshCw
            size={12}
            style={{
              animation: refreshing ? "spin 1s linear infinite" : "none",
            }}
          />
          {refreshing ? "Queuing…" : "Refresh Now"}
        </button>

        <button
          id="stale-banner-dismiss-btn"
          onClick={onDismiss}
          style={{ ...styles.btn, ...styles.dismissBtn }}
          aria-label="Dismiss stale metadata notice for 24 hours"
        >
          <X size={12} />
          Dismiss
        </button>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  banner: {
    display:        "flex",
    alignItems:     "center",
    gap:            12,
    padding:        "10px 20px",
    background:     "color-mix(in srgb, var(--accent, #6366f1) 12%, var(--surface-raised, #1e1e28))",
    borderBottom:   "1px solid color-mix(in srgb, var(--accent, #6366f1) 30%, transparent)",
    fontSize:       13,
    color:          "var(--text-primary, #e2e2e9)",
    flexWrap:       "wrap",
    rowGap:         6,
  },
  iconWrap: {
    flexShrink:  0,
    color:       "var(--accent, #6366f1)",
    display:     "flex",
    alignItems:  "center",
  },
  message: {
    flex:       1,
    minWidth:   180,
    lineHeight: 1.4,
  },
  actions: {
    display:    "flex",
    gap:        8,
    flexShrink: 0,
  },
  btn: {
    display:      "inline-flex",
    alignItems:   "center",
    gap:          5,
    padding:      "5px 12px",
    borderRadius: 4,
    fontSize:     12,
    fontWeight:   600,
    border:       "1px solid transparent",
    transition:   "opacity 150ms, background 150ms",
  },
  refreshBtn: {
    background: "var(--accent, #6366f1)",
    color:      "#fff",
  },
  dismissBtn: {
    background: "transparent",
    border:     "1px solid color-mix(in srgb, var(--text-muted, #888) 40%, transparent)",
    color:      "var(--text-muted, #888)",
    cursor:     "pointer",
  },
};

/**
 * ToastContainer — Renders the active toast queue.
 *
 * Mount once inside AppLayout. Toasts appear bottom-right, auto-dismiss.
 * Per COMPONENTS.md: monochrome, minimal, flat.
 * Per MOTION.md: fade + slide in (150ms), fade out.
 */

import { X, Check, AlertCircle, Info } from "lucide-react";
import { useToastStore, type Toast, type ToastType } from "@/stores/useToastStore";

const ICONS: Record<ToastType, React.ReactNode> = {
  success:     <Check       size={13} />,
  error:       <AlertCircle size={13} />,
  info:        <Info        size={13} />,
  achievement: <span aria-hidden="true">🏆</span>,
};


const TYPE_COLOR: Record<ToastType, React.CSSProperties> = {
  success:     { color: "var(--color-text-secondary)" },
  error:       { color: "var(--color-text-muted)"     },
  info:        { color: "var(--color-text-disabled)"  },
  achievement: { color: "#fbbf24"                     },   // amber-400
};

/** Extra per-type overrides applied to the whole toast card. */
const TYPE_CARD: Partial<Record<ToastType, React.CSSProperties>> = {
  achievement: {
    background:  "linear-gradient(135deg, #1e1040 0%, #2d1a6e 100%)",
    borderColor: "#7c3aed",
    color:       "#fff",
    fontWeight:  600,
  },
};


function ToastItem({ toast }: { toast: Toast }) {
  const { removeToast } = useToastStore();
  const { type, message, id } = toast;

  return (
    <div
      style={{ ...styles.toast, ...TYPE_CARD[type] }}
      className={type === "achievement" ? "toast-achievement" : undefined}
      role="status"
      aria-live="polite"
      aria-atomic="true"
      id={id}
    >
      <span style={{ ...styles.icon, ...TYPE_COLOR[type] }} aria-hidden="true">
        {ICONS[type]}
      </span>
      <p style={styles.message}>{message}</p>
      {toast.action && (
        <button
          type="button"
          id={`${id}-action`}
          onClick={() => { toast.action!.onClick(); removeToast(id); }}
          style={styles.actionBtn}
        >
          {toast.action.label}
        </button>
      )}
      <button
        type="button"
        onClick={() => removeToast(id)}
        style={styles.dismissBtn}
        aria-label="Dismiss notification"
      >
        <X size={11} />
      </button>
    </div>
  );
}

export default function ToastContainer() {
  const { toasts } = useToastStore();
  if (toasts.length === 0) return null;

  return (
    <div
      style={styles.container}
      role="region"
      aria-label="Notifications"
      aria-live="polite"
    >
      {toasts.map(t => <ToastItem key={t.id} toast={t} />)}
    </div>
  );
}

// ── Styles ────────────────────────────────────────────────────────────────────

const styles = {
  container: {
    position:      "fixed" as const,
    bottom:        24,
    right:         24,
    display:       "flex",
    flexDirection: "column" as const,
    gap:           8,
    zIndex:        300,
    pointerEvents: "none" as const,   // allows click-through on the gap
  },
  toast: {
    display:      "flex",
    alignItems:   "center",
    gap:          10,
    padding:      "10px 14px",
    background:   "var(--color-surface)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    minWidth:     260,
    maxWidth:     380,
    boxShadow:    "0 4px 16px rgba(0,0,0,0.4)",
    pointerEvents: "auto" as const,
    animation:    "toast-in 150ms ease forwards",
  },
  icon: {
    display:  "flex",
    flexShrink: 0,
  },
  message: {
    fontFamily: "var(--font-body)",
    fontSize:   13,
    color:      "var(--color-text-primary)",
    margin:     0,
    flex:       1,
    lineHeight: 1.4,
  },
  actionBtn: {
    background:   "color-mix(in srgb, var(--color-accent, #6366f1) 15%, transparent)",
    border:       "1px solid color-mix(in srgb, var(--color-accent, #6366f1) 40%, transparent)",
    borderRadius: 4,
    color:        "var(--color-accent, #6366f1)",
    cursor:       "pointer",
    fontSize:     11,
    fontWeight:   700,
    padding:      "3px 8px",
    flexShrink:   0,
    transition:   "background 150ms",
    whiteSpace:   "nowrap" as const,
  },
  dismissBtn: {
    background:  "none",
    border:      "none",
    color:       "var(--color-text-disabled)",
    cursor:      "pointer",
    padding:     "2px",
    display:     "flex",
    flexShrink:  0,
    transition:  "color 150ms",
    borderRadius: 1,
  },
} satisfies Record<string, React.CSSProperties>;

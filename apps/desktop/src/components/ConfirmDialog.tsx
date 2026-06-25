/**
 * ConfirmDialog — Reusable destructive-action confirmation modal.
 *
 * Per COMPONENTS.md: monochrome, flat, no shadows.
 * Keyboard: Escape dismisses (calls onCancel). Enter fires onConfirm.
 *
 * Usage:
 *   <ConfirmDialog
 *     open={open}
 *     title="Delete game"
 *     message="This cannot be undone."
 *     confirmLabel="Delete"
 *     onConfirm={handleDelete}
 *     onCancel={() => setOpen(false)}
 *     dangerous
 *   />
 */

import { useEffect, useRef } from "react";
import { X, AlertTriangle } from "lucide-react";

export interface ConfirmDialogProps {
  /** Whether the dialog is mounted and visible. */
  open:         boolean;
  /** Short, action-oriented title. */
  title:        string;
  /** Supporting message — can be longer. */
  message:      string;
  /** Label for the confirm button. Defaults to "Confirm". */
  confirmLabel?: string;
  /** Label for the cancel button. Defaults to "Cancel". */
  cancelLabel?:  string;
  /** When true, styles the confirm button as a destructive action. */
  dangerous?:   boolean;
  /** Called when the user confirms. */
  onConfirm:    () => void;
  /** Called when the user cancels or presses Escape. */
  onCancel:     () => void;
}

export default function ConfirmDialog({
  open,
  title,
  message,
  confirmLabel = "Confirm",
  cancelLabel  = "Cancel",
  dangerous    = false,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const confirmRef = useRef<HTMLButtonElement>(null);

  // Focus confirm button when opened, handle keyboard
  useEffect(() => {
    if (!open) return;
    confirmRef.current?.focus();

    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") { e.preventDefault(); onCancel(); }
      if (e.key === "Enter")  { e.preventDefault(); onConfirm(); }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [open, onConfirm, onCancel]);

  if (!open) return null;

  return (
    /* Backdrop */
    <div
      style={styles.backdrop}
      onClick={onCancel}
      role="dialog"
      aria-modal="true"
      aria-labelledby="confirm-dialog-title"
      aria-describedby="confirm-dialog-msg"
    >
      {/* Panel — stop propagation so clicking inside doesn't dismiss */}
      <div style={styles.panel} onClick={e => e.stopPropagation()}>

        {/* Header */}
        <div style={styles.header}>
          {dangerous && (
            <AlertTriangle size={14} style={{ color: "var(--color-text-secondary)", flexShrink: 0 }} aria-hidden="true" />
          )}
          <h2 id="confirm-dialog-title" style={styles.title}>{title}</h2>
          <button
            type="button"
            onClick={onCancel}
            style={styles.closeBtn}
            aria-label="Cancel and close"
          >
            <X size={13} />
          </button>
        </div>

        {/* Message */}
        <p id="confirm-dialog-msg" style={styles.message}>{message}</p>

        {/* Actions */}
        <div style={styles.actions}>
          <button
            type="button"
            onClick={onCancel}
            style={styles.cancelBtn}
          >
            {cancelLabel}
          </button>
          <button
            id="confirm-dialog-confirm-btn"
            ref={confirmRef}
            type="button"
            onClick={onConfirm}
            style={{
              ...styles.confirmBtn,
              ...(dangerous ? styles.confirmBtnDangerous : styles.confirmBtnDefault),
            }}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}

// ── Styles ────────────────────────────────────────────────────────────────────

const styles = {
  backdrop: {
    position:        "fixed" as const,
    inset:           0,
    background:      "rgba(5, 5, 5, 0.72)",
    display:         "flex",
    alignItems:      "center",
    justifyContent:  "center" as const,
    zIndex:          200,
  },
  panel: {
    width:        420,
    maxWidth:     "calc(100vw - 48px)",
    background:   "var(--color-surface)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "24px",
    display:      "flex",
    flexDirection: "column" as const,
    gap:          16,
  },
  header: {
    display:    "flex",
    alignItems: "center",
    gap:        10,
  },
  title: {
    fontFamily:    "var(--font-display)",
    fontSize:      18,
    fontWeight:    700,
    letterSpacing: "-0.01em",
    color:         "var(--color-text-primary)",
    margin:        0,
    flex:          1,
  },
  closeBtn: {
    background:  "none",
    border:      "none",
    color:       "var(--color-text-disabled)",
    cursor:      "pointer",
    padding:     "4px",
    display:     "flex",
    borderRadius: 1,
    transition:  "color 150ms",
    flexShrink:  0,
  },
  message: {
    fontFamily:  "var(--font-body)",
    fontSize:    14,
    lineHeight:  1.6,
    color:       "var(--color-text-muted)",
    margin:      0,
  },
  actions: {
    display:        "flex",
    justifyContent: "flex-end" as const,
    gap:            8,
    marginTop:      4,
  },
  cancelBtn: {
    background:   "none",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "9px 20px",
    fontSize:     12,
    fontFamily:   "var(--font-body)",
    color:        "var(--color-text-muted)",
    cursor:       "pointer",
    transition:   "border-color 150ms, color 150ms",
  },
  confirmBtn: {
    border:        "none",
    borderRadius:  1,
    padding:       "9px 20px",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    fontWeight:    600,
    cursor:        "pointer",
    transition:    "opacity 150ms",
  },
  confirmBtnDefault: {
    background: "var(--color-text-primary)",
    color:      "var(--color-base)",
  },
  confirmBtnDangerous: {
    background: "var(--color-text-secondary)",
    color:      "var(--color-base)",
  },
} satisfies Record<string, React.CSSProperties>;

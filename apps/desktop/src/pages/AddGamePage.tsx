/**
 * AddGamePage — manually add a game to the library.
 *
 * Fields: title (required), exe_path (required), cover image,
 * developer, publisher, genre, status.
 */

import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { ArrowLeft } from "lucide-react";

import { FilePickerButton } from "@/components/FilePickerButton";
import { addGame }          from "@/lib/api";
import type { GameStatus, NewGame } from "@/types";

const STATUS_OPTIONS: { value: GameStatus; label: string }[] = [
  { value: "unplayed",  label: "Unplayed"  },
  { value: "playing",   label: "Playing"   },
  { value: "completed", label: "Completed" },
  { value: "dropped",   label: "Dropped"   },
];

export default function AddGamePage() {
  const navigate = useNavigate();

  // Form state
  const [title,     setTitle]     = useState("");
  const [exePath,   setExePath]   = useState("");
  const [coverPath, setCoverPath] = useState("");
  const [developer, setDeveloper] = useState("");
  const [publisher, setPublisher] = useState("");
  const [genre,     setGenre]     = useState("");
  const [status,    setStatus]    = useState<GameStatus>("unplayed");

  // UI state
  const [saving, setSaving] = useState(false);
  const [error,  setError]  = useState<string | null>(null);

  const canSubmit = title.trim() !== "" && exePath.trim() !== "";

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canSubmit || saving) return;

    setSaving(true);
    setError(null);

    const payload: NewGame = {
      title:       title.trim(),
      exe_path:    exePath.trim(),
      cover_path:  coverPath.trim() || null,
      developer:   developer.trim() || null,
      publisher:   publisher.trim() || null,
      genre:       genre.trim() || null,
      status,
    };

    try {
      const game = await addGame(payload);
      navigate(`/library/${game.id}`);
    } catch (e) {
      setError(String(e));
      setSaving(false);
    }
  };

  return (
    <div className="atlas-enter" style={styles.page}>
      {/* Back */}
      <button
        onClick={() => navigate("/library")}
        style={styles.backBtn}
        aria-label="Back to library"
      >
        <ArrowLeft size={14} />
        Library
      </button>

      {/* Header */}
      <h1 style={styles.title}>Add Game</h1>
      <p style={styles.subtitle}>Manually add a game to your archive.</p>

      {/* Form */}
      <form onSubmit={handleSubmit} style={styles.form} noValidate>

        {/* Title */}
        <Field label="Title" required>
          <input
            id="game-title"
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="e.g. Dark Souls III"
            style={styles.input}
            required
            autoFocus
          />
        </Field>

        {/* Executable */}
        <Field label="Executable" required hint="The .exe file to launch the game">
          <FilePickerButton
            id="game-exe-picker"
            value={exePath}
            onChange={setExePath}
            filters={[{ name: "Executables", extensions: ["exe"] }]}
            placeholder="Browse for .exe file…"
          />
        </Field>

        {/* Cover image */}
        <Field label="Cover Image" hint="Optional — portrait artwork (3:4 ratio works best)">
          <FilePickerButton
            id="game-cover-picker"
            value={coverPath}
            onChange={setCoverPath}
            filters={[
              { name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] },
            ]}
            placeholder="Browse for cover image…"
          />
        </Field>

        {/* Developer */}
        <Field label="Developer">
          <input
            id="game-developer"
            type="text"
            value={developer}
            onChange={(e) => setDeveloper(e.target.value)}
            placeholder="e.g. FromSoftware"
            style={styles.input}
          />
        </Field>

        {/* Publisher */}
        <Field label="Publisher">
          <input
            id="game-publisher"
            type="text"
            value={publisher}
            onChange={(e) => setPublisher(e.target.value)}
            placeholder="e.g. Bandai Namco"
            style={styles.input}
          />
        </Field>

        {/* Genre */}
        <Field label="Genre" hint="Comma-separated for multiple">
          <input
            id="game-genre"
            type="text"
            value={genre}
            onChange={(e) => setGenre(e.target.value)}
            placeholder="e.g. Action RPG"
            style={styles.input}
          />
        </Field>

        {/* Status */}
        <Field label="Status">
          <div style={styles.statusGroup} role="group" aria-label="Game status">
            {STATUS_OPTIONS.map(({ value, label }) => (
              <button
                key={value}
                type="button"
                onClick={() => setStatus(value)}
                style={{
                  ...styles.statusChip,
                  ...(status === value ? styles.statusChipActive : {}),
                }}
                aria-pressed={status === value}
              >
                {label}
              </button>
            ))}
          </div>
        </Field>

        {/* Error */}
        {error && (
          <p style={styles.errorMsg} role="alert">
            {error}
          </p>
        )}

        {/* Actions */}
        <div style={styles.actions}>
          <button
            type="button"
            onClick={() => navigate("/library")}
            style={styles.cancelBtn}
          >
            Cancel
          </button>
          <button
            id="add-game-submit-btn"
            type="submit"
            disabled={!canSubmit || saving}
            style={{
              ...styles.submitBtn,
              opacity: !canSubmit || saving ? 0.4 : 1,
              cursor:  !canSubmit || saving ? "not-allowed" : "pointer",
            }}
          >
            {saving ? "Adding…" : "Add to Library"}
          </button>
        </div>
      </form>
    </div>
  );
}

// ── Field wrapper ─────────────────────────────────────────────────────────────

interface FieldProps {
  label:    string;
  required?: boolean;
  hint?:    string;
  children: React.ReactNode;
}

function Field({ label, required, hint, children }: FieldProps) {
  return (
    <div style={fieldStyles.group}>
      <label style={fieldStyles.label}>
        {label}
        {required && <span style={fieldStyles.required} aria-hidden="true"> *</span>}
      </label>
      {children}
      {hint && <p style={fieldStyles.hint}>{hint}</p>}
    </div>
  );
}

// ── Styles ────────────────────────────────────────────────────────────────────

const styles = {
  page: {
    padding:   "40px 56px",
    maxWidth:  640,
    overflowY: "auto" as const,
    height:    "100%",
    boxSizing: "border-box" as const,
  },
  backBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           6,
    background:    "none",
    border:        "none",
    color:         "var(--color-text-muted)",
    fontSize:      12,
    fontFamily:    "var(--font-mono)",
    letterSpacing: "0.08em",
    textTransform: "uppercase" as const,
    cursor:        "pointer",
    padding:       0,
    marginBottom:  40,
    transition:    "color 150ms",
  },
  title: {
    fontFamily:    "var(--font-display)",
    fontSize:      "clamp(40px, 4vw, 64px)",
    fontWeight:    700,
    letterSpacing: "-0.03em",
    lineHeight:    1.0,
    color:         "var(--color-text-primary)",
    margin:        0,
    marginBottom:  8,
  },
  subtitle: {
    fontFamily:   "var(--font-body)",
    fontSize:     14,
    color:        "var(--color-text-muted)",
    margin:       0,
    marginBottom: 48,
  },
  form: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           28,
  },
  input: {
    width:       "100%",
    background:  "var(--color-surface)",
    border:      "1px solid var(--color-border)",
    borderRadius: 1,
    padding:     "9px 12px",
    fontSize:    13,
    fontFamily:  "var(--font-body)",
    color:       "var(--color-text-primary)",
    outline:     "none",
    boxSizing:   "border-box" as const,
    transition:  "border-color 150ms",
  },
  statusGroup: {
    display: "flex",
    gap:     6,
  },
  statusChip: {
    background:    "none",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "6px 14px",
    fontSize:      11,
    fontFamily:    "var(--font-mono)",
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-muted)",
    cursor:        "pointer",
    transition:    "border-color 150ms, color 150ms",
  },
  statusChipActive: {
    borderColor: "var(--color-text-secondary)",
    color:       "var(--color-text-primary)",
  },
  errorMsg: {
    fontFamily: "var(--font-body)",
    fontSize:   13,
    color:      "var(--color-text-muted)",
    margin:     0,
    padding:    "10px 14px",
    border:     "1px solid var(--color-border)",
    borderRadius: 1,
  },
  actions: {
    display:    "flex",
    gap:        12,
    paddingTop: 12,
  },
  cancelBtn: {
    background:    "none",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "10px 24px",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    color:         "var(--color-text-muted)",
    cursor:        "pointer",
    letterSpacing: "0.04em",
    transition:    "border-color 150ms",
  },
  submitBtn: {
    background:    "var(--color-text-primary)",
    border:        "none",
    borderRadius:  1,
    padding:       "10px 28px",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    fontWeight:    600,
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    color:         "var(--color-base)",
    transition:    "opacity 150ms",
  },
} satisfies Record<string, React.CSSProperties>;

const fieldStyles = {
  group: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           8,
  },
  label: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.10em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-muted)",
    fontWeight:    500,
  },
  required: {
    color: "var(--color-text-disabled)",
  },
  hint: {
    fontFamily: "var(--font-body)",
    fontSize:   12,
    color:      "var(--color-text-disabled)",
    margin:     0,
  },
} satisfies Record<string, React.CSSProperties>;

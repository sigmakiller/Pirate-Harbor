/**
 * AddGamePage — manually add a game to the library.
 *
 * T11: Adds optional RAWG metadata search above the form.
 * When a result is selected it auto-fills title and genre.
 * All fields remain manually editable — metadata is a convenience, not a lock-in.
 */

import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { ArrowLeft, Search, X, Loader } from "lucide-react";

import { FilePickerButton }   from "@/components/FilePickerButton";
import { addGame, searchGameMetadata } from "@/lib/api";
import type { MetadataResult }         from "@/lib/api";
import type { GameStatus, NewGame }    from "@/types";

const STATUS_OPTIONS: { value: GameStatus; label: string }[] = [
  { value: "unplayed",  label: "Unplayed"  },
  { value: "playing",   label: "Playing"   },
  { value: "completed", label: "Completed" },
  { value: "dropped",   label: "Dropped"   },
];

export default function AddGamePage() {
  const navigate = useNavigate();

  // ── Metadata search state ─────────────────────────────────────────────────
  const [metaQuery,    setMetaQuery]    = useState("");
  const [metaResults,  setMetaResults]  = useState<MetadataResult[]>([]);
  const [metaSearching, setMetaSearching] = useState(false);
  const [metaError,    setMetaError]    = useState<string | null>(null);
  const [metaSelected, setMetaSelected] = useState<string | null>(null); // name of selected result

  // ── Form state ────────────────────────────────────────────────────────────
  const [title,     setTitle]     = useState("");
  const [exePath,   setExePath]   = useState("");
  const [coverPath, setCoverPath] = useState("");
  const [developer, setDeveloper] = useState("");
  const [publisher, setPublisher] = useState("");
  const [genre,     setGenre]     = useState("");
  const [status,    setStatus]    = useState<GameStatus>("unplayed");

  // ── Submit state ──────────────────────────────────────────────────────────
  const [saving, setSaving] = useState(false);
  const [error,  setError]  = useState<string | null>(null);

  const canSubmit = title.trim() !== "" && exePath.trim() !== "";

  // ── Metadata search handlers ───────────────────────────────────────────────
  const handleMetaSearch = async () => {
    const q = metaQuery.trim();
    if (!q) return;
    setMetaSearching(true);
    setMetaError(null);
    setMetaResults([]);
    try {
      const results = await searchGameMetadata(q);
      setMetaResults(results);
      if (results.length === 0) setMetaError("No results found.");
    } catch (e) {
      setMetaError(String(e));
    } finally {
      setMetaSearching(false);
    }
  };

  const handleMetaSelect = (result: MetadataResult) => {
    setMetaSelected(result.name);
    setTitle(result.name);
    if (result.genres) setGenre(result.genres);
    // Collapse the search panel after selection
    setMetaResults([]);
    setMetaQuery("");
  };

  // ── Form submit ────────────────────────────────────────────────────────────
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canSubmit || saving) return;

    setSaving(true);
    setError(null);

    const payload: NewGame = {
      title:      title.trim(),
      exe_path:   exePath.trim(),
      cover_path: coverPath.trim() || null,
      developer:  developer.trim() || null,
      publisher:  publisher.trim() || null,
      genre:      genre.trim() || null,
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

      {/* ── Metadata search ────────────────────────────────────────────────── */}
      <div style={styles.metaSection} aria-label="Metadata search">
        <span style={styles.metaLabel}>Search for game info</span>
        <span style={styles.metaHint}>
          Auto-fill title and genre from RAWG · requires API key in Settings
        </span>

        <div style={styles.metaRow}>
          <div style={styles.metaInputWrapper}>
            <input
              id="meta-search-input"
              type="search"
              value={metaQuery}
              onChange={(e) => setMetaQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleMetaSearch()}
              placeholder="Search game name…"
              style={styles.metaInput}
              aria-label="Search for game metadata"
            />
            {metaQuery && (
              <button
                type="button"
                onClick={() => { setMetaQuery(""); setMetaResults([]); setMetaError(null); }}
                style={styles.metaClearBtn}
                aria-label="Clear search"
              >
                <X size={12} />
              </button>
            )}
          </div>
          <button
            id="meta-search-btn"
            type="button"
            onClick={handleMetaSearch}
            disabled={metaSearching || !metaQuery.trim()}
            style={{
              ...styles.metaSearchBtn,
              opacity: metaSearching || !metaQuery.trim() ? 0.4 : 1,
            }}
            aria-label="Search for game metadata"
          >
            {metaSearching
              ? <Loader size={13} style={{ animation: "spin 1s linear infinite" }} />
              : <Search size={13} />
            }
            {metaSearching ? "Searching…" : "Search"}
          </button>
        </div>

        {/* Selected result badge */}
        {metaSelected && (
          <div style={styles.metaSelectedBadge}>
            <span style={styles.metaSelectedText}>Auto-filled from: {metaSelected}</span>
            <button
              type="button"
              onClick={() => setMetaSelected(null)}
              style={styles.metaClearBadgeBtn}
              aria-label="Clear metadata selection"
            >
              <X size={11} />
            </button>
          </div>
        )}

        {/* Error */}
        {metaError && (
          <p style={styles.metaErrorText} role="alert">{metaError}</p>
        )}

        {/* Results */}
        {metaResults.length > 0 && (
          <div style={styles.metaResults} role="listbox" aria-label="Metadata search results">
            {metaResults.map((r) => (
              <button
                key={r.name}
                type="button"
                role="option"
                aria-selected={metaSelected === r.name}
                onClick={() => handleMetaSelect(r)}
                style={styles.metaResultRow}
              >
                {/* Thumbnail */}
                {r.cover_url ? (
                  <img
                    src={r.cover_url}
                    alt=""
                    style={styles.metaThumb}
                    loading="lazy"
                  />
                ) : (
                  <div style={styles.metaThumbPlaceholder} />
                )}

                {/* Info */}
                <div style={styles.metaResultInfo}>
                  <span style={styles.metaResultName}>{r.name}</span>
                  <span style={styles.metaResultSub}>
                    {[r.genres, r.release_year].filter(Boolean).join(" · ")}
                  </span>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>

      {/* ── Manual form ────────────────────────────────────────────────────── */}
      <form onSubmit={handleSubmit} style={styles.form} noValidate>

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

        <Field label="Executable" required hint="The .exe file to launch the game">
          <FilePickerButton
            id="game-exe-picker"
            value={exePath}
            onChange={setExePath}
            filters={[{ name: "Executables", extensions: ["exe"] }]}
            placeholder="Browse for .exe file…"
          />
        </Field>

        <Field label="Cover Image" hint="Optional — portrait artwork (3:4 ratio works best)">
          <FilePickerButton
            id="game-cover-picker"
            value={coverPath}
            onChange={setCoverPath}
            filters={[{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] }]}
            placeholder="Browse for cover image…"
          />
        </Field>

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

        {error && (
          <p style={styles.errorMsg} role="alert">{error}</p>
        )}

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

function Field({
  label,
  required,
  hint,
  children,
}: {
  label:     string;
  required?: boolean;
  hint?:     string;
  children:  React.ReactNode;
}) {
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
    marginBottom: 40,
  },

  // ── Metadata search panel ─────────────────────────────────────────────────
  metaSection: {
    display:      "flex",
    flexDirection: "column" as const,
    gap:          8,
    marginBottom: 40,
    padding:      "20px",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    background:   "var(--color-surface)",
  },
  metaLabel: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.10em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-muted)",
    fontWeight:    500,
  },
  metaHint: {
    fontFamily:   "var(--font-body)",
    fontSize:     12,
    color:        "var(--color-text-disabled)",
    marginBottom: 8,
  },
  metaRow: {
    display: "flex",
    gap:     8,
  },
  metaInputWrapper: {
    position: "relative" as const,
    flex:     1,
  },
  metaInput: {
    width:        "100%",
    background:   "var(--color-elevated)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "8px 32px 8px 12px",
    fontSize:     13,
    fontFamily:   "var(--font-body)",
    color:        "var(--color-text-primary)",
    outline:      "none",
    boxSizing:    "border-box" as const,
  },
  metaClearBtn: {
    position:   "absolute" as const,
    right:      8,
    top:        "50%",
    transform:  "translateY(-50%)",
    background: "none",
    border:     "none",
    color:      "var(--color-text-disabled)",
    cursor:     "pointer",
    padding:    2,
    display:    "flex",
  },
  metaSearchBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           6,
    background:    "none",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "8px 16px",
    fontSize:      12,
    fontFamily:    "var(--font-mono)",
    letterSpacing: "0.06em",
    color:         "var(--color-text-muted)",
    cursor:        "pointer",
    flexShrink:    0,
    transition:    "border-color 150ms",
  },
  metaSelectedBadge: {
    display:      "flex",
    alignItems:   "center",
    gap:          8,
    padding:      "6px 12px",
    background:   "var(--color-elevated)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    alignSelf:    "flex-start" as const,
  },
  metaSelectedText: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    color:         "var(--color-text-muted)",
    letterSpacing: "0.04em",
  },
  metaClearBadgeBtn: {
    background: "none",
    border:     "none",
    color:      "var(--color-text-disabled)",
    cursor:     "pointer",
    padding:    0,
    display:    "flex",
  },
  metaErrorText: {
    fontFamily:  "var(--font-body)",
    fontSize:    12,
    color:       "var(--color-text-muted)",
    margin:      0,
    lineHeight:  1.5,
  },
  metaResults: {
    display:       "flex",
    flexDirection: "column" as const,
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    overflow:      "hidden",
    marginTop:     4,
  },
  metaResultRow: {
    display:    "flex",
    alignItems: "center",
    gap:        12,
    padding:    "10px 12px",
    background: "none",
    border:     "none",
    borderBottom: "1px solid var(--color-border-sub)",
    cursor:     "pointer",
    width:      "100%",
    textAlign:  "left" as const,
    transition: "background 150ms",
  },
  metaThumb: {
    width:        40,
    height:       54, // 3:4 ratio
    objectFit:    "cover" as const,
    borderRadius: 1,
    flexShrink:   0,
    display:      "block",
    background:   "var(--color-elevated)",
  },
  metaThumbPlaceholder: {
    width:        40,
    height:       54,
    borderRadius: 1,
    background:   "var(--color-elevated)",
    flexShrink:   0,
  },
  metaResultInfo: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           3,
    flex:          1,
    minWidth:      0,
  },
  metaResultName: {
    fontFamily:   "var(--font-body)",
    fontSize:     13,
    fontWeight:   500,
    color:        "var(--color-text-primary)",
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
  },
  metaResultSub: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    color:         "var(--color-text-disabled)",
    letterSpacing: "0.04em",
    overflow:      "hidden",
    textOverflow:  "ellipsis",
    whiteSpace:    "nowrap" as const,
  },

  // ── Form ──────────────────────────────────────────────────────────────────
  form: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           28,
  },
  input: {
    width:        "100%",
    background:   "var(--color-surface)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "9px 12px",
    fontSize:     13,
    fontFamily:   "var(--font-body)",
    color:        "var(--color-text-primary)",
    outline:      "none",
    boxSizing:    "border-box" as const,
    transition:   "border-color 150ms",
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
    fontFamily:   "var(--font-body)",
    fontSize:     13,
    color:        "var(--color-text-muted)",
    margin:       0,
    padding:      "10px 14px",
    border:       "1px solid var(--color-border)",
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

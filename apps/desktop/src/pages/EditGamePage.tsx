/**
 * EditGamePage — Edit all game metadata fields.
 *
 * Per Task 14: "Pre-filled form (reuses AddGamePage's Field pattern).
 * All game fields editable. Save → calls updateGame → back to detail.
 * Cancel → back."
 *
 * Route: /library/:id/edit
 */

import { useEffect, useState, useCallback } from "react";
import { useNavigate, useParams }           from "react-router-dom";
import { ArrowLeft, Save, Shield, Download, Plus, Trash2 } from "lucide-react";

import { getGame, updateGame }                from "@/lib/api";
import {
  getAchievementTrackingStatus,
  enableAchievementTracking,
  disableAchievementTracking,
  detectSteamAppId,
  getAchievementMappings,
  addAchievementMapping,
  removeAchievementMapping,
  importAchievementsFromSteam,
  type AchievementMapping,
  type TrackingStatus,
  type AppIdDetectionResult,
}                                            from "@/lib/api";
import { useToastStore }                     from "@/stores/useToastStore";
import { FilePickerButton }                  from "@/components/FilePickerButton";
import type { Game, GameStatus }             from "@/types";

// ─────────────────────────────────────────────────────────────────────────────

const STATUS_OPTIONS: { value: GameStatus; label: string }[] = [
  { value: "unplayed",  label: "Unplayed"  },
  { value: "playing",   label: "Playing"   },
  { value: "completed", label: "Completed" },
  { value: "dropped",   label: "Dropped"   },
];

// ─────────────────────────────────────────────────────────────────────────────

export default function EditGamePage() {
  const { id }        = useParams<{ id: string }>();
  const navigate      = useNavigate();
  const { addToast }  = useToastStore();

  const [loading,   setLoading]   = useState(true);
  const [saving,    setSaving]    = useState(false);
  const [error,     setError]     = useState<string | null>(null);

  // Form fields -- mirroring UpdateGame
  const [title,     setTitle]     = useState("");
  const [exePath,   setExePath]   = useState("");
  const [coverPath, setCoverPath] = useState("");
  const [developer, setDeveloper] = useState("");
  const [publisher, setPublisher] = useState("");
  const [genre,     setGenre]     = useState("");
  const [status,    setStatus]    = useState<GameStatus>("unplayed");
  // Keep a reference to the full game for exe_path-based directory derivation.
  const [game, setGame] = useState<Game | null>(null);

  // Achievement tracking state (T45)
  const [trackingStatus,  setTrackingStatus]  = useState<TrackingStatus | null>(null);
  const [mappings,        setMappings]        = useState<AchievementMapping[]>([]);
  const [steamAppId,      setSteamAppId]      = useState("");
  const [appIdSource,     setAppIdSource]     = useState<string | null>(null);
  const [trackingLoading, setTrackingLoading] = useState(false);
  const [importing,       setImporting]       = useState(false);
  // New mapping row inputs
  const [newSteamId,      setNewSteamId]      = useState("");
  const [newDisplayName,  setNewDisplayName]  = useState("");
  const [newPoints,       setNewPoints]       = useState(10);

  useEffect(() => {
    if (!id) return;
    (async () => {
      try {
        const g: Game = await getGame(id);
        setGame(g);
        setTitle(g.title        ?? "");
        setExePath(g.exe_path   ?? "");
        setCoverPath(g.cover_path ?? "");
        setDeveloper(g.developer  ?? "");
        setPublisher(g.publisher  ?? "");
        setGenre(g.genre          ?? "");
        setStatus(g.status        ?? "unplayed");
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    })();
  }, [id]);

  // Load achievement tracking status and mappings.
  const loadAchievementData = useCallback(async () => {
    if (!id) return;
    try {
      const [statusResult, mapsResult] = await Promise.all([
        getAchievementTrackingStatus(id),
        getAchievementMappings(id),
      ]);
      setTrackingStatus(statusResult);
      setMappings(mapsResult);
      setSteamAppId(statusResult.steam_app_id ?? "");

      // Auto-detect App ID only if not already saved and exe_path is known.
      if (!statusResult.steam_app_id && game?.exe_path) {
        const lastSep = Math.max(
          game.exe_path.lastIndexOf("\\"),
          game.exe_path.lastIndexOf("/"),
        );
        const dir = game.exe_path.substring(0, lastSep);
        const det: AppIdDetectionResult = await detectSteamAppId(id, dir);
        if (det.app_id) {
          setSteamAppId(det.app_id);
          setAppIdSource(det.source);
        }
      }
    } catch {
      // Achievement data is non-critical — silently degrade.
    }
  }, [id, game?.exe_path]);

  useEffect(() => {
    loadAchievementData();
  }, [loadAchievementData]);


  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!id || !title.trim()) return;
    setSaving(true);
    setError(null);
    try {
      await updateGame(id, {
        title:     title.trim(),
        exe_path:  exePath.trim()   || undefined,
        cover_path: coverPath.trim() || null,
        developer:  developer.trim() || null,
        publisher:  publisher.trim() || null,
        genre:      genre.trim()     || null,
        status,
      });
      addToast({ message: `"${title.trim()}" updated`, type: "success" });
      navigate(`/library/${id}`);
    } catch (err) {
      setError(String(err));
      addToast({ message: "Failed to save changes", type: "error" });
    } finally {
      setSaving(false);
    }
  };

  const handleCancel = () => navigate(`/library/${id}`);

  // ── Render ─────────────────────────────────────────────────────────────────

  if (loading) {
    return (
      <div style={styles.loadingShell}>
        <p style={styles.loadingText}>Loading…</p>
      </div>
    );
  }

  return (
    <div className="atlas-enter" style={styles.root}>

      {/* ── Back / Header ──────────────────────────────────────────────────── */}
      <div style={styles.topBar}>
        <button
          type="button"
          onClick={handleCancel}
          style={styles.backBtn}
          aria-label="Cancel and go back"
        >
          <ArrowLeft size={14} />
          Back
        </button>
        <h1 style={styles.pageTitle}>Edit Game</h1>
      </div>

      {/* ── Form ───────────────────────────────────────────────────────────── */}
      <form
        onSubmit={handleSave}
        style={styles.form}
        aria-label="Edit game details"
        noValidate
      >

        {/* Error banner */}
        {error && (
          <div style={styles.errorBanner} role="alert">
            <p style={styles.errorText}>{error}</p>
          </div>
        )}

        {/* Title */}
        <Field label="Title" required>
          <input
            id="edit-title"
            type="text"
            value={title}
            onChange={e => setTitle(e.target.value)}
            style={styles.input}
            required
            autoFocus
            aria-label="Game title"
          />
        </Field>

        {/* Executable */}
        <Field label="Executable Path">
          <div style={styles.filePickerRow}>
            <FilePickerButton
              id="edit-exe-path-picker"
              value={exePath}
              onChange={(path: string) => setExePath(path)}
              filters={[{ name: "Executable", extensions: ["exe"] }]}
              placeholder="Browse for .exe…"
            />
          </div>
        </Field>

        {/* Cover */}
        <Field label="Cover Image">
          <div style={styles.filePickerRow}>
            <FilePickerButton
              id="edit-cover-path-picker"
              value={coverPath}
              onChange={(path: string) => setCoverPath(path)}
              filters={[{ name: "Image", extensions: ["jpg", "jpeg", "png", "webp"] }]}
              placeholder="Browse for cover image…"
            />
          </div>
        </Field>

        {/* Status */}
        <Field label="Status">
          <div style={styles.statusRow} role="radiogroup" aria-label="Game status">
            {STATUS_OPTIONS.map(opt => (
              <label
                key={opt.value}
                style={{
                  ...styles.statusChip,
                  ...(status === opt.value ? styles.statusChipActive : {}),
                }}
              >
                <input
                  type="radio"
                  name="status"
                  value={opt.value}
                  checked={status === opt.value}
                  onChange={() => setStatus(opt.value)}
                  style={{ display: "none" }}
                />
                {opt.label}
              </label>
            ))}
          </div>
        </Field>

        {/* Developer / Publisher — two-column row */}
        <div style={styles.twoCol}>
          <Field label="Developer">
            <input
              id="edit-developer"
              type="text"
              value={developer}
              onChange={e => setDeveloper(e.target.value)}
              style={styles.input}
              aria-label="Developer"
              placeholder="e.g. CD Projekt Red"
            />
          </Field>
          <Field label="Publisher">
            <input
              id="edit-publisher"
              type="text"
              value={publisher}
              onChange={e => setPublisher(e.target.value)}
              style={styles.input}
              aria-label="Publisher"
              placeholder="e.g. Bandai Namco"
            />
          </Field>
        </div>

        {/* Genre */}
        <Field label="Genre">
          <input
            id="edit-genre"
            type="text"
            value={genre}
            onChange={e => setGenre(e.target.value)}
            style={styles.input}
            aria-label="Genre"
            placeholder="e.g. RPG, Action, Strategy"
          />
        </Field>

        {/* ── Achievement Tracking (T45) ─────────────────────────────────── */}
        <section style={achStyles.achievementSection}>
          <h3 style={achStyles.sectionTitle}>
            <Shield size={16} aria-hidden="true" />
            Achievement Tracking
          </h3>

          <p style={achStyles.sectionHint}>
            Only for games with <code>steam_api64.dll</code>. Do{" "}
            <strong>not</strong> enable for games with online anti-cheat
            (EAC, BattlEye).
          </p>

          {/* Enable / Disable toggle */}
          <label style={achStyles.toggleRow}>
            <span style={achStyles.toggleLabel}>Enable Automated Achievement Tracking</span>
            <input
              id="achievement-tracking-toggle"
              type="checkbox"
              checked={trackingStatus?.enabled ?? false}
              disabled={trackingLoading || !exePath}
              onChange={async (e) => {
                setTrackingLoading(true);
                try {
                  if (e.target.checked) {
                    await enableAchievementTracking(id!, exePath, steamAppId);
                  } else {
                    await disableAchievementTracking(id!, exePath);
                  }
                  await loadAchievementData();
                  addToast({
                    message: e.target.checked
                      ? "Achievement tracking enabled"
                      : "Tracking disabled",
                    type: "success",
                  });
                } catch (err) {
                  addToast({ message: String(err), type: "error" });
                } finally {
                  setTrackingLoading(false);
                }
              }}
            />
          </label>

          {/* Steam App ID field */}
          <Field label="Steam App ID">
            <div style={achStyles.inputWithHint}>
              <input
                id="steam-app-id"
                type="text"
                value={steamAppId}
                onChange={(e) => setSteamAppId(e.target.value)}
                style={styles.input}
                placeholder="e.g. 570"
                aria-label="Steam App ID"
              />
              {appIdSource === "rawg" && (
                <span style={achStyles.hintOk}>✓ Auto-detected via RAWG</span>
              )}
              {appIdSource === "local_file" && (
                <span style={achStyles.hintOk}>✓ Found in game folder</span>
              )}
            </div>
          </Field>

          {/* Import from Steam button */}
          <button
            id="import-achievements-btn"
            type="button"
            disabled={!steamAppId || importing}
            style={{
              ...achStyles.secondaryBtn,
              opacity: !steamAppId || importing ? 0.45 : 1,
            }}
            onClick={async () => {
              setImporting(true);
              try {
                const maps = await importAchievementsFromSteam(id!, steamAppId);
                setMappings(maps);
                addToast({
                  message: `Imported ${maps.length} achievement${maps.length === 1 ? "" : "s"}`,
                  type: "success",
                });
              } catch (err) {
                addToast({ message: String(err), type: "error" });
              } finally {
                setImporting(false);
              }
            }}
          >
            <Download size={14} aria-hidden="true" />
            {importing ? "Importing…" : "Import from Steam"}
          </button>

          {/* Mappings table */}
          {mappings.length > 0 && (
            <table style={achStyles.mappingsTable} aria-label="Achievement mappings">
              <thead>
                <tr>
                  <th style={achStyles.th}>Steam ID</th>
                  <th style={achStyles.th}>Display Name</th>
                  <th style={achStyles.th}>Pts</th>
                  <th style={achStyles.th} aria-label="Actions" />
                </tr>
              </thead>
              <tbody>
                {mappings.map((m) => (
                  <tr key={m.id}>
                    <td style={achStyles.td}><code>{m.steam_id}</code></td>
                    <td style={achStyles.td}>{m.display_name}</td>
                    <td style={achStyles.td}>{m.points}</td>
                    <td style={achStyles.td}>
                      <button
                        type="button"
                        style={achStyles.iconBtn}
                        aria-label={`Remove ${m.display_name}`}
                        onClick={async () => {
                          await removeAchievementMapping(m.id);
                          setMappings((prev) => prev.filter((x) => x.id !== m.id));
                        }}
                      >
                        <Trash2 size={12} />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}

          {/* Add mapping row */}
          <div style={achStyles.addMappingRow}>
            <input
              id="new-steam-id"
              type="text"
              placeholder="ACH_ID"
              value={newSteamId}
              onChange={(e) => setNewSteamId(e.target.value)}
              style={{ ...styles.input, flex: 1 }}
              aria-label="New achievement Steam ID"
            />
            <input
              id="new-display-name"
              type="text"
              placeholder="Display name"
              value={newDisplayName}
              onChange={(e) => setNewDisplayName(e.target.value)}
              style={{ ...styles.input, flex: 2 }}
              aria-label="New achievement display name"
            />
            <input
              id="new-points"
              type="number"
              value={newPoints}
              onChange={(e) => setNewPoints(Number(e.target.value))}
              style={{ ...styles.input, width: 64, flex: "none" }}
              aria-label="Points"
              min={0}
            />
            <button
              id="add-mapping-btn"
              type="button"
              disabled={!newSteamId || !newDisplayName}
              style={{
                ...achStyles.secondaryBtn,
                opacity: !newSteamId || !newDisplayName ? 0.45 : 1,
              }}
              onClick={async () => {
                if (!newSteamId || !newDisplayName) return;
                const m = await addAchievementMapping(
                  id!, newSteamId, newDisplayName, null, newPoints,
                );
                setMappings((prev) => [...prev, m]);
                setNewSteamId("");
                setNewDisplayName("");
                setNewPoints(10);
              }}
            >
              <Plus size={14} aria-hidden="true" /> Add
            </button>
          </div>
        </section>

        {/* Form actions */}
        <div style={styles.formActions}>
          <button
            type="button"
            onClick={handleCancel}
            style={styles.cancelBtn}
          >
            Cancel
          </button>
          <button
            id="save-game-btn"
            type="submit"
            disabled={!title.trim() || saving}
            style={{
              ...styles.saveBtn,
              opacity: !title.trim() || saving ? 0.45 : 1,
            }}
          >
            <Save size={13} aria-hidden="true" />
            {saving ? "Saving…" : "Save Changes"}
          </button>
        </div>
      </form>
    </div>
  );
}

// ── Field wrapper ─────────────────────────────────────────────────────────────

function Field({
  label, required, children,
}: {
  label:    string;
  required?: boolean;
  children: React.ReactNode;
}) {
  return (
    <div style={fieldStyles.root}>
      <label style={fieldStyles.label}>
        {label}
        {required && <span style={fieldStyles.required} aria-hidden="true"> *</span>}
      </label>
      {children}
    </div>
  );
}

const fieldStyles = {
  root: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           6,
  },
  label: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.12em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
  },
  required: {
    color: "var(--color-text-muted)",
  },
} satisfies Record<string, React.CSSProperties>;

// ── Page styles ───────────────────────────────────────────────────────────────

const styles = {
  loadingShell: {
    display:        "flex",
    alignItems:     "center",
    justifyContent: "center",
    height:         "100%",
  },
  loadingText: {
    fontFamily:    "var(--font-mono)",
    fontSize:      12,
    color:         "var(--color-text-disabled)",
    letterSpacing: "0.08em",
    margin:        0,
  },
  root: {
    padding:   "40px 56px",
    maxWidth:  680,
    overflowY: "auto" as const,
  },
  topBar: {
    display:    "flex",
    alignItems: "center",
    gap:        20,
    marginBottom: 36,
  },
  backBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           6,
    background:    "none",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "7px 14px",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    color:         "var(--color-text-muted)",
    cursor:        "pointer",
    transition:    "border-color 150ms, color 150ms",
    flexShrink:    0,
  },
  pageTitle: {
    fontFamily:    "var(--font-display)",
    fontSize:      "clamp(32px, 3vw, 48px)",
    fontWeight:    700,
    letterSpacing: "-0.03em",
    lineHeight:    1.0,
    color:         "var(--color-text-primary)",
    margin:        0,
  },
  form: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           24,
  },
  errorBanner: {
    padding:      "12px 16px",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    background:   "var(--color-elevated)",
  },
  errorText: {
    fontFamily: "var(--font-body)",
    fontSize:   13,
    color:      "var(--color-text-muted)",
    margin:     0,
  },
  input: {
    background:   "var(--color-elevated)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "10px 14px",
    fontSize:     14,
    fontFamily:   "var(--font-body)",
    color:        "var(--color-text-primary)",
    outline:      "none",
    width:        "100%",
    boxSizing:    "border-box" as const,
    transition:   "border-color 150ms",
  },
  filePickerRow: {
    display: "flex",
    gap:     10,
  },
  statusRow: {
    display: "flex",
    gap:     8,
    flexWrap: "wrap" as const,
  },
  statusChip: {
    display:       "inline-flex",
    alignItems:    "center",
    gap:           5,
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "7px 16px",
    fontSize:      12,
    fontFamily:    "var(--font-mono)",
    letterSpacing: "0.06em",
    color:         "var(--color-text-muted)",
    cursor:        "pointer",
    transition:    "border-color 150ms, color 150ms",
    userSelect:    "none" as const,
  },
  statusChipActive: {
    borderColor: "var(--color-text-secondary)",
    color:       "var(--color-text-primary)",
  },
  twoCol: {
    display:             "grid",
    gridTemplateColumns: "1fr 1fr",
    gap:                 16,
  },
  formActions: {
    display:        "flex",
    justifyContent: "flex-end" as const,
    gap:            10,
    marginTop:      8,
    paddingTop:     24,
    borderTop:      "1px solid var(--color-border)",
  },
  cancelBtn: {
    background:   "none",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "10px 22px",
    fontSize:     13,
    fontFamily:   "var(--font-body)",
    color:        "var(--color-text-muted)",
    cursor:       "pointer",
    transition:   "border-color 150ms, color 150ms",
  },
  saveBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           7,
    background:    "var(--color-text-primary)",
    border:        "none",
    borderRadius:  1,
    padding:       "10px 24px",
    fontSize:      13,
    fontFamily:    "var(--font-body)",
    fontWeight:    600,
    color:         "var(--color-base)",
    cursor:        "pointer",
    transition:    "opacity 150ms",
  },
} satisfies Record<string, React.CSSProperties>;

// ── Achievement section styles (T45) ──────────────────────────────────────────
// Defined separately because they mix CSSProperties and non-CSSProperties values
// (e.g. table styles require string-typed display).
const achStyles: Record<string, React.CSSProperties> = {
  achievementSection: {
    borderTop:    "1px solid var(--color-border)",
    paddingTop:   24,
    display:      "flex",
    flexDirection: "column",
    gap:          16,
  },
  sectionTitle: {
    display:    "flex",
    alignItems: "center",
    gap:        8,
    fontFamily: "var(--font-display)",
    fontSize:   18,
    fontWeight: 600,
    color:      "var(--color-text-primary)",
    margin:     0,
  },
  sectionHint: {
    fontFamily: "var(--font-body)",
    fontSize:   12,
    color:      "var(--color-text-muted)",
    margin:     0,
    lineHeight: 1.5,
  },
  toggleRow: {
    display:        "flex",
    alignItems:     "center",
    justifyContent: "space-between",
    cursor:         "pointer",
  },
  toggleLabel: {
    fontFamily: "var(--font-body)",
    fontSize:   13,
    color:      "var(--color-text-secondary)",
  },
  inputWithHint: {
    display:       "flex",
    flexDirection: "column",
    gap:           4,
  },
  hintOk: {
    fontFamily: "var(--font-mono)",
    fontSize:   10,
    color:      "var(--color-text-muted)",
    letterSpacing: "0.06em",
  },
  secondaryBtn: {
    display:      "inline-flex",
    alignItems:   "center",
    gap:          6,
    alignSelf:    "flex-start",
    background:   "none",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "8px 18px",
    fontSize:     12,
    fontFamily:   "var(--font-body)",
    color:        "var(--color-text-secondary)",
    cursor:       "pointer",
    transition:   "border-color 150ms, opacity 150ms",
  },
  mappingsTable: {
    width:          "100%",
    borderCollapse: "collapse",
    fontSize:       12,
    fontFamily:     "var(--font-body)",
  },
  th: {
    textAlign:     "left",
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.12em",
    textTransform: "uppercase",
    color:         "var(--color-text-disabled)",
    padding:       "4px 8px",
    borderBottom:  "1px solid var(--color-border)",
  },
  td: {
    padding:      "6px 8px",
    color:        "var(--color-text-secondary)",
    borderBottom: "1px solid var(--color-border)",
    verticalAlign: "middle",
  },
  iconBtn: {
    background:   "none",
    border:       "none",
    cursor:       "pointer",
    color:        "var(--color-text-disabled)",
    padding:      "2px 4px",
    display:      "inline-flex",
    alignItems:   "center",
    transition:   "color 150ms",
  },
  addMappingRow: {
    display:    "flex",
    gap:        8,
    alignItems: "center",
    flexWrap:   "wrap",
  },
};


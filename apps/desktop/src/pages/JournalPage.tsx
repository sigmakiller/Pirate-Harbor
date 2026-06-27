/**
 * JournalPage — Chronological reading-first log.
 *
 * Design spec: "Reading-first layout. Chronological log of play sessions,
 * notes, screenshots and milestones."
 *
 * Layout:
 *   Left: narrow filter sidebar (All / by game)
 *   Right: editorial timeline feed — large body text, wide margins,
 *          entry type badges, timestamps, inline edit/delete
 *   Top:   compose button that expands an inline form above the feed
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { BookOpen, PenLine, Trophy, Gamepad2, X, Trash2, Check } from "lucide-react";
import ConfirmDialog from "@/components/ConfirmDialog";

import {
  getJournalEntries,
  createJournalEntry,
  updateJournalEntry,
  deleteJournalEntry,
  getAllGames,
  type JournalEntry,
  type EntryType,
} from "@/lib/api";
import type { Game } from "@/types";

// ─────────────────────────────────────────────────────────────────────────────

const ENTRY_TYPE_META: Record<EntryType, { label: string; Icon: React.FC<{ size?: number }> }> = {
  note:      { label: "Note",      Icon: PenLine  },
  milestone: { label: "Milestone", Icon: Trophy   },
  session:   { label: "Session",   Icon: Gamepad2 },
};

function formatDate(iso: string) {
  const d = new Date(iso);
  return d.toLocaleDateString("en-GB", {
    day:   "numeric",
    month: "long",
    year:  "numeric",
  });
}

function formatTime(iso: string) {
  return new Date(iso).toLocaleTimeString("en-GB", {
    hour:   "2-digit",
    minute: "2-digit",
  });
}

// ─────────────────────────────────────────────────────────────────────────────

export default function JournalPage() {
  const [entries,       setEntries]       = useState<JournalEntry[]>([]);
  const [games,         setGames]         = useState<Game[]>([]);
  const [loading,       setLoading]       = useState(true);
  const [filterGameId,  setFilterGameId]  = useState<string | null>(null);

  // Compose form
  const [composing,    setComposing]    = useState(false);
  const [compTitle,    setCompTitle]    = useState("");
  const [compBody,     setCompBody]     = useState("");
  const [compType,     setCompType]     = useState<EntryType>("note");
  const [compGameId,   setCompGameId]   = useState<string>("");
  const [saving,       setSaving]       = useState(false);
  const [compError,    setCompError]    = useState<string | null>(null);

  // Inline edit
  const [editingId,    setEditingId]    = useState<string | null>(null);
  const [editTitle,    setEditTitle]    = useState("");
  const [editBody,     setEditBody]     = useState("");
  const [editSaving,   setEditSaving]   = useState(false);

  // Delete confirmation
  const [pendingDeleteEntry, setPendingDeleteEntry] = useState<JournalEntry | null>(null);

  const bodyRef = useRef<HTMLTextAreaElement>(null);

  const load = useCallback(async () => {
    setLoading(true);
    const [es, gs] = await Promise.all([
      getJournalEntries(filterGameId, 200),
      getAllGames({}),
    ]);
    setEntries(es);
    setGames(gs);
    setLoading(false);
  }, [filterGameId]);

  useEffect(() => { load(); }, [load]);

  // Focus textarea when compose opens
  useEffect(() => {
    if (composing && bodyRef.current) bodyRef.current.focus();
  }, [composing]);

  // ── Handlers ───────────────────────────────────────────────────────────────

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!compBody.trim() && !compTitle.trim()) return;
    setSaving(true);
    setCompError(null);
    try {
      const entry = await createJournalEntry({
        game_id:    compGameId || null,
        title:      compTitle.trim() || null,
        body:       compBody.trim(),
        entry_type: compType,
      });
      setEntries(prev => [entry, ...prev]);
      setCompTitle("");
      setCompBody("");
      setCompGameId("");
      setCompType("note");
      setComposing(false);
    } catch (err) {
      setCompError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (entry: JournalEntry) => {
    setPendingDeleteEntry(entry);
  };

  const confirmDeleteEntry = async () => {
    if (!pendingDeleteEntry) return;
    await deleteJournalEntry(pendingDeleteEntry.id);
    setEntries(prev => prev.filter(e => e.id !== pendingDeleteEntry.id));
    setPendingDeleteEntry(null);
  };

  const startEdit = (entry: JournalEntry) => {
    setEditingId(entry.id);
    setEditTitle(entry.title ?? "");
    setEditBody(entry.body);
  };

  const handleSaveEdit = async (entry: JournalEntry) => {
    setEditSaving(true);
    try {
      const updated = await updateJournalEntry(entry.id, {
        title: editTitle.trim() || null,
        body:  editBody.trim(),
      });
      setEntries(prev => prev.map(e => e.id === updated.id ? updated : e));
      setEditingId(null);
    } catch (err) {
      console.error(err);
    } finally {
      setEditSaving(false);
    }
  };

  // ── Group entries by day ───────────────────────────────────────────────────

  type DayGroup = { day: string; entries: JournalEntry[] };
  const grouped: DayGroup[] = [];
  for (const entry of entries) {
    const day = formatDate(entry.created_at);
    const last = grouped[grouped.length - 1];
    if (last?.day === day) {
      last.entries.push(entry);
    } else {
      grouped.push({ day, entries: [entry] });
    }
  }

  // ── Render ─────────────────────────────────────────────────────────────────

  return (
    <div className="atlas-enter" style={styles.root}>

      {/* ── Left sidebar: filter ─────────────────────────────────────────────── */}
      <aside style={styles.sidebar}>
        <p style={styles.sidebarLabel}>Filter</p>
        <button
          type="button"
          onClick={() => setFilterGameId(null)}
          style={{
            ...styles.filterBtn,
            ...(filterGameId === null ? styles.filterBtnActive : {}),
          }}
          aria-pressed={filterGameId === null}
        >
          <BookOpen size={12} aria-hidden="true" />
          All Entries
        </button>
        {games.map(g => (
          <button
            key={g.id}
            type="button"
            onClick={() => setFilterGameId(g.id)}
            style={{
              ...styles.filterBtn,
              ...(filterGameId === g.id ? styles.filterBtnActive : {}),
            }}
            aria-pressed={filterGameId === g.id}
            title={g.title}
          >
            <span style={styles.filterDot} aria-hidden="true" />
            <span style={styles.filterGameTitle}>{g.title}</span>
          </button>
        ))}
      </aside>

      {/* ── Main column ──────────────────────────────────────────────────────── */}
      <main style={styles.main} role="main">

        {/* Page header */}
        <div style={styles.pageHeader}>
          <div>
            <h1 style={styles.pageTitle}>Journal</h1>
            <p style={styles.pageSubtitle}>
              {loading ? "Loading…"
                : filterGameId
                  ? `${entries.length} ${entries.length === 1 ? "entry" : "entries"} — ${games.find(g => g.id === filterGameId)?.title ?? ""}`
                  : `${entries.length} ${entries.length === 1 ? "entry" : "entries"} across all games`
              }
            </p>
          </div>
          <button
            id="compose-entry-btn"
            type="button"
            onClick={() => setComposing(v => !v)}
            style={{
              ...styles.composeBtn,
              ...(composing ? styles.composeBtnActive : {}),
            }}
            aria-expanded={composing}
            aria-label="Compose a new journal entry"
          >
            {composing ? <X size={13} /> : <PenLine size={13} />}
            {composing ? "Cancel" : "New Entry"}
          </button>
        </div>

        {/* ── Compose form ──────────────────────────────────────────────────── */}
        {composing && (
          <form onSubmit={handleCreate} style={styles.composeForm} aria-label="New journal entry">
            {/* Type selector */}
            <div style={styles.typeRow} role="group" aria-label="Entry type">
              {(["note", "milestone", "session"] as EntryType[]).map(t => {
                const { label, Icon } = ENTRY_TYPE_META[t];
                return (
                  <button
                    key={t}
                    type="button"
                    onClick={() => setCompType(t)}
                    style={{
                      ...styles.typeChip,
                      ...(compType === t ? styles.typeChipActive : {}),
                    }}
                    aria-pressed={compType === t}
                  >
                    <Icon size={11} />
                    {label}
                  </button>
                );
              })}

              {/* Game selector */}
              <select
                value={compGameId}
                onChange={e => setCompGameId(e.target.value)}
                style={styles.gameSelect}
                aria-label="Link to game"
              >
                <option value="">No game</option>
                {games.map(g => (
                  <option key={g.id} value={g.id}>{g.title}</option>
                ))}
              </select>
            </div>

            {/* Title */}
            <input
              type="text"
              value={compTitle}
              onChange={e => setCompTitle(e.target.value)}
              placeholder="Title (optional)"
              style={styles.titleInput}
              aria-label="Entry title"
            />

            {/* Body */}
            <textarea
              ref={bodyRef}
              id="journal-body-input"
              value={compBody}
              onChange={e => setCompBody(e.target.value)}
              placeholder="Write your entry…"
              style={styles.bodyTextarea}
              rows={6}
              aria-label="Entry body"
            />

            {compError && (
              <p style={styles.composeErr} role="alert">{compError}</p>
            )}

            <div style={styles.composeActions}>
              <button type="button" onClick={() => setComposing(false)} style={styles.cancelBtn}>
                Cancel
              </button>
              <button
                id="submit-entry-btn"
                type="submit"
                disabled={(!compBody.trim() && !compTitle.trim()) || saving}
                style={{
                  ...styles.submitBtn,
                  opacity: (!compBody.trim() && !compTitle.trim()) || saving ? 0.4 : 1,
                }}
              >
                {saving ? "Saving…" : "Add Entry"}
              </button>
            </div>
          </form>
        )}

        {/* ── Empty state ────────────────────────────────────────────────────── */}
        {!loading && entries.length === 0 && (
          <div style={styles.emptyState}>
            <BookOpen size={40} style={{ color: "var(--color-text-disabled)", marginBottom: 16 }} />
            <p style={styles.emptyTitle}>No entries yet</p>
            <p style={styles.emptyHint}>
              Start writing notes, marking milestones, or logging sessions.
            </p>
          </div>
        )}

        {/* ── Timeline feed ─────────────────────────────────────────────────── */}
        <div style={styles.feed} aria-label="Journal timeline">
          {grouped.map(({ day, entries: dayEntries }) => (
            <div key={day} style={styles.dayGroup}>
              {/* Day separator */}
              <div style={styles.daySeparator} role="heading" aria-level={2}>
                <span style={styles.dayLabel}>{day}</span>
                <span style={styles.daySepLine} aria-hidden="true" />
              </div>

              {/* Entries */}
              {dayEntries.map(entry => {
                const { label: typeLabel, Icon: TypeIcon } = ENTRY_TYPE_META[entry.entry_type];
                const isEditing = editingId === entry.id;

                return (
                  <article
                    key={entry.id}
                    style={styles.entryCard}
                    aria-label={entry.title ?? `${typeLabel} entry`}
                  >
                    {/* Entry meta row */}
                    <div style={styles.entryMeta}>
                      <span style={{
                        ...styles.typeBadge,
                        ...TYPE_BADGE_STYLES[entry.entry_type],
                      }}>
                        <TypeIcon size={10} />
                        {typeLabel}
                      </span>

                      {entry.game_title && (
                        <span style={styles.gameTag}>{entry.game_title}</span>
                      )}

                      <span style={styles.entryTime}>{formatTime(entry.created_at)}</span>

                      {/* Actions */}
                      <div style={styles.entryActions}>
                        <button
                          type="button"
                          onClick={() => isEditing ? setEditingId(null) : startEdit(entry)}
                          style={styles.entryActionBtn}
                          aria-label={isEditing ? "Cancel edit" : "Edit entry"}
                        >
                          {isEditing ? <X size={11} /> : <PenLine size={11} />}
                        </button>
                        {isEditing && (
                          <button
                            type="button"
                            onClick={() => handleSaveEdit(entry)}
                            disabled={editSaving}
                            style={{ ...styles.entryActionBtn, color: "var(--color-text-secondary)" }}
                            aria-label="Save edits"
                          >
                            <Check size={11} />
                          </button>
                        )}
                        <button
                          type="button"
                          onClick={() => handleDelete(entry)}
                          style={styles.entryActionBtn}
                          aria-label="Delete entry"
                        >
                          <Trash2 size={11} />
                        </button>
                      </div>
                    </div>

                    {/* Title */}
                    {isEditing ? (
                      <input
                        type="text"
                        value={editTitle}
                        onChange={e => setEditTitle(e.target.value)}
                        style={styles.editTitleInput}
                        placeholder="Title (optional)"
                        aria-label="Edit entry title"
                        autoFocus
                      />
                    ) : (
                      entry.title && (
                        <h3 style={styles.entryTitle}>{entry.title}</h3>
                      )
                    )}

                    {/* Body */}
                    {isEditing ? (
                      <textarea
                        value={editBody}
                        onChange={e => setEditBody(e.target.value)}
                        style={styles.editBodyTextarea}
                        rows={5}
                        aria-label="Edit entry body"
                      />
                    ) : (
                      <p style={styles.entryBody}>{entry.body}</p>
                    )}
                  </article>
                );
              })}
            </div>
          ))}
        </div>
      </main>

      {/* Confirm delete entry dialog */}
      <ConfirmDialog
        open={pendingDeleteEntry !== null}
        title="Delete entry"
        message="Delete this journal entry permanently? This cannot be undone."
        confirmLabel="Delete"
        dangerous
        onConfirm={confirmDeleteEntry}
        onCancel={() => setPendingDeleteEntry(null)}
      />
    </div>
  );
}

// ── Entry type badge colour overrides ─────────────────────────────────────────

const TYPE_BADGE_STYLES: Record<EntryType, React.CSSProperties> = {
  note:      {},
  milestone: { borderColor: "var(--color-text-secondary)", color: "var(--color-text-secondary)" },
  session:   { borderColor: "var(--color-border)", color: "var(--color-text-muted)" },
};

// ── Styles ────────────────────────────────────────────────────────────────────

const styles = {
  root: {
    display:  "flex",
    height:   "100%",
    overflow: "hidden",
  },

  // ── Sidebar ───────────────────────────────────────────────────────────────
  sidebar: {
    width:        200,
    flexShrink:   0,
    borderRight:  "1px solid var(--color-border)",
    padding:      "40px 20px",
    overflowY:    "auto" as const,
    display:      "flex",
    flexDirection: "column" as const,
    gap:          4,
  },
  sidebarLabel: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.12em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    margin:        "0 0 8px",
  },
  filterBtn: {
    display:      "flex",
    alignItems:   "center",
    gap:          8,
    background:   "none",
    border:       "none",
    borderRadius: 1,
    padding:      "7px 10px",
    fontSize:     12,
    fontFamily:   "var(--font-body)",
    color:        "var(--color-text-muted)",
    cursor:       "pointer",
    textAlign:    "left" as const,
    width:        "100%",
    transition:   "background 150ms, color 150ms",
    overflow:     "hidden",
  },
  filterBtnActive: {
    background: "var(--color-elevated)",
    color:      "var(--color-text-primary)",
  },
  filterDot: {
    width:        5,
    height:       5,
    borderRadius: "50%",
    background:   "var(--color-text-disabled)",
    flexShrink:   0,
    display:      "inline-block",
  },
  filterGameTitle: {
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
    flex:         1,
  },

  // ── Main ──────────────────────────────────────────────────────────────────
  main: {
    flex:      1,
    display:   "flex",
    flexDirection: "column" as const,
    overflowY: "auto" as const,
    padding:   "40px 80px",
    maxWidth:  860,
  },
  pageHeader: {
    display:        "flex",
    justifyContent: "space-between",
    alignItems:     "flex-start",
    marginBottom:   40,
    gap:            16,
  },
  pageTitle: {
    fontFamily:    "var(--font-display)",
    fontSize:      "clamp(40px, 4vw, 72px)",
    fontWeight:    700,
    letterSpacing: "-0.03em",
    lineHeight:    1.0,
    color:         "var(--color-text-primary)",
    margin:        0,
    marginBottom:  6,
  },
  pageSubtitle: {
    fontFamily: "var(--font-body)",
    fontSize:   13,
    color:      "var(--color-text-muted)",
    margin:     0,
  },
  composeBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           6,
    flexShrink:    0,
    background:    "var(--color-text-primary)",
    border:        "1px solid transparent",
    borderRadius:  1,
    padding:       "9px 18px",
    fontSize:      12,
    fontFamily:    "var(--font-mono)",
    fontWeight:    500,
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    color:         "var(--color-base)",
    cursor:        "pointer",
    transition:    "background 150ms, color 150ms, border-color 150ms",
  },
  composeBtnActive: {
    background:  "none",
    color:       "var(--color-text-muted)",
    borderColor: "var(--color-border)",
  },

  // ── Compose form ──────────────────────────────────────────────────────────
  composeForm: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           12,
    marginBottom:  40,
    padding:       "24px",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    background:    "var(--color-surface)",
  },
  typeRow: {
    display:    "flex",
    alignItems: "center",
    gap:        8,
    flexWrap:   "wrap" as const,
  },
  typeChip: {
    display:       "flex",
    alignItems:    "center",
    gap:           5,
    background:    "none",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "5px 12px",
    fontSize:      11,
    fontFamily:    "var(--font-mono)",
    letterSpacing: "0.06em",
    color:         "var(--color-text-muted)",
    cursor:        "pointer",
    transition:    "border-color 150ms, color 150ms",
  },
  typeChipActive: {
    borderColor: "var(--color-text-secondary)",
    color:       "var(--color-text-primary)",
  },
  gameSelect: {
    background:   "var(--color-elevated)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "5px 10px",
    fontSize:     11,
    fontFamily:   "var(--font-mono)",
    color:        "var(--color-text-muted)",
    cursor:       "pointer",
    outline:      "none",
    marginLeft:   "auto",
  },
  titleInput: {
    background:   "transparent",
    border:       "none",
    borderBottom: "1px solid var(--color-border)",
    padding:      "6px 0",
    fontSize:     18,
    fontFamily:   "var(--font-display)",
    fontWeight:   700,
    letterSpacing: "-0.02em",
    color:        "var(--color-text-primary)",
    outline:      "none",
    width:        "100%",
  },
  bodyTextarea: {
    background:  "transparent",
    border:      "none",
    padding:     "8px 0",
    fontSize:    15,
    fontFamily:  "var(--font-body)",
    lineHeight:  1.7,
    color:       "var(--color-text-primary)",
    outline:     "none",
    resize:      "vertical" as const,
    width:       "100%",
  },
  composeErr: {
    fontFamily: "var(--font-body)",
    fontSize:   12,
    color:      "var(--color-text-muted)",
    margin:     0,
  },
  composeActions: {
    display:        "flex",
    justifyContent: "flex-end" as const,
    gap:            8,
  },
  cancelBtn: {
    background:   "none",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "7px 16px",
    fontSize:     12,
    fontFamily:   "var(--font-body)",
    color:        "var(--color-text-muted)",
    cursor:       "pointer",
  },
  submitBtn: {
    background:   "var(--color-text-primary)",
    border:       "none",
    borderRadius: 1,
    padding:      "7px 18px",
    fontSize:     12,
    fontFamily:   "var(--font-body)",
    fontWeight:   600,
    color:        "var(--color-base)",
    cursor:       "pointer",
    transition:   "opacity 150ms",
  },

  // ── Empty state ───────────────────────────────────────────────────────────
  emptyState: {
    display:        "flex",
    flexDirection:  "column" as const,
    alignItems:     "center",
    justifyContent: "center",
    padding:        "80px 0",
    textAlign:      "center" as const,
  },
  emptyTitle: {
    fontFamily:   "var(--font-display)",
    fontSize:     24,
    fontWeight:   700,
    color:        "var(--color-text-secondary)",
    margin:       0,
    marginBottom: 8,
  },
  emptyHint: {
    fontFamily: "var(--font-body)",
    fontSize:   14,
    color:      "var(--color-text-disabled)",
    maxWidth:   320,
    lineHeight: 1.6,
    margin:     0,
  },

  // ── Feed ──────────────────────────────────────────────────────────────────
  feed: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           0,
  },
  dayGroup: {
    marginBottom: 40,
  },
  daySeparator: {
    display:    "flex",
    alignItems: "center",
    gap:        16,
    marginBottom: 20,
  },
  dayLabel: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.10em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    flexShrink:    0,
  },
  daySepLine: {
    flex:       1,
    height:     1,
    background: "var(--color-border)",
    display:    "block",
  },
  entryCard: {
    paddingBottom: 28,
    marginBottom:  28,
    borderBottom:  "1px solid var(--color-border-sub)",
  },
  entryMeta: {
    display:    "flex",
    alignItems: "center",
    gap:        10,
    marginBottom: 12,
  },
  typeBadge: {
    display:       "flex",
    alignItems:    "center",
    gap:           4,
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.10em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "3px 8px",
    flexShrink:    0,
  },
  gameTag: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.06em",
    color:         "var(--color-text-muted)",
    background:    "var(--color-elevated)",
    borderRadius:  1,
    padding:       "3px 8px",
    overflow:      "hidden",
    textOverflow:  "ellipsis",
    whiteSpace:    "nowrap" as const,
    maxWidth:      160,
  },
  entryTime: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.06em",
    color:         "var(--color-text-disabled)",
    marginLeft:    "auto",
  },
  entryActions: {
    display: "flex",
    gap:     4,
  },
  entryActionBtn: {
    background:  "none",
    border:      "none",
    color:       "var(--color-text-disabled)",
    cursor:      "pointer",
    padding:     "3px 5px",
    display:     "flex",
    borderRadius: 1,
    transition:  "color 150ms",
  },

  // Reading body
  entryTitle: {
    fontFamily:    "var(--font-display)",
    fontSize:      "clamp(20px, 2.5vw, 28px)",
    fontWeight:    700,
    letterSpacing: "-0.02em",
    color:         "var(--color-text-primary)",
    margin:        "0 0 10px",
  },
  entryBody: {
    fontFamily:  "var(--font-body)",
    fontSize:    15,
    lineHeight:  1.75,
    color:       "var(--color-text-secondary)",
    margin:      0,
    whiteSpace:  "pre-wrap" as const,
  },

  // Inline edit
  editTitleInput: {
    background:   "transparent",
    border:       "none",
    borderBottom: "1px solid var(--color-border)",
    padding:      "4px 0",
    fontSize:     22,
    fontFamily:   "var(--font-display)",
    fontWeight:   700,
    color:        "var(--color-text-primary)",
    outline:      "none",
    width:        "100%",
    marginBottom: 8,
  },
  editBodyTextarea: {
    background:  "transparent",
    border:      "1px solid var(--color-border)",
    borderRadius: 1,
    padding:     "8px 10px",
    fontSize:    14,
    fontFamily:  "var(--font-body)",
    lineHeight:  1.7,
    color:       "var(--color-text-primary)",
    outline:     "none",
    resize:      "vertical" as const,
    width:       "100%",
    boxSizing:   "border-box" as const,
  },
} satisfies Record<string, React.CSSProperties>;

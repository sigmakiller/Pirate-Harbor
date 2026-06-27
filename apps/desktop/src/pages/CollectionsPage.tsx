/**
 * CollectionsPage — Curated galleries.
 *
 * Design spec: Design/Pages/collections.md
 * "Large covers. Editorial layouts. Museum-like presentation."
 *
 * Layout:
 *   - Top: editorial header + "New Collection" button
 *   - Grid: collection cards (large cover mosaic, name, game count)
 *   - Detail panel: slides in when a collection is selected
 */

import { useCallback, useEffect, useState } from "react";
import { useNavigate }                        from "react-router-dom";
import { Plus, X, Trash2, FolderOpen } from "lucide-react";
import ConfirmDialog from "@/components/ConfirmDialog";
import { FilePickerButton } from "@/components/FilePickerButton";

import {
  getCollections,
  createCollection,
  deleteCollection,
  getAllGames,
  addGameToCollection,
  removeGameFromCollection,
  type Collection,
} from "@/lib/api";
import type { Game } from "@/types";
import { convertFileSrc } from "@tauri-apps/api/core";

// ─────────────────────────────────────────────────────────────────────────────

export default function CollectionsPage() {
  const navigate = useNavigate();

  const [collections, setCollections] = useState<Collection[]>([]);
  const [games,       setGames]       = useState<Game[]>([]);
  const [loading,     setLoading]     = useState(true);

  // Selected collection for detail panel
  const [selected, setSelected] = useState<Collection | null>(null);

  // Create-collection form
  const [creating,       setCreating]       = useState(false);
  const [newName,        setNewName]        = useState("");
  const [newDesc,        setNewDesc]        = useState("");
  const [newCoverMode,   setNewCoverMode]   = useState<'auto' | 'custom'>('auto');
  const [newCoverPath,   setNewCoverPath]   = useState("");
  const [saving,         setSaving]         = useState(false);
  const [createErr,      setCreateErr]      = useState<string | null>(null);

  // Delete confirmation
  const [pendingDelete, setPendingDelete] = useState<Collection | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    const [cols, gs] = await Promise.all([getCollections(), getAllGames({})]);
    setCollections(cols);
    setGames(gs);
    setLoading(false);
  }, []);

  useEffect(() => { load(); }, [load]);

  // Keep selected collection in sync after any mutation
  const refresh = useCallback(async () => {
    const cols = await getCollections();
    setCollections(cols);
    if (selected) {
      const updated = cols.find(c => c.id === selected.id) ?? null;
      setSelected(updated);
    }
  }, [selected]);

  // ── Handlers ───────────────────────────────────────────────────────────────

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim()) return;
    setSaving(true);
    setCreateErr(null);
    try {
      const col = await createCollection({
        name:        newName.trim(),
        description: newDesc.trim() || null,
        cover_mode:  newCoverMode,
        cover_path:  newCoverMode === 'custom' ? (newCoverPath.trim() || null) : null,
      });
      setCollections(prev => [col, ...prev]);
      setNewName("");
      setNewDesc("");
      setNewCoverMode('auto');
      setNewCoverPath("");
      setCreating(false);
    } catch (err) {
      setCreateErr(String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (col: Collection) => {
    setPendingDelete(col);
  };

  const confirmDelete = async () => {
    if (!pendingDelete) return;
    await deleteCollection(pendingDelete.id);
    if (selected?.id === pendingDelete.id) setSelected(null);
    setPendingDelete(null);
    await refresh();
  };

  const handleToggleGame = async (gameId: string) => {
    if (!selected) return;
    const inCollection = selected.game_ids.includes(gameId);
    const updated = inCollection
      ? await removeGameFromCollection(selected.id, gameId)
      : await addGameToCollection(selected.id, gameId);
    setSelected(updated);
    setCollections(prev => prev.map(c => c.id === updated.id ? updated : c));
  };

  // ── Helpers ────────────────────────────────────────────────────────────────

  const gameMap = Object.fromEntries(games.map(g => [g.id, g]));

  const coverSrc = (g: Game | undefined) => {
    if (!g?.cover_path) return null;
    try { return convertFileSrc(g.cover_path); } catch { return null; }
  };

  // Returns up to 4 games in a collection for the mosaic
  const mosaicGames = (col: Collection): (Game | undefined)[] =>
    col.game_ids.slice(0, 4).map(id => gameMap[id]);

  // ── Render ─────────────────────────────────────────────────────────────────

  return (
    <div className="atlas-enter" style={styles.root}>
      {/* ── Main column ──────────────────────────────────────────────────────── */}
      <div style={{ ...styles.main, flex: selected ? "0 0 340px" : "1" }}>

        {/* Header */}
        <div style={styles.header}>
          <div>
            <h1 style={styles.pageTitle}>Collections</h1>
            <p style={styles.pageSubtitle}>
              {loading ? "Loading…" : `${collections.length} curated ${collections.length === 1 ? "gallery" : "galleries"}`}
            </p>
          </div>
          <button
            id="new-collection-btn"
            type="button"
            onClick={() => setCreating(true)}
            style={styles.newBtn}
            aria-label="Create a new collection"
          >
            <Plus size={13} />
            New Collection
          </button>
        </div>

        {/* Create form */}
        {creating && (
          <form onSubmit={handleCreate} style={styles.createForm} aria-label="Create collection">
            <div style={styles.createRow}>
              <input
                id="new-col-name"
                type="text"
                value={newName}
                onChange={e => setNewName(e.target.value)}
                placeholder="Collection name…"
                style={styles.createInput}
                autoFocus
                required
                aria-label="Collection name"
              />
              <input
                id="new-col-desc"
                type="text"
                value={newDesc}
                onChange={e => setNewDesc(e.target.value)}
                placeholder="Description (optional)"
                style={styles.createInput}
                aria-label="Collection description"
              />
            </div>

            {/* Cover mode toggle */}
            <div style={styles.coverModeRow} role="group" aria-label="Cover mode">
              <button
                type="button"
                onClick={() => setNewCoverMode('auto')}
                style={{
                  ...styles.modeChip,
                  ...(newCoverMode === 'auto' ? styles.modeChipActive : {}),
                }}
                aria-pressed={newCoverMode === 'auto'}
              >
                Auto Mosaic
              </button>
              <button
                type="button"
                onClick={() => setNewCoverMode('custom')}
                style={{
                  ...styles.modeChip,
                  ...(newCoverMode === 'custom' ? styles.modeChipActive : {}),
                }}
                aria-pressed={newCoverMode === 'custom'}
              >
                Custom Image
              </button>
            </div>

            {/* Custom cover picker */}
            {newCoverMode === 'custom' && (
              <FilePickerButton
                id="new-col-cover-picker"
                value={newCoverPath}
                onChange={(p: string) => setNewCoverPath(p)}
                filters={[{ name: "Image", extensions: ["jpg", "jpeg", "png", "webp"] }]}
                placeholder="Browse for cover image…"
              />
            )}

            {createErr && <p style={styles.createErr} role="alert">{createErr}</p>}
            <div style={styles.createActions}>
              <button type="button" onClick={() => { setCreating(false); setCreateErr(null); }} style={styles.cancelBtn}>
                Cancel
              </button>
              <button
                id="create-collection-submit"
                type="submit"
                disabled={!newName.trim() || saving}
                style={{ ...styles.submitBtn, opacity: !newName.trim() || saving ? 0.4 : 1 }}
              >
                {saving ? "Creating…" : "Create"}
              </button>
            </div>
          </form>
        )}

        {/* Empty state */}
        {!loading && collections.length === 0 && (
          <div style={styles.emptyState}>
            <FolderOpen size={40} style={{ color: "var(--color-text-disabled)", marginBottom: 16 }} />
            <p style={styles.emptyTitle}>No collections yet</p>
            <p style={styles.emptyHint}>
              Create a collection to curate your library into themed galleries.
            </p>
          </div>
        )}

        {/* Collection grid */}
        <div style={styles.grid} role="list" aria-label="Collections">
          {collections.map(col => {
            const mosaic = mosaicGames(col);
            const isActive = selected?.id === col.id;
            return (
              <div
                key={col.id}
                role="listitem"
                onClick={() => setSelected(isActive ? null : col)}
                style={{
                  ...styles.card,
                  ...(isActive ? styles.cardActive : {}),
                }}
                aria-pressed={isActive}
                aria-label={`${col.name} — ${col.game_count} games`}
                tabIndex={0}
                onKeyDown={e => e.key === "Enter" && setSelected(isActive ? null : col)}
              >
                {/* Cover — auto mosaic or custom image */}
                <div style={styles.mosaic} aria-hidden="true">
                  {col.cover_mode === 'custom' && col.cover_path ? (
                    <img
                      src={convertFileSrc(col.cover_path)}
                      alt=""
                      style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                    />
                  ) : (
                    [0, 1, 2, 3].map(i => {
                      const g = mosaic[i];
                      const src = coverSrc(g);
                      return (
                        <div key={i} style={styles.mosaicCell}>
                          {src
                            ? <img src={src} alt="" style={styles.mosaicImg} />
                            : <div style={styles.mosaicPlaceholder} />
                          }
                        </div>
                      );
                    })
                  )}
                  {/* Dark scrim for legibility */}
                  <div style={styles.mosaicScrim} />
                </div>

                {/* Info */}
                <div style={styles.cardInfo}>
                  <span style={styles.cardName}>{col.name}</span>
                  <span style={styles.cardCount}>
                    {col.game_count} {col.game_count === 1 ? "game" : "games"}
                  </span>
                </div>

                {/* Delete */}
                <button
                  type="button"
                  onClick={e => { e.stopPropagation(); handleDelete(col); }}
                  style={styles.cardDeleteBtn}
                  aria-label={`Delete ${col.name}`}
                  tabIndex={-1}
                >
                  <Trash2 size={11} />
                </button>
              </div>
            );
          })}
        </div>
      </div>

      {/* ── Detail panel ─────────────────────────────────────────────────────── */}
      {selected && (
        <aside style={styles.panel} aria-label={`${selected.name} detail`}>
          <div style={styles.panelHeader}>
            <div>
              <h2 style={styles.panelTitle}>{selected.name}</h2>
              {selected.description && (
                <p style={styles.panelDesc}>{selected.description}</p>
              )}
              <p style={styles.panelMeta}>
                {selected.game_count} {selected.game_count === 1 ? "game" : "games"}
              </p>
            </div>
            <button
              type="button"
              onClick={() => setSelected(null)}
              style={styles.panelCloseBtn}
              aria-label="Close collection detail"
            >
              <X size={14} />
            </button>
          </div>

          {/* Games in collection */}
          <div style={styles.panelSection}>
            <p style={styles.panelSectionLabel}>In Collection</p>
            <div style={styles.gameList} role="list">
              {selected.game_ids.length === 0 && (
                <p style={styles.panelEmpty}>No games yet. Add from the library below.</p>
              )}
              {selected.game_ids.map(gid => {
                const g = gameMap[gid];
                if (!g) return null;
                const src = coverSrc(g);
                return (
                  <div key={gid} style={styles.gameRow} role="listitem">
                    <div style={styles.gameCoverThumb}>
                      {src
                        ? <img src={src} alt="" style={styles.gameCoverImg} />
                        : <div style={styles.gameCoverPlaceholder} />
                      }
                    </div>
                    <span
                      style={styles.gameTitle}
                      onClick={() => navigate(`/library/${gid}`)}
                      title="Open game detail"
                    >
                      {g.title}
                    </span>
                    <button
                      type="button"
                      onClick={() => handleToggleGame(gid)}
                      style={styles.gameRemoveBtn}
                      aria-label={`Remove ${g.title} from collection`}
                    >
                      <X size={11} />
                    </button>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Library — games not in collection */}
          {games.filter(g => !selected.game_ids.includes(g.id)).length > 0 && (
            <div style={styles.panelSection}>
              <p style={styles.panelSectionLabel}>Add from Library</p>
              <div style={styles.gameList} role="list">
                {games
                  .filter(g => !selected.game_ids.includes(g.id))
                  .map(g => {
                    const src = coverSrc(g);
                    return (
                      <div key={g.id} style={{ ...styles.gameRow, opacity: 0.7 }} role="listitem">
                        <div style={styles.gameCoverThumb}>
                          {src
                            ? <img src={src} alt="" style={styles.gameCoverImg} />
                            : <div style={styles.gameCoverPlaceholder} />
                          }
                        </div>
                        <span style={styles.gameTitle}>{g.title}</span>
                        <button
                          type="button"
                          onClick={() => handleToggleGame(g.id)}
                          style={styles.gameAddBtn}
                          aria-label={`Add ${g.title} to collection`}
                        >
                          <Plus size={11} />
                        </button>
                      </div>
                    );
                  })}
              </div>
            </div>
          )}
        </aside>
      )}

      {/* Confirm delete dialog */}
      <ConfirmDialog
        open={pendingDelete !== null}
        title="Delete collection"
        message={`Delete "${pendingDelete?.name}"? Games inside will not be removed.`}
        confirmLabel="Delete"
        dangerous
        onConfirm={confirmDelete}
        onCancel={() => setPendingDelete(null)}
      />
    </div>
  );
}

// ── Styles ────────────────────────────────────────────────────────────────────

const styles = {
  root: {
    display:  "flex",
    height:   "100%",
    overflow: "hidden",
  },
  main: {
    display:       "flex",
    flexDirection: "column" as const,
    padding:       "40px 56px",
    overflowY:     "auto" as const,
    transition:    "flex 250ms ease",
    minWidth:      0,
  },
  header: {
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
    fontFamily:   "var(--font-body)",
    fontSize:     13,
    color:        "var(--color-text-muted)",
    margin:       0,
  },
  newBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           6,
    flexShrink:    0,
    background:    "var(--color-text-primary)",
    border:        "none",
    borderRadius:  1,
    padding:       "9px 18px",
    fontSize:      12,
    fontFamily:    "var(--font-mono)",
    fontWeight:    500,
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    color:         "var(--color-base)",
    cursor:        "pointer",
    transition:    "opacity 150ms",
  },
  createForm: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           10,
    padding:       "20px",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    background:    "var(--color-surface)",
    marginBottom:  32,
  },
  createRow: {
    display: "flex",
    gap:     10,
  },
  createInput: {
    flex:         1,
    background:   "var(--color-elevated)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "8px 12px",
    fontSize:     13,
    fontFamily:   "var(--font-body)",
    color:        "var(--color-text-primary)",
    outline:      "none",
  },
  createErr: {
    fontFamily: "var(--font-body)",
    fontSize:   12,
    color:      "var(--color-text-muted)",
    margin:     0,
  },
  createActions: {
    display: "flex",
    gap:     8,
    justifyContent: "flex-end" as const,
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
    background:    "var(--color-text-primary)",
    border:        "none",
    borderRadius:  1,
    padding:       "7px 18px",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    fontWeight:    600,
    color:         "var(--color-base)",
    cursor:        "pointer",
    transition:    "opacity 150ms",
  },
  coverModeRow: {
    display: "flex",
    gap:     8,
  },
  modeChip: {
    display:       "inline-flex",
    alignItems:    "center",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "5px 14px",
    fontSize:      11,
    fontFamily:    "var(--font-mono)",
    letterSpacing: "0.06em",
    color:         "var(--color-text-muted)",
    background:    "none",
    cursor:        "pointer",
    transition:    "border-color 150ms, color 150ms",
  },
  modeChipActive: {
    borderColor: "var(--color-text-secondary)",
    color:       "var(--color-text-primary)",
  },
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
  grid: {
    display:             "grid",
    gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))",
    gap:                 16,
  },
  card: {
    position:      "relative" as const,
    display:       "flex",
    flexDirection: "column" as const,
    borderRadius:  1,
    overflow:      "hidden",
    border:        "1px solid var(--color-border)",
    cursor:        "pointer",
    transition:    "border-color 150ms, transform 150ms",
  },
  cardActive: {
    borderColor: "var(--color-text-secondary)",
  },
  mosaic: {
    position: "relative" as const,
    display:  "grid",
    gridTemplateColumns: "1fr 1fr",
    gridTemplateRows:    "1fr 1fr",
    height:   200,
    background: "var(--color-elevated)",
  },
  mosaicCell: {
    overflow: "hidden",
  },
  mosaicImg: {
    width:      "100%",
    height:     "100%",
    objectFit:  "cover" as const,
    display:    "block",
  },
  mosaicPlaceholder: {
    width:      "100%",
    height:     "100%",
    background: "var(--color-elevated)",
  },
  mosaicScrim: {
    position:   "absolute" as const,
    inset:      0,
    background: "linear-gradient(to top, rgba(5,5,5,0.7) 0%, transparent 60%)",
  },
  cardInfo: {
    padding:    "12px 14px 10px",
    display:    "flex",
    flexDirection: "column" as const,
    gap:        3,
    background: "var(--color-surface)",
  },
  cardName: {
    fontFamily:   "var(--font-body)",
    fontSize:     13,
    fontWeight:   600,
    color:        "var(--color-text-primary)",
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
  },
  cardCount: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.08em",
    color:         "var(--color-text-disabled)",
  },
  cardDeleteBtn: {
    position:    "absolute" as const,
    top:         8,
    right:       8,
    background:  "rgba(5,5,5,0.6)",
    border:      "1px solid var(--color-border)",
    borderRadius: 1,
    color:       "var(--color-text-muted)",
    cursor:      "pointer",
    padding:     "4px 6px",
    display:     "flex",
    opacity:     0,
    transition:  "opacity 150ms",
  },

  // ── Detail panel ───────────────────────────────────────────────────────────
  panel: {
    width:          "360px",
    flexShrink:     0,
    borderLeft:     "1px solid var(--color-border)",
    display:        "flex",
    flexDirection:  "column" as const,
    overflowY:      "auto" as const,
    background:     "var(--color-surface)",
  },
  panelHeader: {
    display:        "flex",
    justifyContent: "space-between",
    alignItems:     "flex-start",
    padding:        "32px 24px 20px",
    borderBottom:   "1px solid var(--color-border)",
    gap:            12,
    position:       "sticky" as const,
    top:            0,
    background:     "var(--color-surface)",
    zIndex:         1,
  },
  panelTitle: {
    fontFamily:    "var(--font-display)",
    fontSize:      28,
    fontWeight:    700,
    letterSpacing: "-0.02em",
    color:         "var(--color-text-primary)",
    margin:        0,
    marginBottom:  4,
  },
  panelDesc: {
    fontFamily:   "var(--font-body)",
    fontSize:     13,
    color:        "var(--color-text-muted)",
    margin:       0,
    marginBottom: 4,
    lineHeight:   1.5,
  },
  panelMeta: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.10em",
    color:         "var(--color-text-disabled)",
    margin:        0,
  },
  panelCloseBtn: {
    flexShrink:  0,
    background:  "none",
    border:      "1px solid var(--color-border)",
    borderRadius: 1,
    color:       "var(--color-text-muted)",
    cursor:      "pointer",
    padding:     "6px",
    display:     "flex",
    transition:  "color 150ms",
  },
  panelSection: {
    padding: "16px 24px",
    borderBottom: "1px solid var(--color-border-sub)",
  },
  panelSectionLabel: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.12em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    margin:        "0 0 10px",
  },
  panelEmpty: {
    fontFamily:  "var(--font-body)",
    fontSize:    13,
    color:       "var(--color-text-disabled)",
    margin:      0,
    lineHeight:  1.5,
  },
  gameList: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           6,
  },
  gameRow: {
    display:     "flex",
    alignItems:  "center",
    gap:         10,
    padding:     "6px 0",
    borderBottom: "1px solid var(--color-border-sub)",
  },
  gameCoverThumb: {
    width:        32,
    height:       44,
    flexShrink:   0,
    borderRadius: 1,
    overflow:     "hidden",
    background:   "var(--color-elevated)",
  },
  gameCoverImg: {
    width:     "100%",
    height:    "100%",
    objectFit: "cover" as const,
    display:   "block",
  },
  gameCoverPlaceholder: {
    width:      "100%",
    height:     "100%",
    background: "var(--color-elevated)",
  },
  gameTitle: {
    flex:         1,
    fontFamily:   "var(--font-body)",
    fontSize:     12,
    color:        "var(--color-text-primary)",
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
    cursor:       "pointer",
    minWidth:     0,
  },
  gameRemoveBtn: {
    flexShrink:  0,
    background:  "none",
    border:      "none",
    color:       "var(--color-text-disabled)",
    cursor:      "pointer",
    padding:     4,
    display:     "flex",
    transition:  "color 150ms",
  },
  gameAddBtn: {
    flexShrink:  0,
    background:  "none",
    border:      "1px solid var(--color-border)",
    borderRadius: 1,
    color:       "var(--color-text-disabled)",
    cursor:      "pointer",
    padding:     "3px 6px",
    display:     "flex",
    transition:  "border-color 150ms, color 150ms",
  },
} satisfies Record<string, React.CSSProperties>;

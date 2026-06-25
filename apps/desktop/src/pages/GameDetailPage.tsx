/**
 * GameDetailPage — full game view with ambient immersion layer.
 *
 * Layers (bottom to top):
 *   1. Pure black background (#050505) — always present
 *   2. AmbientLayer — desaturated, blurred color extracted from cover art
 *   3. Monochrome UI — metadata, stats, launch button
 */

import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, Play, Star, Pencil, Trash2, FolderPlus, Check } from "lucide-react";

import { AmbientLayer }    from "@/components/AmbientLayer";
import ConfirmDialog        from "@/components/ConfirmDialog";
import { getGame, getSessions, launchGame, toggleFavorite, deleteGame,
         getCollections, addGameToCollection, removeGameFromCollection,
         type Collection } from "@/lib/api";
import { formatPlaytime, formatRelativeDate }   from "@/lib/utils";
import { useToastStore }    from "@/stores/useToastStore";
import type { Game, Session } from "@/types";

export default function GameDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const [game, setGame]         = useState<Game | null>(null);
  const [sessions, setSessions]  = useState<Session[]>([]);
  const [loading, setLoading]    = useState(true);
  const [launching, setLaunching] = useState(false);
  const [error, setError]        = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete]  = useState(false);
  const [collections,   setCollections]    = useState<Collection[]>([]);
  const [memberIds,     setMemberIds]      = useState<Set<string>>(new Set());
  const [collMenuOpen,  setCollMenuOpen]   = useState(false);
  const [collLoading,   setCollLoading]    = useState(false);
  const { addToast } = useToastStore();

  useEffect(() => {
    if (!id) return;

    setLoading(true);
    Promise.all([getGame(id), getSessions(id), getCollections()])
      .then(([g, s, cols]) => {
        setGame(g);
        setSessions(s);
        setCollections(cols);
        setMemberIds(new Set(cols.filter(c => c.game_ids.includes(id)).map(c => c.id)));
      })
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, [id]);

  const handleLaunch = async () => {
    if (!game) return;
    setLaunching(true);
    try {
      await launchGame(game.id);
    } catch (e) {
      setError(String(e));
    } finally {
      setLaunching(false);
    }
  };

  const handleFavorite = async () => {
    if (!game) return;
    const updated = await toggleFavorite(game.id);
    setGame(updated);
  };

  const handleDelete = async () => {
    if (!game) return;
    try {
      await deleteGame(game.id);
      addToast({ message: `"${game.title}" removed from library`, type: "success" });
      navigate("/library");
    } catch (err) {
      addToast({ message: "Failed to delete game", type: "error" });
    }
  };

  const handleToggleCollection = async (col: Collection) => {
    if (!game) return;
    setCollLoading(true);
    try {
      const isMember = memberIds.has(col.id);
      if (isMember) {
        await removeGameFromCollection(col.id, game.id);
        setMemberIds(prev => { const s = new Set(prev); s.delete(col.id); return s; });
        addToast({ message: `Removed from "${col.name}"`, type: "info" });
      } else {
        await addGameToCollection(col.id, game.id);
        setMemberIds(prev => new Set([...prev, col.id]));
        addToast({ message: `Added to "${col.name}"`, type: "success" });
      }
    } catch {
      addToast({ message: "Collection update failed", type: "error" });
    } finally {
      setCollLoading(false);
    }
  };

  // ── Loading ───────────────────────────────────────────────────────────────
  if (loading) {
    return (
      <>
        <AmbientLayer coverPath={null} />
        <div style={styles.page}>
          <p style={{ color: "var(--color-text-muted)", fontSize: 14 }}>
            Loading…
          </p>
        </div>
      </>
    );
  }

  // ── Error / Not found ─────────────────────────────────────────────────────
  if (error || !game) {
    return (
      <>
        <AmbientLayer coverPath={null} />
        <div style={styles.page}>
          <p style={{ color: "var(--color-text-muted)", fontSize: 14 }}>
            {error ?? "Game not found."}
          </p>
          <button onClick={() => navigate("/library")} style={styles.backBtn}>
            ← Back to library
          </button>
        </div>
      </>
    );
  }

  // ── Detail view ───────────────────────────────────────────────────────────
  return (
    <>
      {/* Layer 2 — Ambient immersion. Receives cover art path. */}
      <AmbientLayer coverPath={game.cover_path} />

      {/* Layer 3 — Monochrome UI */}
      <div style={styles.page}>
        {/* Back navigation */}
        <button
          onClick={() => navigate(-1)}
          style={styles.backBtn}
          aria-label="Back to library"
        >
          <ArrowLeft size={16} />
          <span>Library</span>
        </button>

        {/* Hero */}
        <div style={styles.hero}>
          {game.cover_path ? (
            <img
              src={`https://asset.localhost/${encodeURIComponent(game.cover_path.replace(/\\/g, "/"))}`}
              alt={`${game.title} cover`}
              style={styles.cover}
              draggable={false}
            />
          ) : (
            <div style={styles.coverPlaceholder} />
          )}

          <div style={styles.heroMeta}>
            {/* Status tag */}
            <span style={styles.statusTag}>{game.status.toUpperCase()}</span>

            {/* Title */}
            <h1 style={styles.title}>{game.title}</h1>

            {/* Developer / Publisher */}
            {(game.developer || game.publisher) && (
              <p style={styles.byline}>
                {[game.developer, game.publisher].filter(Boolean).join(" · ")}
              </p>
            )}

            {/* Genre */}
            {game.genre && (
              <p style={styles.genre}>{game.genre}</p>
            )}

            {/* Actions */}
            <div style={styles.actions}>
              <button
                onClick={handleLaunch}
                disabled={launching}
                style={styles.launchBtn}
                aria-label={`Launch ${game.title}`}
              >
                <Play size={14} fill="currentColor" />
                {launching ? "Launching…" : "Play"}
              </button>

              <button
                onClick={handleFavorite}
                style={{
                  ...styles.iconBtn,
                  color: game.is_favorite
                    ? "var(--color-text-primary)"
                    : "var(--color-text-disabled)",
                }}
                aria-label={game.is_favorite ? "Remove from favorites" : "Add to favorites"}
              >
                <Star
                  size={16}
                  fill={game.is_favorite ? "currentColor" : "none"}
                />
              </button>

              {/* Add to Collection */}
              <div style={{ position: "relative" }}>
                <button
                  onClick={() => setCollMenuOpen(v => !v)}
                  style={{
                    ...styles.iconBtn,
                    color: collMenuOpen ? "var(--color-text-primary)" : "var(--color-text-disabled)",
                  }}
                  aria-label="Add to collection"
                  aria-expanded={collMenuOpen}
                  aria-haspopup="listbox"
                >
                  <FolderPlus size={15} />
                </button>
                {collMenuOpen && (
                  <div
                    style={styles.collMenu}
                    role="listbox"
                    aria-label="Collections"
                  >
                    {collections.length === 0 && (
                      <p style={styles.collMenuEmpty}>No collections yet</p>
                    )}
                    {collections.map(col => {
                      const isMember = memberIds.has(col.id);
                      return (
                        <button
                          key={col.id}
                          type="button"
                          role="option"
                          aria-selected={isMember}
                          onClick={() => handleToggleCollection(col)}
                          disabled={collLoading}
                          style={styles.collMenuItem}
                        >
                          {isMember
                            ? <Check size={11} style={{ color: "var(--color-text-secondary)", flexShrink: 0 }} />
                            : <span style={{ width: 11, flexShrink: 0 }} />}
                          <span style={styles.collMenuLabel}>{col.name}</span>
                        </button>
                      );
                    })}
                  </div>
                )}
              </div>

              {/* Edit */}
              <button
                onClick={() => navigate(`/library/${game.id}/edit`)}
                style={styles.iconBtn}
                aria-label="Edit game"
              >
                <Pencil size={15} />
              </button>

              {/* Delete */}
              <button
                onClick={() => setConfirmDelete(true)}
                style={{ ...styles.iconBtn, color: "var(--color-text-disabled)" }}
                aria-label="Delete game"
              >
                <Trash2 size={15} />
              </button>
            </div>
          </div>
        </div>

        {/* Confirm delete dialog */}
        <ConfirmDialog
          open={confirmDelete}
          title="Delete game"
          message={`Remove "${game.title}" from your library? This will also delete all play sessions. This cannot be undone.`}
          confirmLabel="Delete"
          dangerous
          onConfirm={handleDelete}
          onCancel={() => setConfirmDelete(false)}
        />

        {/* Stats row */}
        <div style={styles.statsRow}>
          <Stat label="Total playtime" value={formatPlaytime(game.total_playtime_secs)} />
          <Stat label="Sessions"       value={String(sessions.length)} />
          <Stat label="Launches"       value={String(game.launch_count)} />
          {game.last_played && (
            <Stat label="Last played" value={formatRelativeDate(game.last_played)} />
          )}
        </div>

        {/* Recent sessions */}
        {sessions.length > 0 && (
          <section style={styles.section}>
            <h2 style={styles.sectionTitle}>Recent sessions</h2>
            <div style={styles.sessionList}>
              {sessions.slice(0, 10).map((s) => (
                <div key={s.id} style={styles.sessionRow}>
                  <span style={styles.sessionDate}>
                    {formatRelativeDate(s.started_at)}
                  </span>
                  <span style={styles.sessionDuration}>
                    {formatPlaytime(s.duration_secs)}
                  </span>
                </div>
              ))}
            </div>
          </section>
        )}
      </div>
    </>
  );
}

// ── Sub-components ────────────────────────────────────────────────────────────

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div style={styles.stat}>
      <span style={styles.statValue}>{value}</span>
      <span style={styles.statLabel}>{label}</span>
    </div>
  );
}

// ── Styles ────────────────────────────────────────────────────────────────────
// All inline — no Tailwind on this page, full control over z-index layering.

const styles = {
  page: {
    position:  "relative" as const,
    zIndex:    2, // Layer 3 — always above the ambient overlay
    padding:   "40px 56px",
    minHeight: "100vh",
  },
  backBtn: {
    display:        "flex",
    alignItems:     "center",
    gap:            6,
    background:     "none",
    border:         "none",
    color:          "var(--color-text-muted)",
    fontSize:       13,
    fontFamily:     "var(--font-body)",
    cursor:         "pointer",
    padding:        0,
    marginBottom:   40,
    letterSpacing:  "0.04em",
    textTransform:  "uppercase" as const,
    transition:     "color 150ms",
  },
  hero: {
    display:   "flex",
    gap:       48,
    alignItems: "flex-start",
    marginBottom: 56,
  },
  cover: {
    width:        240,
    height:       320,
    objectFit:    "cover" as const,
    flexShrink:   0,
    borderRadius: 2,
  },
  coverPlaceholder: {
    width:       240,
    height:      320,
    flexShrink:  0,
    background:  "var(--color-surface-02)",
    borderRadius: 2,
  },
  heroMeta: {
    display:        "flex",
    flexDirection:  "column" as const,
    justifyContent: "flex-end",
    paddingBottom:  8,
    flex:           1,
  },
  statusTag: {
    fontSize:      11,
    letterSpacing: "0.12em",
    color:         "var(--color-text-disabled)",
    fontFamily:    "var(--font-mono)",
    marginBottom:  12,
  },
  title: {
    fontFamily:    "var(--font-display)",
    fontSize:      "clamp(40px, 5vw, 72px)",
    fontWeight:    700,
    letterSpacing: "-0.03em",
    lineHeight:    1.0,
    color:         "var(--color-text-primary)",
    margin:        0,
    marginBottom:  12,
  },
  byline: {
    fontSize:     14,
    color:        "var(--color-text-muted)",
    margin:       0,
    marginBottom: 6,
    fontFamily:   "var(--font-body)",
  },
  genre: {
    fontSize:     13,
    color:        "var(--color-text-disabled)",
    margin:       0,
    marginBottom: 32,
    fontFamily:   "var(--font-mono)",
    letterSpacing: "0.04em",
  },
  actions: {
    display:    "flex",
    alignItems: "center",
    gap:        12,
  },
  launchBtn: {
    display:        "flex",
    alignItems:     "center",
    gap:            8,
    background:     "var(--color-text-primary)",
    color:          "var(--color-bg-base)",
    border:         "none",
    padding:        "10px 24px",
    fontSize:       13,
    fontFamily:     "var(--font-body)",
    fontWeight:     600,
    letterSpacing:  "0.06em",
    textTransform:  "uppercase" as const,
    cursor:         "pointer",
    borderRadius:   1,
    transition:     "opacity 150ms",
  },
  iconBtn: {
    background:  "none",
    border:      "none",
    cursor:      "pointer",
    padding:     8,
    display:     "flex",
    alignItems:  "center",
    transition:  "color 150ms",
  },
  statsRow: {
    display:       "flex",
    gap:           48,
    paddingBottom: 48,
    borderBottom:  "1px solid var(--color-border)",
    marginBottom:  48,
  },
  stat: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           4,
  },
  statValue: {
    fontFamily:    "var(--font-display)",
    fontSize:      28,
    fontWeight:    600,
    letterSpacing: "-0.02em",
    color:         "var(--color-text-primary)",
  },
  statLabel: {
    fontSize:      12,
    letterSpacing: "0.08em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    fontFamily:    "var(--font-mono)",
  },
  section: {
    marginBottom: 48,
  },
  sectionTitle: {
    fontFamily:    "var(--font-body)",
    fontSize:      11,
    fontWeight:    600,
    letterSpacing: "0.12em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    margin:        0,
    marginBottom:  16,
  },
  sessionList: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           2,
  },
  sessionRow: {
    display:        "flex",
    justifyContent: "space-between",
    padding:        "10px 0",
    borderBottom:   "1px solid var(--color-border)",
    fontSize:       13,
    fontFamily:     "var(--font-body)",
  },
  sessionDate: {
    color: "var(--color-text-muted)",
  },
  sessionDuration: {
    color:      "var(--color-text-primary)",
    fontFamily: "var(--font-mono)",
    fontSize:   12,
  },
  collMenu: {
    position:     "absolute" as const,
    top:          "calc(100% + 8px)",
    left:         "50%",
    transform:    "translateX(-50%)",
    background:   "var(--color-surface)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    minWidth:     180,
    zIndex:       50,
    boxShadow:    "0 8px 24px rgba(0,0,0,0.4)",
    padding:      "6px 0",
    display:      "flex",
    flexDirection: "column" as const,
  },
  collMenuEmpty: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    color:         "var(--color-text-disabled)",
    padding:       "8px 14px",
    margin:        0,
    letterSpacing: "0.06em",
  },
  collMenuItem: {
    display:     "flex",
    alignItems:  "center",
    gap:         8,
    background:  "none",
    border:      "none",
    padding:     "8px 14px",
    cursor:      "pointer",
    width:       "100%",
    textAlign:   "left" as const,
    transition:  "background 100ms",
  },
  collMenuLabel: {
    fontFamily:   "var(--font-body)",
    fontSize:     13,
    color:        "var(--color-text-primary)",
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
  },
} satisfies Record<string, React.CSSProperties | string>;

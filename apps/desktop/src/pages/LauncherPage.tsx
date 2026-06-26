/**
 * LauncherPage — the home screen / Continue Journey.
 *
 * Design spec: Design/Pages/launcher.md
 * "One hero game dominates the screen."
 *
 * Sections:
 *   1. Continue Journey — last-played game as full-width hero
 *   2. Recent Activity  — last 5 played games as a compact row
 *
 * If no games exist, a welcome state prompts the user to add their first game.
 */

import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Play, Plus, Clock } from "lucide-react";

import { launchGame, getAllGames } from "@/lib/api";
import { formatPlaytime, formatRelativeDate } from "@/lib/utils";
import { useGameStoppedListener }             from "@/hooks/useGameStoppedListener";
import type { Game } from "@/types";

export default function LauncherPage() {
  const navigate = useNavigate();

  const [games,     setGames]     = useState<Game[]>([]);
  const [loading,   setLoading]   = useState(true);
  const [launching, setLaunching] = useState(false);
  const [error,     setError]     = useState<string | null>(null);

  /** Fetch all games, sort by last_played descending (nulls last). */
  const loadGames = useCallback(() => {
    getAllGames()
      .then((data) => {
        const sorted = [...data].sort((a, b) => {
          if (!a.last_played && !b.last_played) return 0;
          if (!a.last_played) return 1;
          if (!b.last_played) return -1;
          return new Date(b.last_played).getTime() - new Date(a.last_played).getTime();
        });
        setGames(sorted);
      })
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    loadGames();
  }, [loadGames]);

  // Refresh hero + recent activity whenever any game session ends
  useGameStoppedListener(loadGames);

  const handleLaunch = async (game: Game) => {
    setLaunching(true);
    try {
      await launchGame(game.id);
    } catch (e) {
      setError(String(e));
    } finally {
      setLaunching(false);
    }
  };

  // Hero = most recently played (or most recently added if none played)
  const hero   = games[0] ?? null;
  const recent = games.slice(1, 6);

  // ── Loading ──────────────────────────────────────────────────────────────────
  if (loading) {
    return (
      <div style={styles.page}>
        <p style={{ color: "var(--color-text-disabled)", fontSize: 13 }}>
          Loading…
        </p>
      </div>
    );
  }

  // ── Welcome state (no games) ──────────────────────────────────────────────────
  if (games.length === 0) {
    return (
      <div className="atlas-enter" style={styles.page}>
        <div style={styles.welcomeState}>
          <span style={styles.welcomeEyebrow}>Pirate Harbor</span>
          <h1 style={styles.welcomeTitle}>Your archive awaits.</h1>
          <p style={styles.welcomeBody}>
            Add your first game to begin preserving your gaming history.
          </p>
          <button
            id="launcher-add-game-btn"
            onClick={() => navigate("/library/add")}
            style={styles.primaryBtn}
            aria-label="Add your first game"
          >
            <Plus size={14} aria-hidden="true" />
            Add Game
          </button>
        </div>
      </div>
    );
  }

  // ── Main view ─────────────────────────────────────────────────────────────────
  return (
    <div className="atlas-enter" style={styles.page}>

      {/* ── Error banner ──────────────────────────────────────────────────── */}
      {error && (
        <p style={styles.errorBanner} role="alert">{error}</p>
      )}

      {/* ── Continue Journey — Hero ────────────────────────────────────────── */}
      {hero && (
        <section aria-label="Continue Journey" style={styles.heroSection}>
          <span style={styles.sectionLabel}>Continue Journey</span>

          <div
            style={styles.heroCard}
            onClick={() => navigate(`/library/${hero.id}`)}
            role="button"
            tabIndex={0}
            aria-label={`Open ${hero.title}`}
            onKeyDown={(e) => e.key === "Enter" && navigate(`/library/${hero.id}`)}
          >
            {/* Cover art */}
            {hero.cover_path ? (
              <img
                src={`https://asset.localhost/${encodeURIComponent(hero.cover_path.replace(/\\/g, "/"))}`}
                alt={`${hero.title} cover`}
                style={styles.heroCover}
                draggable={false}
              />
            ) : (
              <div style={styles.heroCoverPlaceholder}>
                <span style={styles.heroCoverInitial}>
                  {hero.title.charAt(0).toUpperCase()}
                </span>
              </div>
            )}

            {/* Meta */}
            <div style={styles.heroMeta}>
              {hero.genre && (
                <span style={styles.heroGenre}>{hero.genre}</span>
              )}
              <h1 style={styles.heroTitle}>{hero.title}</h1>

              <div style={styles.heroStats}>
                <div style={styles.heroStat}>
                  <Clock size={12} aria-hidden="true" />
                  <span>{formatPlaytime(hero.total_playtime_secs)}</span>
                </div>
                {hero.last_played && (
                  <div style={styles.heroStat}>
                    <span>Last played {formatRelativeDate(hero.last_played)}</span>
                  </div>
                )}
              </div>

              {/* Play button */}
              <button
                id="hero-play-btn"
                onClick={(e) => { e.stopPropagation(); handleLaunch(hero); }}
                disabled={launching}
                style={styles.primaryBtn}
                aria-label={`Play ${hero.title}`}
              >
                <Play size={13} fill="currentColor" aria-hidden="true" />
                {launching ? "Launching…" : "Play"}
              </button>
            </div>
          </div>
        </section>
      )}

      {/* ── Recent Activity ────────────────────────────────────────────────── */}
      {recent.length > 0 && (
        <section aria-label="Recent activity" style={styles.recentSection}>
          <span style={styles.sectionLabel}>Recent Activity</span>

          <div style={styles.recentRow} role="list">
            {recent.map((game) => (
              <div
                key={game.id}
                role="listitem"
                style={styles.recentCard}
                onClick={() => navigate(`/library/${game.id}`)}
                tabIndex={0}
                aria-label={`Open ${game.title}`}
                onKeyDown={(e) => e.key === "Enter" && navigate(`/library/${game.id}`)}
              >
                {/* Thumbnail */}
                {game.cover_path ? (
                  <img
                    src={`https://asset.localhost/${encodeURIComponent(game.cover_path.replace(/\\/g, "/"))}`}
                    alt=""
                    style={styles.recentThumb}
                    draggable={false}
                  />
                ) : (
                  <div style={styles.recentThumbPlaceholder}>
                    {game.title.charAt(0).toUpperCase()}
                  </div>
                )}

                {/* Title + time */}
                <div style={styles.recentMeta}>
                  <p style={styles.recentTitle}>{game.title}</p>
                  <span style={styles.recentTime}>
                    {formatPlaytime(game.total_playtime_secs)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}

// ── Styles ────────────────────────────────────────────────────────────────────

const styles = {
  page: {
    padding:   "40px 56px",
    overflowY: "auto" as const,
    height:    "100%",
    boxSizing: "border-box" as const,
  },
  errorBanner: {
    color:        "var(--color-text-muted)",
    fontSize:     13,
    fontFamily:   "var(--font-body)",
    marginBottom: 24,
  },

  // ── Welcome state ─────────────────────────────────────────────────────────
  welcomeState: {
    display:        "flex",
    flexDirection:  "column" as const,
    justifyContent: "center",
    height:         "80vh",
    maxWidth:       480,
  },
  welcomeEyebrow: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.16em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    marginBottom:  24,
    display:       "block",
  },
  welcomeTitle: {
    fontFamily:    "var(--font-display)",
    fontSize:      "clamp(48px, 6vw, 96px)",
    fontWeight:    700,
    letterSpacing: "-0.03em",
    lineHeight:    1.0,
    color:         "var(--color-text-primary)",
    margin:        0,
    marginBottom:  20,
  },
  welcomeBody: {
    fontFamily:   "var(--font-body)",
    fontSize:     16,
    color:        "var(--color-text-muted)",
    margin:       0,
    lineHeight:   1.6,
    marginBottom: 40,
  },

  // ── Hero section ──────────────────────────────────────────────────────────
  heroSection: {
    marginBottom: 64,
  },
  sectionLabel: {
    display:       "block",
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.12em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    marginBottom:  20,
  },
  heroCard: {
    display:      "flex",
    gap:          48,
    alignItems:   "flex-end",
    cursor:       "pointer",
    borderBottom: "1px solid var(--color-border)",
    paddingBottom: 48,
  },
  heroCover: {
    width:        200,
    height:       267, // 3:4 ratio
    objectFit:    "cover" as const,
    flexShrink:   0,
    borderRadius: 1,
    display:      "block",
  },
  heroCoverPlaceholder: {
    width:          200,
    height:         267,
    flexShrink:     0,
    background:     "var(--color-elevated)",
    borderRadius:   1,
    display:        "flex",
    alignItems:     "center",
    justifyContent: "center",
  },
  heroCoverInitial: {
    fontFamily:    "var(--font-display)",
    fontSize:      64,
    fontWeight:    700,
    color:         "var(--color-text-disabled)",
    letterSpacing: "-0.02em",
  },
  heroMeta: {
    flex:          1,
    display:       "flex",
    flexDirection: "column" as const,
    gap:           12,
    paddingBottom: 8,
  },
  heroGenre: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.08em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
  },
  heroTitle: {
    fontFamily:    "var(--font-display)",
    fontSize:      "clamp(36px, 4vw, 64px)",
    fontWeight:    700,
    letterSpacing: "-0.03em",
    lineHeight:    1.0,
    color:         "var(--color-text-primary)",
    margin:        0,
  },
  heroStats: {
    display:    "flex",
    gap:        20,
    alignItems: "center",
    marginTop:  4,
  },
  heroStat: {
    display:    "flex",
    alignItems: "center",
    gap:        6,
    fontFamily: "var(--font-mono)",
    fontSize:   12,
    color:      "var(--color-text-muted)",
  },

  // ── Recent activity ───────────────────────────────────────────────────────
  recentSection: {
    marginBottom: 48,
  },
  recentRow: {
    display: "flex",
    gap:     16,
  },
  recentCard: {
    display:      "flex",
    flexDirection: "column" as const,
    gap:          10,
    cursor:       "pointer",
    flex:         "0 0 120px",
    transition:   "opacity 150ms",
  },
  recentThumb: {
    width:        120,
    height:       160, // 3:4 ratio
    objectFit:    "cover" as const,
    borderRadius: 1,
    display:      "block",
    transition:   "opacity 150ms",
  },
  recentThumbPlaceholder: {
    width:          120,
    height:         160,
    background:     "var(--color-elevated)",
    borderRadius:   1,
    display:        "flex",
    alignItems:     "center",
    justifyContent: "center",
    fontFamily:     "var(--font-display)",
    fontSize:       28,
    fontWeight:     700,
    color:          "var(--color-text-disabled)",
  },
  recentMeta: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           2,
  },
  recentTitle: {
    fontFamily:   "var(--font-body)",
    fontSize:     12,
    fontWeight:   500,
    color:        "var(--color-text-primary)",
    margin:       0,
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
  },
  recentTime: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    color:         "var(--color-text-disabled)",
    letterSpacing: "0.04em",
  },

  // ── Shared ────────────────────────────────────────────────────────────────
  primaryBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           8,
    background:    "var(--color-text-primary)",
    color:         "var(--color-base)",
    border:        "none",
    padding:       "10px 24px",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    fontWeight:    600,
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    cursor:        "pointer",
    borderRadius:  1,
    transition:    "opacity 150ms",
    alignSelf:     "flex-start" as const,
    marginTop:     8,
  },
} satisfies Record<string, React.CSSProperties>;

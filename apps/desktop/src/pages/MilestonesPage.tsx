/**
 * MilestonesPage — Archival achievement records.
 *
 * Design spec: "Display achievements as archival records."
 * "Show: Progress · Rare milestones · Unlock timeline · Statistics"
 *
 * Data sources (all existing APIs, no new backend needed):
 *   - journal_entries WHERE entry_type = 'milestone'  → milestone log
 *   - getAllGames                                       → per-game progress stats
 *   - getSessions                                       → playtime per game
 *
 * Layout:
 *   Left:  Statistics panel  (totals, top game, completion rate)
 *   Right: Two-column content
 *            Top:  Rare / Highlighted milestones strip
 *            Main: Chronological unlock timeline grouped by game
 */

import { useCallback, useEffect, useState } from "react";
import { Trophy, Star, Gamepad2, Clock, BookOpen, ChevronRight } from "lucide-react";
import { useNavigate } from "react-router-dom";

import {
  getJournalEntries,
  getAllGames,
  type JournalEntry,
} from "@/lib/api";
import type { Game } from "@/types";
import { formatPlaytime } from "@/lib/utils";

// ─────────────────────────────────────────────────────────────────────────────

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString("en-GB", {
    day:   "2-digit",
    month: "short",
    year:  "numeric",
  });
}

// ─────────────────────────────────────────────────────────────────────────────

export default function MilestonesPage() {
  const navigate = useNavigate();

  const [milestones, setMilestones] = useState<JournalEntry[]>([]);
  const [games,      setGames]      = useState<Game[]>([]);
  const [loading,    setLoading]    = useState(true);
  const [activeGame, setActiveGame] = useState<string | null>(null); // filter

  const load = useCallback(async () => {
    setLoading(true);
    const [ms, gs] = await Promise.all([
      getJournalEntries(null, 500),
      getAllGames({}),
    ]);
    // Only milestone entries
    setMilestones(ms.filter(e => e.entry_type === "milestone"));
    setGames(gs);
    setLoading(false);
  }, []);

  useEffect(() => { load(); }, [load]);

  // ── Derived data ───────────────────────────────────────────────────────────

  const gameMap = Object.fromEntries(games.map(g => [g.id, g]));

  // Filter milestones by active game
  const visible = activeGame
    ? milestones.filter(m => m.game_id === activeGame)
    : milestones;

  // Per-game milestone count
  const gameCount: Record<string, number> = {};
  for (const m of milestones) {
    if (m.game_id) gameCount[m.game_id] = (gameCount[m.game_id] ?? 0) + 1;
  }

  // Top game by milestone count
  const topGameId = Object.entries(gameCount)
    .sort((a, b) => b[1] - a[1])[0]?.[0] ?? null;

  // Games that have milestones, sorted by count desc
  const gamesWithMilestones = Object.entries(gameCount)
    .sort((a, b) => b[1] - a[1])
    .map(([id, count]) => ({ game: gameMap[id], count }))
    .filter(x => x.game);

  // "Rare" = entries with a non-empty title (manually curated significant events)
  const rareMilestones = milestones.filter(m => m.title && m.title.trim().length > 0).slice(0, 6);

  // Completion stats
  const completedGames = games.filter(g => g.status === "completed").length;
  const totalPlaytime  = games.reduce((s, g) => s + (g.total_playtime_secs ?? 0), 0);

  // Group visible milestones by game (for timeline)
  type GameGroup = { gameId: string | null; gameTitle: string; milestones: JournalEntry[] };
  const gameGroups: GameGroup[] = [];
  for (const m of visible) {
    const gid = m.game_id ?? "__global";
    const last = gameGroups[gameGroups.length - 1];
    if (last?.gameId === gid) {
      last.milestones.push(m);
    } else {
      gameGroups.push({
        gameId:     m.game_id ?? null,
        gameTitle:  m.game_title ?? "Library",
        milestones: [m],
      });
    }
  }

  // ── Render ─────────────────────────────────────────────────────────────────

  return (
    <div style={styles.root}>

      {/* ── Stats sidebar ─────────────────────────────────────────────────── */}
      <aside style={styles.sidebar}>
        <p style={styles.sidebarLabel}>Statistics</p>

        <StatItem
          icon={<Trophy size={14} />}
          label="Total milestones"
          value={milestones.length}
        />
        <StatItem
          icon={<Gamepad2 size={14} />}
          label="Games tracked"
          value={Object.keys(gameCount).length}
        />
        <StatItem
          icon={<BookOpen size={14} />}
          label="Completed games"
          value={completedGames}
        />
        <StatItem
          icon={<Clock size={14} />}
          label="Total playtime"
          value={formatPlaytime(totalPlaytime)}
          mono
        />

        {topGameId && gameMap[topGameId] && (
          <div style={styles.topGameBox}>
            <p style={styles.topGameLabel}>Most milestones</p>
            <p style={styles.topGameName}>{gameMap[topGameId].title}</p>
            <p style={styles.topGameCount}>{gameCount[topGameId]} milestone{gameCount[topGameId] !== 1 ? "s" : ""}</p>
          </div>
        )}

        {/* ── Filter by game ─────────────────────────────────────────────── */}
        {gamesWithMilestones.length > 0 && (
          <div style={{ marginTop: 32 }}>
            <p style={styles.sidebarLabel}>Filter by game</p>
            <button
              type="button"
              onClick={() => setActiveGame(null)}
              style={{
                ...styles.filterBtn,
                ...(activeGame === null ? styles.filterBtnActive : {}),
              }}
              aria-pressed={activeGame === null}
            >
              All games
            </button>
            {gamesWithMilestones.map(({ game, count }) => (
              <button
                key={game.id}
                type="button"
                onClick={() => setActiveGame(game.id === activeGame ? null : game.id)}
                style={{
                  ...styles.filterBtn,
                  ...(activeGame === game.id ? styles.filterBtnActive : {}),
                }}
                aria-pressed={activeGame === game.id}
                title={game.title}
              >
                <span style={styles.filterTitle}>{game.title}</span>
                <span style={styles.filterCount}>{count}</span>
              </button>
            ))}
          </div>
        )}
      </aside>

      {/* ── Main ──────────────────────────────────────────────────────────── */}
      <main style={styles.main} role="main">

        {/* Page header */}
        <div style={styles.header}>
          <div>
            <h1 style={styles.pageTitle}>Milestones</h1>
            <p style={styles.pageSubtitle}>
              {loading ? "Loading…" : `${milestones.length} archived ${milestones.length === 1 ? "achievement" : "achievements"}`}
            </p>
          </div>
        </div>

        {/* Empty state */}
        {!loading && milestones.length === 0 && (
          <div style={styles.emptyState}>
            <Trophy size={40} style={{ color: "var(--color-text-disabled)", marginBottom: 16 }} />
            <p style={styles.emptyTitle}>No milestones yet</p>
            <p style={styles.emptyHint}>
              Create milestone entries in the Journal to archive significant gaming achievements.
            </p>
            <button
              type="button"
              onClick={() => navigate("/journal")}
              style={styles.emptyBtn}
            >
              Open Journal
            </button>
          </div>
        )}

        {/* ── Rare milestones strip ──────────────────────────────────────── */}
        {rareMilestones.length > 0 && (
          <section style={styles.rareSection} aria-label="Highlighted milestones">
            <p style={styles.sectionLabel}>
              <Star size={11} style={{ display: "inline-block", marginRight: 6 }} aria-hidden="true" />
              Highlighted
            </p>
            <div style={styles.rareGrid} role="list">
              {rareMilestones.map(m => (
                <div
                  key={m.id}
                  style={styles.rareCard}
                  role="listitem"
                  aria-label={m.title ?? "Milestone"}
                >
                  <span style={styles.rareDateStamp}>{formatDate(m.created_at)}</span>
                  <Trophy size={16} style={styles.rareTrophyIcon} aria-hidden="true" />
                  <p style={styles.rareTitle}>{m.title}</p>
                  {m.game_title && (
                    <p style={styles.rareGame}>{m.game_title}</p>
                  )}
                </div>
              ))}
            </div>
          </section>
        )}

        {/* ── Timeline ──────────────────────────────────────────────────── */}
        {visible.length > 0 && (
          <section style={styles.timeline} aria-label="Milestone timeline">
            <p style={styles.sectionLabel}>Unlock Timeline</p>

            {gameGroups.map((group, gi) => (
              <div key={`${group.gameId}-${gi}`} style={styles.timelineGroup}>
                {/* Game header */}
                <button
                  type="button"
                  onClick={() => group.gameId ? navigate(`/library/${group.gameId}`) : undefined}
                  style={{
                    ...styles.gameGroupHeader,
                    cursor: group.gameId ? "pointer" : "default",
                  }}
                  aria-label={group.gameId ? `Open ${group.gameTitle}` : group.gameTitle}
                >
                  <span style={styles.gameGroupTitle}>{group.gameTitle}</span>
                  <span style={styles.gameGroupCount}>
                    {group.milestones.length} {group.milestones.length === 1 ? "milestone" : "milestones"}
                  </span>
                  {group.gameId && <ChevronRight size={12} style={{ color: "var(--color-text-disabled)" }} />}
                </button>

                {/* Milestone entries */}
                <div style={styles.milestoneList} role="list">
                  {group.milestones.map((m, i) => (
                    <div key={m.id} style={styles.milestoneRow} role="listitem">
                      {/* Timeline stem */}
                      <div style={styles.stem} aria-hidden="true">
                        <div style={styles.stemDot} />
                        {i < group.milestones.length - 1 && (
                          <div style={styles.stemLine} />
                        )}
                      </div>

                      {/* Content */}
                      <div style={styles.milestoneContent}>
                        <div style={styles.milestoneMeta}>
                          <span style={styles.milestoneDate}>{formatDate(m.created_at)}</span>
                        </div>
                        {m.title && (
                          <p style={styles.milestoneTitle}>{m.title}</p>
                        )}
                        {m.body && (
                          <p style={styles.milestoneBody}>{m.body}</p>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </section>
        )}
      </main>
    </div>
  );
}

// ── StatItem component ────────────────────────────────────────────────────────

function StatItem({
  icon, label, value, mono,
}: {
  icon:   React.ReactNode;
  label:  string;
  value:  string | number;
  mono?:  boolean;
}) {
  return (
    <div style={statStyles.item}>
      <span style={statStyles.icon}>{icon}</span>
      <div style={statStyles.text}>
        <p style={statStyles.label}>{label}</p>
        <p style={{ ...statStyles.value, fontFamily: mono ? "var(--font-mono)" : "var(--font-display)" }}>
          {value}
        </p>
      </div>
    </div>
  );
}

const statStyles = {
  item: {
    display:    "flex",
    alignItems: "center",
    gap:        12,
    padding:    "10px 0",
    borderBottom: "1px solid var(--color-border-sub)",
  },
  icon: {
    color:    "var(--color-text-disabled)",
    display:  "flex",
    flexShrink: 0,
  },
  text: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           2,
  },
  label: {
    fontFamily: "var(--font-mono)",
    fontSize:   10,
    letterSpacing: "0.08em",
    color:      "var(--color-text-disabled)",
    margin:     0,
  },
  value: {
    fontSize:   18,
    fontWeight: 700,
    color:      "var(--color-text-primary)",
    margin:     0,
    lineHeight: 1.1,
  },
} satisfies Record<string, React.CSSProperties>;

// ── Page styles ───────────────────────────────────────────────────────────────

const styles = {
  root: {
    display:  "flex",
    height:   "100%",
    overflow: "hidden",
  },

  // ── Sidebar ───────────────────────────────────────────────────────────────
  sidebar: {
    width:         220,
    flexShrink:    0,
    borderRight:   "1px solid var(--color-border)",
    padding:       "40px 20px",
    overflowY:     "auto" as const,
    display:       "flex",
    flexDirection: "column" as const,
    gap:           0,
  },
  sidebarLabel: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.12em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    margin:        "0 0 8px",
  },
  topGameBox: {
    marginTop:  20,
    padding:    "14px",
    border:     "1px solid var(--color-border)",
    borderRadius: 1,
    background: "var(--color-surface)",
  },
  topGameLabel: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.10em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    margin:        "0 0 4px",
  },
  topGameName: {
    fontFamily:   "var(--font-body)",
    fontSize:     13,
    fontWeight:   600,
    color:        "var(--color-text-primary)",
    margin:       "0 0 2px",
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
  },
  topGameCount: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    color:         "var(--color-text-disabled)",
    margin:        0,
    letterSpacing: "0.04em",
  },
  filterBtn: {
    display:        "flex",
    alignItems:     "center",
    justifyContent: "space-between" as const,
    gap:            8,
    background:     "none",
    border:         "none",
    borderRadius:   1,
    padding:        "7px 8px",
    fontSize:       12,
    fontFamily:     "var(--font-body)",
    color:          "var(--color-text-muted)",
    cursor:         "pointer",
    textAlign:      "left" as const,
    width:          "100%",
    transition:     "background 150ms, color 150ms",
    marginBottom:   2,
  },
  filterBtnActive: {
    background: "var(--color-elevated)",
    color:      "var(--color-text-primary)",
  },
  filterTitle: {
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
    flex:         1,
    fontSize:     12,
  },
  filterCount: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    color:         "var(--color-text-disabled)",
    letterSpacing: "0.06em",
    flexShrink:    0,
  },

  // ── Main ──────────────────────────────────────────────────────────────────
  main: {
    flex:      1,
    display:   "flex",
    flexDirection: "column" as const,
    overflowY: "auto" as const,
    padding:   "40px 56px",
    minWidth:  0,
  },
  header: {
    marginBottom: 40,
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

  // ── Empty ─────────────────────────────────────────────────────────────────
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
    fontFamily:   "var(--font-body)",
    fontSize:     14,
    color:        "var(--color-text-disabled)",
    maxWidth:     300,
    lineHeight:   1.6,
    margin:       "0 0 24px",
  },
  emptyBtn: {
    background:    "none",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "9px 20px",
    fontSize:      12,
    fontFamily:    "var(--font-mono)",
    letterSpacing: "0.06em",
    color:         "var(--color-text-muted)",
    cursor:        "pointer",
    transition:    "border-color 150ms, color 150ms",
  },

  // ── Rare milestones strip ─────────────────────────────────────────────────
  sectionLabel: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.12em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    margin:        "0 0 14px",
    display:       "flex",
    alignItems:    "center",
  },
  rareSection: {
    marginBottom: 48,
  },
  rareGrid: {
    display:             "grid",
    gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))",
    gap:                 12,
  },
  rareCard: {
    padding:      "16px",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    background:   "var(--color-surface)",
    display:      "flex",
    flexDirection: "column" as const,
    gap:          6,
    position:     "relative" as const,
    overflow:     "hidden",
  },
  rareDateStamp: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.08em",
    color:         "var(--color-text-disabled)",
  },
  rareTrophyIcon: {
    color:     "var(--color-text-secondary)",
    alignSelf: "flex-start" as const,
    marginBottom: 2,
  },
  rareTitle: {
    fontFamily:  "var(--font-body)",
    fontSize:    13,
    fontWeight:  600,
    color:       "var(--color-text-primary)",
    margin:      0,
    lineHeight:  1.4,
  },
  rareGame: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    color:         "var(--color-text-disabled)",
    margin:        0,
    letterSpacing: "0.04em",
    overflow:      "hidden",
    textOverflow:  "ellipsis",
    whiteSpace:    "nowrap" as const,
  },

  // ── Timeline ──────────────────────────────────────────────────────────────
  timeline: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           0,
  },
  timelineGroup: {
    marginBottom: 40,
  },
  gameGroupHeader: {
    display:        "flex",
    alignItems:     "center",
    gap:            10,
    background:     "none",
    border:         "none",
    borderRadius:   1,
    padding:        "0 0 12px",
    width:          "100%",
    textAlign:      "left" as const,
    borderBottom:   "1px solid var(--color-border)",
    marginBottom:   20,
    transition:     "opacity 150ms",
  },
  gameGroupTitle: {
    fontFamily:    "var(--font-display)",
    fontSize:      18,
    fontWeight:    700,
    letterSpacing: "-0.01em",
    color:         "var(--color-text-primary)",
  },
  gameGroupCount: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.08em",
    color:         "var(--color-text-disabled)",
    marginLeft:    "auto",
  },
  milestoneList: {
    display:       "flex",
    flexDirection: "column" as const,
  },
  milestoneRow: {
    display:    "flex",
    gap:        20,
    paddingBottom: 24,
  },
  stem: {
    display:       "flex",
    flexDirection: "column" as const,
    alignItems:    "center",
    flexShrink:    0,
    width:         16,
    paddingTop:    3,
  },
  stemDot: {
    width:        8,
    height:       8,
    borderRadius: "50%",
    border:       "2px solid var(--color-text-secondary)",
    background:   "var(--color-base)",
    flexShrink:   0,
    zIndex:       1,
  },
  stemLine: {
    width:      1,
    flex:       1,
    background: "var(--color-border)",
    marginTop:  4,
  },
  milestoneContent: {
    flex:      1,
    minWidth:  0,
    paddingTop: 0,
  },
  milestoneMeta: {
    display:      "flex",
    alignItems:   "center",
    gap:          10,
    marginBottom: 4,
  },
  milestoneDate: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.08em",
    color:         "var(--color-text-disabled)",
  },
  milestoneTitle: {
    fontFamily:   "var(--font-body)",
    fontSize:     15,
    fontWeight:   600,
    color:        "var(--color-text-primary)",
    margin:       "0 0 4px",
    lineHeight:   1.35,
  },
  milestoneBody: {
    fontFamily:  "var(--font-body)",
    fontSize:    13,
    lineHeight:  1.65,
    color:       "var(--color-text-muted)",
    margin:      0,
    whiteSpace:  "pre-wrap" as const,
  },
} satisfies Record<string, React.CSSProperties>;

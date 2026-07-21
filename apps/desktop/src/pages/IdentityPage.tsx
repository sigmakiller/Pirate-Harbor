/**
 * IdentityPage — Player profile overview.
 *
 * Design spec: "Profile · Favorite genres · Runtime · Recent journeys ·
 * Completion Timeline"
 *
 * Data sources (all existing APIs — no new backend):
 *   - getAllGames()              → library stats, genre breakdown, completion timeline
 *   - getJournalEntries()        → journal activity count
 *
 * Layout (two-column editorial):
 *   Left col  (40%): Profile card · Genre breakdown · Runtime hero
 *   Right col (60%): Recent journeys (last played) · Completion timeline
 */

import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  Gamepad2, Trophy, BookOpen, Star,
  Check, TrendingUp, ChevronRight,
} from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";

import { getAllGames, getJournalEntries, getGamingIdentity, getDateHeatmap, getRecentMilestones, type JournalEntry, type DateHeatmapCell, type RecentMilestone } from "@/lib/api";
import { formatPlaytime, formatRelativeDate, STATUS_LABELS } from "@/lib/utils";
import { GhostList } from "@/components/SkeletonLoader";
import type { Game, GameStatus, GamingIdentity } from "@/types";

// ─────────────────────────────────────────────────────────────────────────────

const STATUS_ORDER: GameStatus[] = ["playing", "completed", "dropped", "unplayed"];

const STATUS_BADGE: Record<GameStatus, React.CSSProperties> = {
  playing:   { color: "var(--color-text-secondary)", borderColor: "var(--color-text-secondary)" },
  completed: { color: "var(--color-text-muted)",     borderColor: "var(--color-border)" },
  dropped:   { color: "var(--color-text-disabled)",  borderColor: "var(--color-border-sub)" },
  unplayed:  { color: "var(--color-text-disabled)",  borderColor: "var(--color-border-sub)" },
};

function coverSrc(g: Game): string | null {
  if (!g.cover_path) return null;
  try { return convertFileSrc(g.cover_path); } catch { return null; }
}

function formatYear(iso: string) {
  return new Date(iso).getFullYear().toString();
}

// ─────────────────────────────────────────────────────────────────────────────

export default function IdentityPage() {
  const navigate = useNavigate();

  const [games,    setGames]    = useState<Game[]>([]);
  const [entries,  setEntries]  = useState<JournalEntry[]>([]);
  const [identity, setIdentity] = useState<GamingIdentity | null>(null);
  const [loading,  setLoading]  = useState(true);
  // T53
  const [heatmap,    setHeatmap]    = useState<DateHeatmapCell[]>([]);
  const [milestones, setMilestones] = useState<RecentMilestone[]>([]);

  const load = useCallback(async () => {
    setLoading(true);
    const [gs, es, id, hm, ms] = await Promise.all([
      getAllGames({}),
      getJournalEntries(null, 500),
      getGamingIdentity().catch(() => null), // Fallback to manual calculation if fails
      getDateHeatmap().catch(() => []),
      getRecentMilestones(20).catch(() => []),
    ]);
    setGames(gs);
    setEntries(es);
    setIdentity(id);
    setHeatmap(hm);
    setMilestones(ms);
    setLoading(false);
  }, []);

  useEffect(() => { load(); }, [load]);

  // ── Derived stats ──────────────────────────────────────────────────────────

  const totalGames     = games.length;
  const totalPlaytime  = games.reduce((s, g) => s + (g.total_playtime_secs ?? 0), 0);
  const completedCount = games.filter(g => g.status === "completed").length;
  const playingCount   = games.filter(g => g.status === "playing").length;
  const favCount       = games.filter(g => g.is_favorite).length;
  const milestoneCount = entries.filter(e => e.entry_type === "milestone").length;
  const noteCount      = entries.filter(e => e.entry_type !== "milestone").length;

  // Completion rate (excluding unplayed)
  const engaged      = games.filter(g => g.status !== "unplayed").length;
  const completionPct = engaged > 0 ? Math.round((completedCount / engaged) * 100) : 0;

  // Genre breakdown — split comma-separated genres and count
  const genreCounts: Record<string, number> = {};
  for (const g of games) {
    if (!g.genre) continue;
    for (const raw of g.genre.split(",")) {
      const genre = raw.trim();
      if (genre) genreCounts[genre] = (genreCounts[genre] ?? 0) + 1;
    }
  }
  const topGenres = Object.entries(genreCounts)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 8);
  const maxGenreCount = topGenres[0]?.[1] ?? 1;

  // Status breakdown
  const statusCounts: Record<GameStatus, number> = {
    playing:   playingCount,
    completed: completedCount,
    dropped:   games.filter(g => g.status === "dropped").length,
    unplayed:  games.filter(g => g.status === "unplayed").length,
  };

  // Recent journeys — last 5 played games sorted by last_played desc
  const recentGames = [...games]
    .filter(g => g.last_played)
    .sort((a, b) => {
      const da = a.last_played ? new Date(a.last_played).getTime() : 0;
      const db = b.last_played ? new Date(b.last_played).getTime() : 0;
      return db - da;
    })
    .slice(0, 6);

  // Completion timeline — completed games sorted by last_played (proxy for completion date)
  const completionTimeline = [...games]
    .filter(g => g.status === "completed" && g.last_played)
    .sort((a, b) => {
      const da = new Date(a.last_played!).getTime();
      const db = new Date(b.last_played!).getTime();
      return da - db;
    });

  // ── Render ─────────────────────────────────────────────────────────────────

  if (loading) {
    return (
      <GhostList count={10} />
    );
  }

  return (
    <div className="atlas-enter" style={styles.root}>

      {/* ── Left column ──────────────────────────────────────────────────────── */}
      <div style={styles.leftCol}>

        {/* ── Profile card ─────────────────────────────────────────────────── */}
        <section style={styles.profileCard} aria-label="Profile summary">
          <div style={styles.profileHero}>
            <div style={styles.avatarRing} aria-hidden="true">
              <Gamepad2 size={28} style={{ color: "var(--color-text-secondary)" }} />
            </div>
            <div>
              <p style={styles.profileHandle}>PIRATE HARBOR</p>
              <p style={styles.profileCaption}>Player Profile</p>
            </div>
          </div>

          {/* T52: Year-in-Review link */}
          <button
            id="yir-nav-btn"
            onClick={() => navigate("/identity/year-in-review")}
            style={{
              display:        "flex",
              alignItems:     "center",
              justifyContent: "space-between",
              width:          "100%",
              background:     "color-mix(in srgb, var(--color-accent, #6366f1) 10%, var(--color-surface))",
              border:         "1px solid color-mix(in srgb, var(--color-accent, #6366f1) 30%, transparent)",
              borderRadius:   8,
              padding:        "10px 14px",
              color:          "var(--color-text-primary)",
              fontSize:       13,
              fontWeight:     600,
              cursor:         "pointer",
              marginBottom:   12,
              transition:     "background 150ms",
            }}
          >
            <span>📅 {new Date().getFullYear()} Year in Review</span>
            <span style={{ color: "var(--color-accent, #6366f1)", fontSize: 16 }}>→</span>
          </button>

          {/* Gaming Personality (if available) */}
          {identity?.gaming_personality && (
            <div style={styles.personalityBox}>
              <p style={styles.personalityLabel}>Gaming Personality</p>
              <p style={styles.personalityType}>
                {identity.gaming_personality.primary_type}
                {identity.gaming_personality.secondary_type && (
                  <span style={styles.personalitySecondary}>
                    {" / " + identity.gaming_personality.secondary_type}
                  </span>
                )}
              </p>
              <p style={styles.personalityDesc}>{identity.gaming_personality.description}</p>
              {identity.gaming_personality.traits.length > 0 && (
                <div style={styles.traitsList}>
                  {identity.gaming_personality.traits.slice(0, 4).map(trait => (
                    <span key={trait} style={styles.traitPill}>{trait}</span>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Stat row */}
          <div style={styles.statRow} role="list" aria-label="Quick stats">
            <StatPill icon={<Gamepad2 size={11} />} label="Games"     value={totalGames}       />
            <StatPill icon={<Star     size={11} />} label="Favourites" value={favCount}          />
            <StatPill icon={<Trophy  size={11} />} label="Milestones" value={milestoneCount}    />
            <StatPill icon={<BookOpen size={11} />} label="Notes"     value={noteCount}         />
          </div>
        </section>

        {/* ── Runtime hero ─────────────────────────────────────────────────── */}
        <section style={styles.runtimeHero} aria-label="Total runtime">
          <p style={styles.runtimeLabel}>Total Runtime</p>
          <p style={styles.runtimeValue}>{formatPlaytime(totalPlaytime)}</p>
          
          {/* Streaks display (if analytics available) */}
          {identity?.runtime_statistics && (identity.runtime_statistics.streak_current_days > 0 || identity.runtime_statistics.streak_longest_days > 0) && (
            <div style={styles.streakRow}>
              <div style={styles.streakBox}>
                <span style={styles.streakLabel}>Current Streak</span>
                <span style={styles.streakValue}>{identity.runtime_statistics.streak_current_days} days</span>
              </div>
              <div style={styles.streakBox}>
                <span style={styles.streakLabel}>Longest Streak</span>
                <span style={styles.streakValue}>{identity.runtime_statistics.streak_longest_days} days</span>
              </div>
            </div>
          )}

          <div style={styles.completionRow}>
            <div style={styles.completionBar} role="progressbar"
              aria-valuenow={completionPct} aria-valuemin={0} aria-valuemax={100}
              aria-label={`Completion rate ${completionPct}%`}
            >
              <div style={{ ...styles.completionFill, width: `${completionPct}%` }} />
            </div>
            <span style={styles.completionPct}>{completionPct}% completed</span>
          </div>
          {/* Status breakdown */}
          <div style={styles.statusBreakdown} role="list" aria-label="Library status breakdown">
            {STATUS_ORDER.map(s => (
              <div key={s} style={styles.statusRow} role="listitem">
                <span style={{ ...styles.statusDot, ...(STATUS_BADGE[s as GameStatus]) }} aria-hidden="true" />
                <span style={styles.statusLabel}>{STATUS_LABELS[s as GameStatus]}</span>
                <span style={styles.statusCount}>{statusCounts[s]}</span>
              </div>
            ))}
          </div>
        </section>

        {/* ── Favourite genres ─────────────────────────────────────────────── */}
        <section style={styles.genreSection} aria-label="Favourite genres">
          <p style={styles.sectionLabel}>
            <TrendingUp size={11} style={{ display: "inline-block", marginRight: 6 }} aria-hidden="true" />
            Favourite Genres
          </p>
          {topGenres.length === 0 && (
            <p style={styles.emptyNote}>Add genre info to your games to see your profile.</p>
          )}
          {topGenres.map(([genre, count]) => (
            <div key={genre} style={styles.genreRow} aria-label={`${genre}: ${count} games`}>
              <span style={styles.genreName}>{genre}</span>
              <div style={styles.genreBarTrack} aria-hidden="true">
                <div style={{
                  ...styles.genreBarFill,
                  width: `${(count / maxGenreCount) * 100}%`,
                }} />
              </div>
              <span style={styles.genreCount}>{count}</span>
            </div>
          ))}
        </section>
      </div>

      {/* ── Right column ─────────────────────────────────────────────────────── */}
      <div style={styles.rightCol}>

        {/* ── Recent journeys ──────────────────────────────────────────────── */}
        <section style={styles.recentSection} aria-label="Recent journeys">
          <p style={styles.sectionLabel}>Recent Journeys</p>
          {recentGames.length === 0 && (
            <p style={styles.emptyNote}>No play sessions recorded yet.</p>
          )}
          <div style={styles.recentGrid} role="list">
            {recentGames.map(g => {
              const src = coverSrc(g);
              return (
                <button
                  key={g.id}
                  type="button"
                  onClick={() => navigate(`/library/${g.id}`)}
                  style={styles.recentCard}
                  role="listitem"
                  aria-label={`${g.title} — ${formatPlaytime(g.total_playtime_secs ?? 0)}`}
                >
                  {/* Cover */}
                  <div style={styles.recentCover}>
                    {src
                      ? <img src={src} alt="" style={styles.recentCoverImg} />
                      : <div style={styles.recentCoverPlaceholder}><Gamepad2 size={18} style={{ color: "var(--color-text-disabled)" }} /></div>
                    }
                    <div style={styles.recentScrim} aria-hidden="true" />
                  </div>

                  {/* Info */}
                  <div style={styles.recentInfo}>
                    <span style={styles.recentTitle}>{g.title}</span>
                    <span style={styles.recentMeta}>
                      {formatPlaytime(g.total_playtime_secs ?? 0)}
                      {g.last_played && ` · ${formatRelativeDate(g.last_played)}`}
                    </span>
                    <span style={{ ...styles.recentStatusBadge, ...STATUS_BADGE[g.status] }}>
                      {STATUS_LABELS[g.status]}
                    </span>
                  </div>

                  <ChevronRight size={12} style={{ color: "var(--color-text-disabled)", flexShrink: 0 }} />
                </button>
              );
            })}
          </div>
        </section>

        {/* ── Completion timeline ───────────────────────────────────────────── */}
        <section style={styles.timelineSection} aria-label="Completion timeline">
          <p style={styles.sectionLabel}>
            <Check size={11} style={{ display: "inline-block", marginRight: 6 }} aria-hidden="true" />
            Completion Timeline
          </p>
          {completionTimeline.length === 0 && (
            <p style={styles.emptyNote}>No completed games yet.</p>
          )}
          <div style={styles.timelineList} role="list">
            {completionTimeline.map((g, i) => (
              <button
                key={g.id}
                type="button"
                onClick={() => navigate(`/library/${g.id}`)}
                style={styles.timelineRow}
                role="listitem"
                aria-label={`${g.title}, completed`}
              >
                {/* Stem */}
                <div style={styles.stem} aria-hidden="true">
                  <div style={styles.stemDot} />
                  {i < completionTimeline.length - 1 && <div style={styles.stemLine} />}
                </div>

                {/* Content */}
                <div style={styles.timelineContent}>
                  <span style={styles.timelineYear}>
                    {g.last_played ? formatYear(g.last_played) : "—"}
                  </span>
                  <span style={styles.timelineTitle}>{g.title}</span>
                  <span style={styles.timelinePlaytime}>
                    {formatPlaytime(g.total_playtime_secs ?? 0)}
                  </span>
                </div>

                <ChevronRight size={11} style={{ color: "var(--color-text-disabled)", flexShrink: 0 }} />
              </button>
            ))}
          </div>
        </section>
      </div>

      {/* ─── T53-A: Activity Heatmap ─────────────────────────────────────────── */}
      {heatmap.length > 0 ? (
        <IdentityHeatmap cells={heatmap} />
      ) : !loading && (
        <section style={styles.heatmapSection} aria-label="Activity heatmap">
          <p style={styles.sectionHeading}>Activity — Last 365 Days</p>
          <div style={{ padding: "24px", color: "var(--color-text-disabled)", fontSize: 13, textAlign: "center" }}>
            No sessions recorded yet. Start playing to see your activity heatmap.
          </div>
        </section>
      )}

      {/* ─── T53-B: Milestone Timeline ───────────────────────────────────────── */}
      <IdentityMilestoneTimeline milestones={milestones} onNavigate={navigate} />

    </div>
  );
}

// ── StatPill ──────────────────────────────────────────────────────────────────

function StatPill({ icon, label, value }: {
  icon:  React.ReactNode;
  label: string;
  value: number;
}) {
  return (
    <div style={pillStyles.root} role="listitem">
      <span style={pillStyles.icon}>{icon}</span>
      <span style={pillStyles.value}>{value}</span>
      <span style={pillStyles.label}>{label}</span>
    </div>
  );
}

const pillStyles = {
  root: {
    display:       "flex",
    flexDirection: "column" as const,
    alignItems:    "center",
    gap:           3,
    flex:          1,
    padding:       "12px 8px",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    background:    "var(--color-elevated)",
  },
  icon: {
    color:   "var(--color-text-disabled)",
    display: "flex",
  },
  value: {
    fontFamily:  "var(--font-display)",
    fontSize:    20,
    fontWeight:  700,
    color:       "var(--color-text-primary)",
    lineHeight:  1.1,
  },
  label: {
    fontFamily:    "var(--font-mono)",
    fontSize:      9,
    letterSpacing: "0.10em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
  },
} satisfies Record<string, React.CSSProperties>;

// ── Page styles ───────────────────────────────────────────────────────────────

const styles = {
  heatmapSection: {
    padding: "0 32px 8px",
  },
  sectionHeading: {
    fontSize: 11,
    fontWeight: 700,
    letterSpacing: "0.1em",
    color: "var(--color-text-disabled)",
    textTransform: "uppercase" as const,
    marginBottom: 10,
  },
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
    display:   "flex",
    gap:       32,
    padding:   "40px 56px",
    height:    "100%",
    overflowY: "auto" as const,
    boxSizing: "border-box" as const,
    alignItems: "flex-start" as const,
  },

  // ── Left column ───────────────────────────────────────────────────────────
  leftCol: {
    flex:          "0 0 320px",
    display:       "flex",
    flexDirection: "column" as const,
    gap:           20,
  },

  // Profile card
  profileCard: {
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    background:    "var(--color-surface)",
    padding:       "24px",
    display:       "flex",
    flexDirection: "column" as const,
    gap:           20,
  },
  profileHero: {
    display:    "flex",
    alignItems: "center",
    gap:        16,
  },
  avatarRing: {
    width:        56,
    height:       56,
    borderRadius: "50%",
    border:       "1px solid var(--color-border)",
    display:      "flex",
    alignItems:   "center",
    justifyContent: "center" as const,
    background:   "var(--color-elevated)",
    flexShrink:   0,
  },
  profileHandle: {
    fontFamily:    "var(--font-mono)",
    fontSize:      12,
    letterSpacing: "0.12em",
    fontWeight:    600,
    color:         "var(--color-text-primary)",
    margin:        "0 0 3px",
  },
  profileCaption: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.08em",
    color:         "var(--color-text-disabled)",
    margin:        0,
  },
  statRow: {
    display: "flex",
    gap:     8,
  },
  personalityBox: {
    padding:       "14px",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    background:    "var(--color-elevated)",
  },
  personalityLabel: {
    fontFamily:    "var(--font-mono)",
    fontSize:      9,
    letterSpacing: "0.10em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    margin:        "0 0 6px",
  },
  personalityType: {
    fontFamily:    "var(--font-display)",
    fontSize:      16,
    fontWeight:    700,
    color:         "var(--color-text-primary)",
    margin:        "0 0 6px",
    letterSpacing: "-0.01em",
  },
  personalitySecondary: {
    color:      "var(--color-text-muted)",
    fontWeight: 500,
  },
  personalityDesc: {
    fontFamily: "var(--font-body)",
    fontSize:   12,
    lineHeight: 1.5,
    color:      "var(--color-text-muted)",
    margin:     "0 0 10px",
  },
  traitsList: {
    display:  "flex",
    flexWrap: "wrap" as const,
    gap:      6,
  },
  traitPill: {
    fontFamily:    "var(--font-mono)",
    fontSize:      9,
    letterSpacing: "0.08em",
    color:         "var(--color-text-disabled)",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "3px 8px",
    display:       "inline-block",
  },

  // Runtime hero
  runtimeHero: {
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    background:    "var(--color-surface)",
    padding:       "24px",
  },
  runtimeLabel: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.12em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    margin:        "0 0 6px",
  },
  runtimeValue: {
    fontFamily:    "var(--font-display)",
    fontSize:      "clamp(28px, 3vw, 40px)",
    fontWeight:    700,
    letterSpacing: "-0.03em",
    color:         "var(--color-text-primary)",
    margin:        "0 0 16px",
    lineHeight:    1.0,
  },
  streakRow: {
    display:      "flex",
    gap:          8,
    marginBottom: 16,
  },
  streakBox: {
    flex:          1,
    padding:       "8px 10px",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    background:    "var(--color-elevated)",
    display:       "flex",
    flexDirection: "column" as const,
    gap:           2,
  },
  streakLabel: {
    fontFamily:    "var(--font-mono)",
    fontSize:      9,
    letterSpacing: "0.08em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
  },
  streakValue: {
    fontFamily: "var(--font-display)",
    fontSize:   16,
    fontWeight: 700,
    color:      "var(--color-text-secondary)",
    lineHeight: 1.1,
  },
  completionRow: {
    display:     "flex",
    alignItems:  "center",
    gap:         10,
    marginBottom: 16,
  },
  completionBar: {
    flex:         1,
    height:       2,
    background:   "var(--color-border)",
    borderRadius: 1,
    overflow:     "hidden",
  },
  completionFill: {
    height:     "100%",
    background: "var(--color-text-secondary)",
    borderRadius: 1,
    transition: "width 600ms ease",
  },
  completionPct: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.08em",
    color:         "var(--color-text-disabled)",
    flexShrink:    0,
  },
  statusBreakdown: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           8,
  },
  statusRow: {
    display:    "flex",
    alignItems: "center",
    gap:        10,
  },
  statusDot: {
    width:        6,
    height:       6,
    borderRadius: "50%",
    border:       "1px solid",
    flexShrink:   0,
    display:      "block",
  },
  statusLabel: {
    fontFamily:   "var(--font-body)",
    fontSize:     12,
    color:        "var(--color-text-muted)",
    flex:         1,
  },
  statusCount: {
    fontFamily:    "var(--font-mono)",
    fontSize:      12,
    color:         "var(--color-text-secondary)",
    letterSpacing: "0.04em",
  },

  // Genre section
  genreSection: {
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    background:    "var(--color-surface)",
    padding:       "24px",
  },
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
  genreRow: {
    display:     "flex",
    alignItems:  "center",
    gap:         10,
    marginBottom: 8,
  },
  genreName: {
    fontFamily:   "var(--font-body)",
    fontSize:     12,
    color:        "var(--color-text-muted)",
    width:        100,
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
    flexShrink:   0,
  },
  genreBarTrack: {
    flex:         1,
    height:       2,
    background:   "var(--color-border)",
    borderRadius: 1,
    overflow:     "hidden",
  },
  genreBarFill: {
    height:       "100%",
    background:   "var(--color-text-secondary)",
    borderRadius: 1,
    transition:   "width 600ms ease",
  },
  genreCount: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    color:         "var(--color-text-disabled)",
    width:         16,
    textAlign:     "right" as const,
    flexShrink:    0,
    letterSpacing: "0.04em",
  },
  emptyNote: {
    fontFamily:  "var(--font-body)",
    fontSize:    12,
    color:       "var(--color-text-disabled)",
    margin:      0,
    lineHeight:  1.5,
  },

  // ── Right column ──────────────────────────────────────────────────────────
  rightCol: {
    flex:          1,
    display:       "flex",
    flexDirection: "column" as const,
    gap:           32,
    minWidth:      0,
  },

  // Recent section
  recentSection: {},
  recentGrid: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           8,
  },
  recentCard: {
    display:     "flex",
    alignItems:  "center",
    gap:         14,
    padding:     "12px 14px",
    border:      "1px solid var(--color-border)",
    borderRadius: 1,
    background:  "var(--color-surface)",
    cursor:      "pointer",
    width:       "100%",
    textAlign:   "left" as const,
    transition:  "border-color 150ms, background 150ms",
  },
  recentCover: {
    position:     "relative" as const,
    width:        40,
    height:       54,
    flexShrink:   0,
    borderRadius: 1,
    overflow:     "hidden",
    background:   "var(--color-elevated)",
    display:      "flex",
    alignItems:   "center",
    justifyContent: "center" as const,
  },
  recentCoverImg: {
    width:     "100%",
    height:    "100%",
    objectFit: "cover" as const,
    display:   "block",
  },
  recentCoverPlaceholder: {
    display:        "flex",
    alignItems:     "center",
    justifyContent: "center" as const,
    width:          "100%",
    height:         "100%",
  },
  recentScrim: {
    position:   "absolute" as const,
    inset:      0,
    background: "linear-gradient(to top, rgba(5,5,5,0.3) 0%, transparent 60%)",
  },
  recentInfo: {
    flex:          1,
    display:       "flex",
    flexDirection: "column" as const,
    gap:           3,
    minWidth:      0,
  },
  recentTitle: {
    fontFamily:   "var(--font-body)",
    fontSize:     13,
    fontWeight:   600,
    color:        "var(--color-text-primary)",
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
  },
  recentMeta: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.04em",
    color:         "var(--color-text-disabled)",
  },
  recentStatusBadge: {
    fontFamily:    "var(--font-mono)",
    fontSize:      9,
    letterSpacing: "0.10em",
    textTransform: "uppercase" as const,
    border:        "1px solid",
    borderRadius:  1,
    padding:       "2px 6px",
    alignSelf:     "flex-start" as const,
  },

  // Completion timeline
  timelineSection: {},
  timelineList: {
    display:       "flex",
    flexDirection: "column" as const,
  },
  timelineRow: {
    display:     "flex",
    alignItems:  "flex-start",
    gap:         16,
    padding:     "0",
    background:  "none",
    border:      "none",
    cursor:      "pointer",
    width:       "100%",
    textAlign:   "left" as const,
    transition:  "opacity 150ms",
  },
  stem: {
    display:       "flex",
    flexDirection: "column" as const,
    alignItems:    "center",
    flexShrink:    0,
    width:         14,
    paddingTop:    4,
  },
  stemDot: {
    width:        8,
    height:       8,
    borderRadius: "50%",
    border:       "2px solid var(--color-text-secondary)",
    background:   "var(--color-base)",
    flexShrink:   0,
  },
  stemLine: {
    width:      1,
    height:     28,
    background: "var(--color-border)",
    marginTop:  4,
  },
  timelineContent: {
    flex:        1,
    display:     "flex",
    alignItems:  "baseline",
    gap:         12,
    paddingBottom: 20,
    borderBottom: "1px solid var(--color-border-sub)",
    marginBottom: 4,
    flexWrap:    "wrap" as const,
    minWidth:    0,
  },
  timelineYear: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.08em",
    color:         "var(--color-text-disabled)",
    flexShrink:    0,
  },
  timelineTitle: {
    fontFamily:   "var(--font-body)",
    fontSize:     13,
    fontWeight:   600,
    color:        "var(--color-text-primary)",
    flex:         1,
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
    minWidth:     0,
  },
  timelinePlaytime: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.04em",
    color:         "var(--color-text-disabled)",
    flexShrink:    0,
  },
} satisfies Record<string, React.CSSProperties>;

// ─── T53-A: Activity Heatmap (GitHub-style 52×7 calendar) ─────────────────────

const INTENSITY = [
  "var(--color-surface-raised, #1a1a24)",          // 0 - empty
  "color-mix(in srgb, var(--color-accent, #6366f1) 25%, var(--color-surface))", // 1
  "color-mix(in srgb, var(--color-accent, #6366f1) 50%, var(--color-surface))", // 2
  "color-mix(in srgb, var(--color-accent, #6366f1) 75%, var(--color-surface))", // 3
  "var(--color-accent, #6366f1)",                   // 4 - max
];

function secsToLevel(secs: number): 0 | 1 | 2 | 3 | 4 {
  if (secs === 0) return 0;
  if (secs < 1800)  return 1;   // < 30 min
  if (secs < 7200)  return 2;   // < 2 h
  if (secs < 14400) return 3;   // < 4 h
  return 4;                     // >= 4 h
}

function buildCalendarGrid(cells: import("@/lib/api").DateHeatmapCell[]) {
  const map: Record<string, { secs: number; count: number }> = {};
  for (const c of cells) map[c.date] = { secs: c.secs, count: c.count };

  // Start from 52 full weeks ago (Sunday)
  const today = new Date();
  const start = new Date(today);
  start.setDate(today.getDate() - 364);
  // Rewind to Sunday of that week
  start.setDate(start.getDate() - start.getDay());

  const weeks: { date: string; secs: number }[][] = [];
  let cur = new Date(start);
  while (cur <= today) {
    const week: { date: string; secs: number }[] = [];
    for (let d = 0; d < 7; d++) {
      const iso = cur.toISOString().slice(0, 10);
      const future = cur > today;
      week.push({ date: iso, secs: future ? -1 : (map[iso]?.secs ?? 0) });
      cur.setDate(cur.getDate() + 1);
    }
    weeks.push(week);
  }
  return weeks;
}

function fmtTooltip(date: string, secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const label = h > 0 ? `${h}h ${m}m` : `${m}m`;
  const d = new Date(date + "T12:00:00");
  const fmt = d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
  return secs > 0 ? `${label} · ${fmt}` : fmt;
}

function IdentityHeatmap({ cells }: { cells: import("@/lib/api").DateHeatmapCell[] }) {
  const weeks = buildCalendarGrid(cells);
  const cellSize = 12;
  const gap = 2;

  return (
    <section style={{ padding: "0 32px 8px", maxWidth: 1100 }} aria-label="Activity heatmap — last 365 days">
      <p style={{ fontSize: 11, fontWeight: 700, letterSpacing: "0.1em", color: "var(--color-text-disabled)", textTransform: "uppercase", marginBottom: 10 }}>
        Activity — Last 365 Days
      </p>
      <div style={{ display: "flex", gap, alignItems: "flex-start", overflowX: "auto" }}>
        {weeks.map((week, wi) => (
          <div key={wi} style={{ display: "flex", flexDirection: "column", gap }}>
            {week.map((day) => {
              const level = day.secs < 0 ? 0 : secsToLevel(day.secs);
              return (
                <div
                  key={day.date}
                  title={day.secs >= 0 ? fmtTooltip(day.date, day.secs) : ""}
                  style={{
                    width:        cellSize,
                    height:       cellSize,
                    borderRadius: 2,
                    background:   day.secs < 0 ? "transparent" : INTENSITY[level],
                    cursor:       day.secs > 0 ? "default" : "default",
                    transition:   "background 150ms",
                  }}
                  aria-label={day.secs >= 0 ? fmtTooltip(day.date, day.secs) : ""}
                />
              );
            })}
          </div>
        ))}
      </div>
    </section>
  );
}

// ─── T53-B: Milestone Timeline ─────────────────────────────────────────────────

function groupByMonth(milestones: import("@/lib/api").RecentMilestone[]) {
  const groups: Record<string, import("@/lib/api").RecentMilestone[]> = {};
  for (const m of milestones) {
    const d = new Date(m.achievement_date);
    const key = d.toLocaleDateString("en-US", { month: "long", year: "numeric" });
    if (!groups[key]) groups[key] = [];
    groups[key].push(m);
  }
  return Object.entries(groups);
}

const CATEGORY_ICON: Record<string, string> = {
  milestone:  "🏆",
  completion: "★",
  achievement: "🎯",
  personal:   "💎",
};

function IdentityMilestoneTimeline({
  milestones,
  onNavigate,
}: {
  milestones: import("@/lib/api").RecentMilestone[];
  onNavigate: (path: string) => void;
}) {
  if (milestones.length === 0) return null;

  const groups = groupByMonth(milestones);

  return (
    <section style={{ padding: "8px 32px 48px", maxWidth: 760 }} aria-label="Recent milestones">
      <p style={{ fontSize: 11, fontWeight: 700, letterSpacing: "0.1em", color: "var(--color-text-disabled)", textTransform: "uppercase", marginBottom: 16 }}>
        Recent Milestones
      </p>
      <div style={{ display: "flex", flexDirection: "column", gap: 28 }}>
        {groups.map(([month, items]) => (
          <div key={month}>
            <div style={{ fontSize: 12, fontWeight: 700, color: "var(--color-text-secondary)", marginBottom: 10, letterSpacing: "0.04em" }}>
              {month}
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              {items.map((m) => {
                const day = new Date(m.achievement_date).toLocaleDateString("en-US", { month: "short", day: "numeric" });
                return (
                  <button
                    key={m.id}
                    id={`milestone-timeline-${m.id}`}
                    onClick={() => onNavigate(`/library/${m.game_id}`)}
                    style={{
                      display:        "flex",
                      alignItems:     "center",
                      gap:            10,
                      background:     "none",
                      border:         "none",
                      cursor:         "pointer",
                      padding:        "7px 10px",
                      borderRadius:   6,
                      textAlign:      "left",
                      transition:     "background 150ms",
                      width:          "100%",
                    }}
                    onMouseEnter={(e) => (e.currentTarget.style.background = "var(--color-surface-raised, #1e1e28)")}
                    onMouseLeave={(e) => (e.currentTarget.style.background = "none")}
                  >
                    <span style={{ width: 20, flexShrink: 0, fontSize: 14, textAlign: "center" }}>
                      {CATEGORY_ICON[m.category] ?? "🏅"}
                    </span>
                    <span style={{ flex: 1, fontSize: 13, fontWeight: 500, color: "var(--color-text-primary)", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                      {m.title}
                    </span>
                    <span style={{ fontSize: 12, color: "var(--color-text-disabled)", whiteSpace: "nowrap", flexShrink: 0 }}>
                      {m.game_title}
                    </span>
                    <span style={{ fontSize: 11, color: "var(--color-text-disabled)", flexShrink: 0, fontFamily: "var(--font-mono, monospace)" }}>
                      {day}
                    </span>
                  </button>
                );
              })}
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
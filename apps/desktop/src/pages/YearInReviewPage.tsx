/**
 * YearInReviewPage — T52
 *
 * Displays an annual gaming summary driven by `get_year_in_review` and
 * `get_monthly_playtime` backends.  Year selector defaults to the most-recent
 * year with session data, falling back to the current calendar year.
 *
 * Sections:
 *  1. Hero — year heading + 3 stat cards (playtime / games / sessions)
 *  2. Top Games — horizontal bar chart (top 5 by playtime)
 *  3. Genre Breakdown — percentage bars
 *  4. Monthly Playtime — 12-column bar chart
 *  5. Milestones row — longest session + most active month + completion rate
 */

import { useEffect, useState, useCallback } from "react";
import { useNavigate }  from "react-router-dom";
import { ArrowLeft, Trophy, Gamepad2, Clock, TrendingUp } from "lucide-react";
import {
  getYearInReview,
  getSessionYears,
  getMonthlyPlaytime,
  type YearInReview,
  type MonthlyPlaytime,
} from "@/lib/api";

// ─── Helpers ──────────────────────────────────────────────────────────────────

const MONTHS = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];

function fmtHours(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h === 0) return `${m}m`;
  if (m === 0) return `${h}h`;
  return `${h}h ${m}m`;
}

function fmtPct(v: number): string {
  return `${Math.round(v * 100)}%`;
}

// ─── Sub-components ───────────────────────────────────────────────────────────

function StatCard({ icon, label, value, sub }: {
  icon: React.ReactNode; label: string; value: string; sub?: string;
}) {
  return (
    <div style={styles.statCard}>
      <span style={styles.statIcon}>{icon}</span>
      <div style={styles.statValue}>{value}</div>
      <div style={styles.statLabel}>{label}</div>
      {sub && <div style={styles.statSub}>{sub}</div>}
    </div>
  );
}

function SectionHeading({ title }: { title: string }) {
  return (
    <div style={styles.sectionHeading}>
      <span style={styles.sectionTitle}>{title}</span>
      <div style={styles.sectionRule} />
    </div>
  );
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export default function YearInReviewPage() {
  const navigate = useNavigate();

  const [availableYears, setAvailableYears] = useState<number[]>([]);
  const [selectedYear,   setSelectedYear]   = useState<number | null>(null);
  const [data,           setData]           = useState<YearInReview | null>(null);
  const [monthly,        setMonthly]        = useState<MonthlyPlaytime[]>([]);
  const [loading,        setLoading]        = useState(true);
  const [error,          setError]          = useState<string | null>(null);

  // ── Boot: pick default year ──────────────────────────────────────────────
  useEffect(() => {
    getSessionYears()
      .then((years) => {
        setAvailableYears(years);
        const def = years[0] ?? new Date().getFullYear();
        setSelectedYear(def);
      })
      .catch(() => setSelectedYear(new Date().getFullYear()));
  }, []);

  // ── Load data when year is known / changed ───────────────────────────────
  const load = useCallback(async (year: number) => {
    setLoading(true);
    setError(null);
    try {
      const [yir, mon] = await Promise.all([
        getYearInReview(year),
        getMonthlyPlaytime(year),
      ]);
      setData(yir);
      setMonthly(mon);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (selectedYear !== null) load(selectedYear);
  }, [selectedYear, load]);

  // ── Derived ──────────────────────────────────────────────────────────────
  const maxGameSecs  = data?.top_games[0]?.total_playtime_secs ?? 1;
  const maxMonthSecs = Math.max(...monthly.map((m) => m.secs), 1);
  const maxGenreSecs = data?.top_genres[0]?.total_playtime_secs ?? 1;

  const isEmpty = data && data.sessions === 0;

  // ─── Render ───────────────────────────────────────────────────────────────
  return (
    <div className="atlas-enter" style={styles.page}>

      {/* ── Back + Year selector ── */}
      <div style={styles.topBar}>
        <button
          id="yir-back-btn"
          style={styles.backBtn}
          onClick={() => navigate("/identity")}
          aria-label="Back to Identity"
        >
          <ArrowLeft size={16} />
          Identity
        </button>

        {availableYears.length > 0 && (
          <select
            id="yir-year-select"
            value={selectedYear ?? ""}
            onChange={(e) => setSelectedYear(Number(e.target.value))}
            style={styles.yearSelect}
            aria-label="Select year"
          >
            {availableYears.map((y) => (
              <option key={y} value={y}>{y}</option>
            ))}
          </select>
        )}
      </div>

      {/* ── Hero heading ── */}
      <div style={styles.hero}>
        <div style={styles.heroYear}>{selectedYear}</div>
        <div style={styles.heroLabel}>IN REVIEW</div>
        <div style={styles.heroDivider} />
      </div>

      {loading && (
        <div style={styles.loadingMsg}>Loading…</div>
      )}

      {error && (
        <div style={styles.errorMsg} role="alert">{error}</div>
      )}

      {isEmpty && !loading && (
        <div style={styles.emptyState}>
          <Gamepad2 size={48} style={{ opacity: 0.2 }} />
          <p>No sessions recorded for {selectedYear}.</p>
          <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
            Start playing games and your {selectedYear} recap will appear here.
          </p>
        </div>
      )}

      {!loading && !error && data && !isEmpty && (
        <div style={styles.content}>

          {/* ── 1. Stat cards ── */}
          <div style={styles.statRow}>
            <StatCard
              icon={<Clock size={18} />}
              label="Total Playtime"
              value={fmtHours(data.total_playtime_secs)}
              sub={`${data.sessions} sessions`}
            />
            <StatCard
              icon={<Gamepad2 size={18} />}
              label="Games Played"
              value={String(data.games_played)}
              sub={`${data.games_completed} completed`}
            />
            <StatCard
              icon={<TrendingUp size={18} />}
              label="Games Added"
              value={String(data.games_added)}
            />
            <StatCard
              icon={<Trophy size={18} />}
              label="Completion Rate"
              value={fmtPct(data.completion_rate)}
              sub={`${data.games_completed} of ${data.games_played}`}
            />
          </div>

          {/* ── 2. Top Games ── */}
          {data.top_games.length > 0 && (
            <section aria-label="Top games">
              <SectionHeading title="TOP GAMES" />
              <div style={styles.barList}>
                {data.top_games.map((g, i) => (
                  <div
                    key={g.game_id}
                    style={styles.barRow}
                    role="button"
                    tabIndex={0}
                    onClick={() => navigate(`/library/${g.game_id}`)}
                    onKeyDown={(e) => e.key === "Enter" && navigate(`/library/${g.game_id}`)}
                  >
                    <span style={styles.barRank}>{i + 1}</span>
                    <span style={styles.barLabel}>{g.title}</span>
                    <div style={styles.barTrack}>
                      <div
                        style={{
                          ...styles.barFill,
                          width: `${(g.total_playtime_secs / maxGameSecs) * 100}%`,
                        }}
                      />
                    </div>
                    <span style={styles.barValue}>{fmtHours(g.total_playtime_secs)}</span>
                  </div>
                ))}
              </div>
            </section>
          )}

          {/* ── 3. Genre Breakdown ── */}
          {data.top_genres.length > 0 && (
            <section aria-label="Genre breakdown">
              <SectionHeading title="GENRE BREAKDOWN" />
              <div style={styles.genreGrid}>
                {data.top_genres.map((g) => {
                  const pct = Math.round((g.total_playtime_secs / maxGenreSecs) * 100);
                  return (
                    <div key={g.genre} style={styles.genreRow}>
                      <span style={styles.genreName}>{g.genre}</span>
                      <div style={styles.genreTrack}>
                        <div
                          style={{ ...styles.genreFill, width: `${pct}%` }}
                        />
                      </div>
                      <span style={styles.genrePct}>{pct}%</span>
                      <span style={styles.genreSub}>{fmtHours(g.total_playtime_secs)}</span>
                    </div>
                  );
                })}
              </div>
            </section>
          )}

          {/* ── 4. Monthly Playtime chart ── */}
          {monthly.some((m) => m.secs > 0) && (
            <section aria-label="Monthly playtime">
              <SectionHeading title="MONTHLY PLAYTIME" />
              <div style={styles.monthlyChart}>
                {monthly.map((m) => {
                  const heightPct = (m.secs / maxMonthSecs) * 100;
                  const isActive  = data.most_active_month === m.month;
                  return (
                    <div key={m.month} style={styles.monthCol}>
                      <div
                        title={`${MONTHS[m.month - 1]}: ${fmtHours(m.secs)}`}
                        style={styles.monthBarWrap}
                      >
                        <div
                          style={{
                            ...styles.monthBar,
                            height: `${Math.max(heightPct, 2)}%`,
                            background: isActive
                              ? "var(--accent, #6366f1)"
                              : "color-mix(in srgb, var(--accent, #6366f1) 45%, var(--surface-raised))",
                          }}
                        />
                      </div>
                      <span style={styles.monthLabel}>{MONTHS[m.month - 1]}</span>
                    </div>
                  );
                })}
              </div>
            </section>
          )}

          {/* ── 5. Highlights row ── */}
          <section aria-label="Highlights">
            <SectionHeading title="HIGHLIGHTS" />
            <div style={styles.highlightRow}>
              {data.longest_session_secs > 0 && (
                <div style={styles.highlight}>
                  <div style={styles.highlightValue}>{fmtHours(data.longest_session_secs)}</div>
                  <div style={styles.highlightLabel}>Longest session</div>
                </div>
              )}
              {data.most_active_month !== null && (
                <div style={styles.highlight}>
                  <div style={styles.highlightValue}>{MONTHS[(data.most_active_month ?? 1) - 1]}</div>
                  <div style={styles.highlightLabel}>Most active month</div>
                </div>
              )}
              <div style={styles.highlight}>
                <div style={styles.highlightValue}>{data.sessions}</div>
                <div style={styles.highlightLabel}>Sessions logged</div>
              </div>
            </div>
          </section>

        </div>
      )}
    </div>
  );
}

// ─── Styles ───────────────────────────────────────────────────────────────────

const styles: Record<string, React.CSSProperties> = {
  page: {
    minHeight:  "100vh",
    background: "var(--bg, #0f0f17)",
    color:      "var(--text-primary, #e2e2e9)",
    padding:    "0 0 80px",
    fontFamily: "var(--font-body, Inter, sans-serif)",
  },
  topBar: {
    display:        "flex",
    alignItems:     "center",
    justifyContent: "space-between",
    padding:        "20px 32px 0",
  },
  backBtn: {
    display:    "inline-flex",
    alignItems: "center",
    gap:        6,
    background: "none",
    border:     "none",
    color:      "var(--text-muted, #888)",
    fontSize:   13,
    cursor:     "pointer",
    padding:    0,
    transition: "color 150ms",
  },
  yearSelect: {
    background:   "var(--surface-raised, #1e1e28)",
    border:       "1px solid var(--border, #2e2e3e)",
    color:        "var(--text-primary, #e2e2e9)",
    borderRadius: 6,
    padding:      "6px 12px",
    fontSize:     14,
    cursor:       "pointer",
  },
  hero: {
    textAlign: "center",
    padding:   "48px 32px 0",
  },
  heroYear: {
    fontSize:      72,
    fontWeight:    800,
    letterSpacing: "-2px",
    lineHeight:    1,
    background:    "linear-gradient(135deg, var(--accent, #6366f1) 0%, #a78bfa 100%)",
    WebkitBackgroundClip: "text",
    WebkitTextFillColor: "transparent",
    backgroundClip: "text",
  },
  heroLabel: {
    fontSize:      18,
    fontWeight:    700,
    letterSpacing: "0.3em",
    color:         "var(--text-muted, #888)",
    marginTop:     4,
  },
  heroDivider: {
    width:     80,
    height:    2,
    background:"var(--accent, #6366f1)",
    margin:    "24px auto 0",
    borderRadius: 1,
  },
  loadingMsg: {
    textAlign:  "center",
    padding:    "80px 32px",
    color:      "var(--text-muted, #888)",
    fontSize:   14,
  },
  errorMsg: {
    margin:       "32px auto",
    maxWidth:     500,
    background:   "color-mix(in srgb, red 10%, var(--surface-raised))",
    border:       "1px solid color-mix(in srgb, red 30%, transparent)",
    borderRadius: 8,
    padding:      "16px 20px",
    color:        "#f87171",
    fontSize:     13,
  },
  emptyState: {
    textAlign:  "center",
    padding:    "80px 32px",
    color:      "var(--text-muted, #888)",
    lineHeight: 1.8,
  },
  content: {
    maxWidth: 860,
    margin:   "0 auto",
    padding:  "40px 32px",
    display:  "flex",
    flexDirection: "column",
    gap:      48,
  },
  // Stat cards
  statRow: {
    display:             "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))",
    gap:                 16,
  },
  statCard: {
    background:   "var(--surface-raised, #1e1e28)",
    border:       "1px solid var(--border, #2e2e3e)",
    borderRadius: 12,
    padding:      "20px 16px",
    textAlign:    "center",
    display:      "flex",
    flexDirection: "column",
    alignItems:   "center",
    gap:          4,
  },
  statIcon: {
    color:        "var(--accent, #6366f1)",
    marginBottom: 6,
    display:      "flex",
  },
  statValue: {
    fontSize:   28,
    fontWeight: 800,
    lineHeight: 1.1,
  },
  statLabel: {
    fontSize: 11,
    fontWeight: 600,
    letterSpacing: "0.08em",
    color:    "var(--text-muted, #888)",
    textTransform: "uppercase",
    marginTop: 2,
  },
  statSub: {
    fontSize: 11,
    color:    "var(--text-muted, #888)",
    marginTop: 2,
  },
  // Section headings
  sectionHeading: {
    display:    "flex",
    alignItems: "center",
    gap:        12,
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize:      11,
    fontWeight:    700,
    letterSpacing: "0.12em",
    color:         "var(--text-muted, #888)",
    whiteSpace:    "nowrap",
  },
  sectionRule: {
    flex:       1,
    height:     1,
    background: "var(--border, #2e2e3e)",
  },
  // Top games
  barList: {
    display:       "flex",
    flexDirection: "column",
    gap:           10,
  },
  barRow: {
    display:    "flex",
    alignItems: "center",
    gap:        12,
    cursor:     "pointer",
    padding:    "6px 0",
    borderRadius: 4,
    transition: "background 150ms",
  },
  barRank: {
    width:      20,
    textAlign:  "right",
    fontSize:   13,
    fontWeight: 700,
    color:      "var(--accent, #6366f1)",
    flexShrink: 0,
  },
  barLabel: {
    width:     180,
    fontSize:  14,
    fontWeight: 500,
    whiteSpace: "nowrap",
    overflow:   "hidden",
    textOverflow: "ellipsis",
    flexShrink: 0,
  },
  barTrack: {
    flex:         1,
    height:       6,
    background:   "var(--surface-raised, #1e1e28)",
    borderRadius: 3,
    overflow:     "hidden",
  },
  barFill: {
    height:       "100%",
    background:   "linear-gradient(90deg, var(--accent, #6366f1), #a78bfa)",
    borderRadius: 3,
    transition:   "width 600ms cubic-bezier(.4,0,.2,1)",
  },
  barValue: {
    width:     60,
    textAlign: "right",
    fontSize:  13,
    color:     "var(--text-muted, #888)",
    flexShrink: 0,
    fontFamily: "var(--font-mono, monospace)",
  },
  // Genres
  genreGrid: {
    display:       "flex",
    flexDirection: "column",
    gap:           10,
  },
  genreRow: {
    display:    "flex",
    alignItems: "center",
    gap:        12,
  },
  genreName: {
    width:     100,
    fontSize:  13,
    fontWeight: 500,
    flexShrink: 0,
  },
  genreTrack: {
    flex:         1,
    height:       8,
    background:   "var(--surface-raised, #1e1e28)",
    borderRadius: 4,
    overflow:     "hidden",
  },
  genreFill: {
    height:       "100%",
    background:   "linear-gradient(90deg, #a78bfa, #6366f1)",
    borderRadius: 4,
    transition:   "width 600ms cubic-bezier(.4,0,.2,1)",
  },
  genrePct: {
    width:      36,
    textAlign:  "right",
    fontSize:   12,
    color:      "var(--text-muted, #888)",
    fontFamily: "var(--font-mono, monospace)",
    flexShrink: 0,
  },
  genreSub: {
    width:      52,
    textAlign:  "right",
    fontSize:   11,
    color:      "var(--text-muted, #888)",
    flexShrink: 0,
  },
  // Monthly chart
  monthlyChart: {
    display:    "flex",
    alignItems: "flex-end",
    gap:        6,
    height:     120,
  },
  monthCol: {
    flex:          1,
    display:       "flex",
    flexDirection: "column",
    alignItems:    "center",
    gap:           4,
    height:        "100%",
  },
  monthBarWrap: {
    flex:          1,
    width:         "100%",
    display:       "flex",
    alignItems:    "flex-end",
  },
  monthBar: {
    width:        "100%",
    borderRadius: "3px 3px 0 0",
    transition:   "height 600ms cubic-bezier(.4,0,.2,1)",
    minHeight:    2,
  },
  monthLabel: {
    fontSize:   9,
    color:      "var(--text-muted, #888)",
    letterSpacing: "0.04em",
  },
  // Highlights
  highlightRow: {
    display:             "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))",
    gap:                 16,
  },
  highlight: {
    background:   "var(--surface-raised, #1e1e28)",
    border:       "1px solid var(--border, #2e2e3e)",
    borderRadius: 10,
    padding:      "16px",
    textAlign:    "center",
  },
  highlightValue: {
    fontSize:   24,
    fontWeight: 800,
    color:      "var(--accent, #6366f1)",
  },
  highlightLabel: {
    fontSize:  11,
    color:     "var(--text-muted, #888)",
    marginTop: 4,
    textTransform: "uppercase",
    letterSpacing: "0.08em",
  },
};

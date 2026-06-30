/**
 * LibraryPage — the game archive.
 *
 * UI state is now managed by useLibraryStore (Zustand) so search/filter/view
 * preferences persist when navigating away and back.
 *
 * Features:
 * - Live search (debounced 200ms)
 * - Status + favorites-only filters
 * - Grid / List view toggle
 * - Empty state with CTA
 */

import { useCallback, useEffect, useMemo, useRef } from "react";
import { useNavigate } from "react-router-dom";
import {
  LayoutGrid,
  List,
  Plus,
  Search,
  Star,
  FolderSearch,
  X,
} from "lucide-react";

import { GameCard }        from "@/components/GameCard";
import { GameListRow }     from "@/components/GameListRow";
import { EnrichmentProgressBar } from "@/components/EnrichmentProgressBar";
import { getAllGames, bulkEnrichLibrary }     from "@/lib/api";
import { useLibraryStore } from "@/stores/useLibraryStore";
import { useEnrichmentProgress } from "@/hooks/useEnrichmentProgress";
import { useToastStore } from "@/stores/useToastStore";
import type { Game, GameStatus } from "@/types";
import { useState } from "react";

type SortKey = "title" | "playtime" | "last_played" | "added";

const STATUS_OPTIONS: { value: GameStatus; label: string }[] = [
  { value: "playing",   label: "Playing"   },
  { value: "unplayed",  label: "Unplayed"  },
  { value: "completed", label: "Completed" },
  { value: "dropped",   label: "Dropped"   },
];

export default function LibraryPage() {
  const navigate = useNavigate();

  // ── Data ────────────────────────────────────────────────────────────────────
  const [games,   setGames]   = useState<Game[]>([]);
  const [loading, setLoading] = useState(true);
  const [error,   setError]   = useState<string | null>(null);

  // ── Enrichment ──────────────────────────────────────────────────────────────
  const { progress, isActive: enrichmentActive, reset: resetEnrichment } = useEnrichmentProgress();
  const [enriching, setEnriching] = useState(false);
  const { addToast } = useToastStore();

  // ── UI state — from Zustand store (persists across navigation) ───────────────
  const {
    searchQuery,
    statusFilter,
    favoritesOnly,
    viewMode,
    sortKey,
    setSearchQuery,
    setStatusFilter,
    setFavoritesOnly,
    setViewMode,
    setSortKey,
    clearFilters,
  } = useLibraryStore();

  // Debounced search (200ms) — local ref so we don't need extra state
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [debouncedQuery, setDebouncedQuery] = useState(searchQuery);

  const handleSearchChange = (q: string) => {
    setSearchQuery(q);
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => setDebouncedQuery(q), 200);
  };

  // Keep debouncedQuery in sync if the store query changes externally
  useEffect(() => {
    setDebouncedQuery(searchQuery);
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // Load library
  const loadGames = useCallback(async () => {
    setLoading(true);
    try {
      const data = await getAllGames();
      setGames(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { loadGames(); }, [loadGames]);

  // Enrichment handler
  const handleEnrichLibrary = async () => {
    try {
      setEnriching(true);
      await bulkEnrichLibrary();
      addToast("Library enrichment started", "info");
    } catch (e) {
      addToast(`Enrichment failed: ${e}`, "error");
    } finally {
      setEnriching(false);
    }
  };

  // Game update callback (favorites, etc.)
  const handleGameUpdate = useCallback((updated: Game) => {
    setGames((prev) => prev.map((g) => g.id === updated.id ? updated : g));
  }, []);

  // ── Filtering + sorting (client-side) ───────────────────────────────────────
  const displayed = useMemo(() => {
    let list = [...games];

    if (debouncedQuery.trim()) {
      const q = debouncedQuery.toLowerCase();
      list = list.filter(
        (g) =>
          g.title.toLowerCase().includes(q) ||
          g.developer?.toLowerCase().includes(q) ||
          g.genre?.toLowerCase().includes(q)
      );
    }

    if (statusFilter) {
      list = list.filter((g) => g.status === statusFilter);
    }

    if (favoritesOnly) {
      list = list.filter((g) => g.is_favorite);
    }

    list.sort((a, b) => {
      switch (sortKey as SortKey) {
        case "title":
          return a.title.localeCompare(b.title);
        case "playtime":
          return b.total_playtime_secs - a.total_playtime_secs;
        case "last_played": {
          const ta = a.last_played ? new Date(a.last_played).getTime() : 0;
          const tb = b.last_played ? new Date(b.last_played).getTime() : 0;
          return tb - ta;
        }
        case "added":
          return new Date(b.added_at).getTime() - new Date(a.added_at).getTime();
        default:
          return 0;
      }
    });

    return list;
  }, [games, debouncedQuery, statusFilter, favoritesOnly, sortKey]);

  const hasFilters = debouncedQuery || statusFilter || favoritesOnly;

  // ── Render ──────────────────────────────────────────────────────────────────
  return (
    <div className="atlas-enter" style={styles.page}>
      {/* Progress bar */}
      {progress && enrichmentActive && (
        <EnrichmentProgressBar progress={progress} onDismiss={resetEnrichment} />
      )}

      {/* ── Page header ──────────────────────────────────────────────────── */}
      <div style={styles.header}>
        <div>
          <h1 style={styles.title}>Library</h1>
          <p style={styles.subtitle}>
            {loading
              ? "Loading…"
              : `${games.length} game${games.length !== 1 ? "s" : ""}`}
          </p>
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          <button
            onClick={handleEnrichLibrary}
            disabled={enriching || enrichmentActive}
            style={{
              ...styles.enrichBtn,
              opacity: enriching || enrichmentActive ? 0.5 : 1,
            }}
            aria-label="Enrich library with metadata"
          >
            <Star size={14} />
            {enriching ? "Enriching..." : "Enrich Library"}
          </button>
          <button
            id="scan-folder-btn"
            onClick={() => navigate("/library/scan")}
            style={styles.scanBtn}
            aria-label="Scan a folder for games"
          >
            <FolderSearch size={14} />
            Scan Folder
          </button>
          <button
            id="add-game-btn"
            onClick={() => navigate("/library/add")}
            style={styles.addBtn}
            aria-label="Add a game to your library"
          >
            <Plus size={14} />
            Add Game
          </button>
        </div>
      </div>

      {/* ── Toolbar ──────────────────────────────────────────────────────── */}
      <div style={styles.toolbar}>
        {/* Search */}
        <div style={styles.searchWrapper}>
          <Search size={14} style={styles.searchIcon} aria-hidden="true" />
          <input
            id="library-search"
            type="search"
            placeholder="Search games…"
            value={searchQuery}
            onChange={(e) => handleSearchChange(e.target.value)}
            style={styles.searchInput}
            aria-label="Search library"
          />
          {searchQuery && (
            <button
              onClick={() => { setSearchQuery(""); setDebouncedQuery(""); }}
              style={styles.clearBtn}
              aria-label="Clear search"
            >
              <X size={12} />
            </button>
          )}
        </div>

        {/* Status filter chips */}
        <div
          style={styles.filters}
          role="group"
          aria-label="Filter by status"
        >
          {STATUS_OPTIONS.map(({ value, label }) => (
            <button
              key={value}
              onClick={() => setStatusFilter(statusFilter === value ? null : value)}
              style={{
                ...styles.chip,
                ...(statusFilter === value ? styles.chipActive : {}),
              }}
              aria-pressed={statusFilter === value}
              aria-label={`Filter by ${label}`}
            >
              {label}
            </button>
          ))}

          <button
            onClick={() => setFavoritesOnly(!favoritesOnly)}
            style={{
              ...styles.chip,
              ...(favoritesOnly ? styles.chipActive : {}),
              display: "flex",
              alignItems: "center",
              gap: 4,
            }}
            aria-pressed={favoritesOnly}
            aria-label="Show favorites only"
          >
            <Star size={11} fill={favoritesOnly ? "currentColor" : "none"} aria-hidden="true" />
            Favorites
          </button>
        </div>

        {/* Sort + view toggle */}
        <div style={styles.controls}>
          <select
            id="library-sort"
            value={sortKey}
            onChange={(e) => setSortKey(e.target.value as SortKey)}
            style={styles.select}
            aria-label="Sort games by"
          >
            <option value="title">Title</option>
            <option value="playtime">Playtime</option>
            <option value="last_played">Last played</option>
            <option value="added">Date added</option>
          </select>

          <div style={styles.viewToggle} role="group" aria-label="View mode">
            <button
              id="view-grid-btn"
              onClick={() => setViewMode("grid")}
              style={{
                ...styles.viewBtn,
                ...(viewMode === "grid" ? styles.viewBtnActive : {}),
              }}
              aria-label="Grid view"
              aria-pressed={viewMode === "grid"}
            >
              <LayoutGrid size={14} aria-hidden="true" />
            </button>
            <button
              id="view-list-btn"
              onClick={() => setViewMode("list")}
              style={{
                ...styles.viewBtn,
                ...(viewMode === "list" ? styles.viewBtnActive : {}),
              }}
              aria-label="List view"
              aria-pressed={viewMode === "list"}
            >
              <List size={14} aria-hidden="true" />
            </button>
          </div>
        </div>
      </div>

      {/* ── Error ────────────────────────────────────────────────────────── */}
      {error && (
        <p style={{ color: "var(--color-text-muted)", fontSize: 13 }} role="alert">
          Failed to load library: {error}
        </p>
      )}

      {/* ── Empty state (no games at all) ─────────────────────────────────── */}
      {!loading && !error && games.length === 0 && (
        <div style={styles.emptyState}>
          <span style={styles.emptyMono}>No games yet</span>
          <p style={styles.emptyBody}>
            Add your first game to begin preserving your gaming history.
          </p>
          <button
            id="add-first-game-btn"
            onClick={() => navigate("/library/add")}
            style={styles.addBtn}
          >
            <Plus size={14} aria-hidden="true" />
            Add Game
          </button>
        </div>
      )}

      {/* ── No results (filtered) ─────────────────────────────────────────── */}
      {!loading && !error && games.length > 0 && displayed.length === 0 && (
        <div style={styles.emptyState}>
          <span style={styles.emptyMono}>No results</span>
          <p style={styles.emptyBody}>No games match your current filters.</p>
          {hasFilters && (
            <button onClick={clearFilters} style={styles.ghostBtn}>
              Clear filters
            </button>
          )}
        </div>
      )}

      {/* ── Grid view ────────────────────────────────────────────────────── */}
      {!loading && viewMode === "grid" && displayed.length > 0 && (
        <div style={styles.grid} role="list" aria-label="Game library">
          {displayed.map((game) => (
            <div key={game.id} role="listitem">
              <GameCard game={game} onUpdate={handleGameUpdate} />
            </div>
          ))}
        </div>
      )}

      {/* ── List view ────────────────────────────────────────────────────── */}
      {!loading && viewMode === "list" && displayed.length > 0 && (
        <table style={styles.table} aria-label="Game library">
          <thead>
            <tr>
              <th style={styles.th} scope="col" />
              <th style={{ ...styles.th, textAlign: "left" }} scope="col">Title</th>
              <th style={styles.th} scope="col">Status</th>
              <th style={styles.th} scope="col">Playtime</th>
              <th style={styles.th} scope="col">Last played</th>
              <th style={styles.th} scope="col" />
            </tr>
          </thead>
          <tbody>
            {displayed.map((game) => (
              <GameListRow key={game.id} game={game} onUpdate={handleGameUpdate} />
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

// ── Styles ────────────────────────────────────────────────────────────────────

const styles = {
  page: {
    padding:    "40px 56px",
    overflowY:  "auto" as const,
    height:     "100%",
    boxSizing:  "border-box" as const,
  },
  header: {
    display:        "flex",
    justifyContent: "space-between",
    alignItems:     "flex-start",
    marginBottom:   40,
  },
  title: {
    fontFamily:    "var(--font-display)",
    fontSize:      "clamp(48px, 5vw, 80px)",
    fontWeight:    700,
    letterSpacing: "-0.03em",
    lineHeight:    1.0,
    color:         "var(--color-text-primary)",
    margin:        0,
    marginBottom:  8,
  },
  subtitle: {
    fontFamily:    "var(--font-mono)",
    fontSize:      12,
    color:         "var(--color-text-disabled)",
    letterSpacing: "0.08em",
    margin:        0,
  },
  scanBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           6,
    background:    "none",
    color:         "var(--color-text-muted)",
    border:        "1px solid var(--color-border)",
    padding:       "9px 18px",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    fontWeight:    500,
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    cursor:        "pointer",
    borderRadius:  1,
    transition:    "color 150ms, border-color 150ms",
    flexShrink:    0,
  },
  addBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           6,
    background:    "var(--color-text-primary)",
    color:         "var(--color-base)",
    border:        "none",
    padding:       "9px 20px",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    fontWeight:    600,
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    cursor:        "pointer",
    borderRadius:  1,
    transition:    "opacity 150ms",
    flexShrink:    0,
  },
  enrichBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           6,
    background:    "none",
    color:         "var(--color-text-secondary)",
    border:        "1px solid var(--color-text-secondary)",
    padding:       "9px 18px",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    fontWeight:    500,
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    cursor:        "pointer",
    borderRadius:  1,
    transition:    "opacity 150ms",
    flexShrink:    0,
  },
  toolbar: {
    display:       "flex",
    alignItems:    "center",
    gap:           12,
    marginBottom:  32,
    flexWrap:      "wrap" as const,
    borderBottom:  "1px solid var(--color-border)",
    paddingBottom: 20,
  },
  searchWrapper: {
    position:  "relative" as const,
    flexShrink: 0,
  },
  searchIcon: {
    position:      "absolute" as const,
    left:          10,
    top:           "50%",
    transform:     "translateY(-50%)",
    color:         "var(--color-text-disabled)",
    pointerEvents: "none" as const,
  },
  searchInput: {
    background:   "var(--color-surface)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "8px 32px 8px 32px",
    fontSize:     13,
    fontFamily:   "var(--font-body)",
    color:        "var(--color-text-primary)",
    width:        200,
    outline:      "none",
  },
  clearBtn: {
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
  filters: {
    display:  "flex",
    gap:      6,
    flexWrap: "wrap" as const,
    flex:     1,
  },
  chip: {
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
    textTransform: "uppercase" as const,
  },
  chipActive: {
    borderColor: "var(--color-text-secondary)",
    color:       "var(--color-text-primary)",
  },
  controls: {
    display:    "flex",
    gap:        8,
    alignItems: "center",
    flexShrink: 0,
  },
  select: {
    background:    "var(--color-surface)",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "7px 10px",
    fontSize:      12,
    fontFamily:    "var(--font-mono)",
    letterSpacing: "0.04em",
    color:         "var(--color-text-muted)",
    cursor:        "pointer",
    outline:       "none",
  },
  viewToggle: {
    display:      "flex",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    overflow:     "hidden",
  },
  viewBtn: {
    background: "var(--color-surface)",
    border:     "none",
    padding:    "7px 10px",
    color:      "var(--color-text-disabled)",
    cursor:     "pointer",
    display:    "flex",
    transition: "color 150ms, background 150ms",
  },
  viewBtnActive: {
    background: "var(--color-elevated)",
    color:      "var(--color-text-primary)",
  },
  emptyState: {
    display:        "flex",
    flexDirection:  "column" as const,
    alignItems:     "center",
    justifyContent: "center",
    padding:        "96px 0",
    gap:            16,
    borderTop:      "1px solid var(--color-border)",
  },
  emptyMono: {
    fontFamily:    "var(--font-mono)",
    fontSize:      13,
    color:         "var(--color-text-muted)",
    letterSpacing: "0.1em",
    textTransform: "uppercase" as const,
  },
  emptyBody: {
    fontFamily: "var(--font-body)",
    fontSize:   14,
    color:      "var(--color-text-muted)",
    margin:     0,
    textAlign:  "center" as const,
    maxWidth:   320,
    lineHeight: 1.6,
  },
  ghostBtn: {
    background:    "none",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "8px 20px",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    color:         "var(--color-text-muted)",
    cursor:        "pointer",
    letterSpacing: "0.04em",
    transition:    "border-color 150ms",
  },
  grid: {
    display:             "grid",
    gridTemplateColumns: "repeat(auto-fill, minmax(160px, 1fr))",
    gap:                 24,
  },
  table: {
    width:          "100%",
    borderCollapse: "collapse" as const,
    tableLayout:    "fixed" as const,
  },
  th: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.12em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    padding:       "0 24px 12px 0",
    fontWeight:    500,
    textAlign:     "center" as const,
    borderBottom:  "1px solid var(--color-border)",
  },
} satisfies Record<string, React.CSSProperties>;

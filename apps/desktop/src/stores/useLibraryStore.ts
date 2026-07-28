/**
 * useLibraryStore — client-side library UI state.
 *
 * Manages search query, filters, view mode, and sort preferences.
 * Not persisted — resets on page reload (intentional: fresh state each session).
 *
 * T59: Extended with genre, developer, playtime-range, and never-played filters.
 */

import { create } from "zustand";
import type { GameStatus } from "@/types";

export type ViewMode = "grid" | "list";
export type SortKey  = "title" | "playtime" | "last_played" | "added";

interface LibraryStore {
  // ── Existing filters ─────────────────────────────────────────────────────────
  searchQuery:    string;
  statusFilter:   GameStatus | null;
  favoritesOnly:  boolean;
  viewMode:       ViewMode;
  sortKey:        SortKey;

  // ── T59: Advanced filters ─────────────────────────────────────────────────────
  /** Exact match on comma-split genre segments (e.g. "RPG" matches "RPG, Action") */
  genreFilter:    string | null;
  /** Substring match on developer name */
  developerFilter: string | null;
  /** Minimum playtime in minutes (null = no lower bound) */
  minPlaytime:    number | null;
  /** Maximum playtime in minutes (null = no upper bound) */
  maxPlaytime:    number | null;
  /** When true, show only games with total_playtime_secs === 0 */
  neverPlayed:    boolean;

  // ── Actions ───────────────────────────────────────────────────────────────────
  setSearchQuery:    (q: string)              => void;
  setStatusFilter:   (s: GameStatus | null)   => void;
  setFavoritesOnly:  (v: boolean)             => void;
  setViewMode:       (m: ViewMode)            => void;
  setSortKey:        (k: SortKey)             => void;
  // T59
  setGenreFilter:    (g: string | null)       => void;
  setDeveloperFilter:(d: string | null)       => void;
  setMinPlaytime:    (m: number | null)       => void;
  setMaxPlaytime:    (m: number | null)       => void;
  setNeverPlayed:    (v: boolean)             => void;
  clearFilters:      ()                       => void;

  /** T59: Count of active advanced filters (for the badge on the toggle button). */
  advancedFilterCount: () => number;
}

export const useLibraryStore = create<LibraryStore>((set, get) => ({
  searchQuery:     "",
  statusFilter:    null,
  favoritesOnly:   false,
  viewMode:        "grid",
  sortKey:         "title",
  // T59
  genreFilter:     null,
  developerFilter: null,
  minPlaytime:     null,
  maxPlaytime:     null,
  neverPlayed:     false,

  setSearchQuery:    (q) => set({ searchQuery: q }),
  setStatusFilter:   (s) => set({ statusFilter: s }),
  setFavoritesOnly:  (v) => set({ favoritesOnly: v }),
  setViewMode:       (m) => set({ viewMode: m }),
  setSortKey:        (k) => set({ sortKey: k }),
  // T59
  setGenreFilter:    (g) => set({ genreFilter: g }),
  setDeveloperFilter:(d) => set({ developerFilter: d }),
  setMinPlaytime:    (m) => set({ minPlaytime: m }),
  setMaxPlaytime:    (m) => set({ maxPlaytime: m }),
  setNeverPlayed:    (v) => set({ neverPlayed: v }),

  clearFilters: () => set({
    searchQuery:     "",
    statusFilter:    null,
    favoritesOnly:   false,
    genreFilter:     null,
    developerFilter: null,
    minPlaytime:     null,
    maxPlaytime:     null,
    neverPlayed:     false,
  }),

  advancedFilterCount: () => {
    const s = get();
    let count = 0;
    if (s.genreFilter)        count++;
    if (s.developerFilter)    count++;
    if (s.minPlaytime != null) count++;
    if (s.maxPlaytime != null) count++;
    if (s.neverPlayed)        count++;
    return count;
  },
}));

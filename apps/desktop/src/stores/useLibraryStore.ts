/**
 * useLibraryStore — client-side library UI state.
 *
 * Manages search query, filters, view mode, and sort preferences.
 * Not persisted — resets on page reload (intentional: fresh state each session).
 */

import { create } from "zustand";
import type { GameStatus } from "@/types";

export type ViewMode = "grid" | "list";
export type SortKey  = "title" | "playtime" | "last_played" | "added";

interface LibraryStore {
  searchQuery:    string;
  statusFilter:   GameStatus | null;
  favoritesOnly:  boolean;
  viewMode:       ViewMode;
  sortKey:        SortKey;

  setSearchQuery:   (q: string) => void;
  setStatusFilter:  (s: GameStatus | null) => void;
  setFavoritesOnly: (v: boolean) => void;
  setViewMode:      (m: ViewMode) => void;
  setSortKey:       (k: SortKey) => void;
  clearFilters:     () => void;
}

export const useLibraryStore = create<LibraryStore>((set) => ({
  searchQuery:   "",
  statusFilter:  null,
  favoritesOnly: false,
  viewMode:      "grid",
  sortKey:       "title",

  setSearchQuery:   (q)  => set({ searchQuery: q }),
  setStatusFilter:  (s)  => set({ statusFilter: s }),
  setFavoritesOnly: (v)  => set({ favoritesOnly: v }),
  setViewMode:      (m)  => set({ viewMode: m }),
  setSortKey:       (k)  => set({ sortKey: k }),
  clearFilters:     ()   => set({ searchQuery: "", statusFilter: null, favoritesOnly: false }),
}));

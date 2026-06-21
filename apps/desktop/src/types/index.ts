// Re-export all shared types so the frontend can import from @/types
// e.g. import type { Game, NewGame } from "@/types"
//
// The canonical definitions live in packages/shared/index.ts.
// These are duplicated here because the shared package is not yet
// installed as a workspace dependency in this monorepo.

export type GameStatus = "unplayed" | "playing" | "completed" | "dropped";

export interface Game {
  id:                  string;
  title:               string;
  exe_path:            string;
  cover_path:          string | null;
  banner_path:         string | null;
  developer:           string | null;
  publisher:           string | null;
  genre:               string | null;
  is_favorite:         boolean;
  added_at:            string;
  last_played:         string | null;
  total_playtime_secs: number;
  launch_count:        number;
  status:              GameStatus;
}

export interface NewGame {
  title:        string;
  exe_path:     string;
  cover_path?:  string | null;
  banner_path?: string | null;
  developer?:   string | null;
  publisher?:   string | null;
  genre?:       string | null;
  status?:      GameStatus;
}

export interface UpdateGame {
  title?:       string;
  exe_path?:    string;
  cover_path?:  string | null;
  banner_path?: string | null;
  developer?:   string | null;
  publisher?:   string | null;
  genre?:       string | null;
  status?:      GameStatus;
  is_favorite?: boolean;
}

export interface GameFilters {
  query?:          string;
  status?:         GameStatus;
  genre?:          string;
  favorites_only?: boolean;
}

export interface Session {
  id:            string;
  game_id:       string;
  started_at:    string;
  ended_at:      string | null;
  duration_secs: number;
}

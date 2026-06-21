// ── Shared TypeScript Types ───────────────────────────────────────────────────
// These mirror the Rust structs in src-tauri/src/models.rs exactly.
// This is the single source of truth for the data contract between
// the Tauri backend and the React frontend.

// ── Game Status ───────────────────────────────────────────────────────────────

export type GameStatus = "unplayed" | "playing" | "completed" | "dropped";

// ── Game ─────────────────────────────────────────────────────────────────────

/** Full game record returned from the Rust backend. */
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
  added_at:            string;   // ISO 8601
  last_played:         string | null;
  total_playtime_secs: number;
  launch_count:        number;
  status:              GameStatus;
}

/** Payload to create a new game. */
export interface NewGame {
  title:       string;
  exe_path:    string;
  cover_path?: string | null;
  banner_path?: string | null;
  developer?:  string | null;
  publisher?:  string | null;
  genre?:      string | null;
  status?:     GameStatus;
}

/** Partial update payload — all fields optional. */
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

/** Filter parameters for the library query. */
export interface GameFilters {
  query?:          string;
  status?:         GameStatus;
  genre?:          string;
  favorites_only?: boolean;
}

// ── Session ───────────────────────────────────────────────────────────────────

/** A single recorded play session. */
export interface Session {
  id:            string;
  game_id:       string;
  started_at:    string;
  ended_at:      string | null;
  duration_secs: number;
}

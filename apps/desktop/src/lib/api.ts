/**
 * api.ts — Typed Tauri invoke() wrappers.
 *
 * Every function here maps 1:1 to a Rust #[tauri::command] registered
 * in src-tauri/src/lib.rs. Return types match the Rust model structs.
 *
 * Usage:
 *   import { getAllGames, addGame } from "@/lib/api";
 *   const games = await getAllGames();
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  Game,
  GameFilters,
  NewGame,
  Session,
  UpdateGame,
} from "@/types";

// ── Games ─────────────────────────────────────────────────────────────────────

/**
 * Fetch all games from the library, with optional filters/search.
 */
export async function getAllGames(filters?: GameFilters): Promise<Game[]> {
  return invoke<Game[]>("get_all_games", { filters: filters ?? null });
}

/**
 * Fetch a single game by ID.
 * Throws if the game does not exist.
 */
export async function getGame(id: string): Promise<Game> {
  return invoke<Game>("get_game", { id });
}

/**
 * Add a new game to the library.
 * Returns the created game record (with generated ID and timestamps).
 */
export async function addGame(newGame: NewGame): Promise<Game> {
  return invoke<Game>("add_game", { newGame });
}

/**
 * Partially update a game.
 * Only provided fields are updated; omitted fields are left unchanged.
 */
export async function updateGame(
  id: string,
  updates: UpdateGame
): Promise<Game> {
  return invoke<Game>("update_game", { id, updates });
}

/**
 * Delete a game and all its associated sessions.
 */
export async function deleteGame(id: string): Promise<void> {
  return invoke<void>("delete_game", { id });
}

/**
 * Toggle a game's favorite status.
 * Returns the updated game record.
 */
export async function toggleFavorite(id: string): Promise<Game> {
  return invoke<Game>("toggle_favorite", { id });
}

// ── Settings ──────────────────────────────────────────────────────────────────

/**
 * Get the value of a single setting key.
 * Returns null if the key has not been set.
 */
export async function getSetting(key: string): Promise<string | null> {
  return invoke<string | null>("get_setting", { key });
}

/**
 * Set (insert or update) a setting key-value pair.
 */
export async function setSetting(key: string, value: string): Promise<void> {
  return invoke<void>("set_setting", { key, value });
}

/**
 * Get all settings as a key-value map.
 */
export async function getAllSettings(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>("get_all_settings");
}

// ── Sessions ──────────────────────────────────────────────────────────────────

/**
 * Get all play sessions for a game (newest first).
 */
export async function getSessions(gameId: string): Promise<Session[]> {
  return invoke<Session[]>("get_sessions", { gameId });
}

// ── Launcher ──────────────────────────────────────────────────────────────────

/**
 * Launch a game by its library ID.
 * Spawns the process and starts background playtime tracking.
 * Emits a `game-stopped` event when the process exits.
 */
export async function launchGame(id: string): Promise<void> {
  return invoke<void>("launch_game", { id });
}

/**
 * Returns the game ID of the currently running game, or null if none.
 */
export async function getRunningGame(): Promise<string | null> {
  return invoke<string | null>("get_running_game");
}

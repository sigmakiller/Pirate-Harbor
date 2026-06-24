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

// ── Scanner ───────────────────────────────────────────────────────────────────

export interface ScanResult {
  name:         string;
  exe_path:     string;
  already_added: boolean;
}

/** Return all registered scan directories. */
export async function getScanDirectories(): Promise<string[]> {
  return invoke<string[]>("get_scan_directories");
}

/**
 * Add a directory to the watch list. Returns the full updated list.
 * Throws if the path is not a valid directory.
 */
export async function addScanDirectory(path: string): Promise<string[]> {
  return invoke<string[]>("add_scan_directory", { path });
}

/** Remove a directory from the watch list. Returns the updated list. */
export async function removeScanDirectory(path: string): Promise<string[]> {
  return invoke<string[]>("remove_scan_directory", { path });
}

/**
 * Scan a single directory and return discovered executables.
 * Already-added games are marked with `already_added: true`.
 */
export async function scanDirectory(path: string): Promise<ScanResult[]> {
  return invoke<ScanResult[]>("scan_directory", { path });
}

/**
 * Scan ALL registered directories and return a merged, deduplicated result set.
 * Returns an empty array if no directories are registered.
 */
export async function scanAllDirectories(): Promise<ScanResult[]> {
  return invoke<ScanResult[]>("scan_all_directories");
}

// ── Metadata ──────────────────────────────────────────────────────────────────

export interface MetadataResult {
  name:         string;
  genres:       string;
  cover_url:    string | null;
  release_year: number | null;
}

/**
 * Search the RAWG Video Games Database for game metadata.
 *
 * Results are cached locally for 24 hours.
 * Requires the RAWG API key to be configured in Settings → Integrations.
 *
 * @throws if no API key is configured or the network call fails.
 */
export async function searchGameMetadata(query: string): Promise<MetadataResult[]> {
  return invoke<MetadataResult[]>("search_game_metadata", { query });
}

/**
 * Return the configured RAWG API key, or null if not set.
 * Used by Settings page to pre-populate the key input.
 */
export async function getRawgApiKey(): Promise<string | null> {
  return invoke<string | null>("get_rawg_api_key");
}

// ── Collections ───────────────────────────────────────────────────────────────

export interface Collection {
  id:            string;
  name:          string;
  description:   string | null;
  cover_game_id: string | null;
  created_at:    string;
  updated_at:    string;
  game_ids:      string[];
  game_count:    number;
}

export interface NewCollection {
  name:          string;
  description?:  string | null;
  cover_game_id?: string | null;
}

export interface UpdateCollection {
  name?:          string;
  description?:   string | null;
  cover_game_id?: string | null;
}

export async function getCollections(): Promise<Collection[]> {
  return invoke<Collection[]>("get_collections");
}

export async function getCollection(id: string): Promise<Collection> {
  return invoke<Collection>("get_collection", { id });
}

export async function createCollection(payload: NewCollection): Promise<Collection> {
  return invoke<Collection>("create_collection", { payload });
}

export async function updateCollection(id: string, payload: UpdateCollection): Promise<Collection> {
  return invoke<Collection>("update_collection", { id, payload });
}

export async function deleteCollection(id: string): Promise<void> {
  return invoke<void>("delete_collection", { id });
}

export async function addGameToCollection(collectionId: string, gameId: string): Promise<Collection> {
  return invoke<Collection>("add_game_to_collection", { collectionId, gameId });
}

export async function removeGameFromCollection(collectionId: string, gameId: string): Promise<Collection> {
  return invoke<Collection>("remove_game_from_collection", { collectionId, gameId });
}

export async function getGameCollections(gameId: string): Promise<string[]> {
  return invoke<string[]>("get_game_collections", { gameId });
}

// ── Journal ───────────────────────────────────────────────────────────────────

export type EntryType = "note" | "milestone" | "session";

export interface JournalEntry {
  id:         string;
  game_id:    string | null;
  game_title: string | null;
  title:      string | null;
  body:       string;
  entry_type: EntryType;
  created_at: string;
  updated_at: string;
}

export interface NewJournalEntry {
  game_id?:    string | null;
  title?:      string | null;
  body:        string;
  entry_type?: EntryType;
}

export interface UpdateJournalEntry {
  title?:      string | null;
  body?:       string;
  entry_type?: EntryType;
}

export async function getJournalEntries(
  gameId?: string | null,
  limit?: number
): Promise<JournalEntry[]> {
  return invoke<JournalEntry[]>("get_journal_entries", {
    gameId: gameId ?? null,
    limit:  limit  ?? null,
  });
}

export async function createJournalEntry(payload: NewJournalEntry): Promise<JournalEntry> {
  return invoke<JournalEntry>("create_journal_entry", { payload });
}

export async function updateJournalEntry(
  id: string,
  payload: UpdateJournalEntry
): Promise<JournalEntry> {
  return invoke<JournalEntry>("update_journal_entry", { id, payload });
}

export async function deleteJournalEntry(id: string): Promise<void> {
  return invoke<void>("delete_journal_entry", { id });
}

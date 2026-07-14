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
  name:          string;
  exe_path:      string;
  already_added: boolean;
  /** Heuristic confidence score 0.0–1.0. Higher = more likely a game. */
  confidence:    number;
  /** File size in megabytes. */
  size_mb:       number;
  /** Parent folder name (e.g. "TheWitcher3"). */
  folder_name:   string;
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

/**
 * Bulk-insert scan results as library games, skipping duplicates by exe_path.
 * Returns the list of successfully added Game records.
 */
export async function batchAddGames(games: NewGame[]): Promise<Game[]> {
  return invoke<Game[]>("batch_add_games", { games });
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

/**
 * Trigger background enrichment for the entire library.
 * Fetches metadata and downloads images for all games that haven't
 * been enriched yet. Runs in the background; progress is emitted
 * via Tauri events.
 */
export async function bulkEnrichLibrary(): Promise<void> {
  return invoke<void>("bulk_enrich_library");
}

/**
 * Download and process game images (cover and background).
 * Automatically resizes and converts images to optimal formats.
 * Emits 'image-download-progress' events during processing.
 */
export async function downloadGameImages(
  gameId: string,
  coverUrl?: string | null,
  backgroundUrl?: string | null
): Promise<ImageDownloadResult> {
  return invoke<ImageDownloadResult>("download_game_images", {
    gameId,
    coverUrl: coverUrl ?? null,
    backgroundUrl: backgroundUrl ?? null,
  });
}

export interface ImageDownloadResult {
  game_id: string;
  cover_path: string | null;
  background_path: string | null;
  success: boolean;
  error: string | null;
}

// ── Collections ───────────────────────────────────────────────────────────────

export interface Collection {
  id:            string;
  name:          string;
  description:   string | null;
  /** Absolute path to a user-chosen cover image (used when cover_mode = 'custom'). */
  cover_path:    string | null;
  /** 'auto' = 2×2 mosaic from game covers | 'custom' = cover_path image. */
  cover_mode:    'auto' | 'custom';
  cover_game_id: string | null;
  created_at:    string;
  updated_at:    string;
  game_ids:      string[];
  game_count:    number;
}

export interface NewCollection {
  name:           string;
  description?:   string | null;
  cover_path?:    string | null;
  cover_mode?:    'auto' | 'custom';
  cover_game_id?: string | null;
}

export interface UpdateCollection {
  name?:          string;
  description?:   string | null;
  cover_path?:    string | null;
  cover_mode?:    'auto' | 'custom';
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

// ── Milestones ────────────────────────────────────────────────────────────────

// Import types for use within this file; also re-export so consumers can
// import from a single canonical location instead of going to @/types directly.
import type {
  Milestone,
  NewMilestone,
  MilestoneTemplate,
  NewMilestoneTemplate,
  MilestoneStatistics,
} from "@/types";

export type {
  Milestone,
  NewMilestone,
  MilestoneTemplate,
  NewMilestoneTemplate,
  MilestoneStatistics,
};

/**
 * Create a new milestone.
 */
export async function createMilestone(payload: NewMilestone): Promise<Milestone> {
  return invoke<Milestone>("create_milestone", { payload });
}

/**
 * Get milestones with optional filters.
 * @param gameId - Filter by game ID
 * @param category - Filter by category
 * @param limit - Maximum number of results
 */
export async function getMilestones(
  gameId?: string | null,
  category?: string | null,
  limit?: number | null
): Promise<Milestone[]> {
  return invoke<Milestone[]>("get_milestones", {
    gameId: gameId ?? null,
    category: category ?? null,
    limit: limit ?? null,
  });
}

/**
 * Delete a milestone.
 */
export async function deleteMilestone(id: string): Promise<void> {
  return invoke<void>("delete_milestone", { id });
}

/**
 * Get milestone templates with optional category filter.
 */
export async function getMilestoneTemplates(
  category?: string | null
): Promise<MilestoneTemplate[]> {
  return invoke<MilestoneTemplate[]>("get_milestone_templates", {
    category: category ?? null,
  });
}

/**
 * Create a new milestone template.
 */
export async function createMilestoneTemplate(
  payload: NewMilestoneTemplate
): Promise<MilestoneTemplate> {
  return invoke<MilestoneTemplate>("create_milestone_template", { payload });
}

/**
 * Create a milestone from a template.
 */
export async function createMilestoneFromTemplate(
  templateId: string,
  gameId: string
): Promise<Milestone> {
  return invoke<Milestone>("create_milestone_from_template", {
    templateId,
    gameId,
  });
}

/**
 * Seed default milestone templates.
 * Returns the number of templates created.
 */
export async function seedDefaultTemplates(): Promise<number> {
  return invoke<number>("seed_default_templates");
}

/**
 * Get comprehensive milestone statistics.
 * @param gameId - Optional game ID filter
 */
export async function getMilestoneStatistics(
  gameId?: string | null
): Promise<MilestoneStatistics> {
  return invoke<MilestoneStatistics>("get_milestone_statistics", {
    gameId: gameId ?? null,
  });
}

/**
 * Migrate existing journal entries with entry_type='milestone' to milestones table.
 * This is a one-time migration that runs automatically on first launch.
 * @returns Number of journal entries migrated
 */
export async function migrateJournalToMilestones(): Promise<number> {
  return invoke<number>("migrate_journal_to_milestones");
}

// ── Identity ──────────────────────────────────────────────────────────────────

/**
 * Get comprehensive gaming identity profile.
 * Includes genre preferences, runtime stats, personality analysis, and more.
 */
export async function getGamingIdentity(): Promise<import("@/types").GamingIdentity> {
  return invoke<import("@/types").GamingIdentity>("get_gaming_identity");
}

// ── Background Jobs (T27) ─────────────────────────────────────────────────────

import type { JobInfo } from "@/hooks/useJobProgress";

/**
 * Get the status snapshot for a single background job.
 * Returns null if the job ID is unknown or has been pruned from history.
 */
export async function getJobStatus(jobId: string): Promise<JobInfo | null> {
  return invoke<JobInfo | null>("get_job_status", { jobId });
}

/**
 * Attempt to cancel a queued job before it starts.
 * Returns true if the job was found and removed; false if it was already
 * running, finished, or unknown.
 */
export async function cancelJob(jobId: string): Promise<boolean> {
  return invoke<boolean>("cancel_job", { jobId });
}

/**
 * Return all currently queued or running jobs.
 */
export async function listActiveJobs(): Promise<JobInfo[]> {
  return invoke<JobInfo[]>("list_active_jobs");
}

/**
 * Return all jobs including recently finished ones (last 20).
 */
export async function listAllJobs(): Promise<JobInfo[]> {
  return invoke<JobInfo[]>("list_all_jobs");
}

/**
 * Return the number of jobs currently waiting in the queue.
 */
export async function queueDepth(): Promise<number> {
  return invoke<number>("queue_depth");
}

// ── Asset Management (T28) ────────────────────────────────────────────────────

export type AssetType = "cover" | "background" | "gallery" | "thumbnail";

export interface AssetRef {
  path: string;
  asset_type: AssetType;
  content_hash: string;
}

export interface StorageStats {
  total_bytes: number;
  covers_bytes: number;
  backgrounds_bytes: number;
  gallery_bytes: number;
  thumbnails_bytes: number;
  file_count: number;
}

export interface CleanupResult {
  deleted_count: number;
  bytes_freed: number;
}

/** Store a cover image (resized to 512×512 WebP). Returns AssetRef. */
export async function storeCover(gameId: string, sourcePath: string): Promise<AssetRef> {
  return invoke<AssetRef>("store_cover", { gameId, sourcePath });
}

/** Store a background image (resized to 1920×1080 WebP). Returns AssetRef. */
export async function storeBackground(gameId: string, sourcePath: string): Promise<AssetRef> {
  return invoke<AssetRef>("store_background", { gameId, sourcePath });
}

/** Get the local cover path for a game, or null if none stored. */
export async function getCoverPath(gameId: string): Promise<string | null> {
  return invoke<string | null>("get_cover_path", { gameId });
}

/** Delete the cover image for a game. */
export async function deleteCover(gameId: string): Promise<void> {
  return invoke<void>("delete_cover", { gameId });
}

/** Store a gallery image for a game (converted to WebP, no resize). */
export async function storeGalleryImage(gameId: string, sourcePath: string): Promise<AssetRef> {
  return invoke<AssetRef>("store_gallery_image", { gameId, sourcePath });
}

/** List all gallery images for a game. */
export async function getGalleryImages(gameId: string): Promise<AssetRef[]> {
  return invoke<AssetRef[]>("get_gallery_images", { gameId });
}

/** Delete a single gallery image by path. */
export async function deleteGalleryImage(path: string): Promise<void> {
  return invoke<void>("delete_gallery_image", { path });
}

/** Delete all gallery images for a game. Returns count deleted. */
export async function deleteGameGallery(gameId: string): Promise<number> {
  return invoke<number>("delete_game_gallery", { gameId });
}

/** Get disk usage statistics for all asset directories. */
export async function getStorageStats(): Promise<StorageStats> {
  return invoke<StorageStats>("get_storage_stats");
}

/** Delete orphaned asset files (games no longer in DB). */
export async function cleanupOrphanAssets(): Promise<CleanupResult> {
  return invoke<CleanupResult>("cleanup_orphan_assets");
}

/** Check if a file's content is already stored (dedup check). */
export async function checkDuplicate(sourcePath: string): Promise<AssetRef | null> {
  return invoke<AssetRef | null>("check_duplicate", { sourcePath });
}

// -- Search / FTS5 (T29) -------------------------------------------------------

export interface GameSearchHit {
  id: string;
  title: string;
  developer: string | null;
  genre: string | null;
  status: string;
  cover_path: string | null;
}

export interface JournalSearchHit {
  id: string;
  title: string | null;
  body: string;
  entry_type: string;
  game_id: string | null;
  game_title: string | null;
  created_at: string;
}

export interface MilestoneSearchHit {
  id: string;
  title: string;
  game_id: string;
  game_title: string | null;
  category: string;
}

export interface SearchResults {
  games: GameSearchHit[];
  journal_entries: JournalSearchHit[];
  milestones: MilestoneSearchHit[];
  total: number;
}

export interface RebuildResult {
  games_indexed: number;
  journal_indexed: number;
}

/** Global FTS5 search across games, journal entries, and milestones. */
export async function searchGlobal(query: string, limit?: number): Promise<SearchResults> {
  return invoke<SearchResults>("search_global", { query, limit });
}

/** Rebuild the FTS5 search indexes from scratch. */
export async function rebuildSearchIndex(): Promise<RebuildResult> {
  return invoke<RebuildResult>("rebuild_search_index");
}

// -- Recommendations (T31) -----------------------------------------------------

export interface StrategyContribution {
  strategy: string;
  score: number;
  weight: number;
}

export interface RecommendationResult {
  game_id: string;
  title: string;
  cover_path: string | null;
  genre: string | null;
  developer: string | null;
  status: string;
  /** Composite score 0.0�1.0 across all strategies. */
  score: number;
  /** Human-readable reason for the recommendation. */
  reason: string;
  strategy_contributions: StrategyContribution[];
}

/** Get top recommended unplayed games from the library. */
export async function getRecommendations(limit?: number): Promise<RecommendationResult[]> {
  return invoke<RecommendationResult[]>("get_recommendations", { limit });
}

/** Get games related to a specific game by genre/developer. */
export async function getGameRecommendations(
  gameId: string,
  limit?: number,
): Promise<RecommendationResult[]> {
  return invoke<RecommendationResult[]>("get_game_recommendations", { gameId, limit });
}

// -- Analytics engines (T30) ---------------------------------------------------

export interface GamePlaytime {
  game_id: string;
  title: string;
  total_playtime_secs: number;
  cover_path: string | null;
  status: string;
}

export interface DailyPlaytime {
  date: string;         // YYYY-MM-DD
  playtime_secs: number;
}

export interface HeatmapCell {
  day_of_week: number;  // 0=Sun � 6=Sat
  hour: number;         // 0�23
  sessions: number;
}

export interface ActivityHeatmap {
  cells: HeatmapCell[];
  most_active_day: number | null;
  most_active_hour: number | null;
  total_sessions: number;
}

export interface LibrarySummary {
  total_games: number;
  total_playtime_secs: number;
  completed_games: number;
  playing_games: number;
  unplayed_games: number;
  dropped_games: number;
  favorite_games: number;
  total_sessions: number;
  total_milestones: number;
  average_session_secs: number;
}

export interface GenreStat {
  genre: string;
  game_count: number;
  completed_count: number;
  total_playtime_secs: number;
  milestone_count: number;
  preference_score: number;
}

export interface GenreDistribution {
  genres: GenreStat[];
  total_playtime_secs: number;
  dominant_genre: string | null;
}

export interface YearlyCompletions {
  year: string;
  count: number;
}

export interface CompletionStats {
  total_games: number;
  completed: number;
  playing: number;
  unplayed: number;
  dropped: number;
  completion_rate: number;
  completions_by_year: YearlyCompletions[];
  avg_time_to_complete_secs: number;
}

export interface YearInReview {
  year: string;
  total_playtime_secs: number;
  games_played: number;
  games_completed: number;
  games_added: number;
  sessions: number;
  top_games: GamePlaytime[];
  top_genres: GenreStat[];
  most_active_month: number | null;
  longest_session_secs: number;
  completion_rate: number;
}

export type RelationKind =
  | 'same_genre'
  | 'same_developer'
  | 'same_publisher'
  | 'genre_and_developer';

export interface RelatedGame {
  id: string;
  title: string;
  cover_path: string | null;
  genre: string | null;
  developer: string | null;
  status: string;
  total_playtime_secs: number;
  relation: RelationKind;
}

/** One-stop dashboard summary. */
export async function getLibrarySummary(): Promise<LibrarySummary> {
  return invoke<LibrarySummary>("get_library_summary");
}

/** Top N games by playtime. */
export async function getMostPlayedGames(limit?: number): Promise<GamePlaytime[]> {
  return invoke<GamePlaytime[]>("get_most_played_games", { limit });
}

/** Daily playtime trend � last N days (default 30). */
export async function getPlaytimeTrend(days?: number): Promise<DailyPlaytime[]> {
  return invoke<DailyPlaytime[]>("get_playtime_trend", { days });
}

/** 7�24 activity heatmap (all 168 cells, zeros included). */
export async function getActivityHeatmap(): Promise<ActivityHeatmap> {
  return invoke<ActivityHeatmap>("get_activity_heatmap");
}

/** Genre distribution ordered by preference score. */
export async function getGenreDistribution(): Promise<GenreDistribution> {
  return invoke<GenreDistribution>("get_genre_distribution");
}

/** Library completion statistics. */
export async function getCompletionStats(): Promise<CompletionStats> {
  return invoke<CompletionStats>("get_completion_stats");
}

/** Year-in-Review for a given year (defaults to current year). */
export async function getYearInReview(year?: number): Promise<YearInReview> {
  return invoke<YearInReview>("get_year_in_review", { year });
}

/** Related games by genre/developer for Game Detail page. */
export async function getRelatedGames(gameId: string, limit?: number): Promise<RelatedGame[]> {
  return invoke<RelatedGame[]>("get_related_games", { gameId, limit });
}

// ── Export (T32) ──────────────────────────────────────────────────────────────

/** Preview information shown before starting an export. */
export interface ExportPreview {
  game_count: number;
  milestone_count: number;
  journal_count: number;
  session_count: number;
  estimated_size_bytes: number;
}

/** Returned immediately when an export job is queued. */
export interface ExportQueued {
  /** Poll get_job_status(job_id) for progress. */
  job_id: string;
  /** Absolute path the export file will be written to. */
  output_path: string;
}

/**
 * Get a lightweight preview of the export — game count, milestone count,
 * estimated file size.  Fast; no file I/O.
 */
export async function getExportPreview(): Promise<ExportPreview> {
  return invoke<ExportPreview>("get_export_preview");
}

/**
 * Export the full library to a JSON file (async background job).
 *
 * @param path  Absolute path to write the file, e.g.
 *              "C:\\Users\\name\\Desktop\\library.json"
 * @returns     ExportQueued with a job_id to poll for status.
 */
export async function exportLibraryJson(path: string): Promise<ExportQueued> {
  return invoke<ExportQueued>("export_library_json", { path });
}

/**
 * Export a human-readable profile report to Markdown (async background job).
 *
 * @param path  Absolute path to write the file, e.g.
 *              "C:\\Users\\name\\Desktop\\profile.md"
 * @returns     ExportQueued with a job_id to poll for status.
 */
export async function exportProfileMarkdown(path: string): Promise<ExportQueued> {
  return invoke<ExportQueued>("export_profile_markdown", { path });
}

// ── Backup / Restore (T33) ────────────────────────────────────────────────────

export interface BackupResult {
  path: string;
  size_bytes: number;
  game_count: number;
  duration_ms: number;
}

export interface RestoreResult {
  games_restored: number;
  warnings: string[];
}

export interface BackupInfo {
  path: string;
  created_at: string;
  size_bytes: number;
  game_count: number;
  is_auto: boolean;
}

export interface BackupQueued {
  job_id: string;
  output_path: string;
}

/**
 * Create a .phb backup archive (async background job).
 * @param path  Absolute path ending in .phb, e.g. "C:\\Users\\name\\Desktop\\backup.phb"
 */
export async function createBackup(path: string): Promise<BackupQueued> {
  return invoke<BackupQueued>("create_backup", { path });
}

/**
 * Restore from a .phb archive (synchronous — rewrites DB + images).
 * @param path  Absolute path to the .phb file to restore from.
 */
export async function restoreBackup(path: string): Promise<RestoreResult> {
  return invoke<RestoreResult>("restore_backup", { path });
}

/** List all .phb backups in the default backup directory, newest first. */
export async function listAutoBackups(): Promise<BackupInfo[]> {
  return invoke<BackupInfo[]>("list_auto_backups");
}

// ── Diagnostics (T35) ─────────────────────────────────────────────────────────

export interface TableCounts {
  games: number;
  sessions: number;
  collections: number;
  collection_games: number;
  milestones: number;
  journal_entries: number;
  settings: number;
}

export interface DiagnosticsReport {
  schema_version: number;
  target_schema_version: number;
  schema_up_to_date: boolean;
  db_size_bytes: number;
  wal_enabled: boolean;
  foreign_keys_enabled: boolean;
  table_counts: TableCounts;
  active_job_count: number;
  storage: StorageStats;
  db_path: string;
}

export interface IntegrityResult {
  ok: boolean;
  messages: string[];
}

/** Full DB health report — schema version, table counts, storage stats. */
export async function getDiagnostics(): Promise<DiagnosticsReport> {
  return invoke<DiagnosticsReport>("get_diagnostics");
}

/** Run SQLite PRAGMA integrity_check. Returns ok:true for a healthy DB. */
export async function runIntegrityCheck(): Promise<IntegrityResult> {
  return invoke<IntegrityResult>("run_integrity_check");
}

/** Absolute path to the pirate_harbor.db file. */
export async function getDbPath(): Promise<string> {
  return invoke<string>("get_db_path");
}

// ── Achievement Tracking (Phase 5) ──────────────────────────────────────────

/** A single Steam achievement mapped to a game in the Pirate Harbor library. */
export interface AchievementMapping {
  /** UUID generated by Pirate Harbor. */
  id: string;
  /** FK to games.id. */
  game_id: string;
  /** Steam achievement key, e.g. "ACH_WIN_ONE_GAME". */
  steam_id: string;
  /** Human-readable achievement title. */
  display_name: string;
  /** Optional extended description. */
  description: string | null;
  /** XP-like points value. Defaults to 10. */
  points: number;
  /** ISO-8601 creation timestamp. */
  created_at: string;
}

/** Per-game achievement tracking status, returned by get_tracking_status. */
export interface TrackingStatus {
  /** Whether the Goldberg DLL swap is active for this game. */
  enabled: boolean;
  /** Steam App ID used to locate Goldberg's achievements.json, if set. */
  steam_app_id: string | null;
  /** Number of achievement mappings configured for this game. */
  mapping_count: number;
}

/** A Steam achievement definition as returned by the RAWG or local parse. */
export interface SteamAchievementDef {
  /** Steam internal key, e.g. "ACH_WIN_ONE_GAME". */
  steam_id: string;
  /** Human-readable title. */
  display_name: string;
  /** Optional extended description. */
  description: string | null;
}

/** How the Steam App ID was resolved for a game. */
export type AppIdSource = "rawg" | "local_file" | "not_found";

/** Result of auto-detecting a game's Steam App ID. */
export interface AppIdDetectionResult {
  /** The detected App ID, or null if not found. */
  app_id: string | null;
  /** Which source provided the App ID. */
  source: AppIdSource;
}


// -- Achievement Commands (T42) ------------------------------------------------

/** Enable Goldberg DLL tracking for a game. */
export async function enableAchievementTracking(
  gameId: string,
  exePath: string,
  steamAppId: string,
): Promise<void> {
  return invoke<void>('enable_achievement_tracking', { gameId, exePath, steamAppId });
}

/** Restore the original DLL and stop the file watcher. */
export async function disableAchievementTracking(
  gameId: string,
  exePath: string,
): Promise<void> {
  return invoke<void>('disable_achievement_tracking', { gameId, exePath });
}

/** Get current tracking status for a game. */
export async function getAchievementTrackingStatus(
  gameId: string,
): Promise<TrackingStatus> {
  return invoke<TrackingStatus>('get_achievement_tracking_status', { gameId });
}

/** Add or replace a single achievement mapping. */
export async function addAchievementMapping(
  gameId: string,
  steamId: string,
  displayName: string,
  description: string | null,
  points: number,
): Promise<AchievementMapping> {
  return invoke<AchievementMapping>('add_achievement_mapping', {
    gameId, steamId, displayName, description, points,
  });
}

/** Remove a single achievement mapping by ID. */
export async function removeAchievementMapping(mappingId: string): Promise<void> {
  return invoke<void>('remove_achievement_mapping', { mappingId });
}

/** List all achievement mappings for a game. */
export async function getAchievementMappings(
  gameId: string,
): Promise<AchievementMapping[]> {
  return invoke<AchievementMapping[]>('get_achievement_mappings', { gameId });
}

/** Bulk-import achievement definitions from the Steam public schema API. */
export async function importAchievementsFromSteam(
  gameId: string,
  steamAppId: string,
): Promise<AchievementMapping[]> {
  return invoke<AchievementMapping[]>('import_achievements_from_steam', {
    gameId, steamAppId,
  });
}


// -- T43: Steam App ID auto-detection -----------------------------------------

/** Detect the Steam App ID for a game via 3-tier cascade (RAWG, local file, not found). */
export async function detectSteamAppId(
  gameId: string,
  gameDir: string,
): Promise<AppIdDetectionResult> {
  return invoke<AppIdDetectionResult>('detect_steam_app_id', { gameId, gameDir });
}

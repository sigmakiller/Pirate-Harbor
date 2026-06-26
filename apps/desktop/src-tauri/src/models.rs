//! Domain models for Pirate Harbor.
//!
//! All structs derive `Serialize` + `Deserialize` so they can cross
//! the Tauri IPC boundary transparently between Rust and TypeScript.

use serde::{Deserialize, Serialize};

// ── Game status ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameStatus {
    Unplayed,
    Playing,
    Completed,
    Dropped,
}

impl GameStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameStatus::Unplayed  => "unplayed",
            GameStatus::Playing   => "playing",
            GameStatus::Completed => "completed",
            GameStatus::Dropped   => "dropped",
        }
    }
}

impl Default for GameStatus {
    fn default() -> Self {
        GameStatus::Unplayed
    }
}

impl std::str::FromStr for GameStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unplayed"  => Ok(GameStatus::Unplayed),
            "playing"   => Ok(GameStatus::Playing),
            "completed" => Ok(GameStatus::Completed),
            "dropped"   => Ok(GameStatus::Dropped),
            other       => Err(format!("Unknown status: {}", other)),
        }
    }
}

// ── Game ─────────────────────────────────────────────────────────────────────

/// Full game record — returned to the frontend for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id:                   String,
    pub title:                String,
    pub exe_path:             String,
    pub cover_path:           Option<String>,
    pub banner_path:          Option<String>,
    pub developer:            Option<String>,
    pub publisher:            Option<String>,
    pub genre:                Option<String>,
    pub is_favorite:          bool,
    pub added_at:             String,  // ISO 8601
    pub last_played:          Option<String>,
    pub total_playtime_secs:  i64,
    pub launch_count:         i64,
    pub status:               GameStatus,
}

/// Payload for adding a new game (frontend → Rust).
#[derive(Debug, Deserialize)]
pub struct NewGame {
    pub title:       String,
    pub exe_path:    String,
    pub cover_path:  Option<String>,
    pub banner_path: Option<String>,
    pub developer:   Option<String>,
    pub publisher:   Option<String>,
    pub genre:       Option<String>,
    pub status:      Option<GameStatus>,
}

/// Payload for updating an existing game (all fields optional).
#[derive(Debug, Deserialize)]
pub struct UpdateGame {
    pub title:       Option<String>,
    pub exe_path:    Option<String>,
    pub cover_path:  Option<String>,
    pub banner_path: Option<String>,
    pub developer:   Option<String>,
    pub publisher:   Option<String>,
    pub genre:       Option<String>,
    pub status:      Option<GameStatus>,
    pub is_favorite: Option<bool>,
}

/// Filter parameters for searching the library.
#[derive(Debug, Default, Deserialize)]
pub struct GameFilters {
    pub query:        Option<String>,
    pub status:       Option<GameStatus>,
    pub genre:        Option<String>,
    pub favorites_only: Option<bool>,
}

// ── Session ───────────────────────────────────────────────────────────────────

/// A single play session record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id:            String,
    pub game_id:       String,
    pub started_at:    String,
    pub ended_at:      Option<String>,
    pub duration_secs: i64,
}

// ── Scanner ───────────────────────────────────────────────────────────────────

/// A candidate game executable discovered during a folder scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Display name — derived from the exe filename (no extension)
    pub name:          String,
    /// Absolute path to the executable
    pub exe_path:      String,
    /// True if this exe_path is already registered in the library
    pub already_added: bool,
    /// Confidence score 0.0–1.0 derived from heuristics
    pub confidence:    f64,
    /// File size in megabytes
    pub size_mb:       f64,
    /// Parent folder name (e.g. "TheWitcher3")
    pub folder_name:   String,
}

// ── Metadata ──────────────────────────────────────────────────────────────────

/// A game metadata result returned from the RAWG API search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataResult {
    /// Canonical game title
    pub name:         String,
    /// Comma-separated genre names
    pub genres:       String,
    /// Cover artwork URL (hosted on RAWG CDN)
    pub cover_url:    Option<String>,
    /// Release year (e.g. 2015)
    pub release_year: Option<i32>,
}

// ── Collections ───────────────────────────────────────────────────────────────

/// A named collection of games — the curator's gallery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id:            String,
    pub name:          String,
    pub description:   Option<String>,
    /// Optional path to a user-chosen custom cover image.
    pub cover_path:    Option<String>,
    /// "auto" = 2×2 mosaic from game covers | "custom" = cover_path image.
    pub cover_mode:    String,
    /// ID of the game whose cover is used as the collection hero image.
    pub cover_game_id: Option<String>,
    pub created_at:    String,
    pub updated_at:    String,
    /// IDs of games currently in this collection (populated on query).
    pub game_ids:      Vec<String>,
    /// Count of games in this collection.
    pub game_count:    i64,
}

/// Payload for creating a new collection.
#[derive(Debug, Deserialize)]
pub struct NewCollection {
    pub name:          String,
    pub description:   Option<String>,
    pub cover_path:    Option<String>,
    pub cover_mode:    Option<String>,
    pub cover_game_id: Option<String>,
}

/// Payload for updating an existing collection.
#[derive(Debug, Deserialize)]
pub struct UpdateCollection {
    pub name:          Option<String>,
    pub description:   Option<String>,
    pub cover_path:    Option<String>,
    pub cover_mode:    Option<String>,
    pub cover_game_id: Option<String>,
}

// ── Journal ───────────────────────────────────────────────────────────────────

/// Type of a journal entry — controls visual treatment in the UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    Note,
    Milestone,
    Session,
}

impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::Note      => "note",
            EntryType::Milestone => "milestone",
            EntryType::Session   => "session",
        }
    }
}

impl Default for EntryType {
    fn default() -> Self { EntryType::Note }
}

impl std::str::FromStr for EntryType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "note"      => Ok(EntryType::Note),
            "milestone" => Ok(EntryType::Milestone),
            "session"   => Ok(EntryType::Session),
            other       => Err(format!("Unknown entry type: {}", other)),
        }
    }
}

/// A single journal entry — note, milestone, or session log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id:         String,
    /// Linked game (optional — entries can be game-agnostic)
    pub game_id:    Option<String>,
    /// Denormalised game title for display without an extra join
    pub game_title: Option<String>,
    pub title:      Option<String>,
    pub body:       String,
    pub entry_type: EntryType,
    pub created_at: String,
    pub updated_at: String,
}

/// Payload for creating a journal entry.
#[derive(Debug, Deserialize)]
pub struct NewJournalEntry {
    pub game_id:    Option<String>,
    pub title:      Option<String>,
    pub body:       String,
    pub entry_type: Option<EntryType>,
}

/// Payload for updating a journal entry.
#[derive(Debug, Deserialize)]
pub struct UpdateJournalEntry {
    pub title:      Option<String>,
    pub body:       Option<String>,
    pub entry_type: Option<EntryType>,
}

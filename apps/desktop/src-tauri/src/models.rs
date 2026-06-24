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

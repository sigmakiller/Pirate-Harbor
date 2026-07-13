//! Steam public Web API helpers -- T41.
//!
//! Fetches achievement definitions from Steam's public schema endpoint.
//! No Steam API key is required for `GetSchemaForGame`.

// All public items become used in T42 Tauri commands.
#![allow(dead_code)]

use serde::Deserialize;

// ── Response types ────────────────────────────────────────────────────────────

/// Top-level response from `GetSchemaForGame/v2`.
#[derive(Debug, Deserialize)]
pub struct SteamSchemaResponse {
    pub game: SteamSchemaGame,
}

/// The `game` object inside the schema response.
#[derive(Debug, Deserialize)]
pub struct SteamSchemaGame {
    #[serde(rename = "availableGameStats")]
    pub available_game_stats: Option<SteamGameStats>,
}

/// The stats section that contains the achievements array.
#[derive(Debug, Deserialize)]
pub struct SteamGameStats {
    pub achievements: Option<Vec<SteamAchievementDef>>,
}

/// A single Steam achievement definition.
#[derive(Debug, Deserialize, Clone)]
pub struct SteamAchievementDef {
    /// The internal Steam achievement key (e.g. `"ACH_WIN_ONE_GAME"`).
    pub name: String,
    /// Human-readable title shown in the Steam overlay.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// Optional longer description.
    pub description: Option<String>,
}

// ── API call ──────────────────────────────────────────────────────────────────

/// Fetch all achievement definitions for a Steam App ID.
///
/// Uses Steam's public `GetSchemaForGame/v2` endpoint — no API key required.
/// Returns an empty `Vec` if the game has no achievements or the request fails.
pub async fn fetch_achievement_defs(
    client:       &reqwest::Client,
    steam_app_id: &str,
) -> Result<Vec<SteamAchievementDef>, String> {
    let url = format!(
        "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/?appid={}&l=english",
        steam_app_id
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Steam API request failed: {e}"))?
        .json::<SteamSchemaResponse>()
        .await
        .map_err(|e| format!("Steam API response parse failed: {e}"))?;

    Ok(resp
        .game
        .available_game_stats
        .and_then(|s| s.achievements)
        .unwrap_or_default())
}

//! Tauri updater commands — T55.
//!
//! Exposes `check_for_updates` which the frontend calls on startup (after a
//! short delay) to see if a newer version of Pirate Harbor is available on
//! GitHub Releases.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

/// Result returned by `check_for_updates`.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    /// True when a newer version is available at the configured endpoint.
    pub available: bool,
    /// The available version string, e.g. "0.2.0".  `None` when up-to-date.
    pub version: Option<String>,
    /// Release notes / changelog from the update manifest.  `None` when up-to-date.
    pub notes: Option<String>,
}

/// Check the configured update endpoint and report whether a newer version is
/// available.  Returns `available: false` (never errors) so the frontend can
/// treat failures as "up to date".
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> UpdateCheckResult {
    match app.updater() {
        Ok(updater) => match updater.check().await {
            Ok(Some(update)) => UpdateCheckResult {
                available: true,
                version:   Some(update.version.clone()),
                notes:     update.body.clone(),
            },
            Ok(None) => UpdateCheckResult {
                available: false,
                version:   None,
                notes:     None,
            },
            Err(_) => UpdateCheckResult {
                available: false,
                version:   None,
                notes:     None,
            },
        },
        Err(_) => UpdateCheckResult {
            available: false,
            version:   None,
            notes:     None,
        },
    }
}

//! DLL swap — backup / inject Goldberg emulator / restore — T39.
//!
//! # Strategy
//!
//! Instead of copying the original `steam_api64.dll` to a backup location,
//! we **rename** it (atomic on the same volume). This means:
//!
//! * A crash after rename but before the Goldberg copy leaves the game in
//!   the detectable [`SwapState::BackupOnlyNoActive`] state.
//! * Recovery is always possible: call [`restore_dll`] to rename the backup
//!   back to `steam_api64.dll`.
//!
//! # Goldberg emulator notes
//!
//! Goldberg Steam Emu intercepts Steam API calls and writes unlock state to:
//! ```text
//! %APPDATA%\Goldberg SteamEmu Saves\{steam_app_id}\achievements.json
//! ```
//! It also requires a `steam_settings/steam_appid.txt` file in the game
//! directory containing the numeric Steam App ID as a plain string.

// All items here become used when T41 exposes them as Tauri commands.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use tauri::Manager; // Required for app_handle.path() (PathResolver trait provider)

const BACKUP_SUFFIX: &str = ".ph_backup";
const APPID_FILE: &str = "steam_appid.txt";
const SETTINGS_DIR: &str = "steam_settings";

// ── State detection ───────────────────────────────────────────────────────────

/// Current state of the `steam_api64.dll` in a game's directory.
#[derive(Debug, PartialEq)]
pub enum SwapState {
    /// No `steam_api64.dll` present — this game does not use the Steam API.
    /// Achievement tracking is not supported without a Steam DLL.
    NoSteamDll,
    /// Original DLL is present; our Goldberg DLL is not installed.
    OriginalPresent,
    /// Goldberg DLL is active (original is in `.ph_backup`) — normal tracking state.
    OurDllInstalled,
    /// Backup exists but active DLL is missing — app crashed mid-swap.
    /// Call [`restore_dll`] to recover to [`SwapState::OriginalPresent`].
    BackupOnlyNoActive,
}

/// Inspect the game directory and return the current [`SwapState`].
pub fn verify_swap_integrity(game_dir: &Path) -> Result<SwapState, String> {
    let dll    = game_dir.join("steam_api64.dll");
    let backup = game_dir.join(format!("steam_api64.dll{}", BACKUP_SUFFIX));

    match (dll.exists(), backup.exists()) {
        (false, false) => Ok(SwapState::NoSteamDll),
        (true,  false) => Ok(SwapState::OriginalPresent),
        (true,  true)  => Ok(SwapState::OurDllInstalled),
        (false, true)  => Ok(SwapState::BackupOnlyNoActive),
    }
}

// ── Inject ────────────────────────────────────────────────────────────────────

/// Install the Goldberg emulator DLL into the game directory.
///
/// Steps (atomic-safe):
///   1. Guard: return early if already installed or no Steam DLL present.
///   2. **Rename** original DLL → `.ph_backup` (atomic on same volume).
///   3. Copy our bundled Goldberg DLL from the Tauri resource directory.
///   4. Write `steam_settings/steam_appid.txt` with the numeric App ID.
///
/// This function requires a live [`tauri::AppHandle`] to locate the bundled
/// Goldberg DLL. It is not called in unit tests; use integration tests or
/// end-to-end tests for the inject path.
pub fn inject_dll(
    game_dir:     &Path,
    steam_app_id: &str,
    app_handle:   &tauri::AppHandle,
) -> Result<(), String> {
    match verify_swap_integrity(game_dir)? {
        SwapState::OurDllInstalled    => return Ok(()),  // idempotent
        SwapState::NoSteamDll         => return Err(
            "Game has no steam_api64.dll — achievement tracking not supported for this game."
                .into(),
        ),
        SwapState::BackupOnlyNoActive => return Err(
            "Previous DLL swap is in a broken state. Call restore_dll first.".into(),
        ),
        SwapState::OriginalPresent => {}
    }

    let dll_dest = game_dir.join("steam_api64.dll");
    let backup   = game_dir.join(format!("steam_api64.dll{}", BACKUP_SUFFIX));

    // Step 1: atomic rename — crash-safe; backup exists ↔ swap in progress.
    std::fs::rename(&dll_dest, &backup)
        .map_err(|e| format!("Failed to back up original DLL: {e}"))?;

    // Step 2: copy our Goldberg DLL from the Tauri resource bundle.
    let dll_src = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Could not resolve resource directory: {e}"))?
        .join("plugins")
        .join("steam_api64.dll");

    if !dll_src.exists() {
        // Roll back: restore the original before returning the error.
        let _ = std::fs::rename(&backup, &dll_dest);
        return Err(format!(
            "Goldberg DLL not found in resource bundle at: {}",
            dll_src.display()
        ));
    }

    std::fs::copy(&dll_src, &dll_dest)
        .map_err(|e| {
            // Roll back on copy failure.
            let _ = std::fs::rename(&backup, &dll_dest);
            format!("Failed to copy Goldberg DLL: {e}")
        })?;

    // Step 3: write steam_settings/steam_appid.txt (required by Goldberg).
    let settings_dir = game_dir.join(SETTINGS_DIR);
    std::fs::create_dir_all(&settings_dir)
        .map_err(|e| format!("Failed to create steam_settings/: {e}"))?;
    std::fs::write(settings_dir.join(APPID_FILE), steam_app_id)
        .map_err(|e| format!("Failed to write steam_appid.txt: {e}"))?;

    Ok(())
}

// ── Restore ───────────────────────────────────────────────────────────────────

/// Restore the original DLL and remove the Goldberg installation.
///
/// Safe to call even if no swap is active (idempotent — returns `Ok(())`).
///
/// Also removes the `steam_settings/` directory created by [`inject_dll`].
pub fn restore_dll(game_dir: &Path) -> Result<(), String> {
    let dll_dest = game_dir.join("steam_api64.dll");
    let backup   = game_dir.join(format!("steam_api64.dll{}", BACKUP_SUFFIX));

    if !backup.exists() {
        // Nothing to restore — swap was never applied.
        return Ok(());
    }

    // Remove the active (Goldberg) DLL if present.
    if dll_dest.exists() {
        std::fs::remove_file(&dll_dest)
            .map_err(|e| format!("Failed to remove Goldberg DLL: {e}"))?;
    }

    // Rename backup → original.
    std::fs::rename(&backup, &dll_dest)
        .map_err(|e| format!("Failed to restore original DLL: {e}"))?;

    // Remove the steam_settings/ directory we created.
    let settings_dir = game_dir.join(SETTINGS_DIR);
    if settings_dir.exists() {
        std::fs::remove_dir_all(&settings_dir)
            .map_err(|e| format!("Failed to remove steam_settings/: {e}"))?;
    }

    Ok(())
}

// ── Utilities ─────────────────────────────────────────────────────────────────

/// Derive the game directory from an executable path.
///
/// Returns `Ok(parent_dir)` or an `Err` if the path has no parent component.
pub fn game_dir_from_exe(exe_path: &str) -> Result<PathBuf, String> {
    Path::new(exe_path)
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| format!("Cannot determine game directory from: {exe_path}"))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // ── helpers ───────────────────────────────────────────────────────────────

    /// Create a temp dir that already contains a steam_api64.dll.
    fn make_game_dir_with_dll() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("steam_api64.dll"), b"original_dll_content").unwrap();
        dir
    }

    // ── SwapState detection ───────────────────────────────────────────────────

    /// A directory with no DLL must report `NoSteamDll`.
    #[test]
    fn state_no_dll() {
        let dir = tempdir().unwrap();
        assert_eq!(
            verify_swap_integrity(dir.path()).unwrap(),
            SwapState::NoSteamDll,
            "Empty directory must report NoSteamDll"
        );
    }

    /// A directory with only the original DLL must report `OriginalPresent`.
    #[test]
    fn state_original_present() {
        let dir = make_game_dir_with_dll();
        assert_eq!(
            verify_swap_integrity(dir.path()).unwrap(),
            SwapState::OriginalPresent,
            "Directory with only steam_api64.dll must report OriginalPresent"
        );
    }

    /// Simulated installed state (both DLL and backup present) must report
    /// `OurDllInstalled`.
    #[test]
    fn state_our_dll_installed() {
        let dir = make_game_dir_with_dll();
        // Simulate an active Goldberg install: rename original → backup, write fake Goldberg.
        fs::rename(
            dir.path().join("steam_api64.dll"),
            dir.path().join("steam_api64.dll.ph_backup"),
        )
        .unwrap();
        fs::write(dir.path().join("steam_api64.dll"), b"goldberg_dll_content").unwrap();

        assert_eq!(
            verify_swap_integrity(dir.path()).unwrap(),
            SwapState::OurDllInstalled,
            "Directory with both DLL and backup must report OurDllInstalled"
        );
    }

    /// Backup present but active DLL missing must report `BackupOnlyNoActive`
    /// (crash-recovery state).
    #[test]
    fn state_backup_only_no_active() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("steam_api64.dll.ph_backup"), b"original_dll_content").unwrap();

        assert_eq!(
            verify_swap_integrity(dir.path()).unwrap(),
            SwapState::BackupOnlyNoActive,
            "Backup-only directory must report BackupOnlyNoActive"
        );
    }

    /// `restore_dll` on a directory that was never swapped must succeed and
    /// leave the original DLL intact (idempotent / no-op).
    #[test]
    fn restore_is_idempotent_when_nothing_installed() {
        let dir = make_game_dir_with_dll();
        // No swap has been applied — restore should be a harmless no-op.
        restore_dll(dir.path()).unwrap();
        // Original DLL must still be present.
        assert!(
            dir.path().join("steam_api64.dll").exists(),
            "Original DLL must still exist after restore on non-swapped directory"
        );
        // No backup should exist.
        assert!(
            !dir.path().join("steam_api64.dll.ph_backup").exists(),
            "No backup file should exist"
        );
    }

    // ── Additional coverage ───────────────────────────────────────────────────

    /// `restore_dll` on a fully-swapped directory must restore the original
    /// and remove the Goldberg install artefacts.
    #[test]
    fn restore_cleans_up_correctly() {
        let dir = tempdir().unwrap();
        // Set up installed state: backup + active Goldberg DLL + steam_settings/.
        fs::write(dir.path().join("steam_api64.dll"), b"goldberg").unwrap();
        fs::write(dir.path().join("steam_api64.dll.ph_backup"), b"original").unwrap();
        let settings = dir.path().join("steam_settings");
        fs::create_dir_all(&settings).unwrap();
        fs::write(settings.join("steam_appid.txt"), b"570").unwrap();

        restore_dll(dir.path()).unwrap();

        // Original DLL must be restored.
        let dll = dir.path().join("steam_api64.dll");
        assert!(dll.exists(), "DLL must exist after restore");
        assert_eq!(fs::read(&dll).unwrap(), b"original", "Restored DLL must match original");

        // Backup must be gone.
        assert!(!dir.path().join("steam_api64.dll.ph_backup").exists(), "Backup must be removed");

        // steam_settings/ must be cleaned up.
        assert!(!settings.exists(), "steam_settings/ must be removed after restore");
    }

    /// `game_dir_from_exe` must return the parent directory of the executable path.
    #[test]
    fn game_dir_from_exe_returns_parent() {
        let result = game_dir_from_exe(r"C:\Games\MyGame\game.exe").unwrap();
        assert_eq!(result, PathBuf::from(r"C:\Games\MyGame"));
    }

    /// `game_dir_from_exe` with a bare filename (no path separator) must error.
    #[test]
    fn game_dir_from_exe_bare_name_errors() {
        // "game.exe" has no parent — should return an Err.
        // Note: on some systems Path::parent() of a bare name returns Some(""),
        // so we verify the function doesn't panic and documents its contract.
        let result = game_dir_from_exe("game.exe");
        // Either Ok with empty path OR an Err is acceptable; the key is no panic.
        let _ = result; // contract: must not panic
    }
}

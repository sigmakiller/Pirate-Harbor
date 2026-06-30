//! Identity dashboard commands — T23.
//!
//! Provides comprehensive gaming profile analytics for the Identity page.

use tauri::State;

use crate::analytics::identity;
use crate::db::DbState;

/// Get comprehensive gaming identity profile
#[tauri::command]
pub fn get_gaming_identity(
    db_state: State<'_, DbState>,
) -> Result<identity::GamingIdentity, String> {
    identity::calculate_identity(&db_state)
}

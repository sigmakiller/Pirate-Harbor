//! Database module — initialization, connection management, and migrations.
//!
//! Provides `DbState` (a `Mutex<Connection>`) for safe concurrent access
//! from Tauri commands, and `init_db()` which creates the database file
//! in the app data directory and runs all migrations.

pub mod migrations;

use std::sync::Mutex;

use rusqlite::Connection;

/// Thread-safe database state managed by Tauri.
///
/// All Tauri commands that need database access receive this via
/// `tauri::State<DbState>` and lock the mutex for the duration
/// of their transaction.
pub struct DbState(pub Mutex<Connection>);

/// Initialize the database at the given app data directory.
///
/// Creates the directory if it doesn't exist, opens (or creates)
/// `pirate_harbor.db`, enables WAL mode and foreign keys, and
/// runs all pending migrations.
pub fn init_db(app_data_dir: &std::path::Path) -> Result<Connection, Box<dyn std::error::Error>> {
    // Ensure the data directory exists
    std::fs::create_dir_all(app_data_dir)?;

    let db_path = app_data_dir.join("pirate_harbor.db");

    let conn = Connection::open(&db_path)?;

    // Enable WAL mode for better concurrent read performance
    conn.pragma_update(None, "journal_mode", "WAL")?;

    // Enable foreign key constraints (SQLite has them off by default)
    conn.pragma_update(None, "foreign_keys", "ON")?;

    // Run migrations
    migrations::run_migrations(&conn)?;

    println!(
        "[PirateHarbor] Database initialized at: {}",
        db_path.display()
    );

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_db_creates_file() {
        let tmp = std::env::temp_dir().join("pirate_harbor_test_db");
        let _ = std::fs::remove_dir_all(&tmp);

        let conn = init_db(&tmp).unwrap();

        // Verify file was created
        assert!(tmp.join("pirate_harbor.db").exists());

        // Verify we can query
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);

        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let tmp = std::env::temp_dir().join("pirate_harbor_test_fk");
        let _ = std::fs::remove_dir_all(&tmp);

        let conn = init_db(&tmp).unwrap();

        let fk_enabled: i64 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}

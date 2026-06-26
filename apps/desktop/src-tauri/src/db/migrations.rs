/// Database migrations — embedded SQL for schema creation.
///
/// Each migration is a static SQL string applied in order.
/// `run_migrations` is idempotent (uses IF NOT EXISTS).

use rusqlite::Connection;

/// All migrations in order. Each is applied sequentially on first run.
const MIGRATIONS: &[&str] = &[MIGRATION_001, MIGRATION_002, MIGRATION_003, MIGRATION_004];

/// 001 — Initial schema: games, sessions, settings + indexes
const MIGRATION_001: &str = r#"
CREATE TABLE IF NOT EXISTS games (
    id                  TEXT PRIMARY KEY,
    title               TEXT NOT NULL,
    exe_path            TEXT NOT NULL,
    cover_path          TEXT,
    banner_path         TEXT,
    developer           TEXT,
    publisher           TEXT,
    genre               TEXT,
    is_favorite         INTEGER NOT NULL DEFAULT 0,
    added_at            TEXT NOT NULL,
    last_played         TEXT,
    total_playtime_secs INTEGER NOT NULL DEFAULT 0,
    launch_count        INTEGER NOT NULL DEFAULT 0,
    status              TEXT NOT NULL DEFAULT 'unplayed'
);

CREATE TABLE IF NOT EXISTS sessions (
    id            TEXT PRIMARY KEY,
    game_id       TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    started_at    TEXT NOT NULL,
    ended_at      TEXT,
    duration_secs INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_game ON sessions(game_id);
CREATE INDEX IF NOT EXISTS idx_games_title ON games(title);
CREATE INDEX IF NOT EXISTS idx_games_status ON games(status);
CREATE INDEX IF NOT EXISTS idx_games_favorite ON games(is_favorite);
"#;

/// 002 — Metadata cache: stores RAWG search results keyed by lowercase query
const MIGRATION_002: &str = r#"
CREATE TABLE IF NOT EXISTS metadata_cache (
    query       TEXT PRIMARY KEY,
    results_json TEXT NOT NULL,
    cached_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

/// 003 — Collections: curated galleries of games
const MIGRATION_003: &str = r#"
CREATE TABLE IF NOT EXISTS collections (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    description   TEXT,
    cover_path    TEXT,
    cover_mode    TEXT NOT NULL DEFAULT 'auto',
    cover_game_id TEXT REFERENCES games(id) ON DELETE SET NULL,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS collection_games (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    game_id       TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    added_at      TEXT NOT NULL,
    PRIMARY KEY (collection_id, game_id)
);

CREATE INDEX IF NOT EXISTS idx_collection_games_coll ON collection_games(collection_id);
CREATE INDEX IF NOT EXISTS idx_collection_games_game ON collection_games(game_id);
"#;

/// 004 — Journal: chronological log of notes, milestones, and session records
const MIGRATION_004: &str = r#"
CREATE TABLE IF NOT EXISTS journal_entries (
    id          TEXT PRIMARY KEY,
    game_id     TEXT REFERENCES games(id) ON DELETE CASCADE,
    game_title  TEXT,
    title       TEXT,
    body        TEXT NOT NULL DEFAULT '',
    entry_type  TEXT NOT NULL DEFAULT 'note',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_journal_game    ON journal_entries(game_id);
CREATE INDEX IF NOT EXISTS idx_journal_created ON journal_entries(created_at DESC);
"#;

/// Apply all migrations to the given connection.
pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    for migration in MIGRATIONS {
        conn.execute_batch(migration)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_apply_cleanly() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Verify tables exist by querying sqlite_master
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"games".to_string()));
        assert!(tables.contains(&"sessions".to_string()));
        assert!(tables.contains(&"settings".to_string()));
        assert!(tables.contains(&"metadata_cache".to_string()));
        assert!(tables.contains(&"collections".to_string()));
        assert!(tables.contains(&"collection_games".to_string()));
        assert!(tables.contains(&"journal_entries".to_string()));
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        // Running again should not error
        run_migrations(&conn).unwrap();
    }
}

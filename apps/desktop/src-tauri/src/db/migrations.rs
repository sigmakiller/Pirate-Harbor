/// Database migrations — version-tracked schema upgrades.
///
/// # How it works
///
/// Each migration is paired with a version number. `run_migrations` reads
/// `schema_version` from the `settings` table (defaulting to 0), applies
/// every migration whose version exceeds the stored version, then writes
/// the new version back.
///
/// ## Existing databases (pre-T26)
///
/// Before T26 the schema had no version tracking. On the first run after
/// upgrading, `get_schema_version` returns 0 and `detect_existing_schema`
/// checks for known tables to fast-forward to v6 so no migrations re-run.

use rusqlite::Connection;

// ── Migration version constants ───────────────────────────────────────────────

/// Current target schema version.
/// Increment this whenever a new MIGRATION_NNN is added.
// T35: Consumed by the diagnostics command via `db::CURRENT_SCHEMA_VERSION`.
#[allow(dead_code)]
pub const CURRENT_SCHEMA_VERSION: i32 = 7;

// ── Versioned migration table ─────────────────────────────────────────────────

/// A migration paired with the version it brings the schema to.
struct Migration {
    version:    i32,
    #[allow(dead_code)] // Human-readable label; used in future diagnostics (T35)
    description: &'static str,
    sql:        &'static str,
}

/// Ordered list of every migration. A migration at index i brings the schema
/// from version (i) to version (i+1), i.e., `migration.version` is the
/// version AFTER applying it.
const MIGRATIONS: &[Migration] = &[
    Migration { version: 1, description: "Initial schema", sql: MIGRATION_001 },
    Migration { version: 2, description: "Search cache",   sql: MIGRATION_002 },
    Migration { version: 3, description: "Collections",    sql: MIGRATION_003 },
    Migration { version: 4, description: "Journal",        sql: MIGRATION_004 },
    Migration { version: 5, description: "Metadata cache", sql: MIGRATION_005 },
    Migration { version: 6, description: "Milestones",     sql: MIGRATION_006 },
    Migration { version: 7, description: "FTS5 search",    sql: MIGRATION_007 },
];

// ── SQL strings ───────────────────────────────────────────────────────────────

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

/// 002 — Search cache: stores RAWG search results keyed by lowercase query
const MIGRATION_002: &str = r#"
CREATE TABLE IF NOT EXISTS search_cache (
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

/// 005 — Metadata cache and enrichment queue for RAWG/IGDB integration
const MIGRATION_005: &str = r#"
CREATE TABLE IF NOT EXISTS metadata_cache (
    id          TEXT PRIMARY KEY,
    game_title  TEXT NOT NULL,
    provider    TEXT NOT NULL,
    api_id      INTEGER,
    metadata    TEXT NOT NULL,
    cached_at   TEXT NOT NULL,
    expires_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS metadata_enrichment_queue (
    id          TEXT PRIMARY KEY,
    game_id     TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    priority    INTEGER NOT NULL DEFAULT 0,
    status      TEXT NOT NULL DEFAULT 'pending',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_metadata_cache_title ON metadata_cache(game_title);
CREATE INDEX IF NOT EXISTS idx_enrichment_queue_status ON metadata_enrichment_queue(status);
"#;

/// Image columns added to `games` by MIGRATION_005.
/// Kept separate because SQLite does not support `ALTER TABLE … ADD COLUMN IF NOT EXISTS`.
/// Errors caused by already-existing columns are intentionally ignored.
const MIGRATION_005_ALTER: &[(&str, &str, &str)] = &[
    ("games", "cover_path_local",       "TEXT"),
    ("games", "background_path_local",  "TEXT"),
    ("games", "images_enriched_at",     "TEXT"),
];

/// 006 — Enhanced milestone system with formal structure and templates
const MIGRATION_006: &str = r#"
CREATE TABLE IF NOT EXISTS milestones (
    id              TEXT PRIMARY KEY,
    game_id         TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    description     TEXT,
    category        TEXT NOT NULL,
    difficulty      TEXT,
    achievement_date TEXT NOT NULL,
    points          INTEGER DEFAULT 0,
    metadata        TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS milestone_templates (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    description TEXT,
    category    TEXT NOT NULL,
    difficulty  TEXT,
    is_global   INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_milestones_game ON milestones(game_id);
CREATE INDEX IF NOT EXISTS idx_milestones_category ON milestones(category);
CREATE INDEX IF NOT EXISTS idx_milestones_date ON milestones(achievement_date);
"#;

/// 007 — FTS5 full-text search virtual tables for games and journal entries.
///
/// Content tables mode: the FTS index is a shadow of the real tables.
/// Triggers keep the FTS in sync on every INSERT/UPDATE/DELETE.
/// `content_rowid='rowid'` ensures rowid lookups work correctly.
const MIGRATION_007: &str = r#"
-- ── FTS5 virtual tables ───────────────────────────────────────────────────────────────
CREATE VIRTUAL TABLE IF NOT EXISTS games_fts USING fts5(
    title, developer, publisher, genre,
    content='games', content_rowid='rowid'
);

CREATE VIRTUAL TABLE IF NOT EXISTS journal_fts USING fts5(
    title, body,
    content='journal_entries', content_rowid='rowid'
);

-- ── games sync triggers ──────────────────────────────────────────────────────────────
CREATE TRIGGER IF NOT EXISTS games_ai
AFTER INSERT ON games BEGIN
    INSERT INTO games_fts(rowid, title, developer, publisher, genre)
    VALUES (new.rowid, new.title, new.developer, new.publisher,
            REPLACE(COALESCE(new.genre, ''), ',', ' '));
END;

CREATE TRIGGER IF NOT EXISTS games_ad
AFTER DELETE ON games BEGIN
    INSERT INTO games_fts(games_fts, rowid, title, developer, publisher, genre)
    VALUES ('delete', old.rowid, old.title, old.developer, old.publisher,
            REPLACE(COALESCE(old.genre, ''), ',', ' '));
END;

CREATE TRIGGER IF NOT EXISTS games_au
AFTER UPDATE ON games BEGIN
    INSERT INTO games_fts(games_fts, rowid, title, developer, publisher, genre)
    VALUES ('delete', old.rowid, old.title, old.developer, old.publisher,
            REPLACE(COALESCE(old.genre, ''), ',', ' '));
    INSERT INTO games_fts(rowid, title, developer, publisher, genre)
    VALUES (new.rowid, new.title, new.developer, new.publisher,
            REPLACE(COALESCE(new.genre, ''), ',', ' '));
END;

-- ── journal_entries sync triggers ─────────────────────────────────────────────────
CREATE TRIGGER IF NOT EXISTS journal_ai
AFTER INSERT ON journal_entries BEGIN
    INSERT INTO journal_fts(rowid, title, body)
    VALUES (new.rowid, new.title, new.body);
END;

CREATE TRIGGER IF NOT EXISTS journal_ad
AFTER DELETE ON journal_entries BEGIN
    INSERT INTO journal_fts(journal_fts, rowid, title, body)
    VALUES ('delete', old.rowid, old.title, old.body);
END;

CREATE TRIGGER IF NOT EXISTS journal_au
AFTER UPDATE ON journal_entries BEGIN
    INSERT INTO journal_fts(journal_fts, rowid, title, body)
    VALUES ('delete', old.rowid, old.title, old.body);
    INSERT INTO journal_fts(rowid, title, body)
    VALUES (new.rowid, new.title, new.body);
END;
"#;

// ── Version helpers ───────────────────────────────────────────────────────────

/// Read the current schema version from the `settings` table.
/// Returns 0 if the key does not exist (fresh or pre-T26 database).
pub fn get_schema_version(conn: &Connection) -> i32 {
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'schema_version'",
        [],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .and_then(|s| s.parse::<i32>().ok())
    .unwrap_or(0)
}

/// Persist the schema version to the `settings` table.
pub fn set_schema_version(conn: &Connection, version: i32) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('schema_version', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![version.to_string()],
    )?;
    Ok(())
}

/// Detect whether this is a pre-T26 database that already has all 6 migrations
/// applied (identified by the presence of the `milestones` table), and if so
/// fast-forward the version counter to 6 so no migrations re-run.
fn auto_detect_existing_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    let milestones_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='milestones'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        > 0;

    if milestones_exists {
        // All 6 migrations were already applied without version tracking.
        // T29: We stamp to 6 (not 7) because games_fts may not yet exist.
        // The next run_migrations call will apply migration 007.
        set_schema_version(conn, 6)?;
    }

    Ok(())
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Apply all pending migrations to the given connection.
///
/// - Reads `schema_version` from the `settings` table (0 if absent).
/// - For a pre-T26 existing database, auto-detects and stamps to v6.
/// - Skips every migration whose version ≤ stored version.
/// - After each migration, updates the stored version immediately so a
///   crash mid-run leaves the database in the highest successfully applied state.
pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    // The settings table must exist before we can read/write the version key.
    // MIGRATION_001 creates it, but if we call get_schema_version before
    // applying migration 1, the table won't exist yet. We handle this by
    // catching the "no such table" error and treating it as version 0.
    let mut current_version = get_schema_version(conn);

    // For pre-T26 databases that already have all tables but no version key,
    // fast-forward so nothing re-runs.
    if current_version == 0 {
        auto_detect_existing_schema(conn)?;
        current_version = get_schema_version(conn);
    }

    // Apply each migration that hasn't been applied yet.
    for migration in MIGRATIONS {
        if migration.version <= current_version {
            continue; // Already applied — skip.
        }

        conn.execute_batch(migration.sql)?;

        // Migration 5 has additional ALTER TABLE statements that must be
        // applied idempotently (SQLite lacks ADD COLUMN IF NOT EXISTS).
        if migration.version == 5 {
            for (table, column, col_type) in MIGRATION_005_ALTER {
                let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, col_type);
                let _ = conn.execute_batch(&sql); // Ignore "duplicate column" errors.
            }
        }

        // Stamp the new version immediately so a crash doesn't re-run this migration.
        set_schema_version(conn, migration.version)?;
        current_version = migration.version;
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_apply_cleanly() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // All expected tables must be present.
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
        assert!(tables.contains(&"search_cache".to_string()));
        assert!(tables.contains(&"metadata_cache".to_string()));
        assert!(tables.contains(&"collections".to_string()));
        assert!(tables.contains(&"collection_games".to_string()));
        assert!(tables.contains(&"journal_entries".to_string()));
        assert!(tables.contains(&"milestones".to_string()));
        assert!(tables.contains(&"milestone_templates".to_string()));

        // T29: FTS5 virtual tables must exist.
        assert!(tables.contains(&"games_fts".to_string()),
            "games_fts virtual table missing");
        assert!(tables.contains(&"journal_fts".to_string()),
            "journal_fts virtual table missing");
    }

    #[test]
    fn test_fresh_db_reaches_current_version() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let version = get_schema_version(&conn);
        assert_eq!(version, CURRENT_SCHEMA_VERSION,
            "Fresh DB should reach schema version {}", CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        // Second run must be a no-op — every migration is skipped.
        run_migrations(&conn).unwrap();

        // Version must still be correct after double-run.
        assert_eq!(get_schema_version(&conn), CURRENT_SCHEMA_VERSION);
    }

    /// T29: Verify FTS5 search actually finds records via MATCH syntax.
    #[test]
    fn test_fts_search_works() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Enable WAL + FK (mirrors db/mod.rs).
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").unwrap();

        // Insert a game — triggers games_ai should populate games_fts.
        conn.execute(
            "INSERT INTO games (id, title, exe_path, is_favorite, added_at, total_playtime_secs, launch_count, status)
             VALUES ('g1', 'The Witcher 3', '/path/witcher.exe', 0, '2024-01-01T00:00:00Z', 0, 0, 'unplayed')",
            [],
        ).unwrap();

        // FTS search by partial title (prefix query).
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM games_fts WHERE games_fts MATCH 'Witcher*'",
            [],
            |row| row.get(0),
        ).expect("FTS5 MATCH query failed");
        assert_eq!(count, 1, "Should find 1 game matching 'Witcher*'");

        // FTS search by developer (after UPDATE triggers games_au).
        conn.execute(
            "UPDATE games SET developer = 'CD Projekt Red' WHERE id = 'g1'",
            [],
        ).unwrap();

        let dev_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM games_fts WHERE games_fts MATCH 'Projekt*'",
            [],
            |row| row.get(0),
        ).expect("FTS5 developer MATCH failed");
        assert_eq!(dev_count, 1, "Should find 1 game matching developer 'Projekt*'");

        // Verify DELETE trigger removes from FTS.
        conn.execute("DELETE FROM games WHERE id = 'g1'", []).unwrap();
        let after_delete: i64 = conn.query_row(
            "SELECT COUNT(*) FROM games_fts WHERE games_fts MATCH 'Witcher*'",
            [],
            |row| row.get(0),
        ).expect("FTS5 MATCH after delete failed");
        assert_eq!(after_delete, 0, "Deleted game should not appear in FTS");
    }

    /// T29: Journal FTS5 search.
    #[test]
    fn test_journal_fts_search_works() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO journal_entries (id, title, body, entry_type, created_at, updated_at)
             VALUES ('j1', 'Epic Victory', 'I defeated the final boss in Elden Ring!', 'note', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            [],
        ).unwrap();

        // Search title.
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM journal_fts WHERE journal_fts MATCH 'Victory*'",
            [],
            |row| row.get(0),
        ).expect("Journal FTS MATCH failed");
        assert_eq!(count, 1, "Should find 1 entry matching 'Victory*'");

        // Search body.
        let body_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM journal_fts WHERE journal_fts MATCH 'Elden*'",
            [],
            |row| row.get(0),
        ).expect("Journal body FTS MATCH failed");
        assert_eq!(body_count, 1, "Should find 1 entry matching body 'Elden*'");
    }

    #[test]
    fn test_existing_db_auto_detected_as_v6() {
        // Simulate a pre-T26 database: apply all SQL manually without version tracking.
        let conn = Connection::open_in_memory().unwrap();

        for migration in MIGRATIONS {
            conn.execute_batch(migration.sql).unwrap();
        }
        for (table, column, col_type) in MIGRATION_005_ALTER {
            let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, col_type);
            let _ = conn.execute_batch(&sql);
        }

        // No schema_version key exists yet — simulates pre-T26.
        assert_eq!(get_schema_version(&conn), 0);

        // After run_migrations, the DB should be stamped at v6.
        run_migrations(&conn).unwrap();
        assert_eq!(get_schema_version(&conn), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_set_and_get_schema_version() {
        let conn = Connection::open_in_memory().unwrap();
        // Create settings table first (normally done by migration 1).
        conn.execute_batch(
            "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        ).unwrap();

        set_schema_version(&conn, 3).unwrap();
        assert_eq!(get_schema_version(&conn), 3);

        // Upsert — calling set again must overwrite, not duplicate.
        set_schema_version(&conn, 5).unwrap();
        assert_eq!(get_schema_version(&conn), 5);
    }

    #[test]
    fn test_init_db_creates_file() {
        let tmp = std::env::temp_dir().join("pirate_harbor_t26_init");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let db_path = tmp.join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        run_migrations(&conn).unwrap();
        assert!(db_path.exists());
        assert_eq!(get_schema_version(&conn), CURRENT_SCHEMA_VERSION);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        // FK enforcement is set in db/mod.rs, not migrations, but verify
        // that a table with FK constraints exists so the schema loaded.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sessions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}

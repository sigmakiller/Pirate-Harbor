//! Metadata resolver — T30.
//!
//! Unified metadata source with precedence: local cache → RAWG API.
//! Designed for future extension to IGDB (Phase 5).
//!
//! This module provides a lightweight facade that callers use without
//! caring which source provided the data.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

/// Unified metadata record returned regardless of source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedMetadata {
    pub title:        String,
    pub developer:    Option<String>,
    pub publisher:    Option<String>,
    pub genre:        Option<String>,
    pub description:  Option<String>,
    pub release_date: Option<String>,
    pub cover_url:    Option<String>,
    /// Which source produced this record.
    pub source:       MetadataSource,
}

/// The data source that fulfilled the resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataSource {
    /// Returned from the local `metadata_cache` table.
    Cache,
    /// Returned from the RAWG REST API.
    Rawg,
    /// No external data found; only library data available.
    Library,
}

// ── Resolver ──────────────────────────────────────────────────────────────────

/// Attempt to resolve metadata for `game_id`, preferring the local cache.
///
/// Resolution order:
/// 1. `metadata_cache` table (populated by T16 enrichment engine).
/// 2. Library record itself (developer, publisher, genre already stored).
///
/// RAWG live-fetch is intentionally left to the existing enrichment commands
/// (T15/T16) so the resolver stays synchronous and lock-safe.
pub fn resolve(conn: &Connection, game_id: &str) -> Result<ResolvedMetadata, String> {
    // 1. Check metadata_cache.
    let cache_result = conn.query_row(
        r#"SELECT title, developer, publisher, genres, description,
                  release_date, cover_url
           FROM metadata_cache
           WHERE game_id = ?1
           ORDER BY fetched_at DESC
           LIMIT 1"#,
        rusqlite::params![game_id],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        },
    );

    if let Ok((title, developer, publisher, genre, description, release_date, cover_url)) =
        cache_result
    {
        return Ok(ResolvedMetadata {
            title,
            developer,
            publisher,
            genre,
            description,
            release_date,
            cover_url,
            source: MetadataSource::Cache,
        });
    }

    // 2. Fall back to the library record.
    let lib_result = conn.query_row(
        "SELECT title, developer, publisher, genre FROM games WHERE id = ?1",
        rusqlite::params![game_id],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        },
    );

    match lib_result {
        Ok((title, developer, publisher, genre)) => Ok(ResolvedMetadata {
            title,
            developer,
            publisher,
            genre,
            description: None,
            release_date: None,
            cover_url: None,
            source: MetadataSource::Library,
        }),
        Err(e) => Err(format!("Game not found in library: {e}")),
    }
}

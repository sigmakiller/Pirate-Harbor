//! Related-game lookup — T30.
//!
//! Shared by the Recommendation engine (T31) and the Game Detail page (T34).
//! All functions operate on `&Connection` so they compose without re-locking.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// ── Output types ──────────────────────────────────────────────────────────────

/// A related game returned by lookup queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedGame {
    pub id:                  String,
    pub title:               String,
    pub cover_path:          Option<String>,
    pub genre:               Option<String>,
    pub developer:           Option<String>,
    pub status:              String,
    pub total_playtime_secs: i64,
    /// Why this game is considered related.
    pub relation:            RelationKind,
}

/// The basis for the relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    SameGenre,
    SameDeveloper,
    SamePublisher,
    GenreAndDeveloper,
}

// ── Query helpers ─────────────────────────────────────────────────────────────

fn query_related(
    conn: &Connection,
    sql: &str,
    params: &[&dyn rusqlite::types::ToSql],
    relation: RelationKind,
) -> Result<Vec<RelatedGame>, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params, |r| {
            Ok(RelatedGame {
                id:                  r.get(0)?,
                title:               r.get(1)?,
                cover_path:          r.get(2)?,
                genre:               r.get(3)?,
                developer:           r.get(4)?,
                status:              r.get::<_, String>(5).unwrap_or_default(),
                total_playtime_secs: r.get::<_, i64>(6).unwrap_or(0),
                relation:            relation.clone(),
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Find up to `limit` games related to `game_id` by genre or developer.
///
/// Priority order: genre+developer match → genre only → developer only.
/// The source game itself is always excluded.
pub fn find_related_games(
    conn: &Connection,
    game_id: &str,
    limit: usize,
) -> Result<Vec<RelatedGame>, String> {
    // Fetch source game's attributes.
    let (genre, developer, publisher): (Option<String>, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT genre, developer, publisher FROM games WHERE id = ?1",
            rusqlite::params![game_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|e| format!("Source game not found: {e}"))?;

    let mut results: Vec<RelatedGame> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    seen.insert(game_id.to_string());

    // 1. Genre + developer match (strongest signal).
    if genre.is_some() && developer.is_some() {
        let sql = "SELECT id, title, cover_path_local, genre, developer, status, total_playtime_secs
                   FROM games
                   WHERE id != ?1 AND genre = ?2 AND developer = ?3
                   ORDER BY total_playtime_secs DESC LIMIT ?4";
        let rows = query_related(
            conn, sql,
            &[&game_id, &genre.as_deref().unwrap_or(""), &developer.as_deref().unwrap_or(""), &(limit as i64)],
            RelationKind::GenreAndDeveloper,
        )?;
        for r in rows {
            if seen.insert(r.id.clone()) {
                results.push(r);
            }
        }
    }

    // 2. Same genre only.
    if results.len() < limit {
        if let Some(ref g) = genre {
            let sql = "SELECT id, title, cover_path_local, genre, developer, status, total_playtime_secs
                       FROM games
                       WHERE id != ?1 AND genre = ?2
                       ORDER BY total_playtime_secs DESC LIMIT ?3";
            let rows = query_related(
                conn, sql,
                &[&game_id, g, &(limit as i64)],
                RelationKind::SameGenre,
            )?;
            for r in rows {
                if seen.insert(r.id.clone()) && results.len() < limit {
                    results.push(r);
                }
            }
        }
    }

    // 3. Same developer only.
    if results.len() < limit {
        if let Some(ref dev) = developer {
            let sql = "SELECT id, title, cover_path_local, genre, developer, status, total_playtime_secs
                       FROM games
                       WHERE id != ?1 AND developer = ?2
                       ORDER BY total_playtime_secs DESC LIMIT ?3";
            let rows = query_related(
                conn, sql,
                &[&game_id, dev, &(limit as i64)],
                RelationKind::SameDeveloper,
            )?;
            for r in rows {
                if seen.insert(r.id.clone()) && results.len() < limit {
                    results.push(r);
                }
            }
        }
    }

    // 4. Same publisher only (fallback).
    if results.len() < limit {
        if let Some(ref pub_) = publisher {
            let sql = "SELECT id, title, cover_path_local, genre, developer, status, total_playtime_secs
                       FROM games
                       WHERE id != ?1 AND publisher = ?2
                       ORDER BY total_playtime_secs DESC LIMIT ?3";
            let rows = query_related(
                conn, sql,
                &[&game_id, pub_, &(limit as i64)],
                RelationKind::SamePublisher,
            )?;
            for r in rows {
                if seen.insert(r.id.clone()) && results.len() < limit {
                    results.push(r);
                }
            }
        }
    }

    Ok(results)
}

/// Games in the given genres, excluding `exclude_id`.
pub fn find_by_genre_overlap(
    conn: &Connection,
    genres: &[String],
    exclude_id: &str,
    limit: usize,
) -> Result<Vec<RelatedGame>, String> {
    if genres.is_empty() {
        return Ok(vec![]);
    }
    // Build `WHERE genre IN (?, ?, …)` dynamically.
    let placeholders = genres
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 2))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT id, title, cover_path_local, genre, developer, status, total_playtime_secs
         FROM games
         WHERE id != ?1 AND genre IN ({})
         ORDER BY total_playtime_secs DESC LIMIT ?{}",
        placeholders,
        genres.len() + 2
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    params.push(Box::new(exclude_id.to_string()));
    for g in genres {
        params.push(Box::new(g.clone()));
    }
    params.push(Box::new(limit as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|b| b.as_ref()).collect();

    let rows = stmt
        .query_map(param_refs.as_slice(), |r| {
            Ok(RelatedGame {
                id:                  r.get(0)?,
                title:               r.get(1)?,
                cover_path:          r.get(2)?,
                genre:               r.get(3)?,
                developer:           r.get(4)?,
                status:              r.get::<_, String>(5).unwrap_or_default(),
                total_playtime_secs: r.get::<_, i64>(6).unwrap_or(0),
                relation:            RelationKind::SameGenre,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// All games by `developer`, excluding `exclude_id`.
pub fn find_by_developer(
    conn: &Connection,
    developer: &str,
    exclude_id: &str,
) -> Result<Vec<RelatedGame>, String> {
    let sql = "SELECT id, title, cover_path_local, genre, developer, status, total_playtime_secs
               FROM games
               WHERE developer = ?1 AND id != ?2
               ORDER BY total_playtime_secs DESC";
    query_related(
        conn, sql,
        &[&developer, &exclude_id],
        RelationKind::SameDeveloper,
    )
}

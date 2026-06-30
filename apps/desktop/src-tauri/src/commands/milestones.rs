//! Milestone commands — T21.
//!
//! Formal milestone tracking system with categories, difficulty levels,
//! and reusable templates.

use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::db::DbState;
use crate::models::{
    Milestone, MilestoneTemplate,
    NewMilestone, NewMilestoneTemplate,
};
use crate::analytics::milestones as milestone_analytics;

// ── CRUD Commands ─────────────────────────────────────────────────────────────

/// Create a new milestone
#[tauri::command]
pub fn create_milestone(
    db_state: State<'_, DbState>,
    payload: NewMilestone,
) -> Result<Milestone, String> {
    if payload.title.trim().is_empty() {
        return Err("Milestone title cannot be empty.".to_string());
    }

    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let achievement_date = payload
        .achievement_date
        .unwrap_or_else(|| now.clone());

    let category_str = payload.category.as_str();
    let difficulty_str = payload.difficulty.as_ref().map(|d| d.as_str());
    let points = payload.points.unwrap_or(0);

    conn.execute(
        "INSERT INTO milestones
             (id, game_id, title, description, category, difficulty,
              achievement_date, points, metadata, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
        rusqlite::params![
            id,
            payload.game_id,
            payload.title.trim(),
            payload.description,
            category_str,
            difficulty_str,
            achievement_date,
            points,
            payload.metadata,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Milestone {
        id,
        game_id: payload.game_id,
        title: payload.title.trim().to_string(),
        description: payload.description,
        category: payload.category,
        difficulty: payload.difficulty,
        achievement_date,
        points,
        metadata: payload.metadata,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// Get milestones with optional filters
#[tauri::command]
pub fn get_milestones(
    db_state: State<'_, DbState>,
    game_id: Option<String>,
    category: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<Milestone>, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    let mut query = String::from(
        "SELECT id, game_id, title, description, category, difficulty,
                achievement_date, points, metadata, created_at, updated_at
         FROM milestones WHERE 1=1",
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(gid) = game_id {
        query.push_str(" AND game_id = ?");
        params.push(Box::new(gid));
    }

    if let Some(cat) = category {
        query.push_str(" AND category = ?");
        params.push(Box::new(cat));
    }

    query.push_str(" ORDER BY achievement_date DESC");

    if let Some(lim) = limit {
        query.push_str(" LIMIT ?");
        params.push(Box::new(lim));
    }

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let milestones: Vec<Milestone> = stmt
        .query_map(param_refs.as_slice(), |row| {
            let category_str: String = row.get(4)?;
            let difficulty_str: Option<String> = row.get(5)?;

            let category = category_str.parse().unwrap_or_default();
            let difficulty = difficulty_str.and_then(|s| s.parse().ok());

            Ok(Milestone {
                id: row.get(0)?,
                game_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                category,
                difficulty,
                achievement_date: row.get(6)?,
                points: row.get(7)?,
                metadata: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(milestones)
}

/// Delete a milestone
#[tauri::command]
pub fn delete_milestone(
    db_state: State<'_, DbState>,
    id: String,
) -> Result<(), String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM milestones WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Template Commands ─────────────────────────────────────────────────────────

/// Get milestone templates with optional category filter
#[tauri::command]
pub fn get_milestone_templates(
    db_state: State<'_, DbState>,
    category: Option<String>,
) -> Result<Vec<MilestoneTemplate>, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    let (query, params): (String, Vec<String>) = if let Some(cat) = category {
        (
            "SELECT id, title, description, category, difficulty, is_global, created_at
             FROM milestone_templates WHERE category = ?1 ORDER BY title"
                .to_string(),
            vec![cat],
        )
    } else {
        (
            "SELECT id, title, description, category, difficulty, is_global, created_at
             FROM milestone_templates ORDER BY category, title"
                .to_string(),
            vec![],
        )
    };

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

    let templates: Vec<MilestoneTemplate> = stmt
        .query_map(param_refs.as_slice(), |row| {
            let category_str: String = row.get(3)?;
            let difficulty_str: Option<String> = row.get(4)?;
            let is_global_int: i64 = row.get(5)?;

            let category = category_str.parse().unwrap_or_default();
            let difficulty = difficulty_str.and_then(|s| s.parse().ok());

            Ok(MilestoneTemplate {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                category,
                difficulty,
                is_global: is_global_int != 0,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(templates)
}

/// Create a milestone template
#[tauri::command]
pub fn create_milestone_template(
    db_state: State<'_, DbState>,
    payload: NewMilestoneTemplate,
) -> Result<MilestoneTemplate, String> {
    if payload.title.trim().is_empty() {
        return Err("Template title cannot be empty.".to_string());
    }

    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let category_str = payload.category.as_str();
    let difficulty_str = payload.difficulty.as_ref().map(|d| d.as_str());
    let is_global_int = if payload.is_global { 1 } else { 0 };

    conn.execute(
        "INSERT INTO milestone_templates
             (id, title, description, category, difficulty, is_global, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            id,
            payload.title.trim(),
            payload.description,
            category_str,
            difficulty_str,
            is_global_int,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(MilestoneTemplate {
        id,
        title: payload.title.trim().to_string(),
        description: payload.description,
        category: payload.category,
        difficulty: payload.difficulty,
        is_global: payload.is_global,
        created_at: now,
    })
}

/// Create a milestone from a template
#[tauri::command]
pub fn create_milestone_from_template(
    db_state: State<'_, DbState>,
    template_id: String,
    game_id: String,
) -> Result<Milestone, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    // Fetch template
    let template: MilestoneTemplate = conn
        .query_row(
            "SELECT id, title, description, category, difficulty, is_global, created_at
             FROM milestone_templates WHERE id = ?1",
            rusqlite::params![template_id],
            |row| {
                let category_str: String = row.get(3)?;
                let difficulty_str: Option<String> = row.get(4)?;
                let is_global_int: i64 = row.get(5)?;

                let category = category_str.parse().unwrap_or_default();
                let difficulty = difficulty_str.and_then(|s| s.parse().ok());

                Ok(MilestoneTemplate {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    category,
                    difficulty,
                    is_global: is_global_int != 0,
                    created_at: row.get(6)?,
                })
            },
        )
        .map_err(|e| format!("Template not found: {}", e))?;

    // Create milestone from template
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let category_str = template.category.as_str();
    let difficulty_str = template.difficulty.as_ref().map(|d| d.as_str());

    conn.execute(
        "INSERT INTO milestones
             (id, game_id, title, description, category, difficulty,
              achievement_date, points, metadata, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, NULL, ?8, ?8)",
        rusqlite::params![
            id,
            game_id,
            template.title,
            template.description,
            category_str,
            difficulty_str,
            now,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Milestone {
        id,
        game_id,
        title: template.title,
        description: template.description,
        category: template.category,
        difficulty: template.difficulty,
        achievement_date: now.clone(),
        points: 0,
        metadata: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

// ── Seed default templates ────────────────────────────────────────────────────

/// Seed default milestone templates.
/// Uses deterministic IDs (UUID v5 namespaced by title+category) and
/// `INSERT OR IGNORE` so this function is safe to call multiple times.
#[tauri::command]
pub fn seed_default_templates(db_state: State<'_, DbState>) -> Result<usize, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    let templates = vec![
        // Completion
        ("First Completion", "Completed the game for the first time", "completion", Some("normal")),
        ("100% Completion", "Achieved 100% completion", "completion", Some("legendary")),
        ("Perfect Score", "Obtained a perfect score", "completion", Some("hard")),
        // Progress
        ("Reached Halfway Point", "Reached 50% completion", "progress", Some("easy")),
        ("Unlocked All Areas", "Discovered and unlocked all game areas", "progress", Some("normal")),
        ("Max Level", "Reached maximum character level", "progress", Some("normal")),
        // Exploration
        ("Found Secret Area", "Discovered a hidden or secret location", "exploration", Some("normal")),
        ("Discovered All Collectibles", "Found every collectible in the game", "exploration", Some("hard")),
        ("Map Completion", "Revealed the entire game map", "exploration", Some("easy")),
        // Mastery
        ("Speedrun Personal Best", "Set a new personal speedrun record", "mastery", Some("legendary")),
        ("No Death Run", "Completed without dying", "mastery", Some("legendary")),
        ("Hardest Difficulty", "Completed on the highest difficulty setting", "mastery", Some("hard")),
        // Social
        ("Played with Friends", "Enjoyed a gaming session with friends", "social", Some("trivial")),
        ("Community Achievement", "Participated in a community event or challenge", "social", Some("normal")),
    ];

    let mut inserted = 0;

    for (title, desc, category, difficulty) in templates {
        // Deterministic ID: hash category+title with std DefaultHasher to derive
        // a stable pseudo-UUID. Avoids needing the uuid `v5` feature flag.
        let seed = format!("{}:{}", category, title);
        let hash = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            seed.hash(&mut h);
            h.finish()
        };
        // Build a deterministic UUID-shaped string from the hash
        let id = format!(
            "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
            (hash >> 32) as u32,
            ((hash >> 16) & 0xFFFF) as u16,
            (hash & 0x0FFF) as u16,
            (((hash >> 48) & 0x3FFF) | 0x8000) as u16,
            hash & 0x0000_FFFF_FFFF,
        );

        let result = conn.execute(
            "INSERT OR IGNORE INTO milestone_templates
                 (id, title, description, category, difficulty, is_global, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
            rusqlite::params![id, title, desc, category, difficulty, now],
        );

        if result.is_ok() {
            inserted += 1;
        }
    }

    Ok(inserted)
}

// ── Statistics ────────────────────────────────────────────────────────────────

/// Get comprehensive milestone statistics
#[tauri::command]
pub fn get_milestone_statistics(
    db_state: State<'_, DbState>,
    game_id: Option<String>,
) -> Result<milestone_analytics::MilestoneStatistics, String> {
    milestone_analytics::calculate_statistics(
        &db_state,
        game_id.as_deref(),
    )
}

/// Migrate existing journal entries with entry_type='milestone' to milestones table.
/// This is a one-time migration that runs automatically on first launch after MIGRATION_006.
/// Preserves original journal entries and links them via metadata.
#[tauri::command]
pub fn migrate_journal_to_milestones(db_state: State<'_, DbState>) -> Result<usize, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    // Check if migration has already run
    let migrated_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM milestones WHERE metadata LIKE '%migrated_from_journal%'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if migrated_count > 0 {
        return Ok(0); // Already migrated
    }

    let now = Utc::now().to_rfc3339();
    let mut inserted = 0;

    // Fetch journal entries with entry_type='milestone'
    let mut stmt = conn
        .prepare(
            "SELECT id, game_id, game_title, title, body, created_at
             FROM journal_entries
             WHERE entry_type = 'milestone'
             ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let journal_entries: Vec<(String, Option<String>, Option<String>, Option<String>, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?, // id
                row.get::<_, Option<String>>(1)?, // game_id
                row.get::<_, Option<String>>(2)?, // game_title
                row.get::<_, Option<String>>(3)?, // title
                row.get::<_, String>(4)?, // body
                row.get::<_, String>(5)?, // created_at
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    drop(stmt); // Release borrow

    // Insert into milestones table
    for (journal_id, game_id, _game_title, title, body, created_at) in journal_entries {
        let milestone_id = Uuid::new_v4().to_string();
        let milestone_title = title.unwrap_or_else(|| "Milestone".to_string());
        let description = if !body.is_empty() { Some(body) } else { None };

        // Create metadata JSON with reference to original journal entry
        let metadata = serde_json::json!({
            "migrated_from_journal": true,
            "original_journal_id": journal_id
        })
        .to_string();

        let result = conn.execute(
            "INSERT INTO milestones
                 (id, game_id, title, description, category, difficulty,
                  achievement_date, points, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'completion', NULL, ?5, 0, ?6, ?7, ?7)",
            rusqlite::params![
                milestone_id,
                game_id,
                milestone_title,
                description,
                created_at,
                metadata,
                now
            ],
        );

        if result.is_ok() {
            inserted += 1;
        }
    }

    Ok(inserted)
}

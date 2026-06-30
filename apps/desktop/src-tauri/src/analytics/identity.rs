//! Identity dashboard analytics — T23.
//!
//! Provides comprehensive gaming profile analysis including genre preferences,
//! session patterns, completion behavior, and gaming personality classification.

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::db::DbState;

// ── Models ────────────────────────────────────────────────────────────────────

/// Comprehensive gaming identity profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingIdentity {
    pub profile_summary: ProfileSummary,
    pub favorite_genres: Vec<GenrePreference>,
    pub runtime_statistics: RuntimeStats,
    pub recent_journeys: Vec<RecentJourney>,
    pub completion_timeline: Vec<CompletionEvent>,
    pub gaming_personality: GamingPersonality,
}

/// High-level profile statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSummary {
    pub total_games: i64,
    pub total_playtime_secs: i64,
    pub completed_games: i64,
    pub playing_games: i64,
    pub favorite_games: i64,
    pub completion_rate: f64,
    pub gaming_since: Option<String>,
    pub total_sessions: i64,
    pub total_milestones: i64,
}

/// Genre preference with playtime weighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenrePreference {
    pub genre: String,
    pub game_count: i64,
    pub total_playtime_secs: i64,
    pub milestone_count: i64,
    pub preference_score: f64,
}

/// Runtime patterns and trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    pub total_playtime_secs: i64,
    pub average_session_secs: f64,
    pub longest_session_secs: i64,
    pub total_sessions: i64,
    pub sessions_last_30_days: i64,
    pub playtime_last_30_days_secs: i64,
    pub average_daily_playtime_secs: f64,
    pub most_active_hour: Option<i32>,
    pub streak_current_days: i64,
    pub streak_longest_days: i64,
}

/// Recent gaming activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentJourney {
    pub game_id: String,
    pub game_title: String,
    pub last_played: String,
    pub total_playtime_secs: i64,
    pub session_count: i64,
    pub status: String,
    pub progress_indicator: Option<String>,
}

/// Major completion milestones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionEvent {
    pub game_id: String,
    pub game_title: String,
    pub completed_at: String,
    pub total_playtime_secs: i64,
    pub milestone_count: i64,
}

/// Gaming personality classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingPersonality {
    pub primary_type: String,
    pub secondary_type: Option<String>,
    pub traits: Vec<String>,
    pub description: String,
}

// ── Analytics Functions ───────────────────────────────────────────────────────

/// Calculate comprehensive gaming identity profile
pub fn calculate_identity(db_state: &DbState) -> Result<GamingIdentity, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    // Profile summary
    let profile_summary = calculate_profile_summary(&conn)?;
    
    // Favorite genres
    let favorite_genres = calculate_genre_preferences(&conn)?;
    
    // Runtime statistics
    let runtime_statistics = calculate_runtime_stats(&conn)?;
    
    // Recent journeys
    let recent_journeys = calculate_recent_journeys(&conn)?;
    
    // Completion timeline
    let completion_timeline = calculate_completion_timeline(&conn)?;
    
    // Gaming personality
    let gaming_personality = classify_personality(&profile_summary, &favorite_genres, &runtime_statistics)?;

    Ok(GamingIdentity {
        profile_summary,
        favorite_genres,
        runtime_statistics,
        recent_journeys,
        completion_timeline,
        gaming_personality,
    })
}

/// Calculate high-level profile summary
fn calculate_profile_summary(conn: &rusqlite::Connection) -> Result<ProfileSummary, String> {
    let total_games: i64 = conn
        .query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))
        .unwrap_or(0);

    let total_playtime_secs: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_playtime_secs), 0) FROM games",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let completed_games: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE status = 'completed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let playing_games: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE status = 'playing'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let favorite_games: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE is_favorite = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let engaged_games: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE status != 'unplayed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1); // Avoid division by zero

    let completion_rate = if engaged_games > 0 {
        (completed_games as f64 / engaged_games as f64) * 100.0
    } else {
        0.0
    };

    let gaming_since: Option<String> = conn
        .query_row(
            "SELECT MIN(added_at) FROM games",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok();

    let total_sessions: i64 = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .unwrap_or(0);

    let total_milestones: i64 = conn
        .query_row("SELECT COUNT(*) FROM milestones", [], |row| row.get(0))
        .unwrap_or(0);

    Ok(ProfileSummary {
        total_games,
        total_playtime_secs,
        completed_games,
        playing_games,
        favorite_games,
        completion_rate,
        gaming_since,
        total_sessions,
        total_milestones,
    })
}

/// Calculate genre preferences weighted by playtime and milestones
fn calculate_genre_preferences(conn: &rusqlite::Connection) -> Result<Vec<GenrePreference>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT genre, 
                    COUNT(*) as game_count,
                    SUM(total_playtime_secs) as total_playtime
             FROM games
             WHERE genre IS NOT NULL AND genre != ''
             GROUP BY genre
             ORDER BY total_playtime DESC
             LIMIT 10",
        )
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    let mut genres = Vec::new();

    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let genre: String = row.get(0).unwrap_or_default();
        let game_count: i64 = row.get(1).unwrap_or(0);
        let total_playtime_secs: i64 = row.get(2).unwrap_or(0);

        // Count milestones for games in this genre
        let milestone_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM milestones 
                 WHERE game_id IN (SELECT id FROM games WHERE genre = ?1)",
                rusqlite::params![genre],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Calculate preference score: weighted by playtime and milestones
        let preference_score = (total_playtime_secs as f64 * 0.7) + (milestone_count as f64 * 500.0);

        genres.push(GenrePreference {
            genre,
            game_count,
            total_playtime_secs,
            milestone_count,
            preference_score,
        });
    }

    // Re-sort by preference score
    genres.sort_by(|a, b| b.preference_score.partial_cmp(&a.preference_score).unwrap());

    Ok(genres)
}

/// Calculate runtime statistics and patterns
fn calculate_runtime_stats(conn: &rusqlite::Connection) -> Result<RuntimeStats, String> {
    let total_playtime_secs: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_playtime_secs), 0) FROM games",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let total_sessions: i64 = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .unwrap_or(0);

    let average_session_secs = if total_sessions > 0 {
        total_playtime_secs as f64 / total_sessions as f64
    } else {
        0.0
    };

    let longest_session_secs: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(duration_secs), 0) FROM sessions",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Last 30 days activity
    let thirty_days_ago = (Utc::now() - Duration::days(30)).to_rfc3339();

    let sessions_last_30_days: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE started_at >= ?1",
            rusqlite::params![thirty_days_ago],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let playtime_last_30_days_secs: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM sessions WHERE started_at >= ?1",
            rusqlite::params![thirty_days_ago],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let average_daily_playtime_secs = playtime_last_30_days_secs as f64 / 30.0;

    // Most active hour (placeholder - requires more complex query)
    let most_active_hour = None;

    // Current and longest streak
    let streak_current_days = calculate_current_streak(conn)?;
    let streak_longest_days = calculate_longest_streak(conn)?;

    Ok(RuntimeStats {
        total_playtime_secs,
        average_session_secs,
        longest_session_secs,
        total_sessions,
        sessions_last_30_days,
        playtime_last_30_days_secs,
        average_daily_playtime_secs,
        most_active_hour,
        streak_current_days,
        streak_longest_days,
    })
}

/// Calculate current gaming streak (consecutive days with sessions)
fn calculate_current_streak(conn: &rusqlite::Connection) -> Result<i64, String> {
    let mut stmt = conn
        .prepare("SELECT DATE(started_at) as date FROM sessions ORDER BY started_at DESC")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    let mut dates = Vec::new();

    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        if let Ok(date) = row.get::<_, String>(0) {
            dates.push(date);
        }
    }

    if dates.is_empty() {
        return Ok(0);
    }

    let now = Utc::now();
    let mut streak = 0i64;
    let mut current_date = now.date_naive();

    for date_str in &dates {
        if let Ok(date_time) = DateTime::parse_from_rfc3339(date_str) {
            let date = date_time.date_naive();
            let diff = current_date.signed_duration_since(date).num_days();

            if diff <= 1 {
                streak += 1;
                current_date = date - Duration::days(1);
            } else {
                break;
            }
        }
    }

    Ok(streak)
}

/// Calculate longest gaming streak in history
fn calculate_longest_streak(conn: &rusqlite::Connection) -> Result<i64, String> {
    // Simplified: return current streak for now (full implementation would analyze all dates)
    calculate_current_streak(conn)
}

/// Calculate recent gaming journeys (last played games)
fn calculate_recent_journeys(conn: &rusqlite::Connection) -> Result<Vec<RecentJourney>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT g.id, g.title, g.last_played, g.total_playtime_secs, g.status,
                    (SELECT COUNT(*) FROM sessions WHERE game_id = g.id) as session_count
             FROM games g
             WHERE g.last_played IS NOT NULL
             ORDER BY g.last_played DESC
             LIMIT 10",
        )
        .map_err(|e| e.to_string())?;

    let journeys = stmt
        .query_map([], |row| {
            Ok(RecentJourney {
                game_id: row.get(0)?,
                game_title: row.get(1)?,
                last_played: row.get(2)?,
                total_playtime_secs: row.get(3)?,
                session_count: row.get(5)?,
                status: row.get(4)?,
                progress_indicator: None, // Could be enhanced based on status/milestones
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(journeys)
}

/// Calculate completion timeline (completed games in chronological order)
fn calculate_completion_timeline(conn: &rusqlite::Connection) -> Result<Vec<CompletionEvent>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT g.id, g.title, g.last_played, g.total_playtime_secs,
                    (SELECT COUNT(*) FROM milestones WHERE game_id = g.id) as milestone_count
             FROM games g
             WHERE g.status = 'completed' AND g.last_played IS NOT NULL
             ORDER BY g.last_played ASC",
        )
        .map_err(|e| e.to_string())?;

    let events = stmt
        .query_map([], |row| {
            Ok(CompletionEvent {
                game_id: row.get(0)?,
                game_title: row.get(1)?,
                completed_at: row.get(2)?,
                total_playtime_secs: row.get(3)?,
                milestone_count: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(events)
}

/// Classify gaming personality based on behavior patterns
fn classify_personality(
    profile: &ProfileSummary,
    genres: &[GenrePreference],
    runtime: &RuntimeStats,
) -> Result<GamingPersonality, String> {
    let mut traits = Vec::new();
    let mut scores: HashMap<&str, f64> = HashMap::new();

    // Completionist score
    if profile.completion_rate > 70.0 {
        traits.push("Completionist".to_string());
        scores.insert("completionist", profile.completion_rate);
    }

    // Explorer score (based on diverse genres)
    if genres.len() >= 5 {
        traits.push("Explorer".to_string());
        scores.insert("explorer", genres.len() as f64 * 10.0);
    }

    // Achiever score (based on milestones)
    let milestones_per_game = if profile.total_games > 0 {
        profile.total_milestones as f64 / profile.total_games as f64
    } else {
        0.0
    };
    if milestones_per_game > 2.0 {
        traits.push("Achiever".to_string());
        scores.insert("achiever", milestones_per_game * 20.0);
    }

    // Dedicated score (based on long sessions and streaks)
    if runtime.average_session_secs > 7200.0 || runtime.streak_longest_days > 7 {
        traits.push("Dedicated".to_string());
        scores.insert("dedicated", runtime.average_session_secs / 360.0);
    }

    // Casual score (shorter sessions, lower frequency)
    if runtime.average_session_secs < 3600.0 && runtime.sessions_last_30_days < 15 {
        traits.push("Casual".to_string());
        scores.insert("casual", 50.0);
    }

    // Collector score (many games in library)
    if profile.total_games > 50 {
        traits.push("Collector".to_string());
        scores.insert("collector", profile.total_games as f64);
    }

    // Determine primary and secondary types
    let mut sorted_scores: Vec<_> = scores.into_iter().collect();
    sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let primary_type = sorted_scores
        .get(0)
        .map(|(t, _)| t.to_string())
        .unwrap_or_else(|| "Casual".to_string());

    let secondary_type = sorted_scores.get(1).map(|(t, _)| t.to_string());

    let description = generate_personality_description(&primary_type, secondary_type.as_deref());

    Ok(GamingPersonality {
        primary_type,
        secondary_type,
        traits,
        description,
    })
}

/// Generate personality description based on classification
fn generate_personality_description(primary: &str, secondary: Option<&str>) -> String {
    let base = match primary {
        "completionist" => "You strive to see everything a game has to offer, achieving full completion",
        "explorer" => "You love discovering new experiences across diverse gaming genres",
        "achiever" => "You're driven by challenges and milestones, constantly pushing your limits",
        "dedicated" => "You commit deeply to your games with long, immersive sessions",
        "casual" => "You enjoy gaming at your own pace, fitting it into your lifestyle",
        "collector" => "You appreciate building a diverse library of gaming experiences",
        _ => "You have a unique gaming style",
    };

    if let Some(sec) = secondary {
        format!("{}, with a {} streak.", base, sec.to_lowercase())
    } else {
        format!("{}.", base)
    }
}

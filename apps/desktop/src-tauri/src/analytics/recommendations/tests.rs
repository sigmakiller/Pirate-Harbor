//! Unit tests for the T31 recommendation engine.

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::analytics::recommendations::{
        build_user_context, fetch_candidates,
        combiner::StrategyCombiner,
        content_based::ContentBasedStrategy,
        genre_strategy::GenreStrategy,
        playtime_strategy::PlaytimeStrategy,
        recency_strategy::RecencyStrategy,
        strategy::{RecommendationStrategy, UserContext},
        Candidate,
    };
    use crate::db::migrations::run_migrations;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn open_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        conn
    }

    fn insert_game(
        conn: &Connection,
        id: &str,
        title: &str,
        genre: Option<&str>,
        developer: Option<&str>,
        status: &str,
        playtime: i64,
        launch_count: i64,
        added_at: &str,
    ) {
        conn.execute(
            "INSERT INTO games (id, title, exe_path, genre, developer, status,
                               total_playtime_secs, launch_count, is_favorite,
                               added_at)
             VALUES (?1,?2,'/exe',?3,?4,?5,?6,?7,0,?8)",
            rusqlite::params![id, title, genre, developer, status, playtime, launch_count, added_at],
        ).unwrap();
    }

    fn make_candidate(
        genre: Option<&str>,
        playtime: i64,
        launch_count: i64,
        added_at: &str,
    ) -> Candidate {
        Candidate {
            id:            "test".to_string(),
            title:         "Test Game".to_string(),
            cover_path:    None,
            genre:         genre.map(String::from),
            developer:     None,
            publisher:     None,
            status:        "unplayed".to_string(),
            total_playtime: playtime,
            added_at:      added_at.to_string(),
            launch_count,
        }
    }

    fn make_ctx_with_genre(genre: &str, playtime: i64) -> UserContext {
        let mut map = std::collections::HashMap::new();
        map.insert(genre.to_string(), playtime);
        UserContext {
            genre_playtime:      map,
            total_playtime_secs: playtime,
            completion_rate:     0.5,
        }
    }

    // ── ContentBasedStrategy ──────────────────────────────────────────────────

    #[test]
    fn content_based_no_genre_returns_zero() {
        let conn = open_db();
        let ctx  = make_ctx_with_genre("RPG", 3600);
        let c    = make_candidate(None, 0, 0, "2024-01-01");
        let s    = ContentBasedStrategy.score(&conn, &c, &ctx);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn content_based_matching_genre_returns_positive() {
        let conn = open_db();
        let ctx  = make_ctx_with_genre("RPG", 3600);
        let c    = make_candidate(Some("RPG"), 0, 0, "2024-01-01");
        let s    = ContentBasedStrategy.score(&conn, &c, &ctx);
        assert_eq!(s, 1.0, "full playtime in genre should return 1.0");
    }

    #[test]
    fn content_based_partial_playtime() {
        let conn = open_db();
        // 1800s of RPG out of 3600s total → 0.5
        let mut map = std::collections::HashMap::new();
        map.insert("RPG".to_string(), 1800_i64);
        let ctx = UserContext { genre_playtime: map, total_playtime_secs: 3600, completion_rate: 0.0 };
        let c   = make_candidate(Some("RPG"), 0, 0, "2024-01-01");
        let s   = ContentBasedStrategy.score(&conn, &c, &ctx);
        assert!((s - 0.5).abs() < 1e-9);
    }

    #[test]
    fn content_based_unmatched_genre_returns_zero() {
        let conn = open_db();
        let ctx  = make_ctx_with_genre("RPG", 3600);
        let c    = make_candidate(Some("Shooter"), 0, 0, "2024-01-01");
        let s    = ContentBasedStrategy.score(&conn, &c, &ctx);
        assert_eq!(s, 0.0);
    }

    // ── GenreStrategy ─────────────────────────────────────────────────────────

    #[test]
    fn genre_strategy_matching_returns_one() {
        let conn = open_db();
        let ctx  = make_ctx_with_genre("RPG", 3600);
        let c    = make_candidate(Some("RPG"), 0, 0, "2024-01-01");
        assert_eq!(GenreStrategy.score(&conn, &c, &ctx), 1.0);
    }

    #[test]
    fn genre_strategy_no_match_returns_zero() {
        let conn = open_db();
        let ctx  = make_ctx_with_genre("RPG", 3600);
        let c    = make_candidate(Some("Shooter"), 0, 0, "2024-01-01");
        assert_eq!(GenreStrategy.score(&conn, &c, &ctx), 0.0);
    }

    #[test]
    fn genre_strategy_empty_history_returns_half() {
        let conn = open_db();
        let ctx  = UserContext { genre_playtime: Default::default(), total_playtime_secs: 0, completion_rate: 0.0 };
        let c    = make_candidate(Some("RPG"), 0, 0, "2024-01-01");
        assert_eq!(GenreStrategy.score(&conn, &c, &ctx), 0.5);
    }

    // ── PlaytimeStrategy ──────────────────────────────────────────────────────

    #[test]
    fn playtime_strategy_virgin_game_returns_one() {
        let conn = open_db();
        let ctx  = make_ctx_with_genre("RPG", 3600);
        let c    = make_candidate(None, 0, 0, "2024-01-01");
        assert_eq!(PlaytimeStrategy.score(&conn, &c, &ctx), 1.0);
    }

    #[test]
    fn playtime_strategy_launched_no_time_returns_half() {
        let conn = open_db();
        let ctx  = make_ctx_with_genre("RPG", 3600);
        let c    = make_candidate(None, 0, 1, "2024-01-01");
        assert_eq!(PlaytimeStrategy.score(&conn, &c, &ctx), 0.5);
    }

    #[test]
    fn playtime_strategy_has_playtime_returns_zero() {
        let conn = open_db();
        let ctx  = make_ctx_with_genre("RPG", 3600);
        let c    = make_candidate(None, 100, 1, "2024-01-01");
        assert_eq!(PlaytimeStrategy.score(&conn, &c, &ctx), 0.0);
    }

    // ── RecencyStrategy ───────────────────────────────────────────────────────

    #[test]
    fn recency_strategy_today_returns_one() {
        let conn = open_db();
        let ctx  = make_ctx_with_genre("RPG", 3600);
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let c    = make_candidate(None, 0, 0, &today);
        let s    = RecencyStrategy.score(&conn, &c, &ctx);
        assert_eq!(s, 1.0);
    }

    #[test]
    fn recency_strategy_old_game_returns_zero() {
        let conn = open_db();
        let ctx  = make_ctx_with_genre("RPG", 3600);
        let c    = make_candidate(None, 0, 0, "2020-01-01");
        let s    = RecencyStrategy.score(&conn, &c, &ctx);
        assert_eq!(s, 0.0);
    }

    // ── StrategyCombiner ─────────────────────────────────────────────────────

    #[test]
    fn combiner_ranks_matching_game_higher() {
        let conn = open_db();
        insert_game(&conn, "g1", "RPG Gem",     Some("RPG"),     None, "unplayed", 0, 0, "2025-01-01");
        insert_game(&conn, "g2", "Old Shooter", Some("Shooter"), None, "unplayed", 0, 0, "2020-01-01");
        // User loves RPG.
        let ctx = make_ctx_with_genre("RPG", 7200);
        let candidates = fetch_candidates(&conn).unwrap();
        let combiner   = StrategyCombiner::default_combiner();
        let results    = combiner.rank(&conn, &candidates, &ctx, 10);

        assert!(!results.is_empty(), "Should produce recommendations");
        // RPG Gem should rank higher than Old Shooter.
        let rpg_rank     = results.iter().position(|r| r.game_id == "g1");
        let shooter_rank = results.iter().position(|r| r.game_id == "g2");
        if let (Some(r), Some(s)) = (rpg_rank, shooter_rank) {
            assert!(r < s, "RPG should rank above Shooter when user loves RPG");
        }
    }

    #[test]
    fn combiner_excludes_played_games() {
        let conn = open_db();
        insert_game(&conn, "g1", "Unplayed",  Some("RPG"), None, "unplayed",   0,    0, "2024-01-01");
        insert_game(&conn, "g2", "Completed", Some("RPG"), None, "completed", 3600, 10, "2024-01-01");
        let ctx        = make_ctx_with_genre("RPG", 3600);
        let candidates = fetch_candidates(&conn).unwrap();
        assert!(candidates.iter().all(|c| c.id != "g2"), "Completed games should not be candidates");

        let combiner = StrategyCombiner::default_combiner();
        let results  = combiner.rank(&conn, &candidates, &ctx, 10);
        assert!(results.iter().all(|r| r.game_id != "g2"), "Completed game must not appear in results");
    }

    #[test]
    fn combiner_handles_empty_library() {
        let conn       = open_db();
        let ctx        = make_ctx_with_genre("RPG", 0);
        let candidates = fetch_candidates(&conn).unwrap();
        let combiner   = StrategyCombiner::default_combiner();
        let results    = combiner.rank(&conn, &candidates, &ctx, 10);
        assert!(results.is_empty(), "Empty library should produce no recommendations");
    }

    #[test]
    fn build_user_context_works_on_empty_db() {
        let conn = open_db();
        let ctx  = build_user_context(&conn).unwrap();
        assert_eq!(ctx.total_playtime_secs, 0);
        assert_eq!(ctx.completion_rate, 0.0);
        assert!(ctx.genre_playtime.is_empty());
    }

    #[test]
    fn build_user_context_captures_genre_playtime() {
        let conn = open_db();
        insert_game(&conn, "g1", "RPG A", Some("RPG"), None, "completed", 3600, 5, "2024-01-01");
        insert_game(&conn, "g2", "RPG B", Some("RPG"), None, "playing",   1800, 2, "2024-06-01");
        insert_game(&conn, "g3", "Shooter", Some("Shooter"), None, "unplayed", 0, 0, "2024-09-01");

        let ctx = build_user_context(&conn).unwrap();
        assert_eq!(ctx.total_playtime_secs, 5400);
        assert_eq!(*ctx.genre_playtime.get("RPG").unwrap(), 5400);
        assert!(!ctx.genre_playtime.contains_key("Shooter"), "Unplayed games contribute 0 to genre_playtime");
    }

    #[test]
    fn results_include_reason_strings() {
        let conn = open_db();
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        insert_game(&conn, "g1", "Fresh RPG", Some("RPG"), None, "unplayed", 0, 0, &today);
        let ctx        = make_ctx_with_genre("RPG", 3600);
        let candidates = fetch_candidates(&conn).unwrap();
        let combiner   = StrategyCombiner::default_combiner();
        let results    = combiner.rank(&conn, &candidates, &ctx, 5);

        assert!(!results.is_empty());
        assert!(!results[0].reason.is_empty(), "Reason string must not be empty");
    }
}

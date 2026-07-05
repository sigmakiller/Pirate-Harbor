# T26–T31 Review — Remaining Items

**Status:** 8/10 fixes verified ✅  
**Remaining:** 2 items (M5, m5) — low risk, no blocker

---

## M5 — Missing unit test for `year_in_review`

**Severity:** 🟡 Moderate  
**File:** `apps/desktop/src-tauri/src/analytics/year_in_review.rs`

### Problem

`year_in_review.rs` has no unit tests. The `most_active_month` and
`longest_session_secs` calculations have untested edge cases:

- Empty library (no sessions)
- Sessions spanning multiple years
- Tie-breaking when two months have equal session counts

### Required Fix

Add a `#[cfg(test)]` module to `year_in_review.rs` with at least these tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::migrations::run_migrations;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn year_in_review_empty_db_returns_zeroes() {
        let conn = setup();
        let year = chrono::Utc::now().format("%Y").to_string();
        let result = build_year_in_review(&conn, &year).unwrap();
        assert_eq!(result.total_playtime_secs, 0);
        assert_eq!(result.games_played, 0);
        assert_eq!(result.games_completed, 0);
        assert_eq!(result.longest_session_secs, 0);
        assert!(result.most_active_month.is_none());
    }

    #[test]
    fn most_active_month_detected_correctly() {
        let conn = setup();
        // Insert a game.
        conn.execute(
            "INSERT INTO games (id, title, status, added_at) VALUES ('g1', 'Game A', 'playing', datetime('now'))",
            [],
        ).unwrap();
        // Insert sessions: 3 in January, 1 in February.
        for _ in 0..3 {
            conn.execute(
                "INSERT INTO sessions (id, game_id, started_at, duration_secs)
                 VALUES (lower(hex(randomblob(16))), 'g1', '2025-01-15T10:00:00', 3600)",
                [],
            ).unwrap();
        }
        conn.execute(
            "INSERT INTO sessions (id, game_id, started_at, duration_secs)
             VALUES (lower(hex(randomblob(16))), 'g1', '2025-02-10T10:00:00', 3600)",
            [],
        ).unwrap();

        let result = build_year_in_review(&conn, "2025").unwrap();
        assert_eq!(result.most_active_month, Some(1)); // January = month 1
    }

    #[test]
    fn longest_session_recorded_correctly() {
        let conn = setup();
        conn.execute(
            "INSERT INTO games (id, title, status, added_at) VALUES ('g1', 'Game A', 'playing', datetime('now'))",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO sessions (id, game_id, started_at, duration_secs)
             VALUES (lower(hex(randomblob(16))), 'g1', '2025-06-01T10:00:00', 7200)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO sessions (id, game_id, started_at, duration_secs)
             VALUES (lower(hex(randomblob(16))), 'g1', '2025-06-02T10:00:00', 3600)",
            [],
        ).unwrap();

        let result = build_year_in_review(&conn, "2025").unwrap();
        assert_eq!(result.longest_session_secs, 7200);
    }
}
```

### Acceptance Criteria

- `cargo test` passes with the new tests
- All 3 test cases above are covered (or equivalent)

---

## m5 — Custom `Pipe` trait in `asset_manager.rs`

**Severity:** ⚪ Minor (code style)  
**File:** `apps/desktop/src-tauri/src/assets/asset_manager.rs`  
**Lines:** 546–552

### Problem

A custom `Pipe` trait is defined at the end of the file and used only once
in `dir_size()`. This is unconventional Rust idiom and adds noise.

```rust
// Current (lines 546–552):
trait Pipe: Sized {
    fn pipe<F: FnOnce(Self) -> R, R>(self, f: F) -> R { f(self) }
}
impl<T> Pipe for T {}
```

```rust
// Current usage in dir_size() line 490:
.sum::<u64>()
.pipe(Ok)
```

### Required Fix

Remove the `Pipe` trait and replace the single usage with a direct `Ok(...)` wrap:

```rust
// In dir_size(), replace:
.sum::<u64>()
.pipe(Ok)

// With:
let total = entries
    .filter_map(|e| e.ok())
    .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
    .sum::<u64>();
Ok(total)
```

Then delete the entire `// ── Pipe helper ──` section (lines 546–552).

### Acceptance Criteria

- `Pipe` trait and its `impl` are deleted
- `dir_size()` uses `Ok(total)` directly
- `cargo check` passes with no new warnings

---

## Checklist

- [ ] M5: Add 3 unit tests to `year_in_review.rs`
- [ ] m5: Remove `Pipe` trait, refactor `dir_size()` to use `Ok(total)`
- [ ] Run `cargo test` — all tests must pass
- [ ] Run `cargo check` — no new warnings

# Phase 4 Review — T26 through T31

**Reviewer:** Architect  
**Date:** 2026-07-03  
**Scope:** Tasks 26–31 (Infrastructure + Service layers)

---

## Build Status

| Check | Result |
|-------|--------|
| `cargo check` | ✅ Compiles (14 dead-code warnings) |
| `cargo test` | ✅ **34/34 passed** (0.17s) |
| `tsc --noEmit` | ✅ PASS |

---

## Per-Task Audit Summary

| Task | Verdict | Issues |
|------|---------|--------|
| **T26** Migration Versioning | ✅ Excellent | 0 |
| **T27** Background Jobs | ✅ Good | M3 |
| **T28** Asset Manager | ✅ Good | M4 |
| **T29** FTS5 Search | ⚠️ Has Bugs | **C1** |
| **T30** Analytics/Metadata | ⚠️ Has Bugs | **C3**, M1, M2, M5 |
| **T31** Recommendations | ⚠️ Has Bugs | **C2**, **C4** |

---

## Critical Issues (Must Fix)

### C1 — FTS5 genre search misses comma-separated genres

**File:** `commands/search.rs` → `query_games()` + Migration 007 triggers  
**Line:** 224 (FTS trigger), 98 (search query)

The FTS5 index stores the raw `genre` column as-is (e.g. `"Action, RPG, Adventure"`). When a user searches for `"RPG"`, FTS5 tokenizes on whitespace, so it matches. **However**, the `games_fts` trigger inserts the raw genre string including commas. This means FTS5 sees tokens like `"Action,"` with a trailing comma, which won't match a bare `"Action"` search.

**Fix:** Strip commas in the FTS trigger, or normalize genres before insertion:

```sql
-- In games_ai trigger, replace:
VALUES (new.rowid, new.title, new.developer, new.publisher, new.genre);
-- With:
VALUES (new.rowid, new.title, new.developer, new.publisher,
        REPLACE(new.genre, ',', ' '));
```

Apply the same fix to `games_au` and the `rebuild` command.

---

### C2 — Recommendation genre matching uses exact string comparison

**File:** `commands/recommendations.rs:70-73`

```rust
let genre_match = genre.as_ref()
    .zip(c.genre.as_ref())
    .map(|(a, b)| a == b)
    .unwrap_or(false);
```

This compares the **entire** genre string (`"Action, RPG"` == `"Action, RPG"`). If two games have even slightly different genre orderings or subsets (`"RPG, Action"` vs `"Action, RPG"`), they won't match. The same problem exists in `content_based.rs` and `genre_strategy.rs`.

**Fix:** Split genre strings by comma and check for **any overlap**:

```rust
fn genres_overlap(a: Option<&str>, b: Option<&str>) -> bool {
    let (Some(a), Some(b)) = (a, b) else { return false };
    let set_a: HashSet<&str> = a.split(',').map(str::trim).collect();
    let set_b: HashSet<&str> = b.split(',').map(str::trim).collect();
    !set_a.is_disjoint(&set_b)
}
```

---

### C3 — `cover_path_local` may be NULL for most games

**File:** `analytics/recommendations/mod.rs:155`, `analytics/gaming_stats.rs:98`, `metadata/game_lookup.rs:89`

Multiple queries use `cover_path_local` instead of `cover_path`. The `cover_path_local` column was added by ALTER TABLE in Migration 005 and is only populated after metadata enrichment. For games added manually or via scanner, this column is NULL — causing recommendations and related games to always show no cover art.

**Fix:** Use `COALESCE(cover_path_local, cover_path)` in all SELECT statements, or alias consistently:

```sql
SELECT id, title, COALESCE(cover_path_local, cover_path) AS cover,
       genre, developer, status, total_playtime_secs
FROM games ...
```

**Affected files:**
- `analytics/recommendations/mod.rs` line 155
- `analytics/gaming_stats.rs` line 98
- `metadata/game_lookup.rs` lines 89, 108, 128, 148, 186, 231
- `commands/search.rs` line 98

---

### C4 — Dead `Recommendation` struct in strategy.rs causes confusion

**File:** `analytics/recommendations/strategy.rs:30`

There are **two** Recommendation types:
1. `strategy::Recommendation` (lines 30-34) — never constructed, generates warning
2. `mod::RecommendationResult` (lines 40-53) — the actual public type

The dead `Recommendation` struct is confusing and generates a compiler warning. Either remove it entirely or use it as the internal type it was meant to be.

**Fix:** Delete `strategy::Recommendation` struct (lines 29-34). It serves no purpose since `RecommendationResult` is the actual output type.

---

## Moderate Issues (Should Fix)

### M1 — `build_user_context` groups genres by raw column, not individual genres

**File:** `analytics/recommendations/mod.rs:94`

```sql
SELECT genre, SUM(total_playtime_secs) as pt
FROM games
WHERE genre IS NOT NULL ...
GROUP BY genre
```

This groups by the full genre string (`"Action, RPG"` vs `"RPG"`), so a game tagged `"Action, RPG"` contributes 0 playtime to the `"RPG"` key. The UserContext's `genre_playtime` map will have entries like `"Action, RPG": 50000` instead of `"Action": 50000, "RPG": 50000`.

**Fix:** Compute in Rust by splitting the genre column after fetching per-game playtime:

```rust
let mut stmt = conn.prepare(
    "SELECT genre, total_playtime_secs FROM games WHERE genre IS NOT NULL AND total_playtime_secs > 0"
)?;
for (genre_str, pt) in rows {
    for g in genre_str.split(',').map(str::trim) {
        *genre_playtime.entry(g.to_string()).or_insert(0) += pt;
    }
}
```

---

### M2 — Heatmap uses UTC timestamps, not local time

**File:** `analytics/gaming_stats.rs:168`

`strftime('%H', started_at)` operates on the stored ISO 8601 UTC timestamp. A session at 11 PM local time (IST, UTC+5:30) is stored as `17:30 UTC`, so it appears in the 17:00 hour bucket — misleading for the user.

**Fix:** Add `'localtime'` modifier: `strftime('%H', started_at, 'localtime')`. Same for `%w`.

---

### M3 — Worker has no concurrency limit

**File:** `background/worker.rs:31-107`

The worker loop picks up one job at a time (sequentially), but there's no concurrency cap if multiple `spawn_blocking` calls overlap. Currently safe because the loop `await`s each job, but if the loop is ever made concurrent (e.g., for parallelism), there's no guard.

**Fix:** Add a comment documenting that the sequential design is intentional, or add a `Semaphore` for future-proofing.

---

### M4 — `deduplicate()` returns thumbnail path, not original asset path

**File:** `assets/asset_manager.rs:155-160`

When a duplicate is detected, the returned `AssetRef` points to the thumbnail file, not the original cover/background. Callers expecting a cover path will get a thumbnail path.

**Fix:** Store the original asset path in the dedup marker (write path as content instead of empty bytes), or document that callers must reconstruct the cover path from the hash.

---

### M5 — `year_in_review.rs` not verified by any test

**File:** `analytics/year_in_review.rs`

This module has no unit tests. The `most_active_month` and `longest_session_secs` calculations should be verified.

**Fix:** Add at least one integration test that inserts sessions across months and verifies the output.

---

## Minor / Nitpick (Nice to Have)

### m1 — 14 dead-code warnings should be suppressed

The 14 warnings come from `metadata/` and `strategy.rs` modules that are pre-wired but not yet consumed by commands. Add `#[allow(dead_code)]` at the module level for `metadata/` files, or suppress with `#![allow(dead_code)]` at the top of each file.

### m2 — `background/queue.rs` Priority enum only has Normal and High

No `Low` priority exists. Consider adding it for auto-backup and cleanup jobs.

### m3 — `fts_escape` wraps query in phrase mode quotes

`fts_escape("witcher 3")` produces `"witcher 3"*` which is a prefix phrase query. This means searching for `"red dead"` works, but `"red witcher"` won't find separate games. Consider offering both phrase and OR modes.

### m4 — `SearchOverlay` component not verified in review

The `SearchOverlay.tsx` component was not audited. Verify it handles empty results, loading states, and keyboard navigation (Escape to close, arrow keys to navigate results).

### m5 — `Pipe` trait in `asset_manager.rs:524-528` is unconventional

A custom `Pipe` trait for `.pipe(Ok)` is unexpected in a Rust codebase. Use `Ok(sum)` directly or use the `tap` crate.

---

## Action Items for Engineer

| ID | Severity | Task | File(s) |
|----|----------|------|---------|
| **C1** | 🔴 Critical | Fix FTS5 trigger to strip commas from genre | `db/migrations.rs` (triggers) |
| **C2** | 🔴 Critical | Fix recommendation genre matching to use overlap | `commands/recommendations.rs`, strategy files |
| **C3** | 🔴 Critical | Use `COALESCE(cover_path_local, cover_path)` everywhere | 8 files listed above |
| **C4** | 🔴 Critical | Delete dead `Recommendation` struct | `strategy.rs` |
| **M1** | 🟡 Moderate | Split comma genres in `build_user_context` | `recommendations/mod.rs` |
| **M2** | 🟡 Moderate | Add `'localtime'` to heatmap `strftime` calls | `gaming_stats.rs` |
| **M3** | 🟡 Moderate | Document sequential worker design or add semaphore | `background/worker.rs` |
| **M4** | 🟡 Moderate | Fix dedup return to point at original, not thumbnail | `asset_manager.rs` |
| **M5** | 🟡 Moderate | Add unit test for year_in_review | `year_in_review.rs` |
| m1 | ⚪ Minor | Suppress dead-code warnings on metadata module | `metadata/*.rs` |
| m5 | ⚪ Minor | Remove custom `Pipe` trait | `asset_manager.rs` |

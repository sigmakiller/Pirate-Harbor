# Architect Review — Phase 3 Implementation (T18–T24)

**Date:** 2026-06-30  
**Reviewer:** Architect  
**Commits:** `f078b54` (T18) → `2cecc7a` (T19) → `9a1921a` (T20) → `ad016b8` (T21) → `c2eaedf` (T22) → `48a6a0c` (fix) → `b37cfe1` (T23) → `530d910` (T24)  
**Build:** `cargo check` ✅ (18 warnings) | `tsc --noEmit` ❌ **4 ERRORS**

---

## Executive Summary

| Task | Description | Backend | Frontend | Verdict |
|------|-------------|---------|----------|---------|
| **T18** | Tech debt (M1 tx, M2 cover_mode) | ✅ | ✅ | ✅ Pass |
| **T19** | Metadata API (RAWG/IGDB) | ✅ | ⚠️ | ⚠️ Issues |
| **T20** | Image download system | ✅ | N/A | ✅ Pass |
| **T21** | Milestone schema + templates | ✅ | ✅ | ✅ Pass |
| **T22** | Milestone analytics | ✅ | ✅ | ✅ Pass |
| **T23** | Identity dashboard | ✅ | ✅ | ✅ Pass |
| **T24** | Enrichment UI integration | N/A | ❌ | ❌ **Broken** |

---

## Phase 3 Plan Quality Assessment

The plan (`docs/phase3_plan.md`) is architecturally comprehensive with good dependency graphs and acceptance criteria. However, it had a critical synchronization problem: **it specified creating migrations that already existed** (C1–C6 in the pre-impl review). The Kiro review (`review_phase3_plan_t18_t22.md`) correctly caught this, and the fix commit `48a6a0c` addressed the critical issues.

**Verdict:** Plan was solid conceptually but needed the code-state audit. The review process worked.

---

## CRITICAL ISSUES (Blocking)

### ❌ C1 — `tsc --noEmit` FAILS: 4 compilation errors

These are **blocking** — the app will not build.

#### Error 1: `bulkEnrichLibrary` not exported from `@/lib/api`

**File:** `src/pages/LibraryPage.tsx:29`
```typescript
import { getAllGames, bulkEnrichLibrary } from "@/lib/api";
//                    ^^^^^^^^^^^^^^^^^ TS2305: has no exported member
```

**Root cause:** `api.ts` does not define or export `bulkEnrichLibrary`. The Rust command `bulk_enrich_library` exists and is registered, but the TypeScript wrapper was never added.

**Fix:** Add to `src/lib/api.ts`:
```typescript
export async function bulkEnrichLibrary(): Promise<void> {
  return invoke<void>("bulk_enrich_library");
}
```

#### Error 2–3: `addToast` signature mismatch

**File:** `src/pages/LibraryPage.tsx:109,111`
```typescript
addToast("Library enrichment started", "info");   // TS2554: Expected 1 argument
addToast(`Enrichment failed: ${e}`, "error");      // TS2554: Expected 1 argument
```

**Root cause:** `useToastStore.addToast` signature is `(toast: Omit<Toast, "id">) => void` which takes an **object** `{ message, type }`, not two positional args.

**Fix:** Change to:
```typescript
addToast({ message: "Library enrichment started", type: "info" });
addToast({ message: `Enrichment failed: ${e}`, type: "error" });
```

#### Error 4: `Milestone` type not re-exported from `@/lib/api`

**File:** `src/pages/MilestonesPage.tsx:27`
```typescript
import { getMilestones, migrateJournalToMilestones, getAllGames, type Milestone } from "@/lib/api";
//                                                                    ^^^^^^^^^ TS2459: declares locally, not exported
```

**Root cause:** `api.ts` uses `import type { Milestone } from "@/types"` (line 353) but doesn't re-export it. MilestonesPage tries to import it from `@/lib/api`.

**Fix:** Change the import in `api.ts` from:
```typescript
import type { Milestone, ... } from "@/types";
```
to:
```typescript
export type { Milestone, NewMilestone, MilestoneTemplate, NewMilestoneTemplate, MilestoneStatistics } from "@/types";
```
Or: change MilestonesPage to import from `@/types` directly.

---

### ⚠️ C2 — Duplicate `metadata_cache` Table Definition

**File:** `src-tauri/src/db/migrations.rs`

MIGRATION_002 (line 51) creates `metadata_cache` with columns `(query, results_json, cached_at)`.
MIGRATION_005 (line 101) creates `metadata_cache` with columns `(id, game_title, provider, api_id, metadata, cached_at, expires_at)`.

Due to `CREATE TABLE IF NOT EXISTS`, only MIGRATION_002's schema takes effect. **MIGRATION_005's `metadata_cache` never gets created.** Any code using `provider`, `api_id`, `expires_at` columns will fail at runtime.

**Fix (pick one):**
- **Option A (Recommended):** Rename MIGRATION_002's table from `metadata_cache` to `search_cache` (matches its existing comment "Search cache"). Update all MIGRATION_002 references.
- **Option B:** Rename MIGRATION_005's table to `game_metadata_cache`.

> [!CAUTION]
> This is a **runtime crash** waiting to happen. The `cache_metadata()` function in `commands/metadata.rs:107` inserts into `metadata_cache` with MIGRATION_005 columns — this will fail with a schema mismatch on any real database.

---

## MODERATE ISSUES

### M1 — 18 Dead-Code Warnings in `cargo check`

| Module | Dead items |
|--------|-----------|
| `api/igdb.rs` | `IgdbClient`, `IgdbGame`, `IgdbCover`, `IgdbGenre`, `IgdbReleaseDate`, `search_games`, `rate_limit`, `new` |
| `api/rawg.rs` | `get_game` method |
| `images/` | `ImageType::Thumbnail`, `filename_suffix`, `download_images_batch`, `convert_format`, `local_path`, `size_bytes` |
| `analytics/milestones.rs` | `TimelineEntry`, `DistributionEntry`, `MetadataResult` |
| `commands/metadata.rs` | `fetch_from_igdb` |

**Impact:** Not blocking, but indicates IGDB client and image processor are scaffolded but not wired in. This is expected for Phase 3 partial delivery but should be cleaned up.

**Fix:** Either wire the dead code into active commands, or add `#[allow(dead_code)]` with `// T25: will be used in polish phase` comments to suppress warnings intentionally.

---

### M2 — IGDB Integration is Placeholder

`api/igdb.rs` is fully coded but `fetch_from_igdb()` in `metadata.rs:164` returns a hardcoded error:
```rust
Err("IGDB integration not yet implemented".to_string())
```

The plan specified IGDB as a **fallback** provider. Currently there is no fallback — if RAWG fails, enrichment fails entirely.

**Impact:** Reduced resilience. RAWG outages = no enrichment.

**Fix:** Either wire IGDB properly or document as T25 scope.

---

### M3 — `seed_default_templates` Creates Duplicates on Re-run

`milestones.rs:364-376` uses `INSERT INTO` with a new UUID each time. If called twice, templates duplicate.

**Fix:** Use `INSERT OR IGNORE` with deterministic template IDs (e.g., UUID from hash of title+category) or check for existing templates before inserting.

---

### M4 — `calculate_longest_streak` Returns Current Streak

`analytics/identity.rs:373`:
```rust
fn calculate_longest_streak(conn: &rusqlite::Connection) -> Result<i64, String> {
    calculate_current_streak(conn) // ← Placeholder, not actual longest
}
```

**Impact:** `streak_longest_days` will always equal `streak_current_days`. Misleading for users who had longer historical streaks.

**Fix:** Implement proper distinct-day analysis with gap detection.

---

### M5 — `most_active_hour` Always `None`

`analytics/identity.rs:311`:
```rust
let most_active_hour = None; // placeholder
```

**Impact:** Identity dashboard can't show preferred gaming time. Minor UX gap.

**Fix:** Add `SELECT strftime('%H', started_at) as hour, COUNT(*) FROM sessions GROUP BY hour ORDER BY COUNT(*) DESC LIMIT 1`.

---

### M6 — `EnrichmentProgressBar` and `useEnrichmentProgress` Not Verified

`LibraryPage.tsx:28,31` imports these components but they were not included in the diff stats check. Need to verify they exist:

---

### M7 — Migration Idempotency Issue with `ALTER TABLE`

MIGRATION_005 has:
```sql
ALTER TABLE games ADD COLUMN cover_path_local TEXT;
ALTER TABLE games ADD COLUMN background_path_local TEXT;
ALTER TABLE games ADD COLUMN images_enriched_at TEXT;
```

`ALTER TABLE ADD COLUMN` will **fail** on second run if column already exists (SQLite doesn't support `IF NOT EXISTS` for `ALTER TABLE`). However, `run_migrations` runs all migrations every time.

> [!WARNING]
> The migration test `test_migrations_are_idempotent` passes because it runs on an in-memory database — columns are added fresh. But on a **persisted database** where MIGRATION_005 was already applied, re-running will error.

**Fix:** Wrap ALTER TABLE in a check:
```rust
// Check if column exists before adding
fn add_column_if_not_exists(conn: &Connection, table: &str, column: &str, col_type: &str) {
    let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, col_type);
    let _ = conn.execute_batch(&sql); // Ignore error if already exists
}
```
Or restructure migrations to run once with a version table.

---

## MINOR ISSUES

| # | File | Issue |
|---|------|-------|
| m1 | `identity.rs:355` | `DATE(started_at)` uses `DateTime::parse_from_rfc3339` on a `DATE()` result — these are `YYYY-MM-DD` strings, not RFC3339. Will never parse successfully. Streak calculation is broken. |
| m2 | `milestones.rs:400-413` | `migrate_journal_to_milestones` checks for prior migration via `LIKE '%migrated_from_journal%'` — fragile text search. Should use settings table flag. |
| m3 | `metadata.rs:154` | `RawgClient::new()` is called per-request in `fetch_from_rawg`. Should be a shared/cached client to maintain rate limiter state across calls. |
| m4 | `MilestonesPage.tsx:56-60` | `migrateJournalToMilestones()` runs on **every page load**. Even though it's idempotent, it's a wasted DB query on each render. Should check a settings flag first. |

---

## What's Working Well

1. **Migration schema** — All 6 migrations are correctly structured with proper FK constraints, cascades, and indexes.
2. **Milestones CRUD** — Complete and correct. Dynamic query building with `WHERE 1=1` pattern is clean.
3. **Template seeding** — Good coverage of default templates across 5 categories.
4. **Journal→Milestone migration** — Thoughtful approach using metadata JSON for backward linking.
5. **Identity analytics** — Comprehensive struct design (`GamingIdentity`) with personality classification.
6. **RAWG client** — Proper rate limiting with sliding window + exponential backoff.
7. **MilestonesPage.tsx** — Excellent UI: sidebar stats, game filter, rare milestones strip, grouped timeline with stem dots. Follows Atlas OS design consistently.

---

## Engineer Action Items

### BLOCKING (must fix before deployment)

| # | Fix | File(s) | Est. |
|---|-----|---------|------|
| **C1a** | Add `bulkEnrichLibrary` export to `api.ts` | `src/lib/api.ts` | 2 min |
| **C1b** | Fix `addToast` calls to use object syntax | `src/pages/LibraryPage.tsx:109,111` | 2 min |
| **C1c** | Re-export `Milestone` type from `api.ts` | `src/lib/api.ts:353` | 2 min |
| **C2** | Rename MIGRATION_002 table to `search_cache` | `src-tauri/src/db/migrations.rs:51` | 5 min |

### HIGH PRIORITY

| # | Fix | File(s) | Est. |
|---|-----|---------|------|
| **M3** | Use deterministic IDs for template seeding | `commands/milestones.rs:364-376` | 10 min |
| **M7** | Handle `ALTER TABLE` idempotency | `db/migrations.rs:124-126` | 15 min |
| **m1** | Fix streak date parsing (DATE vs RFC3339) | `analytics/identity.rs:355` | 10 min |
| **m3** | Cache `RawgClient` instance across requests | `commands/metadata.rs:154` | 15 min |

### NICE-TO-HAVE (T25 Polish)

| # | Fix |
|---|-----|
| **M1** | Clean up 18 dead-code warnings |
| **M2** | Wire IGDB fallback or document as future scope |
| **M4** | Implement real `calculate_longest_streak` |
| **M5** | Implement `most_active_hour` query |
| **m2** | Use settings flag for migration check |
| **m4** | Gate `migrateJournalToMilestones` behind settings flag |

---

## Verification Checklist

After fixing blocking issues, run:

```bash
# Must both pass clean
cargo check                                    # 0 errors expected
pnpm --filter desktop exec tsc --noEmit        # 0 errors expected

# Rust tests
cargo test                                     # migration tests pass
```

Then manually test:
- [ ] Library → "Enrich Library" button → no crash
- [ ] Milestones page loads without errors
- [ ] Identity page shows gaming profile
- [ ] New database (delete DB, restart) → all 6 migrations run cleanly
- [ ] Existing database → re-run migrations without errors (idempotency)

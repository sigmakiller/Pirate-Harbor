# Architect Review — Phase 3 Plan: T18–T22

**Date:** 2026-06-29  
**Reviewer:** Architect  
**Status:** Pre-implementation architectural review

---

## Executive Summary

| Task | Status | Severity | Issues Found |
|------|--------|----------|--------------|
| **T18** | ⚠️ **ISSUES FOUND** | CRITICAL | 3 critical, 2 moderate |
| **T19** | ⚠️ **ISSUES FOUND** | CRITICAL | 2 critical, 3 moderate |
| **T20** | ⚠️ **ISSUES FOUND** | MODERATE | 0 critical, 4 moderate |
| **T21** | ⚠️ **ISSUES FOUND** | MODERATE | 1 critical, 2 moderate |
| **T22** | ✅ **APPROVED** | MINOR | 0 critical, 1 minor |

**Overall Verdict:** ❌ **REVISIONS REQUIRED** — Critical issues must be addressed before implementation begins.

---

## Task 18 — Technical Debt Resolution

### ✅ **APPROVED ITEMS**

1. **M1 Transaction Wrapping** — Already implemented in scanner.rs (lines 343-414)
   - Code review shows transaction is already wrapped properly
   - Uses `conn.transaction()` → `tx.commit()` pattern
   - **Status:** ✅ NO WORK NEEDED

### ❌ **CRITICAL ISSUES**

#### C1 — Migration Numbering Conflict

**Problem:** Plan specifies adding `MIGRATION_004` for cover_mode, but migrations.rs shows:
- MIGRATION_001 through MIGRATION_006 already exist
- MIGRATION_002 already exists (metadata_cache for RAWG search)
- MIGRATION_003 already includes `cover_path` and `cover_mode` columns
- MIGRATION_004 already exists (journal_entries)
- MIGRATION_005 already exists (metadata enrichment queue + image columns)
- MIGRATION_006 already exists (milestones + templates)

**Current State:**
```rust
const MIGRATIONS: &[&str] = &[
    MIGRATION_001,  // games, sessions, settings
    MIGRATION_002,  // metadata_cache (RAWG)
    MIGRATION_003,  // collections with cover_path + cover_mode ✅
    MIGRATION_004,  // journal_entries
    MIGRATION_005,  // metadata_enrichment_queue + image columns ✅
    MIGRATION_006,  // milestones + milestone_templates ✅
];
```

**Impact:** Following the plan would create duplicate/conflicting migrations.

**Resolution Required:**
- **M2 Fix is ALREADY COMPLETE** — MIGRATION_003 includes cover_mode/cover_path
- T18 should ONLY verify the implementation, not add new migrations
- Plan must be updated to reflect current migration state

#### C2 — M2 Frontend Implementation Missing

**Problem:** While the database schema includes cover_mode, the plan doesn't verify if the frontend actually implements the auto mosaic vs custom rendering logic.

**Missing Verification Steps:**
1. Check if `CollectionCard.tsx` implements 2×2 mosaic rendering
2. Verify `cover_mode` enum handling in TypeScript types
3. Confirm collection edit UI has cover mode toggle

**Resolution Required:**
- Add explicit verification step to T18
- Engineer must check existing implementation status
- If incomplete, complete the frontend rendering logic

#### C3 — Incomplete Dependency Specification for T19

**Problem:** T18 is marked as a dependency for T19, but:
- M1 is already complete (transaction wrapping exists)
- M2 database schema is complete (cover_mode exists)
- Only frontend verification remains

**Impact:** T19 can begin immediately if only frontend polish is needed for T18.

**Resolution Required:**
- Clarify which T18 deliverables are blocking vs. nice-to-have
- Update dependency graph if T19 can proceed in parallel

### ⚠️ **MODERATE ISSUES**

#### M1 — Verification Scope Unclear

**Problem:** Plan states "cargo check, collections work with both cover modes" but doesn't specify:
- How to manually test mosaic vs custom mode
- What constitutes "working" for each mode
- Expected visual output or acceptance criteria

**Resolution Required:**
- Add detailed verification steps with screenshots or descriptions
- Specify test data requirements (e.g., collection with 4+ games, collection with custom cover)

#### M2 — No Rollback Plan

**Problem:** If M2 frontend changes break existing collections, there's no rollback strategy specified.

**Resolution Required:**
- Document current collection behavior as baseline
- Add rollback procedure if frontend changes cause regressions

---

## Task 19 — Metadata API Integration Layer

### ❌ **CRITICAL ISSUES**

#### C4 — MIGRATION_005 Already Exists

**Problem:** Plan specifies creating `MIGRATION_005` for metadata_cache and enrichment_queue, but this migration already exists in migrations.rs (lines 87-112).

**Current MIGRATION_005 Contents:**
- `metadata_cache` table ✅
- `metadata_enrichment_queue` table ✅  
- Image tracking columns (`cover_path_local`, `background_path_local`, `images_enriched_at`) ✅

**Impact:** T19 database work is ALREADY COMPLETE.

**Resolution Required:**
- Update plan to reflect that schema is in place
- T19 should focus ONLY on Rust implementation and API integration
- Remove migration creation from T19 scope

#### C5 — Duplicate metadata_cache Table Definition

**Problem:** Two different metadata_cache table definitions exist:
1. **MIGRATION_002** (line 50): Simple RAWG search cache
   ```sql
   query TEXT PRIMARY KEY,
   results_json TEXT NOT NULL,
   cached_at TEXT NOT NULL
   ```
2. **MIGRATION_005** (line 90): Enhanced metadata cache
   ```sql
   id TEXT PRIMARY KEY,
   game_title TEXT NOT NULL,
   provider TEXT NOT NULL,
   api_id INTEGER,
   metadata TEXT NOT NULL,
   cached_at TEXT NOT NULL,
   expires_at TEXT NOT NULL
   ```

**Current Behavior:** Both migrations attempt to CREATE TABLE IF NOT EXISTS, so only the first wins.

**Impact:**
- MIGRATION_005's enhanced schema never gets created
- Code expecting MIGRATION_005 schema will fail
- Database inconsistency between plan and reality

**Resolution Required:**
- **Option A:** Drop MIGRATION_002's metadata_cache, rename in MIGRATION_005
- **Option B:** Rename MIGRATION_005 table to `metadata_cache_v2`
- **Option C:** Add ALTER TABLE migration to evolve MIGRATION_002 schema
- **RECOMMENDED:** Option A — deprecate simple cache, use enhanced cache everywhere

### ⚠️ **MODERATE ISSUES**

#### M3 — Missing API Key Management

**Problem:** Plan mentions RAWG and IGDB integration but doesn't specify:
- Where API keys are stored (environment variables? settings table?)
- How users provide API keys (Settings page? first-run wizard?)
- Fallback behavior if keys are missing or invalid

**Resolution Required:**
- Add API key storage design (recommend settings table with encryption)
- Add UI for key configuration in SettingsPage
- Document free tier limitations and required key acquisition steps

#### M4 — Rate Limiting Implementation Details Missing

**Problem:** Plan states "max 10 requests per minute per API" but doesn't specify:
- Token bucket vs sliding window algorithm
- Per-user vs global rate limiting
- How rate limit state persists across app restarts
- Behavior when queue is full (reject? wait? timeout?)

**Resolution Required:**
- Specify rate limiting algorithm (recommend token bucket with refill)
- Define queue behavior and maximum queue size
- Document persistence strategy (in-memory vs database)

#### M5 — No Conflict Resolution Strategy

**Problem:** When metadata from RAWG conflicts with IGDB (or user-entered data), plan doesn't specify:
- Which source takes precedence
- Whether user can choose preferred source
- How conflicts are presented in UI

**Resolution Required:**
- Define precedence hierarchy (user manual > RAWG > IGDB > defaults)
- Add conflict resolution UI design
- Preserve user edits even when auto-enrichment runs

#### M6 — Background Task Lifecycle Unclear

**Problem:** Plan mentions `bulk_enrich_library()` as background task but doesn't specify:
- How tasks survive app restarts
- Cancellation mechanism
- Resource limits (concurrent downloads, memory usage)

**Resolution Required:**
- Define task persistence model (queue in database)
- Add pause/resume/cancel commands
- Specify resource limits and throttling

---

## Task 20 — Image Download & Management System

### ⚠️ **MODERATE ISSUES**

#### M7 — Image Storage Location Platform-Specific

**Problem:** Plan specifies `%APPDATA%/com.pirate-harbor.app/images/` but:
- This is Windows-specific path format
- Tauri provides cross-platform `app_data_dir()` API
- Direct path usage will break on Linux/macOS

**Resolution Required:**
- Use Tauri's `app_data_dir()` API instead of hardcoded paths
- Document actual resolved paths per platform for debugging

#### M8 — No Disk Space Management

**Problem:** Unlimited image downloads could fill user's disk. Plan doesn't specify:
- Maximum total image storage size
- Cleanup policy for unused images
- User control over cache size

**Resolution Required:**
- Add disk space limit (e.g., 2 GB default, configurable)
- Implement LRU cleanup when limit exceeded
- Add "Clear Image Cache" button in Settings

#### M9 — Image Format Selection Logic Missing

**Problem:** Plan says "convert to optimal format" but doesn't define:
- What "optimal" means (file size? quality? compatibility?)
- When to use JPG vs PNG vs WebP
- Quality/compression settings

**Resolution Required:**
- Specify format selection rules (JPG for photos, PNG for transparency needs, WebP when supported)
- Define quality targets (e.g., JPEG quality 85, WebP quality 80)
- Add user preference for quality vs size tradeoff

#### M10 — Concurrent Download Limits Not Specified

**Problem:** Bulk enrichment could spawn hundreds of simultaneous downloads, overwhelming network and memory.

**Resolution Required:**
- Limit concurrent image downloads (recommend 3-5)
- Use semaphore or queue-based concurrency control
- Progress reporting must account for queued downloads

---

## Task 21 — Enhanced Milestone Database Schema

### ❌ **CRITICAL ISSUES**

#### C6 — MIGRATION_006 Already Exists

**Problem:** Plan specifies creating `MIGRATION_006` for milestones, but migrations.rs shows it already exists (lines 114-142).

**Current MIGRATION_006 Contents:**
- `milestones` table ✅
- `milestone_templates` table ✅
- All specified indexes ✅

**Impact:** T21 database work is ALREADY COMPLETE.

**Resolution Required:**
- Update plan to remove migration creation
- T21 should focus ONLY on:
  1. Rust CRUD implementation
  2. Template seeding
  3. Migration of existing journal entries
  4. Frontend integration

### ⚠️ **MODERATE ISSUES**

#### M11 — Journal Entry Migration Strategy Incomplete

**Problem:** Plan mentions "migrate existing journal entries with entry_type='milestone'" but doesn't specify:
- **When** migration happens (one-time on first launch? on-demand? background?)
- **What** happens to original journal entries (preserved? deleted? marked as migrated?)
- **How** to handle user-created milestones after migration (keep both systems in sync?)

**Current MilestonesPage Implementation:**
```typescript
// MilestonesPage currently reads directly from journal_entries:
const [ms, gs] = await Promise.all([
  getJournalEntries(null, 500),  // ← Still using journal_entries
  getAllGames({}),
]);
setMilestones(ms.filter(e => e.entry_type === "milestone"));
```

**Resolution Required:**
- Define migration trigger (recommend: automatic on app startup after migration 006)
- Preserve journal entries, add `migrated_to_milestone_id` column for linking
- Update MilestonesPage to read from `milestones` table instead of filtering journal_entries
- Provide dual-read fallback during transition period

#### M12 — Template Seeding Implementation Missing

**Problem:** Plan lists default templates but doesn't specify:
- Where seeding code lives (migration? separate command? first-run setup?)
- How to avoid duplicate templates on multiple app launches
- User ability to modify or delete default templates

**Resolution Required:**
- Add template seeding to MIGRATION_006 (one-time with conflict handling)
- Use `INSERT OR IGNORE` with deterministic template IDs
- Allow users to edit/delete templates (mark system templates as `is_system` flag)

---

## Task 22 — Milestone Statistics & Analytics Engine

### ✅ **APPROVED** (with minor suggestions)

#### ✅ **WELL-DESIGNED ELEMENTS**

1. **Statistics struct is comprehensive** — covers all key metrics
2. **Analytics separation** — clean module structure (analytics/milestones.rs)
3. **Frontend integration** — enhances existing MilestonesPage rather than replacing
4. **Performance considerations** — caching and pagination mentioned

### 💡 **MINOR SUGGESTIONS**

#### m1 — Cache Invalidation Strategy

**Suggestion:** Specify cache invalidation triggers more precisely:
- Invalidate on: milestone create, update, delete
- Invalidate on: game delete (cascades to milestones)
- Consider: partial cache updates vs full recalculation

**Recommendation:** Use versioned cache with last_modified tracking.

---

## Summary of Required Actions

### ✅ **APPROVED FOR IMPLEMENTATION (No Changes)**
- **T22** — Milestone Statistics & Analytics Engine

### 🔄 **REQUIRES PLAN UPDATES (Before Implementation)**

| Task | Action Required | Urgency |
|------|----------------|---------|
| **T18** | Remove migration work (already complete), focus on frontend verification | 🔴 **BLOCKING** |
| **T19** | Remove MIGRATION_005 creation, resolve metadata_cache duplication, add API key management | 🔴 **BLOCKING** |
| **T20** | Use platform-agnostic paths, add disk space management, specify format rules | 🟡 **HIGH** |
| **T21** | Remove MIGRATION_006 creation, detail migration strategy, add template seeding | 🔴 **BLOCKING** |

---

## Critical Path Revisions

**Original Critical Path:**
```
T18 → T19 → T20 → T24 → T25
```

**Revised Critical Path (based on actual migration state):**
```
T18 (Frontend Only) ─┬→ T19 (Rust API Implementation)
                      │   ↓
T21 (Migration Logic) → T20 (Image Downloads)
                      ↓
                    T22 (Analytics)
                      ↓
                   T24 (UI Integration)
                      ↓
                   T25 (Polish)
```

**Key Changes:**
1. T18 is mostly complete (transaction wrapping done, schema complete)
2. T19 and T21 can start immediately (schemas exist)
3. T18 frontend verification can run in parallel with backend work
4. T20 depends on T19 (needs metadata for image URLs)
5. T22 depends on T21 (needs milestones table populated)

---

## Recommended Implementation Order

1. **Update Phase 3 Plan** (30 minutes)
   - Remove duplicate migration specifications
   - Add missing design details (API keys, disk management, migration logic)
   - Correct dependency graph

2. **T18 Frontend Verification** (1 hour)
   - Verify CollectionCard mosaic rendering
   - Test cover_mode switching
   - Document any missing implementation

3. **T19 Rust Implementation** (2 days)
   - Resolve metadata_cache duplication
   - Implement RAWG/IGDB clients
   - Add API key management
   - Rate limiting and queue processing

4. **T21 Migration Implementation** (1 day)
   - Write journal→milestone migration logic
   - Seed milestone templates
   - Update MilestonesPage to use new table

5. **T20 Image Management** (1.5 days)
   - Implement platform-agnostic storage
   - Add disk space management
   - Image download and processing

6. **T22 Analytics** (1 day)
   - Statistics calculations
   - Frontend integration

7. **T24 UI Integration** (1.5 days)
   - Connect all pieces in UI
   - Progress indicators

8. **T25 Polish & Testing** (2 days)
   - Integration testing
   - Performance optimization
   - Acceptance test execution

**Total Estimated Time:** 9.5 days of development + 0.5 days of plan updates

---

## Conclusion

The Phase 3 plan demonstrates excellent high-level architectural thinking and comprehensive feature coverage. However, **critical synchronization issues exist between the plan and the current codebase state**:

1. **Migrations 003, 005, 006 already exist** — plan incorrectly assumes they need creation
2. **M1 fix is already implemented** — transaction wrapping is in scanner.rs
3. **metadata_cache table duplication** — two incompatible definitions exist

These issues are **blocking** and must be resolved before implementation begins. The good news: ~40% of the "database work" is already complete, which should accelerate the schedule.

**Recommendation:** 
1. Spend 1 hour updating the plan to match reality
2. Start T19/T21 implementation immediately (schemas ready)
3. Parallel T18 frontend verification (non-blocking)
4. Proceed with revised critical path

Once plan updates are complete, this is a solid, implementable Phase 3 roadmap.

---

**Review Status:** ⚠️ **REVISIONS REQUIRED**  
**Next Step:** Update phase3_plan.md to address critical issues C1-C6  
**Estimated Revision Time:** 30-60 minutes

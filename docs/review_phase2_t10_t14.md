# Architect Review — Phase 2: T10–T14

**Date:** 2026-06-25  
**Scope:** All commits from `6ebbf3b` (T10) through `a9461c9` (T14)  
**Build status:** ✅ `cargo check` — clean | ✅ `tsc --noEmit` — clean

---

## Summary

The Engineer completed **7 commits** covering:

| Plan Task | Engineer Commit | Status |
|-----------|----------------|--------|
| T10 (Collections Schema) | `3481d00` Collections migration + CRUD | ✅ Done |
| T11 (Collections UI) | `3481d00` (same commit) CollectionsPage | ✅ Done |
| T12 (Journal Schema) | `d9e1a4d` Journal migration + CRUD | ✅ Done |
| T13 (Journal UI) | `d9e1a4d` (same commit) JournalPage | ✅ Done |
| T14 (Edit/Delete/Toast) | `a9461c9` EditGamePage, ConfirmDialog, Toast | ✅ Done |
| *Out-of-scope*: Scanner (T16) | `6ebbf3b` scanner.rs + SettingsPage UI | ✅ (bonus) |
| *Out-of-scope*: Metadata API | `ee2b7d7` RAWG integration + caching | ✅ (bonus) |
| *Out-of-scope*: Milestones | `eb4628b` MilestonesPage | ⚠️ Not in plan |
| *Out-of-scope*: Identity | `c5e7d4a` IdentityPage | ⚠️ Not in plan |

---

## ✅ What's Correct

### Backend (Rust)

1. **Migrations:** 4 migrations (`001`–`004`), all idempotent (`IF NOT EXISTS`), verified by unit tests (`test_migrations_apply_cleanly`, `test_migrations_are_idempotent`).
2. **Collections CRUD:** Full CRUD + junction table management. `add_game_to_collection` is idempotent (`INSERT OR IGNORE`). `get_game_collections` returns the reverse lookup. CASCADE on delete is correctly set.
3. **Journal CRUD:** Clean `map_row` pattern. Denormalized `game_title` avoids joins at read time. `EntryType` enum with proper `FromStr` + `Default` implementations.
4. **Scanner:** Blocklist of 19 utility executables. `MAX_DEPTH = 4` prevents runaway walks. Deduplication by `exe_path`. Known-paths check prevents re-importing.
5. **Metadata:** RAWG integration with 24hr cache TTL, proper error handling for 401/403, `urlencoding` for safe query params. Non-fatal cache write (line 194: `let _ = write_cache(...)`).
6. **Command Registration:** All 24 commands properly registered in `lib.rs` with organized section comments.

### Frontend (TypeScript/React)

1. **API Layer:** 100% 1:1 coverage of all Rust commands. Types are inline in `api.ts` (acceptable — no desync risk).
2. **ToastStore:** Clean Zustand pattern, auto-dismiss after 4s, unique IDs.
3. **ToastContainer:** Proper `aria-live="polite"`, `role="status"` on each toast. Mounted once in `AppLayout` (line 45).
4. **ConfirmDialog:** Keyboard accessible (Escape cancels, Enter confirms). `aria-modal`, `aria-labelledby`, `aria-describedby`. Focus trap on confirm button.
5. **EditGamePage:** Pre-fills from `getGame()`, all fields editable, proper error banner, toast on success/error. Route `/library/:id/edit` correctly placed BEFORE `/library/:id` in App.tsx.
6. **GameDetailPage:** Edit (pencil) and Delete (trash) buttons wired. ConfirmDialog for destructive delete. Toast on delete success/failure.
7. **CollectionsPage:** Create, delete, toggle games, mosaic cover from `game_ids` first 4 covers. Detail panel with game picker.
8. **JournalPage:** Compose form with game selector, entry type picker, inline edit, delete with confirmation. Date grouping. Filter by game.
9. **CSS:** `toast-in` keyframe defined (lines 162–165). `prefers-reduced-motion` reduces all animations.

---

## ⚠️ Issues Found

### CRITICAL — Must Fix

**None.** Both compilers pass clean.

### MODERATE — Should Fix

| # | File | Issue | Fix |
|---|------|-------|-----|
| M1 | `CollectionsPage.tsx:92` | Uses `window.confirm()` instead of `ConfirmDialog` for collection delete. Breaks design consistency — native browser dialog breaks the monochrome UI. | Replace with `ConfirmDialog` component (already available). |
| M2 | `models.rs` / `migrations.rs` | Plan specified `cover_mode TEXT NOT NULL DEFAULT 'auto'` column on `collections`. Engineer used `cover_game_id` instead (single game reference). This **deviates from the approved modification** where we decided on `cover_mode = 'auto' | 'custom'` with 2×2 mosaic default. | Add `cover_mode` column to migration, model, and frontend. The current `cover_game_id` approach works as a stopgap but doesn't implement the auto-mosaic design decision. |
| M3 | `scanner.rs` | Plan specified confidence scoring (0.0–1.0) with 5 scoring factors and 20MB size filter. Engineer implemented a simpler blocklist-only approach with no confidence score and no size filter. `ScanResult` has no `confidence` field. | Add confidence scoring as specified in plan. Not a blocker but deviates from approved spec. |
| M4 | `JournalPage.tsx` | Journal delete uses inline confirm (click-to-confirm pattern) but not the `ConfirmDialog` component. Inconsistent with GameDetailPage delete pattern. | Use `ConfirmDialog` for journal entry deletion too. |

### MINOR — Nice to Have

| # | File | Issue |
|---|------|-------|
| m1 | `ConfirmDialog.tsx:60` | Enter key fires `onConfirm` globally while dialog is open. If user is typing in another field (unlikely in confirm dialog, but possible in future reuse), Enter would trigger. Consider only firing from the confirm button's own `onKeyDown`. |
| m2 | `EditGamePage.tsx:18` | Imports `GameStatus` from `@/types` but uses string literal `"unplayed"` as the default (line 47). Type-safe but could use `GameStatus` constant. |
| m3 | Out-of-scope commits | Milestones and Identity pages were not in the Phase 2 plan. They seem functional and don't break anything, but they weren't reviewed against a spec. They should get a dedicated review when their phase arrives. |
| m4 | `scanner.rs:159` | Blocklist uses `contains()` — so a game called "setuptown" would be filtered. Should use exact match on stem or prefix match. |
| m5 | `GameDetailPage.tsx` | Missing "Add to Collection" menu/button per T11 plan. The integration was specified in the plan but not implemented. |

---

## Action Items for Engineer

### Required Fixes (M1–M5)

1. **M1:** Replace `window.confirm()` in `CollectionsPage.tsx` with `ConfirmDialog`.
2. **M2:** Add `cover_mode` column to migration 003, `Collection` model, and frontend type. Default to `'auto'`. This is the approved design decision. *(Can be deferred to T17 polish if preferred.)*
3. **M3:** Add confidence scoring to `scanner.rs` and surface in frontend. *(Can be deferred to T17.)*
4. **M4:** Use `ConfirmDialog` for journal entry deletion.
5. **M5:** Add "Add to Collection" dropdown/popover on `GameDetailPage`.

### Verdict

**Phase 2 T10–T14: CONDITIONALLY APPROVED ✅**

The core work is solid — clean compilation, proper CRUD, good accessibility patterns, consistent Atlas OS styling. The 5 moderate issues (M1–M5) are real but non-blocking deviations from the approved plan. They can be folded into T17 (Polish) or fixed immediately.

The bonus work (Scanner, Metadata, Milestones, Identity) is appreciated but will need its own review pass.

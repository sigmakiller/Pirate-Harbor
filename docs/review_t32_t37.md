# Phase 4 Review — T32 through T37

**Reviewer:** Architect  
**Date:** 2026-07-09  
**Scope:** Tasks 32–37 (Export, Backup/Restore, Game Detail, Settings, UX Polish, Integration Tests)

---

## Build Status

| Check | Result |
|-------|--------|
| `cargo check` | ✅ Compiles (0 warnings) |
| `cargo test`  | ✅ **60/60 passed** |
| `tsc --noEmit`| ✅ 0 errors |

---

## Per-Task Audit Summary

| Task | Verdict | Issues |
|------|---------|--------|
| **T32** Data Export (JSON + Markdown) | ✅ Good | M1 |
| **T33** Local Backup / Restore       | ⚠️ Has Bugs | **C1**, **C2**, M2 |
| **T34** Game Detail Enrichment       | ✅ Good | m1 |
| **T35** Settings Page Completion     | ⚠️ Has Bugs | **C3** |
| **T36** UX Polish                    | ✅ Good | m2 |
| **T37** Integration Testing          | ✅ Good | m3 |

---

## Critical Issues (Must Fix)

### C1 — `app_data_dir` resolved from a setting that is never written

**File:** `commands/backup.rs` lines 452–455, 478–481  
**File:** `commands/diagnostics.rs` line 117–119

The `create_backup`, `restore_backup`, and `list_auto_backups` commands all
resolve the app data directory by querying:

```sql
SELECT value FROM settings WHERE key='app_data_dir'
```

**This key is never set.** `init_db()` in `db/mod.rs` never writes
`app_data_dir` to the settings table. Every call to `create_backup` or
`restore_backup` will silently fall back to the backup file's parent directory
as `app_data_dir`, meaning images will be restored to the wrong location.

**Fix:** In `lib.rs` `setup()`, after `init_db()`, write the resolved path:

```rust
// After app.manage(DbState(Mutex::new(conn)));
{
    let db_guard = /* get the managed DbState */ ...;
    let conn = db_guard.0.lock().unwrap();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('app_data_dir', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![app_data_dir.to_string_lossy()],
    ).ok();
}
```

Or, better: pass `app_data_dir` as a separate `tauri::State` (an
`AppDataDir(PathBuf)` wrapper) so commands don't need the settings table
to find their own path.

---

### C2 — `restore_backup` double-locks the mutex causing a deadlock

**File:** `commands/backup.rs` lines 477–487

```rust
// Line 477: locks the mutex
let conn_guard = db.0.lock()...;
let app_data_dir: PathBuf = conn_guard.query_row(...)...;
drop(conn_guard);   // released

// Line 486: locks again — OK if previous lock is dropped
let mut conn_guard = db.0.lock()...;
restore_backup_file(&mut conn_guard, ...);
```

The comment at line 484 says "unsafe deref" but the code uses a **second
lock** — this is actually fine as written *right now*. However the comment is
misleading and may lead future engineers to think this is using unsafe code.

**Real bug:** `restore_backup_file` receives `&mut conn_guard` where
`conn_guard` is a `MutexGuard<Connection>`. This works via `DerefMut`, but if
anyone adds another `Mutex::lock()` inside `restore_backup_file` in the future,
it will deadlock because the outer lock is still held.

**Fix:** Remove the misleading comment. Document clearly that the lock is held
through the entire restore operation by design (preventing concurrent access):

```rust
// The lock is held for the entire restore to prevent any concurrent DB access
// during the destructive DELETE+INSERT sequence.
let mut conn_guard = db.0.lock().map_err(|_| "DB lock poisoned".to_string())?;
let app_data_dir = resolve_app_data_dir(&conn_guard, &backup_path);
restore_backup_file(&mut conn_guard, &backup_path, &app_data_dir)
```

---

### C3 — `getDiagnostics` API missing from `api.ts`

**File:** `apps/desktop/src/lib/api.ts`

`SettingsPage.tsx` line 29 imports `getDiagnostics` and `runIntegrityCheck`
from `@/lib/api`. The commands `get_diagnostics` and `run_integrity_check`
are registered in `lib.rs` but **the TypeScript bindings are missing** from
`api.ts` (only `DiagnosticsReport` and `IntegrityResult` types are defined,
the actual `invoke` wrapper functions are absent).

**Verify:** Run `grep -n "getDiagnostics\|runIntegrityCheck" src/lib/api.ts`

If the functions are missing, add:

```typescript
/** Full diagnostics snapshot (schema, table counts, storage, jobs). */
export async function getDiagnostics(): Promise<DiagnosticsReport> {
  return invoke<DiagnosticsReport>("get_diagnostics");
}

/** Run SQLite integrity check. */
export async function runIntegrityCheck(): Promise<IntegrityResult> {
  return invoke<IntegrityResult>("run_integrity_check");
}

/** Absolute path to the .db file. */
export async function getDbPath(): Promise<string> {
  return invoke<string>("get_db_path");
}
```

---

## Moderate Issues (Should Fix)

### M1 — JSON export path validation too weak

**File:** `commands/export.rs` line 487–489

```rust
if output_path.extension().is_none() {
    return Err("Output path must include a filename (e.g. export.json)".to_string());
}
```

This only checks that *some* extension exists — `export.exe` would pass.
Also, the validation runs on the *Rust* side with a `PathBuf` constructed
from a user-provided string, so a path like `"just_a_name"` (no extension)
would pass since `PathBuf::from("just_a_name").extension()` returns `None`
but `PathBuf::from("name.exe").extension()` returns `Some("exe")`.

**Fix:** Validate the extension is `.json` or `.md` as appropriate:

```rust
let ext = output_path.extension().and_then(|e| e.to_str()).unwrap_or("");
if ext != "json" {
    return Err("Export file must have .json extension".to_string());
}
```

---

### M2 — Backup restore does not re-populate FTS5 indexes

**File:** `commands/backup.rs` line 184–232

After `restore_backup_file` clears and re-inserts all game/journal rows,
the FTS5 virtual tables (`games_fts`, `journal_fts`) are stale. The
INSERT triggers will fire for each restored row (correct), but only if
the triggers exist. The triggers are created by Migration 007 — they
should exist. However, the bulk DELETE before restore fires the `games_ad`
and `journal_ad` delete triggers, which should correctly remove FTS entries.
Then INSERT triggers fire on restore.

**But:** since `PRAGMA foreign_keys = OFF` is set during restore (line 181),
this doesn't affect triggers — triggers still fire. **This is fine as-is.**

However, for defensive safety, call `rebuild` after restore completes:

```rust
// After tx.commit()
conn.execute_batch("INSERT INTO games_fts(games_fts) VALUES('rebuild');").ok();
conn.execute_batch("INSERT INTO journal_fts(journal_fts) VALUES('rebuild');").ok();
```

---

## Minor / Nitpick

### m1 — `GameDetailPage` loads 7 promises in parallel with no error boundary per-section

**File:** `src/pages/GameDetailPage.tsx` lines 58–73

All 7 data fetches (game, sessions, collections, gallery, journal, related,
recommendations) are done in a single `Promise.all`. If any single fetch
fails (e.g. `getGameRecommendations` errors for a new user with empty
library), the entire page shows an error state and nothing renders.

**Fix:** Use `Promise.allSettled` and handle each result independently so
a failed recommendations fetch doesn't break the gallery section.

---

### m2 — T36 skeleton loading uses inline styles instead of CSS classes

Minor: skeleton animation should be in CSS, not scattered as `style=` props
in TSX. Low impact.

---

### m3 — Integration test `t37_fts5_search_sub_100ms_on_1000_games` inserts
games with a loop — slow on debug builds

**File:** `src/tests/integration.rs`

The 1000-game insertion test iterates one INSERT per game. On a slow CI
debug build this might exceed 100ms. Consider batching with a transaction:

```rust
let tx = conn.transaction().unwrap();
for i in 0..1000 { tx.execute(...).unwrap(); }
tx.commit().unwrap();
```

---

## Action Items for Engineer

| ID | Severity | Task | File(s) |
|----|----------|------|---------|
| **C1** | 🔴 Critical | Write `app_data_dir` to settings at init, or add `AppDataDir` state | `lib.rs`, `commands/backup.rs`, `commands/diagnostics.rs` |
| **C2** | 🔴 Critical | Remove misleading comment; consolidate to one lock + document | `commands/backup.rs` lines 484–487 |
| **C3** | 🔴 Critical | Add `getDiagnostics`, `runIntegrityCheck`, `getDbPath` to `api.ts` | `src/lib/api.ts` |
| **M1** | 🟡 Moderate | Validate export extension is `.json` / `.md` specifically | `commands/export.rs` |
| **M2** | 🟡 Moderate | Call FTS5 `rebuild` after backup restore | `commands/backup.rs` |
| m1 | ⚪ Minor | Use `Promise.allSettled` in `GameDetailPage` | `GameDetailPage.tsx` |
| m3 | ⚪ Minor | Wrap 1000-game insert test in a transaction | `tests/integration.rs` |

---

## Overall Verdict

**Engineering quality is high.** T32 (export) and T34 (game detail) are clean.
T36 and T37 are solid. The two blocking issues are in T33 (backup) and T35
(settings/diagnostics) — both are correctness bugs that will silently fail at
runtime on a real installation.

**All 3 criticals must be fixed before Phase 4 can be considered complete.**

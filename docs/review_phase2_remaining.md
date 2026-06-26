# Architect Review — Remaining Fixes (M2, M3)

**Date:** 2026-06-25  
**Status:** M1 ✅ M4 ✅ M5 ✅ fixed. **M2 ❌ M3 ❌ still outstanding.**  
**Build:** `cargo check` ✅ | `tsc --noEmit` ✅  

---

## M2 — `cover_mode` Column on Collections (Plan Deviation)

### Problem

The approved plan specifies a `cover_mode` column on the `collections` table to support two display modes:

- `'auto'` (default) — frontend renders a 2×2 mosaic from the first 4 game covers in the collection
- `'custom'` — uses a user-selected `cover_path` image

The current implementation only has `cover_game_id` (a single game reference for the cover) and no `cover_mode` column. This was an **explicitly approved design decision** from the user.

### Required Changes

#### 1. Migration — `src-tauri/src/db/migrations.rs`

Add `cover_mode` and `cover_path` columns to migration `MIGRATION_003`. The collections table should be:

```sql
CREATE TABLE IF NOT EXISTS collections (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    description   TEXT,
    cover_path    TEXT,
    cover_mode    TEXT NOT NULL DEFAULT 'auto',
    cover_game_id TEXT REFERENCES games(id) ON DELETE SET NULL,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);
```

> [!IMPORTANT]
> Keep `cover_game_id` — it remains useful as the user-selected hero game. The new `cover_mode` controls **how the card is rendered**, not what data is stored.

#### 2. Rust Model — `src-tauri/src/models.rs`

Update the `Collection` struct to include:

```rust
pub struct Collection {
    pub id:            String,
    pub name:          String,
    pub description:   Option<String>,
    pub cover_path:    Option<String>,       // NEW
    pub cover_mode:    String,               // NEW — "auto" | "custom"
    pub cover_game_id: Option<String>,
    pub created_at:    String,
    pub updated_at:    String,
    pub game_ids:      Vec<String>,
    pub game_count:    i64,
}
```

Update `NewCollection` and `UpdateCollection` to include `cover_path` and `cover_mode` as optional fields.

#### 3. Rust Commands — `src-tauri/src/commands/collections.rs`

- Update all `SELECT` queries to include `cover_path` and `cover_mode` columns
- Update `row_to_collection` helper to map the new columns
- Update `create_collection` INSERT to include `cover_path` and `cover_mode`
- Update `update_collection` to allow changing `cover_path` and `cover_mode`

#### 4. Frontend Type — `src/lib/api.ts`

Update the `Collection` interface:

```typescript
export interface Collection {
  id:            string;
  name:          string;
  description:   string | null;
  cover_path:    string | null;     // NEW
  cover_mode:    'auto' | 'custom'; // NEW
  cover_game_id: string | null;
  created_at:    string;
  updated_at:    string;
  game_ids:      string[];
  game_count:    number;
}
```

Update `NewCollection` and `UpdateCollection` similarly.

#### 5. Frontend UI — `src/pages/CollectionsPage.tsx`

In the collection card rendering:

- If `cover_mode === 'auto'`: render a 2×2 CSS grid of the first 4 game covers from `game_ids`. Use `convertFileSrc()` for each. Empty slots get `background: var(--color-elevated)`.
- If `cover_mode === 'custom'`: render `cover_path` as a single full-bleed image.
- In the create/edit collection form: add a toggle or dropdown for cover mode, and a file picker for custom cover.

**Verify:** Create collection → auto mosaic shows from game covers → switch to custom → single image shows → switch back to auto → mosaic returns.

---

## M3 — Scanner Confidence Scoring (Plan Deviation)

### Problem

The approved plan specifies confidence scoring for the folder scanner. The current implementation is a simple blocklist filter with no scoring, no size filter, and no confidence UI. This was an **explicitly approved user modification**.

### Required Changes

#### 1. Model — `src-tauri/src/models.rs`

Update `ScanResult` to include confidence and size:

```rust
pub struct ScanResult {
    pub name:          String,
    pub exe_path:      String,
    pub already_added: bool,
    pub confidence:    f64,     // NEW — 0.0 to 1.0
    pub size_mb:       f64,     // NEW — file size in megabytes
    pub folder_name:   String,  // NEW — parent folder name
}
```

#### 2. Scanner Logic — `src-tauri/src/commands/scanner.rs`

**A) Add minimum size constant:**

```rust
/// Minimum executable size in bytes (20 MB). Files smaller are skipped.
const MIN_EXE_SIZE: u64 = 20 * 1024 * 1024;
```

**B) Add size filter** in `do_scan()` after the blocklist filter:

```rust
// Skip executables under 20 MB
let metadata = std::fs::metadata(path).ok()?;
if metadata.len() < MIN_EXE_SIZE {
    return None;
}
```

**C) Add confidence scoring function:**

```rust
fn compute_confidence(path: &Path, exe_stem: &str) -> f64 {
    let mut score: f64 = 0.0;
    
    // +0.3 if exe is in a named subfolder (not scan root)
    if let Some(parent) = path.parent() {
        if parent.file_name().is_some() {
            score += 0.3;
        }
    }
    
    // +0.2 if folder contains typical game files
    if let Some(parent) = path.parent() {
        if let Ok(entries) = std::fs::read_dir(parent) {
            let siblings: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_lowercase()))
                .collect();
            let game_extensions = ["dll", "pak", "uasset", "unity3d", "pck"];
            if siblings.iter().any(|ext| game_extensions.contains(&ext.as_str())) {
                score += 0.2;
            }
        }
    }
    
    // +0.2 if exe name matches parent folder name
    if let Some(parent) = path.parent() {
        if let Some(folder_name) = parent.file_name().and_then(|f| f.to_str()) {
            if exe_stem.to_lowercase() == folder_name.to_lowercase() {
                score += 0.2;
            }
        }
    }
    
    // +0.2 if exe is inside a known game binary directory
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains("\\bin\\") || path_str.contains("\\binaries\\") 
       || path_str.contains("/bin/") || path_str.contains("/binaries/") {
        score += 0.2;
    }
    
    // +0.1 if exe > 50 MB
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > 50 * 1024 * 1024 {
            score += 0.1;
        }
    }
    
    score.min(1.0)
}
```

**D) Populate the new fields** in the `do_scan()` `filter_map` closure:

```rust
let metadata = std::fs::metadata(path).ok()?;
let size_bytes = metadata.len();
if size_bytes < MIN_EXE_SIZE {
    return None;
}

let confidence = compute_confidence(path, &stem);
let size_mb = size_bytes as f64 / (1024.0 * 1024.0);
let folder_name = path.parent()
    .and_then(|p| p.file_name())
    .and_then(|f| f.to_str())
    .unwrap_or("")
    .to_string();

Some(ScanResult {
    name: stem,
    exe_path,
    already_added,
    confidence,
    size_mb,
    folder_name,
})
```

**E) Sort by confidence** (descending) instead of name:

```rust
results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
```

#### 3. Frontend Type — `src/lib/api.ts`

Update `ScanResult`:

```typescript
export interface ScanResult {
  name:          string;
  exe_path:      string;
  already_added: boolean;
  confidence:    number;    // NEW — 0.0 to 1.0
  size_mb:       number;    // NEW
  folder_name:   string;    // NEW
}
```

#### 4. Frontend UI — `src/pages/SettingsPage.tsx` (Scan Results section)

In the scan results list, for each discovered game row:

- Show a **confidence bar** — a horizontal bar filled proportionally to confidence (0–100%). Use `var(--color-text-secondary)` for the fill, `var(--color-elevated)` for the track.
- Show the **size** in MB (e.g. "142 MB").
- Show the **folder name** as secondary text.
- **Pre-select** games with confidence ≥ 0.7. **Deselect** games with confidence < 0.4.
- Games with `already_added: true` show as "Already added" disabled rows.

Example row layout:
```
[checkbox] Game Name                    142 MB   [████████░░] 82%
           C:\Games\GameName\game.exe    └ GameName folder
```

**Verify:** Scan a folder → files < 20MB filtered out → confidence scores displayed → high confidence pre-selected → add → appear in library.

---

## Verification Checklist

After implementing both fixes, run:

```bash
# Rust
cargo check
cargo test

# TypeScript
pnpm --filter desktop exec tsc --noEmit
```

All must pass clean. Then manually test:

- [ ] M2: Create collection → auto mosaic renders from game covers → set custom cover → image changes → switch back to auto → mosaic returns
- [ ] M3: Scan folder → small exes (<20MB) filtered → confidence bars visible → high-confidence pre-selected → add works

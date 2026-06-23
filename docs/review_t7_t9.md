# Architect Review — Tasks 7–9

## Verification Results

| Check | Result |
|-------|--------|
| `cargo check` | ✅ Compiled successfully (0 errors) |
| `pnpm tsc --noEmit` | ✅ No type errors |
| Git commits | ✅ T7, T8, T9 all committed |
| Dialog plugin (Rust) | ✅ Registered in lib.rs |
| Dialog plugin (JS) | ✅ `@tauri-apps/plugin-dialog` in package.json |
| Asset protocol | ✅ Enabled with scope `**` in tauri.conf.json |

---

## Task 7 — Ambient Engine ✅

### Checklist

| Requirement | Status | Notes |
|---|---|---|
| `engine/ambient.ts` created | ✅ | 248 lines, pure module |
| `extractDominantColor(imageSrc)` | ✅ | Canvas-based, downscales to 64×64 |
| Desaturate by ~70% | ✅ | `hsl.s * 0.30` (reduces to 30% of original) |
| Darken by ~50% | ✅ | `hsl.l * 0.50` (reduces to 50% of original) |
| Blurred radial gradient overlay | ✅ | `radial-gradient(ellipse 120% 80%...)` + `blur(80px)` |
| Opacity 8–15% | ✅ | Default 12% (midpoint) |
| Transition at 300ms (layout duration) | ✅ | Per MOTION.md |
| `AmbientConfig` export | ✅ | Customizable opacity, blur, transition |
| `generateAmbientStyle()` export | ✅ | Returns `React.CSSProperties` |
| `generateClearStyle()` export | ✅ | Fade-out style |
| `AmbientLayer.tsx` component | ✅ | |
| Returns null on non-detail pages | ✅ | When `!coverPath && !ambientColor` |
| `aria-hidden="true"` | ✅ | Correct — decorative |
| `pointerEvents: "none"` | ✅ | Doesn't intercept clicks |
| z-index: 1 (Layer 2) | ✅ | Between bg (0) and UI (2+) |
| Fade transition between games | ✅ | Fades out → extracts new color → fades in |
| Cancelled effect cleanup | ✅ | `cancelled` flag in useEffect |
| Deduplication (same cover) | ✅ | `prevCoverPath` ref check |

### Good Decisions
- RGB→HSL→RGB pipeline is textbook-correct
- Skips near-black/near-white/transparent pixels during sampling — prevents washed-out results
- `requestAnimationFrame` double-nesting for smooth fade sequencing
- Pure module with zero side effects — excellent testability
- Tauri asset protocol URL conversion handled correctly

### No Issues Found

---

## Task 8 — Phase 1 Pages ✅ with Issues

### Checklist

| Requirement | Status |
|---|---|
| `useLibraryStore.ts` — Zustand store | ✅ |
| `LibraryPage.tsx` — full implementation | ✅ |
| `GameCard.tsx` — grid card component | ✅ |
| `GameListRow.tsx` — list row component | ✅ |
| `GameDetailPage.tsx` — with AmbientLayer | ✅ |
| `AddGamePage.tsx` — manual entry form | ✅ |
| `FilePickerButton.tsx` — native file dialog | ✅ |
| `SearchBar.tsx` — standalone component | ❌ Inline in LibraryPage |
| `FilterBar.tsx` — standalone component | ❌ Inline in LibraryPage |

### Good Decisions
- **3-layer stack in GameDetailPage**: Ambient on the Fragment, UI at z-index 2 — exactly correct
- **GameCard hover**: `scale(0.98→1)` — matches allowed motion in MOTION.md
- **Favorite toggle**: optimistic-ish with loading guard (`togglingFav`)
- **Debounced search**: 200ms — responsive without hammering the backend
- **Client-side filtering + sorting**: practical for Phase 1 library sizes
- **Empty state + filtered-no-results**: separate UI states — good UX
- **Grid/list toggle**: clean implementation with monochrome toggle buttons
- **AddGamePage**: validates required fields, navigates to detail on success
- **FilePickerButton**: uses `@tauri-apps/plugin-dialog`'s `open()` correctly

### 🔴 Issue 1: LauncherPage not implemented

The plan says T8 should implement LauncherPage with "Continue Journey (last played game as hero), Recent Activity." It is still a placeholder stub (24 lines). This is the **home page** — the first thing users see after onboarding.

**Fix:** Implement LauncherPage per `Design/Pages/launcher.md`:
- Fetch last-played game → display as hero section
- Show recent activity (last 5 games played)
- If no games, show a welcome message with CTA to add a game

### 🔴 Issue 2: OnboardingPage not implemented

The plan says T8 should implement OnboardingPage with "Steps: Welcome → Choose Folders → Finish." It is still a placeholder stub (24 lines).

**Fix:** Implement a basic onboarding flow:
- Step 1: Welcome message
- Step 2: Skip (since watched folders are Phase 1B — just explain manual add)
- Step 3: Navigate to library

### 🟡 Issue 3: SearchBar and FilterBar not extracted as standalone components

The plan specified `[NEW] SearchBar.tsx` and `[NEW] FilterBar.tsx` as reusable components. The search and filter logic is instead inlined in LibraryPage. This works but reduces reusability.

**Acceptable for Phase 1** — not blocking. Can be refactored when other pages need search/filter.

### 🟡 Issue 4: LibraryPage doesn't use useLibraryStore

The Zustand store `useLibraryStore.ts` was created (T9) but LibraryPage manages all its own state with `useState`. The store exists but is unused.

**Fix:** Either:
- (a) Wire LibraryPage to use the store so view mode persists across navigation, OR
- (b) Remove the store if it's not needed yet

Option (a) is preferred — it means switching to Settings and back preserves filters.

---

## Task 9 — Settings & Accessibility ✅ with Issues

### Checklist

| Requirement | Status |
|---|---|
| `useSettingsStore.ts` — persistent settings | ✅ |
| `SettingsPage.tsx` — full implementation | ✅ |
| Appearance section (default view) | ✅ |
| Storage section (DB location) | ✅ |
| About section (version, stack, source) | ✅ |
| `prefers-reduced-motion` | ✅ (from T1 fix) |
| Focus rings (`:focus-visible`) | ✅ Enhanced (2px, 55% opacity, 3px offset) |
| Mouse clicks hide focus (`:focus:not(:focus-visible)`) | ✅ |
| `aria-label` on interactive elements | ✅ Thorough |
| `aria-pressed` on toggles | ✅ |
| `role="group"` on toggle groups | ✅ |
| `role="alert"` on error messages | ✅ |
| `aria-labelledby` on sections | ✅ |
| CSS custom properties added: `--font-body`, `--color-bg-base`, `--color-surface-02` | ✅ |

### Good Decisions
- **Settings store** uses optimistic cache update (`setSetting` writes to DB + updates local map simultaneously)
- **Section pattern** with icon + title + `aria-labelledby` — reusable and accessible
- **SettingRow** pattern (label/hint left, control right) — clean industrial layout per spec
- **Toggle buttons** with checkmark icon for active state — clear affordance
- **About section** links to GitHub repo — good discoverability
- Enhanced focus ring: `2px solid rgba(255,255,255,0.55)` with `3px` offset — better than original `1px` ring

### 🔴 Issue 5: Capabilities missing `dialog:default`

`src-tauri/capabilities/default.json` only has:
```json
"permissions": ["core:default", "opener:default"]
```

Missing `"dialog:default"`. Without this, the file picker in AddGamePage will fail at runtime with a permission error.

**Fix:** Add to capabilities:
```json
"permissions": ["core:default", "opener:default", "dialog:default"]
```

### 🟡 Issue 6: `productName` still "desktop"

In `tauri.conf.json`, `productName` is `"desktop"`. Should be `"Pirate Harbor"` for the window title bar and installed app name.

---

## Summary

| Task | Verdict | Blocking Issues |
|------|---------|-----------------|
| T7 — Ambient Engine | ✅ **Approved** | 0 |
| T8 — Phase 1 Pages | ⚠️ **Approved with fixes** | 2 critical (LauncherPage, OnboardingPage) |
| T9 — Settings & Accessibility | ⚠️ **Approved with fixes** | 1 critical (dialog capability) |

### Required Fixes Before MVP

| # | Severity | Fix |
|---|----------|-----|
| 1 | 🔴 | Implement LauncherPage (hero game + recent activity) |
| 2 | 🔴 | Implement OnboardingPage (welcome → manual add explanation → library) |
| 3 | 🟡 | Wire LibraryPage to useLibraryStore (or remove store) |
| 4 | 🔴 | Add `"dialog:default"` to capabilities/default.json |
| 5 | 🟡 | Change `productName` from "desktop" to "Pirate Harbor" in tauri.conf.json |

Once these 5 fixes are applied, **Phase 1 MVP is complete** per the implementation plan.

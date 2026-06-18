# Architect Review — Tasks 1 & 2

## Verdict: ✅ Approved with Issues

Both tasks are **structurally solid** and the foundational architecture is correct. However, there are **6 issues** the Engineer must fix before moving to Task 3.

---

## Task 1 — Frontend Dependencies & Tooling

### ✅ Correct

| Item | Status |
|------|--------|
| Dependencies installed (react-router-dom, zustand, lucide-react, clsx, tailwind-merge) | ✅ |
| Tailwind v4 (`@tailwindcss/vite` + `tailwindcss` ^4.3.1) | ✅ |
| Vite config: Tailwind plugin registered | ✅ |
| Vite config: `@/` path alias | ✅ |
| `tsconfig.json`: path alias `@/*` → `./src/*` | ✅ |
| `main.tsx`: imports `index.css` | ✅ |
| `index.html`: title "Pirate Harbor", meta description, Google Fonts preconnect | ✅ |
| `index.css`: design tokens match `Design/colors.md` exactly | ✅ |
| `index.css`: typography tokens match `Design/typography.md` | ✅ |
| `index.css`: spacing tokens match `Design/spacing.md` | ✅ |
| `index.css`: motion tokens match `Design/MOTION.md` | ✅ |
| `index.css`: focus ring, scrollbar, selection styles | ✅ |
| `index.css`: `prefers-reduced-motion` | ❌ Missing |

### 🔴 Issue 1: Missing `prefers-reduced-motion` media query

`Design/accessibility.md` requires "Motion reduction support". The `index.css` has animations (`atlas-fade-in`, `atlas-enter`) but no `@media (prefers-reduced-motion: reduce)` block to disable them.

**Fix:** Add to `index.css`:
```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

### 🟡 Issue 2: Missing JetBrains Mono and Space Grotesk font imports

`index.html` only imports **Inter**. The design system uses three fonts (`Design/typography.md`): Inter, JetBrains Mono, Space Grotesk. The Sidebar already uses `--font-display` (Space Grotesk) and `--font-mono` (JetBrains Mono), but they'll fall back to system fonts since they're not loaded.

**Fix:** Update the Google Fonts `<link>` in `index.html`:
```html
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600&family=Space+Grotesk:wght@400;500;600;700&display=swap" rel="stylesheet" />
```

### 🟡 Issue 3: `App.css` not deleted

The plan specified `[DELETE] App.css`. It still exists (1,855 bytes). No file imports it anymore, but it should be removed for cleanliness.

---

## Task 2 — App Shell, Routing & Navigation

### ✅ Correct

| Item | Status |
|------|--------|
| `AppLayout.tsx`: sidebar + top bar + `<Outlet />` | ✅ |
| `Sidebar.tsx`: persistent nav, Lucide icons (1.5px stroke), version footer | ✅ |
| `TopBar.tsx`: persistent, JetBrains Mono title, uppercase | ✅ |
| `App.tsx`: BrowserRouter, routes for all pages, catch-all fallback | ✅ |
| Placeholder pages: editorial H1 title + subtitle | ✅ |
| Design token usage: all components use CSS variables | ✅ |
| Hover interactions: opacity transitions (per `Design/interactions.md`) | ✅ |
| Primary button style: filled white (Library empty state) | ✅ |

### 🔴 Issue 4: Missing pages

The plan requires **10 pages** (6 implemented + 4 scaffolded). Only **5** were created:

| Page | Status |
|------|--------|
| LauncherPage | ✅ Created |
| LibraryPage | ✅ Created |
| GameDetailPage | ✅ Created |
| JournalPage | ✅ Created |
| SettingsPage | ✅ Created |
| **AddGamePage** | ❌ Missing |
| **OnboardingPage** | ❌ Missing |
| **CollectionsPage** | ❌ Missing (scaffolded) |
| **MilestonesPage** | ❌ Missing (scaffolded) |
| **IdentityPage** | ❌ Missing (scaffolded) |

**Fix:** Create all 5 missing pages as placeholders with the same editorial H1 pattern. Deferred pages should include a "Coming soon" note. Add routes in `App.tsx`.

### 🔴 Issue 5: Sidebar branding says "Atlas OS" instead of "Pirate Harbor"

`Sidebar.tsx` line 57: `Atlas OS`. The product name is **Pirate Harbor**.

Similarly, `TopBar.tsx` line 14: fallback title is `"Atlas OS"` — should be `"Pirate Harbor"`.

**Fix:** Replace both occurrences.

### 🟡 Issue 6: Sidebar nav items incomplete

The sidebar has 4 nav items: Library, Launcher, Journal, Settings.

Per the plan, the sidebar should include **all pages** for architectural completeness (even deferred ones):
- Launcher
- Library  
- Collections (deferred)
- Journal (deferred)
- Milestones (deferred)
- Identity (deferred)
- Settings

The ordering should also match the plan: Launcher first (it's the home page).

---

## Summary of Required Fixes

| # | Severity | Fix |
|---|----------|-----|
| 1 | 🔴 | Add `prefers-reduced-motion` media query to `index.css` |
| 2 | 🟡 | Add JetBrains Mono + Space Grotesk to Google Fonts `<link>` |
| 3 | 🟡 | Delete `App.css` |
| 4 | 🔴 | Create 5 missing placeholder pages + add routes |
| 5 | 🔴 | Change "Atlas OS" → "Pirate Harbor" in Sidebar and TopBar |
| 6 | 🟡 | Add all nav items to Sidebar (including deferred pages) |

Once these are fixed, Tasks 1 & 2 are complete. The Engineer can then proceed to **Task 3 — SQLite Database & Rust Migrations**.

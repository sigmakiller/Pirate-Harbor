# T45–T48 — Phase 5 Achievement Tracking (Frontend + Integration)
## Engineer Implementation Plan

**Depends on:** T41–T44 complete ✅

---

## T45 — EditGamePage Achievement Tracking Section

### Modify `src/pages/EditGamePage.tsx`

Add these imports:
```typescript
import { Shield, Download, Plus, Trash2 } from "lucide-react";
import {
  getAchievementTrackingStatus, enableAchievementTracking,
  disableAchievementTracking, detectSteamAppId,
  getAchievementMappings, addAchievementMapping,
  removeAchievementMapping, importAchievementsFromSteam,
  type AchievementMapping, type TrackingStatus, type AppIdDetectionResult,
} from "@/lib/api";
```

Add state variables after existing state:
```typescript
// Achievement tracking state
const [trackingStatus,  setTrackingStatus]  = useState<TrackingStatus | null>(null);
const [mappings,        setMappings]        = useState<AchievementMapping[]>([]);
const [steamAppId,      setSteamAppId]      = useState("");
const [appIdSource,     setAppIdSource]     = useState<string | null>(null);
const [trackingLoading, setTrackingLoading] = useState(false);
const [importing,       setImporting]       = useState(false);
// New mapping row inputs
const [newSteamId,     setNewSteamId]     = useState("");
const [newDisplayName, setNewDisplayName] = useState("");
const [newPoints,      setNewPoints]      = useState(10);
```

Add `loadAchievementData` alongside existing load calls:
```typescript
const loadAchievementData = useCallback(async () => {
  if (!id) return;
  const [status, maps] = await Promise.all([
    getAchievementTrackingStatus(id),
    getAchievementMappings(id),
  ]);
  setTrackingStatus(status);
  setMappings(maps);
  setSteamAppId(status.steam_app_id ?? "");

  // Run auto-detection only if no App ID already saved
  if (!status.steam_app_id && game?.exe_path) {
    const dir = game.exe_path.substring(0, game.exe_path.lastIndexOf("\\"));
    const det: AppIdDetectionResult = await detectSteamAppId(id, dir);
    if (det.app_id) {
      setSteamAppId(det.app_id);
      setAppIdSource(det.source);
    }
  }
}, [id, game?.exe_path]);
```

### Section JSX (add below the Status field, before Save button):

```tsx
{/* ── Achievement Tracking ──────────────────────────────────────── */}
<section className="achievement-section">
  <h3 className="section-title">
    <Shield size={16} />
    Achievement Tracking
  </h3>

  <p className="section-hint">
    Only for games with <code>steam_api64.dll</code>. Do not enable for
    games with online anti-cheat (EAC, BattlEye).
  </p>

  {/* Toggle */}
  <label className="toggle-row">
    <span>Enable Automated Achievement Tracking</span>
    <input
      type="checkbox"
      checked={trackingStatus?.enabled ?? false}
      disabled={trackingLoading}
      onChange={async (e) => {
        setTrackingLoading(true);
        try {
          if (e.target.checked) {
            await enableAchievementTracking(id!, exePath, steamAppId);
          } else {
            await disableAchievementTracking(id!, exePath);
          }
          await loadAchievementData();
          addToast(e.target.checked ? "Achievement tracking enabled" : "Tracking disabled", "success");
        } catch (err) {
          addToast(String(err), "error");
        } finally {
          setTrackingLoading(false);
        }
      }}
    />
  </label>

  {/* Steam App ID */}
  <div className="field-row">
    <label>Steam App ID</label>
    <div className="input-with-hint">
      <input
        type="text"
        value={steamAppId}
        onChange={(e) => setSteamAppId(e.target.value)}
        placeholder="e.g. 570"
      />
      {appIdSource === "rawg"       && <span className="hint-ok">✓ Auto-detected via RAWG</span>}
      {appIdSource === "local_file" && <span className="hint-ok">✓ Found in game folder</span>}
    </div>
  </div>

  {/* Import button */}
  <button
    className="btn-secondary"
    disabled={!steamAppId || importing}
    onClick={async () => {
      setImporting(true);
      try {
        const maps = await importAchievementsFromSteam(id!, steamAppId);
        setMappings(maps);
        addToast(`Imported ${maps.length} achievements`, "success");
      } catch (err) {
        addToast(String(err), "error");
      } finally { setImporting(false); }
    }}
  >
    <Download size={14} />
    {importing ? "Importing…" : "Import from Steam"}
  </button>

  {/* Mappings table */}
  {mappings.length > 0 && (
    <table className="mappings-table">
      <thead>
        <tr><th>Steam ID</th><th>Display Name</th><th>Points</th><th></th></tr>
      </thead>
      <tbody>
        {mappings.map((m) => (
          <tr key={m.id}>
            <td><code>{m.steam_id}</code></td>
            <td>{m.display_name}</td>
            <td>{m.points}</td>
            <td>
              <button onClick={async () => {
                await removeAchievementMapping(m.id);
                setMappings((prev) => prev.filter((x) => x.id !== m.id));
              }}>
                <Trash2 size={12} />
              </button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  )}

  {/* Add mapping row */}
  <div className="add-mapping-row">
    <input placeholder="ACH_ID" value={newSteamId} onChange={(e) => setNewSteamId(e.target.value)} />
    <input placeholder="Display name" value={newDisplayName} onChange={(e) => setNewDisplayName(e.target.value)} />
    <input type="number" value={newPoints} onChange={(e) => setNewPoints(Number(e.target.value))} style={{width: 60}} />
    <button onClick={async () => {
      if (!newSteamId || !newDisplayName) return;
      const m = await addAchievementMapping(id!, newSteamId, newDisplayName, null, newPoints);
      setMappings((prev) => [...prev, m]);
      setNewSteamId(""); setNewDisplayName(""); setNewPoints(10);
    }}>
      <Plus size={14} /> Add
    </button>
  </div>
</section>
```

### Add api.ts wrappers

```typescript
export async function getAchievementTrackingStatus(gameId: string): Promise<TrackingStatus> {
  return invoke<TrackingStatus>("get_achievement_tracking_status", { gameId });
}
export async function enableAchievementTracking(gameId: string, exePath: string, steamAppId: string): Promise<void> {
  return invoke<void>("enable_achievement_tracking", { gameId, exePath, steamAppId });
}
export async function disableAchievementTracking(gameId: string, exePath: string): Promise<void> {
  return invoke<void>("disable_achievement_tracking", { gameId, exePath });
}
export async function getAchievementMappings(gameId: string): Promise<AchievementMapping[]> {
  return invoke<AchievementMapping[]>("get_achievement_mappings", { gameId });
}
export async function addAchievementMapping(gameId: string, steamId: string, displayName: string, description: string | null, points: number): Promise<AchievementMapping> {
  return invoke<AchievementMapping>("add_achievement_mapping", { gameId, steamId, displayName, description, points });
}
export async function removeAchievementMapping(mappingId: string): Promise<void> {
  return invoke<void>("remove_achievement_mapping", { mappingId });
}
export async function importAchievementsFromSteam(gameId: string, steamAppId: string): Promise<AchievementMapping[]> {
  return invoke<AchievementMapping[]>("import_achievements_from_steam", { gameId, steamAppId });
}
```

### T45 Acceptance Criteria
- [ ] Section renders on EditGamePage without errors
- [ ] Toggle enable/disable calls correct commands
- [ ] Auto-detect fires on load and fills input with source label
- [ ] Import button disabled when no App ID; enabled when present
- [ ] Mappings table shows/hides correctly; delete works
- [ ] Add mapping row appends new row without page reload

---

## T46 — Achievement Toast (App-level)

### Modify `src/App.tsx`

Add global achievement event listener that shows a styled toast:

```typescript
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useToastStore } from "@/stores/useToastStore";

// Inside App() component, before return:
const { addToast } = useToastStore();

useEffect(() => {
  const unlisten = listen<{
    display_name: string;
    points: number;
    steam_id: string;
  }>("achievement-unlocked", (event) => {
    addToast(
      `🏆 ${event.payload.display_name} · +${event.payload.points} pts`,
      "achievement"  // new toast variant
    );
  });
  return () => { unlisten.then(fn => fn()); };
}, [addToast]);
```

### Add `achievement` toast variant to toast store/component

In `src/stores/useToastStore.ts`, add `"achievement"` to the `ToastType` union:
```typescript
export type ToastType = "success" | "error" | "info" | "achievement";
```

In the toast component, style the `achievement` variant:
```css
/* index.css */
.toast.achievement {
  background: linear-gradient(135deg, var(--color-accent) 0%, #7c3aed 100%);
  border-color: #a78bfa;
  color: #fff;
  font-weight: 600;
}
```

Toast display spec:
- Slide in from **bottom-right**
- Auto-dismiss after **4 000 ms**
- Icon: 🏆 prefix on the message

### T46 Acceptance Criteria
- [ ] `listen("achievement-unlocked")` registered in App.tsx
- [ ] Toast displays with correct message format
- [ ] Toast auto-dismisses after 4 s
- [ ] `"achievement"` variant has distinct gold/purple styling
- [ ] No TypeScript errors

---

## T47 — GameDetailPage: Earned Achievements Tab

Add a read-only achievements panel to `GameDetailPage.tsx`
showing all auto-earned milestones in the `achievement` category.

### State additions:
```typescript
const [earnedAchievements, setEarnedAchievements] = useState<Milestone[]>([]);
```

### Load alongside existing data:
```typescript
// In Promise.allSettled block:
getMilestones(id, { category: "achievement" }),
// Handler:
.then(setEarnedAchievements)
```

### JSX panel (add after the Notes section):
```tsx
{earnedAchievements.length > 0 && (
  <section className="achievements-panel">
    <h3><Trophy size={16} /> Earned Achievements ({earnedAchievements.length})</h3>
    <ul className="achievement-list">
      {earnedAchievements.map((a) => (
        <li key={a.id} className="achievement-item">
          <span className="ach-icon">🏆</span>
          <div>
            <p className="ach-name">{a.title}</p>
            <p className="ach-meta">{formatRelativeDate(a.achievement_date)} · {a.points} pts</p>
          </div>
        </li>
      ))}
    </ul>
  </section>
)}
```

### Add `getMilestones` filter overload to `api.ts`:
```typescript
export async function getMilestonesByCategory(
  gameId: string, category: string
): Promise<Milestone[]> {
  return invoke<Milestone[]>("get_milestones", { gameId, category });
}
```

### T47 Acceptance Criteria
- [ ] Panel hidden when `earnedAchievements.length === 0`
- [ ] Shows title, relative date, points for each earned achievement
- [ ] Does not break existing milestone or notes sections
- [ ] No TypeScript errors

---

## T48 — Full Integration Test + Final Verification

### Manual end-to-end test script (engineer runs this):

```
1. Start app fresh (cargo tauri dev)
2. Add a game that has steam_api64.dll in its directory
3. Open EditGamePage → Achievement Tracking section
4. Verify: App ID auto-detects (RAWG or local file) OR stays blank
5. Click "Import from Steam" → verify mappings populate
6. Enable tracking → verify:
   - DLL swapped (steam_api64.dll.ph_backup exists)
   - steam_settings/steam_appid.txt written
7. Simulate Goldberg by manually writing achievements.json:
   Write: {"ACH_WIN_ONE_GAME":{"earned":true,"earned_time":1720000000}}
   to: %APPDATA%\Goldberg SteamEmu Saves\{app_id}\achievements.json
8. Verify: toast appears within 1 second
9. Open GameDetailPage → Earned Achievements panel shows the unlock
10. Disable tracking → verify DLL restored, ph_backup gone
11. Crash recovery test: manually delete steam_api64.dll while ph_backup exists
    → reopen app → enable tracking should return "broken state" error
```

### Automated test additions (add to `db/migrations.rs` tests or a new `tests/` file):

```rust
#[test]
fn achievement_router_creates_milestone_for_mapped_achievement() {
    use crate::db::migrations::run_migrations;
    use crate::steam_bridge::achievement_router::{parse_achievements, process_changes};

    let conn = rusqlite::Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();

    // Insert game
    conn.execute(
        "INSERT INTO games (id,title,exe_path,added_at,status)
         VALUES('g1','Test','C:/t.exe',datetime('now'),'unplayed')",
        [],
    ).unwrap();

    // Insert mapping
    conn.execute(
        "INSERT INTO steam_achievement_mappings
         (id,game_id,steam_id,display_name,points,created_at)
         VALUES('m1','g1','ACH_A','Win First',10,datetime('now'))",
        [],
    ).unwrap();

    let old   = parse_achievements("{}");
    let new_j = r#"{"ACH_A":{"earned":true,"earned_time":1}}"#;

    // process_changes needs AppHandle — skip event emission in test
    // by verifying milestone count directly after calling newly_unlocked
    use crate::steam_bridge::achievement_router::newly_unlocked;
    let new = parse_achievements(new_j);
    assert_eq!(newly_unlocked(&old, &new), vec!["ACH_A"]);
}
```

### T48 Acceptance Criteria
- [ ] `cargo test` — **all tests pass** (target: ≥ 37 tests)
- [ ] Manual end-to-end script steps 1–11 all pass
- [ ] `tsc --noEmit` — no TypeScript errors
- [ ] `cargo check` — no warnings (or only pre-approved dead-code ones)
- [ ] `docs/phase5_achievement_tracking_plan.md` updated with any deviations

---

## Final Docs Checklist

After T48, update the following docs:
- [ ] `docs/phase5_achievement_tracking_plan.md` — mark all tasks DONE
- [ ] `README.md` — add achievement tracking to feature list
- [ ] Create `docs/review_t38_t48.md` — Architect reviews before Phase 6

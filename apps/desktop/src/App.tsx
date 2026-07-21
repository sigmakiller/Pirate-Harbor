import { useEffect }                              from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { listen }                                  from "@tauri-apps/api/event";
import AppLayout      from "@/layouts/AppLayout";
import LauncherPage   from "@/pages/LauncherPage";
import LibraryPage    from "@/pages/LibraryPage";
import GameDetailPage from "@/pages/GameDetailPage";
import AddGamePage    from "@/pages/AddGamePage";
import SettingsPage   from "@/pages/SettingsPage";
import OnboardingPage from "@/pages/OnboardingPage";
import CollectionsPage from "@/pages/CollectionsPage";
import JournalPage    from "@/pages/JournalPage";
import MilestonesPage from "@/pages/MilestonesPage";
import IdentityPage         from "@/pages/IdentityPage";
import YearInReviewPage     from "@/pages/YearInReviewPage";
import EditGamePage         from "@/pages/EditGamePage";
import ScanPage             from "@/pages/ScanPage";
import { useToastStore } from "@/stores/useToastStore";

/** Payload emitted by the Rust `process_changes` router (T41). */
interface AchievementUnlockedPayload {
  display_name: string;
  points:       number;
  steam_id:     string;
  game_id:      string;
  milestone_id: string;
}

export default function App() {
  const { addToast } = useToastStore();

  // Global achievement-unlocked listener (T46).
  // Registered once at the App root so it survives route changes.
  useEffect(() => {
    const unlisten = listen<AchievementUnlockedPayload>(
      "achievement-unlocked",
      (event) => {
        const { display_name, points } = event.payload;
        addToast({
          message: `\ud83c\udfc6 ${display_name} \u00b7 +${points} pts`,
          type: "achievement",
        });
      },
    );

    // Clean up the listener when the component unmounts (hot-reload safety).
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [addToast]);

  return (
    <BrowserRouter>
      <Routes>
        {/* Redirect root to launcher (home screen) */}
        <Route path="/" element={<Navigate to="/launcher" replace />} />

        {/* Onboarding — outside AppLayout (no sidebar/topbar on first-run) */}
        <Route path="/onboarding" element={<OnboardingPage />} />

        {/* All main routes inside persistent AppLayout */}
        <Route element={<AppLayout />}>
          <Route path="/launcher"            element={<LauncherPage />} />
          <Route path="/library"             element={<LibraryPage />} />
          <Route path="/library/add"         element={<AddGamePage />} />
          <Route path="/library/scan"        element={<ScanPage />} />
          <Route path="/library/:id/edit"     element={<EditGamePage />} />
          <Route path="/library/:id"         element={<GameDetailPage />} />
          <Route path="/collections"         element={<CollectionsPage />} />
          <Route path="/journal"             element={<JournalPage />} />
          <Route path="/milestones"          element={<MilestonesPage />} />
          <Route path="/identity"                    element={<IdentityPage />} />
          <Route path="/identity/year-in-review"      element={<YearInReviewPage />} />
          <Route path="/settings"                     element={<SettingsPage />} />
        </Route>

        {/* Catch-all fallback */}
        <Route path="*" element={<Navigate to="/launcher" replace />} />
      </Routes>
    </BrowserRouter>
  );
}

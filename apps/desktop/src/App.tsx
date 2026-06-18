import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import AppLayout from "@/layouts/AppLayout";
import LibraryPage from "@/pages/LibraryPage";
import LauncherPage from "@/pages/LauncherPage";
import JournalPage from "@/pages/JournalPage";
import SettingsPage from "@/pages/SettingsPage";
import GameDetailPage from "@/pages/GameDetailPage";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        {/* Redirect root to launcher (home screen) */}
        <Route path="/" element={<Navigate to="/launcher" replace />} />

        {/* All main routes inside persistent AppLayout */}
        <Route element={<AppLayout />}>
          <Route path="/launcher"        element={<LauncherPage />} />
          <Route path="/library"         element={<LibraryPage />} />
          <Route path="/library/:id"     element={<GameDetailPage />} />
          <Route path="/journal"         element={<JournalPage />} />
          <Route path="/settings"        element={<SettingsPage />} />
        </Route>

        {/* Catch-all fallback */}
        <Route path="*" element={<Navigate to="/launcher" replace />} />
      </Routes>
    </BrowserRouter>
  );
}

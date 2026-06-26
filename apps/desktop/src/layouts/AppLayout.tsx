import { useCallback }                   from "react";
import { Outlet }                         from "react-router-dom";
import Sidebar                            from "@/components/Sidebar";
import TopBar                             from "@/components/TopBar";
import ToastContainer                     from "@/components/ToastContainer";
import { useGameStoppedListener }         from "@/hooks/useGameStoppedListener";
import { useToastStore }                  from "@/stores/useToastStore";
import { getGame, getSessions }           from "@/lib/api";
import { formatPlaytime }                 from "@/lib/utils";

export default function AppLayout() {
  const { addToast } = useToastStore();

  // ── Global game-stopped handler ───────────────────────────────────────────
  //
  // Runs regardless of which page is active. Fetches the stopped game's latest
  // data so we can report an accurate playtime in the toast message.
  const handleGameStopped = useCallback(async (gameId: string) => {
    try {
      const [game, sessions] = await Promise.all([
        getGame(gameId),
        getSessions(gameId),
      ]);

      // Most recent session = last element (sessions are ordered by started_at)
      const lastSession = sessions[sessions.length - 1];
      const duration    = lastSession?.duration_secs ?? 0;

      addToast({
        message: `"${game.title}" — session ended · ${formatPlaytime(duration)} recorded`,
        type:    "success",
      });
    } catch {
      // Non-fatal: the game may have been deleted between stop and fetch.
      addToast({ message: "Game session ended.", type: "info" });
    }
  }, [addToast]);

  useGameStoppedListener(handleGameStopped);

  // ── Layout ────────────────────────────────────────────────────────────────
  return (
    <div
      style={{
        display:    "flex",
        width:      "100vw",
        height:     "100vh",
        background: "var(--color-base)",
        overflow:   "hidden",
      }}
    >
      {/* ── Persistent Sidebar ── */}
      <Sidebar />

      {/* ── Main area: TopBar + Page Content ── */}
      <div
        style={{
          flex:          1,
          display:       "flex",
          flexDirection: "column",
          overflow:      "hidden",
          minWidth:      0,
        }}
      >
        <TopBar />

        {/* Page content */}
        <main
          style={{
            flex:      1,
            overflowY: "auto",
            overflowX: "hidden",
          }}
        >
          <Outlet />
        </main>
      </div>

      {/* ── Toast notifications (always mounted, fixed position) ── */}
      <ToastContainer />
    </div>
  );
}

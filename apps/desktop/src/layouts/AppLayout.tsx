import { Outlet } from "react-router-dom";
import Sidebar        from "@/components/Sidebar";
import TopBar         from "@/components/TopBar";
import ToastContainer from "@/components/ToastContainer";

export default function AppLayout() {
  return (
    <div
      style={{
        display: "flex",
        width: "100vw",
        height: "100vh",
        background: "var(--color-base)",
        overflow: "hidden",
      }}
    >
      {/* ── Persistent Sidebar ── */}
      <Sidebar />

      {/* ── Main area: TopBar + Page Content ── */}
      <div
        style={{
          flex: 1,
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
          minWidth: 0,
        }}
      >
        <TopBar />

        {/* Page content */}
        <main
          style={{
            flex: 1,
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

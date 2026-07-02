import { useState, useEffect } from "react";
import { useLocation } from "react-router-dom";
import { useJobProgress } from "@/hooks/useJobProgress";
import SearchOverlay from "@/components/SearchOverlay";

const PAGE_TITLES: Record<string, string> = {
  "/launcher":     "Launcher",
  "/library/add":  "Add Game",
  "/library":      "Library",
  "/collections":  "Collections",
  "/journal":      "Journal",
  "/milestones":   "Milestones",
  "/identity":     "Identity",
  "/settings":     "Settings",
  "/onboarding":   "Welcome",
};

function getTitle(pathname: string): string {
  for (const [key, label] of Object.entries(PAGE_TITLES)) {
    if (pathname.startsWith(key)) return label;
  }
  return "Pirate Harbor";
}

/** Subtle animated spinner shown when background jobs are running. */
function JobSpinner({ count }: { count: number }) {
  return (
    <div
      title={`${count} background job${count === 1 ? "" : "s"} running`}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 6,
        opacity: 0.75,
      }}
    >
      <span
        aria-hidden="true"
        style={{
          width: 12,
          height: 12,
          borderRadius: "50%",
          border: "2px solid var(--color-primary)",
          borderTopColor: "transparent",
          display: "inline-block",
          animation: "topbar-spin 0.9s linear infinite",
        }}
      />
      <span
        style={{
          fontSize: 11,
          color: "var(--color-text-secondary)",
          fontFamily: "var(--font-mono)",
          letterSpacing: "0.04em",
        }}
      >
        {count} {count === 1 ? "job" : "jobs"}
      </span>
      <style>{`@keyframes topbar-spin { to { transform: rotate(360deg); } }`}</style>
    </div>
  );
}

export default function TopBar() {
  const location = useLocation();
  const title = getTitle(location.pathname);
  const { hasActiveJobs, activeJobs } = useJobProgress();
  const [searchOpen, setSearchOpen] = useState(false);

  // ── Keyboard shortcut: Ctrl+K / Cmd+K ────────────────────────────────────
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        setSearchOpen((o) => !o);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  return (
    <>
      <header
        style={{
          height: "var(--topbar-height)",
          background: "var(--color-surface)",
          borderBottom: "1px solid var(--color-border)",
          display: "flex",
          alignItems: "center",
          padding: "0 28px",
          gap: 16,
          flexShrink: 0,
        }}
      >
        {/* Page title */}
        <span
          style={{
            fontSize: 13,
            fontWeight: 500,
            letterSpacing: "0.08em",
            textTransform: "uppercase",
            color: "var(--color-text-secondary)",
            fontFamily: "var(--font-mono)",
          }}
        >
          {title}
        </span>

        {/* Spacer */}
        <div style={{ flex: 1 }} />

        {/* Background job indicator */}
        {hasActiveJobs && <JobSpinner count={activeJobs.length} />}

        {/* Search button (T29) */}
        <button
          id="topbar-search-btn"
          onClick={() => setSearchOpen(true)}
          title="Search (Ctrl+K)"
          style={{
            display: "flex",
            alignItems: "center",
            gap: 8,
            background: "var(--color-bg)",
            border: "1px solid var(--color-border)",
            borderRadius: 8,
            padding: "5px 12px",
            cursor: "pointer",
            color: "var(--color-text-secondary)",
            fontSize: 13,
            fontFamily: "var(--font-sans)",
            transition: "border-color 0.15s, color 0.15s",
          }}
          onMouseEnter={(e) => {
            (e.currentTarget as HTMLButtonElement).style.borderColor = "var(--color-primary)";
            (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-primary)";
          }}
          onMouseLeave={(e) => {
            (e.currentTarget as HTMLButtonElement).style.borderColor = "var(--color-border)";
            (e.currentTarget as HTMLButtonElement).style.color = "var(--color-text-secondary)";
          }}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" strokeWidth="2"
            strokeLinecap="round" strokeLinejoin="round">
            <circle cx="11" cy="11" r="8"/>
            <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
          <span>Search</span>
          <kbd style={{
            fontSize: 10,
            background: "var(--color-surface)",
            border: "1px solid var(--color-border)",
            borderRadius: 4,
            padding: "1px 5px",
            lineHeight: 1.6,
          }}>
            Ctrl+K
          </kbd>
        </button>
      </header>

      {/* Search modal (T29) */}
      {searchOpen && <SearchOverlay onClose={() => setSearchOpen(false)} />}
    </>
  );
}

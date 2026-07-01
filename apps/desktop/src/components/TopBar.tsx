import { useLocation } from "react-router-dom";
import { useJobProgress } from "@/hooks/useJobProgress";

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
        marginLeft: "auto",
        opacity: 0.75,
      }}
    >
      {/* CSS spinner — no extra dep needed */}
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

      <style>{`
        @keyframes topbar-spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}

export default function TopBar() {
  const location = useLocation();
  const title = getTitle(location.pathname);
  const { hasActiveJobs, activeJobs } = useJobProgress();

  return (
    <header
      style={{
        height: "var(--topbar-height)",
        background: "var(--color-surface)",
        borderBottom: "1px solid var(--color-border)",
        display: "flex",
        alignItems: "center",
        padding: "0 28px",
        flexShrink: 0,
      }}
    >
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

      {hasActiveJobs && <JobSpinner count={activeJobs.length} />}
    </header>
  );
}

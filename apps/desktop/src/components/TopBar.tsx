import { useLocation } from "react-router-dom";

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

export default function TopBar() {
  const location = useLocation();
  const title = getTitle(location.pathname);

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
    </header>
  );
}

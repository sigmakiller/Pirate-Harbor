import { NavLink, useLocation } from "react-router-dom";
import {
  Gamepad2,
  Library,
  FolderOpen,
  BookOpen,
  Trophy,
  User,
  Settings,
  Anchor,
} from "lucide-react";

const NAV_ITEMS = [
  { label: "Launcher",     icon: Gamepad2,    to: "/launcher",     deferred: false },
  { label: "Library",      icon: Library,     to: "/library",      deferred: false },
  { label: "Collections",  icon: FolderOpen,  to: "/collections",  deferred: true  },
  { label: "Journal",      icon: BookOpen,    to: "/journal",      deferred: true  },
  { label: "Milestones",   icon: Trophy,      to: "/milestones",   deferred: true  },
  { label: "Identity",     icon: User,        to: "/identity",     deferred: true  },
  { label: "Settings",     icon: Settings,    to: "/settings",     deferred: false },
];

export default function Sidebar() {
  const location = useLocation();

  return (
    <aside
      style={{
        width: "var(--sidebar-width)",
        minWidth: "var(--sidebar-width)",
        height: "100vh",
        background: "var(--color-surface)",
        borderRight: "1px solid var(--color-border)",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
        flexShrink: 0,
      }}
    >
      {/* ── Brand ── */}
      <div
        style={{
          height: "var(--topbar-height)",
          display: "flex",
          alignItems: "center",
          padding: "0 20px",
          borderBottom: "1px solid var(--color-border)",
          gap: 10,
          flexShrink: 0,
        }}
      >
        <Anchor size={18} color="var(--color-text-primary)" strokeWidth={1.5} />
        <span
          style={{
            fontFamily: "var(--font-display)",
            fontSize: 14,
            fontWeight: 600,
            letterSpacing: "0.06em",
            textTransform: "uppercase",
            color: "var(--color-text-primary)",
          }}
        >
          Pirate Harbor
        </span>
      </div>

      {/* ── Navigation ── */}
      <nav
        style={{
          flex: 1,
          padding: "12px 8px",
          display: "flex",
          flexDirection: "column",
          gap: 2,
          overflowY: "auto",
        }}
      >
        {NAV_ITEMS.map(({ label, icon: Icon, to, deferred }) => {
          const isActive = location.pathname.startsWith(to);
          const baseColor = deferred
            ? "var(--color-text-disabled)"
            : isActive
            ? "var(--color-text-primary)"
            : "var(--color-text-secondary)";
          return (
            <NavLink
              key={to}
              to={to}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 10,
                padding: "8px 12px",
                borderRadius: "var(--radius-md)",
                textDecoration: "none",
                color: baseColor,
                background: isActive ? "rgba(255,255,255,0.06)" : "transparent",
                transition: `background var(--duration-fast) var(--ease-default),
                             color var(--duration-fast) var(--ease-default)`,
                fontSize: 14,
                fontWeight: isActive ? 500 : 400,
              }}
              onMouseEnter={(e) => {
                if (!isActive) {
                  (e.currentTarget as HTMLAnchorElement).style.background =
                    "rgba(255,255,255,0.04)";
                  (e.currentTarget as HTMLAnchorElement).style.color =
                    deferred ? "var(--color-text-muted)" : "var(--color-text-primary)";
                }
              }}
              onMouseLeave={(e) => {
                if (!isActive) {
                  (e.currentTarget as HTMLAnchorElement).style.background =
                    "transparent";
                  (e.currentTarget as HTMLAnchorElement).style.color = baseColor;
                }
              }}
            >
              <Icon size={16} strokeWidth={1.5} />
              <span>{label}</span>
            </NavLink>
          );
        })}
      </nav>

      {/* ── Footer ── */}
      <div
        style={{
          padding: "12px 20px",
          borderTop: "1px solid var(--color-border)",
          flexShrink: 0,
        }}
      >
        <span
          style={{
            fontSize: "var(--text-meta)",
            color: "var(--color-text-muted)",
            fontFamily: "var(--font-mono)",
          }}
        >
          v0.1.0
        </span>
      </div>
    </aside>
  );
}

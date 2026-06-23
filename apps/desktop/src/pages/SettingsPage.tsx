/**
 * SettingsPage — System Configuration.
 *
 * Design spec: Design/Pages/settings.md
 * "Industrial, uncluttered layout."
 *
 * Sections:
 *   1. Appearance — default library view mode
 *   2. Storage    — database location info
 *   3. About      — version, commit, links
 */

import { useEffect } from "react";
import { Monitor, Database, Info, Check } from "lucide-react";

import { useSettingsStore } from "@/stores/useSettingsStore";
import type { ViewMode } from "@/stores/useLibraryStore";

const APP_VERSION = "0.1.0";

export default function SettingsPage() {
  const { settings, loading, loadSettings, setSetting, getSetting } =
    useSettingsStore();

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  const defaultView = (getSetting("default_view", "grid") as ViewMode) ?? "grid";

  if (loading && Object.keys(settings).length === 0) {
    return (
      <div className="atlas-enter" style={styles.page}>
        <p style={{ color: "var(--color-text-disabled)", fontSize: 13 }}>
          Loading settings…
        </p>
      </div>
    );
  }

  return (
    <div className="atlas-enter" style={styles.page}>
      {/* ── Page Header ──────────────────────────────────────────────────── */}
      <h1 style={styles.pageTitle}>Settings</h1>
      <p style={styles.pageSubtitle}>System configuration.</p>

      {/* ── Section: Appearance ──────────────────────────────────────────── */}
      <Section icon={<Monitor size={14} />} title="Appearance">
        <SettingRow
          label="Default library view"
          hint="Grid or list when opening the Library"
        >
          <div
            style={styles.toggleGroup}
            role="group"
            aria-label="Default library view"
          >
            <ToggleBtn
              id="view-grid-btn"
              active={defaultView === "grid"}
              onClick={() => setSetting("default_view", "grid")}
              aria-label="Grid view"
            >
              Grid
            </ToggleBtn>
            <ToggleBtn
              id="view-list-btn"
              active={defaultView === "list"}
              onClick={() => setSetting("default_view", "list")}
              aria-label="List view"
            >
              List
            </ToggleBtn>
          </div>
        </SettingRow>
      </Section>

      {/* ── Section: Storage ─────────────────────────────────────────────── */}
      <Section icon={<Database size={14} />} title="Storage">
        <SettingRow
          label="Database"
          hint="SQLite database file — pirate_harbor.db in your app data directory"
        >
          <code style={styles.pathCode}>
            %APPDATA%\com.pirateharbor.app\pirate_harbor.db
          </code>
        </SettingRow>

        <SettingRow
          label="Cover images"
          hint="Cover art is stored as file paths — no copying occurs"
        >
          <span style={styles.valueMono}>Referenced locally</span>
        </SettingRow>
      </Section>

      {/* ── Section: About ───────────────────────────────────────────────── */}
      <Section icon={<Info size={14} />} title="About">
        <SettingRow label="Version">
          <span style={styles.valueMono}>v{APP_VERSION}</span>
        </SettingRow>

        <SettingRow label="Design system">
          <span style={styles.valueMono}>Atlas OS — monochrome</span>
        </SettingRow>

        <SettingRow label="Stack">
          <span style={styles.valueMono}>Tauri v2 · React 19 · SQLite</span>
        </SettingRow>

        <SettingRow label="Source">
          <a
            href="https://github.com/sigmakiller/Pirate-Harbor"
            target="_blank"
            rel="noopener noreferrer"
            style={styles.link}
            aria-label="Open source code on GitHub (opens in browser)"
          >
            github.com/sigmakiller/Pirate-Harbor
          </a>
        </SettingRow>
      </Section>
    </div>
  );
}

// ── Sub-components ─────────────────────────────────────────────────────────────

function Section({
  icon,
  title,
  children,
}: {
  icon: React.ReactNode;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section style={styles.section} aria-labelledby={`section-${title.toLowerCase()}`}>
      <div style={styles.sectionHeader}>
        <span style={styles.sectionIcon} aria-hidden="true">{icon}</span>
        <h2
          id={`section-${title.toLowerCase()}`}
          style={styles.sectionTitle}
        >
          {title}
        </h2>
      </div>
      <div style={styles.sectionBody}>{children}</div>
    </section>
  );
}

function SettingRow({
  label,
  hint,
  children,
}: {
  label:    string;
  hint?:    string;
  children: React.ReactNode;
}) {
  return (
    <div style={styles.settingRow}>
      <div style={styles.settingLabel}>
        <span style={styles.labelText}>{label}</span>
        {hint && <span style={styles.hintText}>{hint}</span>}
      </div>
      <div style={styles.settingControl}>{children}</div>
    </div>
  );
}

function ToggleBtn({
  id,
  active,
  onClick,
  children,
  "aria-label": ariaLabel,
}: {
  id:          string;
  active:      boolean;
  onClick:     () => void;
  children:    React.ReactNode;
  "aria-label": string;
}) {
  return (
    <button
      id={id}
      type="button"
      onClick={onClick}
      aria-pressed={active}
      aria-label={ariaLabel}
      style={{
        ...styles.toggleBtn,
        ...(active ? styles.toggleBtnActive : {}),
      }}
    >
      {active && (
        <Check
          size={11}
          aria-hidden="true"
          style={{ flexShrink: 0 }}
        />
      )}
      {children}
    </button>
  );
}

// ── Styles ────────────────────────────────────────────────────────────────────

const styles = {
  page: {
    padding:   "40px 56px",
    maxWidth:  720,
    overflowY: "auto" as const,
    height:    "100%",
    boxSizing: "border-box" as const,
  },
  pageTitle: {
    fontFamily:    "var(--font-display)",
    fontSize:      "clamp(40px, 4vw, 64px)",
    fontWeight:    700,
    letterSpacing: "-0.03em",
    lineHeight:    1.0,
    color:         "var(--color-text-primary)",
    margin:        0,
    marginBottom:  8,
  },
  pageSubtitle: {
    fontFamily:   "var(--font-body)",
    fontSize:     14,
    color:        "var(--color-text-muted)",
    margin:       0,
    marginBottom: 56,
  },
  section: {
    marginBottom: 48,
  },
  sectionHeader: {
    display:       "flex",
    alignItems:    "center",
    gap:           8,
    marginBottom:  16,
    paddingBottom: 10,
    borderBottom:  "1px solid var(--color-border)",
  },
  sectionIcon: {
    color:   "var(--color-text-disabled)",
    display: "flex",
  },
  sectionTitle: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    fontWeight:    500,
    letterSpacing: "0.12em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-muted)",
    margin:        0,
  },
  sectionBody: {
    display:       "flex",
    flexDirection: "column" as const,
  },
  settingRow: {
    display:        "flex",
    justifyContent: "space-between",
    alignItems:     "flex-start",
    padding:        "14px 0",
    borderBottom:   "1px solid var(--color-border-sub)",
    gap:            24,
  },
  settingLabel: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           4,
    flex:          1,
    minWidth:      0,
  },
  labelText: {
    fontFamily: "var(--font-body)",
    fontSize:   13,
    color:      "var(--color-text-primary)",
    fontWeight: 500,
  },
  hintText: {
    fontFamily:  "var(--font-body)",
    fontSize:    12,
    color:       "var(--color-text-disabled)",
    lineHeight:  1.5,
    maxWidth:    360,
  },
  settingControl: {
    flexShrink: 0,
    display:    "flex",
    alignItems: "center",
  },
  toggleGroup: {
    display: "flex",
    gap:     4,
  },
  toggleBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           5,
    background:    "none",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "6px 14px",
    fontSize:      12,
    fontFamily:    "var(--font-mono)",
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-muted)",
    cursor:        "pointer",
    transition:    "border-color 150ms, color 150ms",
  },
  toggleBtnActive: {
    borderColor: "var(--color-text-secondary)",
    color:       "var(--color-text-primary)",
  },
  pathCode: {
    fontFamily:  "var(--font-mono)",
    fontSize:    12,
    color:       "var(--color-text-muted)",
    background:  "var(--color-surface)",
    border:      "1px solid var(--color-border)",
    padding:     "4px 10px",
    borderRadius: 1,
    maxWidth:    260,
    overflow:    "hidden",
    textOverflow: "ellipsis",
    whiteSpace:  "nowrap" as const,
    display:     "block",
  },
  valueMono: {
    fontFamily:    "var(--font-mono)",
    fontSize:      12,
    color:         "var(--color-text-secondary)",
    letterSpacing: "0.04em",
  },
  link: {
    fontFamily:    "var(--font-mono)",
    fontSize:      12,
    color:         "var(--color-text-muted)",
    textDecoration: "none",
    letterSpacing: "0.02em",
    transition:    "color 150ms",
    borderBottom:  "1px solid var(--color-border)",
    paddingBottom: 1,
  },
} satisfies Record<string, React.CSSProperties>;

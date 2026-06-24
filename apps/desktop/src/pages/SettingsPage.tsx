/**
 * SettingsPage — System Configuration.
 *
 * Design spec: Design/Pages/settings.md
 * "Industrial, uncluttered layout."
 *
 * Sections:
 *   1. Appearance        — default library view mode
 *   2. Scan Directories  — T10: watched folder management + scan results
 *   3. Storage           — database location info
 *   4. About             — version, commit, links
 */

import { useCallback, useEffect, useState } from "react";
import { Monitor, Database, Info, Check, FolderSearch, Plus, X, Search, RefreshCw, Key } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";

import { useSettingsStore }   from "@/stores/useSettingsStore";
import { addGame }             from "@/lib/api";
import {
  getScanDirectories,
  addScanDirectory,
  removeScanDirectory,
  scanAllDirectories,
  type ScanResult,
} from "@/lib/api";
import type { ViewMode }      from "@/stores/useLibraryStore";
import type { NewGame }       from "@/types";

const APP_VERSION = "0.1.0";

export default function SettingsPage() {
  const { settings, loading, loadSettings, setSetting, getSetting } =
    useSettingsStore();

  // ── Scan directories state ─────────────────────────────────────────────────
  const [scanDirs,     setScanDirs]     = useState<string[]>([]);
  const [scanResults,  setScanResults]  = useState<ScanResult[]>([]);
  const [scanning,     setScanning]     = useState(false);
  const [scanError,    setScanError]    = useState<string | null>(null);
  const [importing,    setImporting]    = useState<Set<string>>(new Set());
  const [imported,     setImported]     = useState<Set<string>>(new Set());

  // ── RAWG API key state ──────────────────────────────────────────────────────────
  const [rawgKey,       setRawgKey]       = useState("");
  const [rawgKeySaved,  setRawgKeySaved]  = useState(false);

  // Load settings + scan dirs on mount
  useEffect(() => {
    loadSettings();
    getScanDirectories()
      .then(setScanDirs)
      .catch(() => {/* non-fatal */});
    // Load RAWG key into local state
    const savedKey = getSetting("rawg_api_key", "");
    if (typeof savedKey === "string") setRawgKey(savedKey);
  }, [loadSettings]);

  const defaultView = (getSetting("default_view", "grid") as ViewMode) ?? "grid";

  // ── Scan directory handlers ────────────────────────────────────────────────
  const handleAddDirectory = async () => {
    const selected = await open({ multiple: false, directory: true });
    if (!selected || typeof selected !== "string") return;
    try {
      const updated = await addScanDirectory(selected);
      setScanDirs(updated);
      setScanResults([]);  // clear stale results
    } catch (e) {
      setScanError(String(e));
    }
  };

  const handleRemoveDirectory = async (path: string) => {
    const updated = await removeScanDirectory(path);
    setScanDirs(updated);
    setScanResults([]);
  };

  const handleScanAll = useCallback(async () => {
    setScanning(true);
    setScanError(null);
    setScanResults([]);
    try {
      const results = await scanAllDirectories();
      setScanResults(results);
    } catch (e) {
      setScanError(String(e));
    } finally {
      setScanning(false);
    }
  }, []);

  const handleImport = async (result: ScanResult) => {
    if (importing.has(result.exe_path) || imported.has(result.exe_path)) return;
    setImporting((prev) => new Set([...prev, result.exe_path]));
    try {
      const payload: NewGame = {
        title:      result.name,
        exe_path:   result.exe_path,
        cover_path: null,
        developer:  null,
        publisher:  null,
        genre:      null,
        status:     "unplayed",
      };
      await addGame(payload);
      setImported((prev) => new Set([...prev, result.exe_path]));
      // Refresh scan results to mark game as already_added
      setScanResults((prev) =>
        prev.map((r) =>
          r.exe_path === result.exe_path ? { ...r, already_added: true } : r
        )
      );
    } catch {
      // non-fatal — user can retry
    } finally {
      setImporting((prev) => {
        const next = new Set(prev);
        next.delete(result.exe_path);
        return next;
      });
    }
  };

  if (loading && Object.keys(settings).length === 0) {
    return (
      <div className="atlas-enter" style={styles.page}>
        <p style={{ color: "var(--color-text-disabled)", fontSize: 13 }}>
          Loading settings…
        </p>
      </div>
    );
  }

  // Separate results into new vs already-added
  const newResults   = scanResults.filter((r) => !r.already_added);
  const knownResults = scanResults.filter((r) => r.already_added);

  return (
    <div className="atlas-enter" style={styles.page}>
      <h1 style={styles.pageTitle}>Settings</h1>
      <p style={styles.pageSubtitle}>System configuration.</p>

      {/* ── Section: Appearance ──────────────────────────────────────────── */}
      <Section icon={<Monitor size={14} />} title="Appearance">
        <SettingRow
          label="Default library view"
          hint="Grid or list when opening the Library"
        >
          <div style={styles.toggleGroup} role="group" aria-label="Default library view">
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

      {/* ── Section: Scan Directories ─────────────────────────────────────── */}
      <Section icon={<FolderSearch size={14} />} title="Scan Directories">
        <SettingRow
          label="Watched folders"
          hint="Pirate Harbor scans these folders for .exe files you can import"
        >
          <button
            id="add-scan-dir-btn"
            type="button"
            onClick={handleAddDirectory}
            style={styles.outlineBtn}
            aria-label="Add a folder to watch"
          >
            <Plus size={12} aria-hidden="true" />
            Add Folder
          </button>
        </SettingRow>

        {/* Directory list */}
        {scanDirs.length > 0 && (
          <div style={styles.dirList} role="list" aria-label="Watched folders">
            {scanDirs.map((dir) => (
              <div key={dir} style={styles.dirRow} role="listitem">
                <code style={styles.dirPath} title={dir}>{dir}</code>
                <button
                  type="button"
                  onClick={() => handleRemoveDirectory(dir)}
                  style={styles.removeBtn}
                  aria-label={`Remove ${dir} from watched folders`}
                >
                  <X size={12} aria-hidden="true" />
                </button>
              </div>
            ))}
          </div>
        )}

        {/* Scan button */}
        {scanDirs.length > 0 && (
          <div style={{ paddingTop: 12 }}>
            <button
              id="scan-now-btn"
              type="button"
              onClick={handleScanAll}
              disabled={scanning}
              style={{
                ...styles.outlineBtn,
                opacity: scanning ? 0.5 : 1,
                display: "flex",
                gap: 6,
              }}
              aria-label="Scan all watched folders for games"
            >
              {scanning
                ? <RefreshCw size={12} style={{ animation: "spin 1s linear infinite" }} aria-hidden="true" />
                : <Search size={12} aria-hidden="true" />
              }
              {scanning ? "Scanning…" : "Scan Now"}
            </button>
          </div>
        )}

        {scanError && (
          <p style={styles.scanError} role="alert">{scanError}</p>
        )}

        {/* Scan results */}
        {scanResults.length > 0 && (
          <div style={styles.resultsPanel} role="region" aria-label="Scan results">
            <p style={styles.resultsHeader}>
              {newResults.length} new · {knownResults.length} already in library
            </p>

            {newResults.length > 0 && (
              <div style={styles.resultsList} role="list">
                {newResults.map((result) => {
                  const isImporting = importing.has(result.exe_path);
                  const isDone      = imported.has(result.exe_path) || result.already_added;
                  return (
                    <div key={result.exe_path} style={styles.resultRow} role="listitem">
                      <div style={styles.resultMeta}>
                        <span style={styles.resultName}>{result.name}</span>
                        <code style={styles.resultPath} title={result.exe_path}>
                          {result.exe_path}
                        </code>
                      </div>
                      <button
                        type="button"
                        onClick={() => handleImport(result)}
                        disabled={isImporting || isDone}
                        style={{
                          ...styles.importBtn,
                          opacity: isImporting || isDone ? 0.4 : 1,
                          cursor:  isImporting || isDone ? "default" : "pointer",
                        }}
                        aria-label={`Add ${result.name} to library`}
                      >
                        {isDone ? "Added" : isImporting ? "Adding…" : "Add"}
                      </button>
                    </div>
                  );
                })}
              </div>
            )}

            {knownResults.length > 0 && (
              <details style={styles.knownDetails}>
                <summary style={styles.knownSummary}>
                  {knownResults.length} already in library
                </summary>
                <div style={styles.resultsList} role="list">
                  {knownResults.map((result) => (
                    <div key={result.exe_path} style={{ ...styles.resultRow, opacity: 0.4 }} role="listitem">
                      <div style={styles.resultMeta}>
                        <span style={styles.resultName}>{result.name}</span>
                        <code style={styles.resultPath}>{result.exe_path}</code>
                      </div>
                      <span style={styles.alreadyTag}>In library</span>
                    </div>
                  ))}
                </div>
              </details>
            )}
          </div>
        )}

        {/* No dirs empty state */}
        {scanDirs.length === 0 && (
          <p style={styles.emptyHint}>
            Add a folder above to start auto-detecting installed games.
          </p>
        )}
      </Section>

      {/* ── Section: Integrations ───────────────────────────────────── */}
      <Section icon={<Key size={14} />} title="Integrations">
        <SettingRow
          label="RAWG API Key"
          hint="Used to auto-fill game metadata in Add Game. Free key at rawg.io/apidocs"
        >
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input
              id="rawg-api-key-input"
              type="password"
              value={rawgKey}
              onChange={(e) => { setRawgKey(e.target.value); setRawgKeySaved(false); }}
              onBlur={() => {
                setSetting("rawg_api_key", rawgKey.trim());
                setRawgKeySaved(true);
                setTimeout(() => setRawgKeySaved(false), 2000);
              }}
              placeholder="Paste your RAWG API key…"
              autoComplete="off"
              style={styles.apiKeyInput}
              aria-label="RAWG API key"
            />
            {rawgKeySaved && (
              <span style={styles.savedBadge} aria-live="polite">Saved</span>
            )}
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
    <section style={styles.section} aria-labelledby={`section-${title.toLowerCase().replace(/ /g, "-")}`}>
      <div style={styles.sectionHeader}>
        <span style={styles.sectionIcon} aria-hidden="true">{icon}</span>
        <h2
          id={`section-${title.toLowerCase().replace(/ /g, "-")}`}
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
  id:           string;
  active:       boolean;
  onClick:      () => void;
  children:     React.ReactNode;
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
      {active && <Check size={11} aria-hidden="true" style={{ flexShrink: 0 }} />}
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
    fontFamily: "var(--font-body)",
    fontSize:   12,
    color:      "var(--color-text-disabled)",
    lineHeight: 1.5,
    maxWidth:   360,
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
  outlineBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           6,
    background:    "none",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "7px 14px",
    fontSize:      11,
    fontFamily:    "var(--font-mono)",
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-muted)",
    cursor:        "pointer",
    transition:    "border-color 150ms, color 150ms",
  },
  dirList: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           4,
    marginTop:     8,
  },
  dirRow: {
    display:        "flex",
    alignItems:     "center",
    justifyContent: "space-between",
    padding:        "8px 12px",
    background:     "var(--color-surface)",
    border:         "1px solid var(--color-border)",
    borderRadius:   1,
    gap:            8,
  },
  dirPath: {
    fontFamily:   "var(--font-mono)",
    fontSize:     11,
    color:        "var(--color-text-muted)",
    flex:         1,
    minWidth:     0,
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
  },
  removeBtn: {
    background:  "none",
    border:      "none",
    color:       "var(--color-text-disabled)",
    cursor:      "pointer",
    padding:     4,
    display:     "flex",
    flexShrink:  0,
    transition:  "color 150ms",
  },
  scanError: {
    fontFamily:  "var(--font-body)",
    fontSize:    12,
    color:       "var(--color-text-muted)",
    margin:      "8px 0 0",
  },
  emptyHint: {
    fontFamily:  "var(--font-body)",
    fontSize:    13,
    color:       "var(--color-text-disabled)",
    margin:      "12px 0 0",
    paddingTop:  12,
  },
  resultsPanel: {
    marginTop:    16,
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    overflow:     "hidden",
  },
  resultsHeader: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.08em",
    color:         "var(--color-text-disabled)",
    margin:        0,
    padding:       "10px 14px",
    borderBottom:  "1px solid var(--color-border)",
    background:    "var(--color-surface)",
  },
  resultsList: {
    display:       "flex",
    flexDirection: "column" as const,
  },
  resultRow: {
    display:        "flex",
    alignItems:     "center",
    justifyContent: "space-between",
    padding:        "10px 14px",
    borderBottom:   "1px solid var(--color-border-sub)",
    gap:            16,
  },
  resultMeta: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           2,
    flex:          1,
    minWidth:      0,
  },
  resultName: {
    fontFamily:   "var(--font-body)",
    fontSize:     13,
    fontWeight:   500,
    color:        "var(--color-text-primary)",
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
  },
  resultPath: {
    fontFamily:   "var(--font-mono)",
    fontSize:     10,
    color:        "var(--color-text-disabled)",
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
  },
  importBtn: {
    flexShrink:    0,
    background:    "var(--color-elevated)",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "5px 14px",
    fontSize:      11,
    fontFamily:    "var(--font-mono)",
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-muted)",
    transition:    "border-color 150ms",
  },
  alreadyTag: {
    flexShrink:    0,
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    color:         "var(--color-text-disabled)",
    letterSpacing: "0.08em",
  },
  knownDetails: {
    borderTop: "1px solid var(--color-border-sub)",
  },
  knownSummary: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.06em",
    color:         "var(--color-text-disabled)",
    cursor:        "pointer",
    padding:       "10px 14px",
    listStyle:     "none",
  },
  pathCode: {
    fontFamily:   "var(--font-mono)",
    fontSize:     12,
    color:        "var(--color-text-muted)",
    background:   "var(--color-surface)",
    border:       "1px solid var(--color-border)",
    padding:      "4px 10px",
    borderRadius: 1,
    maxWidth:     260,
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
    display:      "block",
  },
  valueMono: {
    fontFamily:    "var(--font-mono)",
    fontSize:      12,
    color:         "var(--color-text-secondary)",
    letterSpacing: "0.04em",
  },
  link: {
    fontFamily:     "var(--font-mono)",
    fontSize:       12,
    color:          "var(--color-text-muted)",
    textDecoration: "none",
    letterSpacing:  "0.02em",
    transition:     "color 150ms",
    borderBottom:   "1px solid var(--color-border)",
    paddingBottom:  1,
  },
  apiKeyInput: {
    width:        220,
    background:   "var(--color-surface)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "7px 12px",
    fontSize:     12,
    fontFamily:   "var(--font-mono)",
    color:        "var(--color-text-primary)",
    outline:      "none",
    letterSpacing: "0.04em",
    boxSizing:    "border-box" as const,
    transition:   "border-color 150ms",
  },
  savedBadge: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.10em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
  },
} satisfies Record<string, React.CSSProperties>;

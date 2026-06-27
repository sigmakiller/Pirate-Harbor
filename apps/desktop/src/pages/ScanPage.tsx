/**
 * ScanPage — Folder scanner workflow (T16).
 *
 * Workflow:
 *   1. User picks a folder with the native file dialog.
 *   2. Backend walks the folder, scores each .exe, and returns candidates.
 *   3. Results are displayed sorted by confidence (highest first).
 *   4. High-confidence results (≥ 0.7) are pre-selected; low (< 0.4) are deselected.
 *   5. User selects/deselects individual games or uses "Select All High Confidence".
 *   6. "Add Selected" bulk-inserts and navigates back to library.
 */

import React, { useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { FolderSearch, CheckSquare, Square, AlertCircle, ArrowLeft } from "lucide-react";

import { FilePickerButton } from "@/components/FilePickerButton";
import {
  scanDirectory,
  batchAddGames,
  type ScanResult,
} from "@/lib/api";
import type { NewGame } from "@/types";
import { useToastStore } from "@/stores/useToastStore";

// ── Helpers ───────────────────────────────────────────────────────────────────

/** Confidence tier for colour coding. */
function confTier(c: number): "high" | "mid" | "low" {
  if (c >= 0.7) return "high";
  if (c >= 0.4) return "mid";
  return "low";
}

const TIER_COLOR = {
  high: "var(--color-text-secondary)",
  mid:  "var(--color-text-muted)",
  low:  "var(--color-text-disabled)",
} as const;

// ─────────────────────────────────────────────────────────────────────────────

export default function ScanPage() {
  const navigate  = useNavigate();
  const { addToast } = useToastStore();

  const [folder,       setFolder]       = useState("");
  const [results,      setResults]      = useState<ScanResult[]>([]);
  const [scanning,     setScanning]     = useState(false);
  const [scanError,    setScanError]    = useState<string | null>(null);
  const [selected,     setSelected]     = useState<Set<string>>(new Set());
  const [adding,       setAdding]       = useState(false);
  const [scanned,      setScanned]      = useState(false);

  // ── Scan ──────────────────────────────────────────────────────────────────

  const handleScan = useCallback(async () => {
    if (!folder) return;
    setScanning(true);
    setScanError(null);
    setResults([]);
    setSelected(new Set());
    setScanned(false);

    try {
      const found = await scanDirectory(folder);
      setResults(found);
      setScanned(true);

      // Auto-select high-confidence, unimported games
      const autoSelect = new Set(
        found
          .filter(r => !r.already_added && r.confidence >= 0.7)
          .map(r => r.exe_path)
      );
      setSelected(autoSelect);
    } catch (e) {
      setScanError(String(e));
    } finally {
      setScanning(false);
    }
  }, [folder]);

  // ── Selection helpers ─────────────────────────────────────────────────────

  const toggle = (exe: string) => {
    setSelected(prev => {
      const next = new Set(prev);
      if (next.has(exe)) next.delete(exe); else next.add(exe);
      return next;
    });
  };

  const selectAllHighConfidence = () => {
    const highConf = new Set(
      results
        .filter(r => !r.already_added && r.confidence >= 0.7)
        .map(r => r.exe_path)
    );
    setSelected(highConf);
  };

  const deselectAll = () => setSelected(new Set());

  // ── Add selected ──────────────────────────────────────────────────────────

  const handleAddSelected = async () => {
    const toAdd: NewGame[] = results
      .filter(r => selected.has(r.exe_path) && !r.already_added)
      .map(r => ({
        title:       r.name,
        exe_path:    r.exe_path,
        cover_path:  null,
        banner_path: null,
        developer:   null,
        publisher:   null,
        genre:       null,
        status:      undefined,
      }));

    if (toAdd.length === 0) return;

    setAdding(true);
    try {
      const added = await batchAddGames(toAdd);
      addToast({
        message: `${added.length} game${added.length === 1 ? "" : "s"} added to your library`,
        type:    "success",
      });
      navigate("/library");
    } catch (e) {
      addToast({ message: `Failed to add games: ${e}`, type: "error" });
    } finally {
      setAdding(false);
    }
  };

  // ── Derived ───────────────────────────────────────────────────────────────

  const newResults     = results.filter(r => !r.already_added);
  const knownResults   = results.filter(r =>  r.already_added);
  const selectedCount  = [...selected].filter(e => !results.find(r => r.exe_path === e)?.already_added).length;

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <div className="atlas-enter" style={s.page}>

      {/* ── Header ──────────────────────────────────────────────────────── */}
      <div style={s.header}>
        <button
          type="button"
          onClick={() => navigate("/library")}
          style={s.backBtn}
          aria-label="Back to library"
        >
          <ArrowLeft size={14} />
          Library
        </button>
        <div>
          <h1 style={s.title}>Scan Folder</h1>
          <p style={s.subtitle}>
            Detect games automatically using confidence scoring.
          </p>
        </div>
      </div>

      {/* ── Folder picker ───────────────────────────────────────────────── */}
      <div style={s.pickerRow}>
        <div style={{ flex: 1 }}>
          <FilePickerButton
            id="scan-folder-picker"
            value={folder}
            onChange={(p) => { setFolder(p); setScanned(false); setResults([]); }}
            directory={true}
            placeholder="Choose a folder to scan…"
          />
        </div>
        <button
          id="scan-start-btn"
          type="button"
          onClick={handleScan}
          disabled={!folder || scanning}
          style={{
            ...s.primaryBtn,
            opacity: !folder || scanning ? 0.4 : 1,
            cursor:  !folder || scanning ? "default" : "pointer",
          }}
          aria-label="Start scanning"
        >
          <FolderSearch size={13} aria-hidden="true" />
          {scanning ? "Scanning…" : "Scan"}
        </button>
      </div>

      {/* ── Error ───────────────────────────────────────────────────────── */}
      {scanError && (
        <div style={s.errorBanner} role="alert">
          <AlertCircle size={13} />
          {scanError}
        </div>
      )}

      {/* ── Results ─────────────────────────────────────────────────────── */}
      {scanned && !scanning && (
        <>
          {/* Summary + bulk actions */}
          <div style={s.summaryRow}>
            <p style={s.summaryText}>
              Found <strong>{newResults.length}</strong> potential game{newResults.length !== 1 ? "s" : ""} in{" "}
              <code style={s.folderCode}>{folder}</code>
              {knownResults.length > 0 && (
                <span style={s.alreadyNote}>
                  &nbsp;· {knownResults.length} already in library
                </span>
              )}
            </p>

            <div style={s.bulkBtns}>
              <button type="button" onClick={selectAllHighConfidence} style={s.ghostBtn}>
                <CheckSquare size={12} /> Select High Confidence
              </button>
              <button type="button" onClick={deselectAll} style={s.ghostBtn}>
                <Square size={12} /> Deselect All
              </button>
            </div>
          </div>

          {/* New games */}
          {newResults.length > 0 ? (
            <div style={s.resultList} role="list" aria-label="Discovered games">
              {newResults.map(r => {
                const tier     = confTier(r.confidence);
                const color    = TIER_COLOR[tier];
                const confPct  = Math.round(r.confidence * 100);
                const isSel    = selected.has(r.exe_path);
                return (
                  <div
                    key={r.exe_path}
                    role="listitem"
                    style={{
                      ...s.resultRow,
                      borderColor: isSel ? "var(--color-border)" : "var(--color-border-sub)",
                      opacity:     1,
                    }}
                  >
                    {/* Checkbox */}
                    <input
                      type="checkbox"
                      id={`scan-row-${r.exe_path}`}
                      checked={isSel}
                      onChange={() => toggle(r.exe_path)}
                      style={s.checkbox}
                      aria-label={`Select ${r.name}`}
                    />

                    {/* Meta */}
                    <label htmlFor={`scan-row-${r.exe_path}`} style={s.resultMeta}>
                      <span style={s.resultName}>{r.name}</span>

                      <div style={s.pathRow}>
                        <code style={s.resultPath} title={r.exe_path}>{r.exe_path}</code>
                        {r.folder_name && (
                          <span style={s.folderTag}>└ {r.folder_name}</span>
                        )}
                      </div>

                      {/* Confidence bar */}
                      <div style={s.confRow}>
                        <div style={s.confTrack}>
                          <div
                            style={{
                              ...s.confFill,
                              width:      `${confPct}%`,
                              background: color,
                            }}
                          />
                        </div>
                        <span style={{ ...s.confLabel, color }}>{confPct}%</span>
                        <span style={s.sizeBadge}>{r.size_mb.toFixed(0)} MB</span>
                        {tier === "high" && (
                          <span style={{ ...s.tierBadge, color }}>High</span>
                        )}
                        {tier === "low" && (
                          <span style={{ ...s.tierBadge, color }}>Low confidence</span>
                        )}
                      </div>
                    </label>
                  </div>
                );
              })}
            </div>
          ) : (
            <div style={s.emptyState}>
              <p style={s.emptyText}>No new games found in this folder.</p>
            </div>
          )}

          {/* Already-in-library (collapsed) */}
          {knownResults.length > 0 && (
            <details style={s.knownSection}>
              <summary style={s.knownSummary}>
                {knownResults.length} already in library
              </summary>
              <div role="list">
                {knownResults.map(r => (
                  <div key={r.exe_path} style={{ ...s.resultRow, opacity: 0.4 }} role="listitem">
                    <div style={s.resultMeta}>
                      <span style={s.resultName}>{r.name}</span>
                      <code style={s.resultPath}>{r.exe_path}</code>
                    </div>
                    <span style={s.inLibBadge}>In library</span>
                  </div>
                ))}
              </div>
            </details>
          )}

          {/* Add selected */}
          {newResults.length > 0 && (
            <div style={s.footer}>
              <button
                id="scan-add-selected-btn"
                type="button"
                onClick={handleAddSelected}
                disabled={selectedCount === 0 || adding}
                style={{
                  ...s.primaryBtn,
                  opacity: selectedCount === 0 || adding ? 0.4 : 1,
                  cursor:  selectedCount === 0 || adding ? "default" : "pointer",
                }}
                aria-label={`Add ${selectedCount} selected games to library`}
              >
                {adding
                  ? "Adding…"
                  : `Add ${selectedCount > 0 ? `${selectedCount} ` : ""}Selected`}
              </button>
            </div>
          )}
        </>
      )}

      {/* ── Empty start state ────────────────────────────────────────────── */}
      {!scanned && !scanning && !scanError && (
        <div style={s.startHint}>
          <FolderSearch size={36} style={{ color: "var(--color-text-disabled)", marginBottom: 16 }} />
          <p style={s.hintText}>Choose a folder above and click Scan to discover games.</p>
          <ul style={s.hintList}>
            <li>Files under 20 MB are skipped (launchers, helpers)</li>
            <li>Results are sorted by confidence — highest first</li>
            <li>High confidence (≥ 70%) games are pre-selected</li>
          </ul>
        </div>
      )}
    </div>
  );
}

// ── Styles ────────────────────────────────────────────────────────────────────

const s: Record<string, React.CSSProperties> = {
  page: {
    padding:   "40px 56px",
    maxWidth:   900,
    height:     "100%",
    boxSizing: "border-box",
    overflowY: "auto",
  },
  header: {
    marginBottom: 36,
    display:      "flex",
    flexDirection:"column",
    gap:          20,
  },
  backBtn: {
    display:       "inline-flex",
    alignItems:    "center",
    gap:           6,
    background:    "none",
    border:        "none",
    color:         "var(--color-text-disabled)",
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.08em",
    textTransform: "uppercase",
    cursor:        "pointer",
    padding:       0,
  },
  title: {
    fontFamily:    "var(--font-display)",
    fontSize:      32,
    fontWeight:    700,
    letterSpacing: "-0.02em",
    color:         "var(--color-text-primary)",
    margin:        0,
  },
  subtitle: {
    fontFamily: "var(--font-body)",
    fontSize:   14,
    color:      "var(--color-text-muted)",
    margin:     "8px 0 0",
  },
  pickerRow: {
    display:    "flex",
    gap:        12,
    alignItems: "stretch",
    marginBottom: 28,
  },
  primaryBtn: {
    display:       "inline-flex",
    alignItems:    "center",
    gap:           8,
    background:    "var(--color-text-primary)",
    color:         "var(--color-base)",
    border:        "none",
    padding:       "10px 24px",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    fontWeight:    600,
    letterSpacing: "0.06em",
    textTransform: "uppercase",
    borderRadius:  1,
    transition:    "opacity 150ms",
    flexShrink:    0,
  },
  errorBanner: {
    display:      "flex",
    alignItems:   "center",
    gap:          8,
    color:        "var(--color-text-muted)",
    fontFamily:   "var(--font-body)",
    fontSize:     13,
    marginBottom: 24,
  },
  summaryRow: {
    display:        "flex",
    alignItems:     "flex-start",
    justifyContent: "space-between",
    gap:            16,
    marginBottom:   20,
    flexWrap:       "wrap",
  },
  summaryText: {
    fontFamily: "var(--font-body)",
    fontSize:   14,
    color:      "var(--color-text-muted)",
    margin:     0,
  },
  folderCode: {
    fontFamily:   "var(--font-mono)",
    fontSize:     12,
    color:        "var(--color-text-secondary)",
    background:   "var(--color-elevated)",
    padding:      "1px 6px",
    borderRadius: 2,
  },
  alreadyNote: {
    color:      "var(--color-text-disabled)",
    fontSize:   12,
    fontFamily: "var(--font-mono)",
  },
  bulkBtns: {
    display: "flex",
    gap:     8,
    flexShrink: 0,
  },
  ghostBtn: {
    display:       "inline-flex",
    alignItems:    "center",
    gap:           6,
    background:    "none",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    padding:       "5px 12px",
    fontSize:      11,
    fontFamily:    "var(--font-mono)",
    letterSpacing: "0.06em",
    color:         "var(--color-text-muted)",
    cursor:        "pointer",
  },
  resultList: {
    display:      "flex",
    flexDirection:"column",
    gap:          8,
    marginBottom: 24,
  },
  resultRow: {
    display:      "flex",
    alignItems:   "flex-start",
    gap:          14,
    padding:      "14px 16px",
    background:   "var(--color-surface)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
  },
  checkbox: {
    flexShrink:  0,
    marginTop:   3,
    accentColor: "var(--color-text-secondary)",
    cursor:      "pointer",
    width:       16,
    height:      16,
  },
  resultMeta: {
    flex:          1,
    display:       "flex",
    flexDirection: "column",
    gap:           4,
    cursor:        "pointer",
    minWidth:      0,
  },
  resultName: {
    fontFamily:   "var(--font-body)",
    fontSize:     14,
    fontWeight:   500,
    color:        "var(--color-text-primary)",
  },
  pathRow: {
    display:    "flex",
    alignItems: "center",
    gap:        10,
    minWidth:   0,
  },
  resultPath: {
    fontFamily:   "var(--font-mono)",
    fontSize:     11,
    color:        "var(--color-text-disabled)",
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap",
    minWidth:     0,
  },
  folderTag: {
    fontFamily: "var(--font-mono)",
    fontSize:   10,
    color:      "var(--color-text-disabled)",
    flexShrink: 0,
  },
  confRow: {
    display:    "flex",
    alignItems: "center",
    gap:        8,
    marginTop:  4,
  },
  confTrack: {
    flex:         "0 0 100px",
    height:       3,
    background:   "var(--color-elevated)",
    borderRadius: 99,
    overflow:     "hidden",
  },
  confFill: {
    height:       "100%",
    borderRadius: 99,
    transition:   "width 300ms",
  },
  confLabel: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.06em",
    flexShrink:    0,
  },
  sizeBadge: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    color:         "var(--color-text-disabled)",
    letterSpacing: "0.04em",
    flexShrink:    0,
  },
  tierBadge: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.06em",
    flexShrink:    0,
  },
  inLibBadge: {
    flexShrink:    0,
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.08em",
    color:         "var(--color-text-disabled)",
    alignSelf:     "center",
  },
  knownSection: {
    borderTop:    "1px solid var(--color-border-sub)",
    paddingTop:   16,
    marginBottom: 24,
  },
  knownSummary: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.06em",
    color:         "var(--color-text-disabled)",
    cursor:        "pointer",
    marginBottom:  12,
    listStyle:     "none",
  },
  emptyState: {
    padding:   "60px 0",
    textAlign: "center",
  },
  emptyText: {
    fontFamily: "var(--font-body)",
    fontSize:   14,
    color:      "var(--color-text-disabled)",
    margin:     0,
  },
  footer: {
    display:      "flex",
    justifyContent: "flex-end",
    paddingTop:   16,
    borderTop:    "1px solid var(--color-border)",
    marginTop:    8,
  },
  startHint: {
    display:        "flex",
    flexDirection:  "column",
    alignItems:     "center",
    justifyContent: "center",
    padding:        "80px 0",
    textAlign:      "center",
  },
  hintText: {
    fontFamily: "var(--font-body)",
    fontSize:   14,
    color:      "var(--color-text-muted)",
    margin:     "0 0 20px",
  },
  hintList: {
    fontFamily:  "var(--font-mono)",
    fontSize:    12,
    color:       "var(--color-text-disabled)",
    listStyle:   "none",
    padding:     0,
    lineHeight:  2.0,
    letterSpacing: "0.04em",
  },
};

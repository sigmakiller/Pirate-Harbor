/**
 * OnboardingPage — v2 first-run experience (T61).
 *
 * 5-step flow:
 *   Step 0  Welcome             — hero tagline, "Get Started"
 *   Step 1  Connect Metadata    — RAWG API key input + validation, skip option
 *   Step 2  Find Your Games     — folder picker → scanner → game-count preview
 *   Step 3  Library Ready       — summary stats (N games found)
 *   Step 4  Let's Go            — navigate to /library
 *
 * Design: Atlas OS monochrome palette. Consistent with app-wide styles.
 */

import { useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import {
  ArrowRight, ArrowLeft, Check, Zap,
  Key, FolderSearch, BookOpen, Sparkles,
} from "lucide-react";

import { FilePickerButton } from "@/components/FilePickerButton";
import {
  setSetting,
  testRawgKey,
  scanDirectory,
  batchAddGames,
  type ScanResult,
} from "@/lib/api";
import type { NewGame } from "@/types";

// ─────────────────────────────────────────────────────────────────────────────

type Step = 0 | 1 | 2 | 3 | 4;

const STEP_META = [
  { id: "welcome",  label: "Welcome",         icon: Sparkles   },
  { id: "rawg",     label: "Metadata",        icon: Key        },
  { id: "scan",     label: "Find Games",      icon: FolderSearch },
  { id: "ready",    label: "Library Ready",   icon: BookOpen   },
  { id: "launch",   label: "Let's Go",        icon: Zap        },
] as const;

// ─────────────────────────────────────────────────────────────────────────────

export default function OnboardingPage() {
  const navigate = useNavigate();
  const [step, setStep] = useState<Step>(0);

  // ── Step 1: RAWG key state ────────────────────────────────────────────────
  const [rawgKey,       setRawgKey]       = useState("");
  const [rawgTesting,   setRawgTesting]   = useState(false);
  const [rawgStatus,    setRawgStatus]    = useState<"idle" | "valid" | "invalid">("idle");
  const [rawgError,     setRawgError]     = useState<string | null>(null);

  // ── Step 2: Scan state ───────────────────────────────────────────────────
  const [folder,        setFolder]        = useState("");
  const [scanning,      setScanning]      = useState(false);
  const [scanResults,   setScanResults]   = useState<ScanResult[]>([]);
  const [scanError,     setScanError]     = useState<string | null>(null);
  const [adding,        setAdding]        = useState(false);
  const [addedCount,    setAddedCount]    = useState(0);

  // ─────────────────────────────────────────────────────────────────────────

  const next = () => setStep(s => Math.min(4, s + 1) as Step);
  const back = () => setStep(s => Math.max(0, s - 1) as Step);

  // ── RAWG validation ───────────────────────────────────────────────────────
  const handleTestRawg = async () => {
    if (!rawgKey.trim()) return;
    setRawgTesting(true);
    setRawgStatus("idle");
    setRawgError(null);
    try {
      const result = await testRawgKey(rawgKey.trim());
      if (result.valid) {
        await setSetting("rawg_api_key", rawgKey.trim());
        setRawgStatus("valid");
      } else {
        setRawgStatus("invalid");
        setRawgError(result.error ?? "Invalid API key.");
      }
    } catch (e) {
      setRawgStatus("invalid");
      setRawgError(String(e));
    } finally {
      setRawgTesting(false);
    }
  };

  const handleSkipRawg = () => {
    setRawgStatus("idle");
    next();
  };

  // ── Scan ──────────────────────────────────────────────────────────────────
  const handleScan = useCallback(async () => {
    if (!folder) return;
    setScanning(true);
    setScanError(null);
    setScanResults([]);
    try {
      const results = await scanDirectory(folder);
      // Auto-select all high-confidence results (≥ 0.7)
      setScanResults(results);
    } catch (e) {
      setScanError(String(e));
    } finally {
      setScanning(false);
    }
  }, [folder]);

  const highConf = scanResults.filter(r => r.confidence >= 0.7);
  const totalSelected = highConf.length;

  const handleAddGames = async () => {
    if (highConf.length === 0) {
      next();
      return;
    }
    setAdding(true);
    try {
      const games: NewGame[] = highConf.map(r => ({
        title:    r.name || r.folder_name,
        exe_path: r.exe_path,
        status:   "unplayed",
      }));
      const added = await batchAddGames(games);
      setAddedCount(added.length);
    } catch {
      // Still advance — user can scan again from library
    } finally {
      setAdding(false);
      next();
    }
  };

  // ── Finish ────────────────────────────────────────────────────────────────
  const handleFinish = async () => {
    await setSetting("onboarding_complete", "true");
    navigate("/library");
  };

  // ─────────────────────────────────────────────────────────────────────────

  return (
    <div style={S.page}>
      {/* Step indicator */}
      <nav style={S.stepNav} aria-label="Onboarding steps">
        {STEP_META.map((s, i) => (
          <div
            key={s.id}
            style={{
              ...S.stepDot,
              ...(i < step ? S.stepDotDone : {}),
              ...(i === step ? S.stepDotActive : {}),
            }}
            aria-current={i === step ? "step" : undefined}
            aria-label={`Step ${i + 1}: ${s.label}${i < step ? " (complete)" : ""}`}
          >
            {i < step && <Check size={8} />}
          </div>
        ))}
      </nav>

      {/* ── Step 0: Welcome ────────────────────────────────────────────────── */}
      {step === 0 && (
        <div className="atlas-enter" style={S.content}>
          <div style={S.heroIcon}>
            <Sparkles size={32} />
          </div>
          <span style={S.eyebrow}>Pirate Harbor</span>
          <h1 style={S.title}>
            A personal OS for<br />gaming history.
          </h1>
          <p style={S.body}>
            Track every game you play. Preserve your history.
            No accounts, no cloud, no noise — just you and your library.
          </p>
          <button
            id="onboarding-welcome-next"
            onClick={next}
            style={S.primaryBtn}
            aria-label="Get started"
          >
            Get started
            <ArrowRight size={14} aria-hidden="true" />
          </button>
        </div>
      )}

      {/* ── Step 1: Connect Metadata (RAWG) ───────────────────────────────── */}
      {step === 1 && (
        <div className="atlas-enter" style={S.content}>
          <div style={S.heroIcon}>
            <Key size={28} />
          </div>
          <span style={S.eyebrow}>Step 1 of 3 — Optional</span>
          <h1 style={S.title}>Connect metadata.</h1>
          <p style={S.body}>
            Pirate Harbor enriches your library with cover art, genres, and
            playtime data from RAWG — the world's largest game database.
            Enter your free API key to enable this.
          </p>

          <div style={S.keyRow}>
            <input
              id="onboarding-rawg-key"
              type="text"
              value={rawgKey}
              onChange={e => { setRawgKey(e.target.value); setRawgStatus("idle"); }}
              placeholder="Paste your RAWG API key…"
              style={{
                ...S.keyInput,
                ...(rawgStatus === "valid"   ? S.keyInputValid   : {}),
                ...(rawgStatus === "invalid" ? S.keyInputInvalid : {}),
              }}
              aria-label="RAWG API key"
              autoComplete="off"
              spellCheck={false}
            />
            <button
              id="onboarding-test-rawg"
              type="button"
              onClick={handleTestRawg}
              disabled={!rawgKey.trim() || rawgTesting}
              style={{
                ...S.testBtn,
                opacity: !rawgKey.trim() || rawgTesting ? 0.4 : 1,
              }}
              aria-label="Test RAWG API key"
            >
              {rawgTesting ? "Testing…" : "Test"}
            </button>
          </div>

          {rawgStatus === "valid" && (
            <p style={S.successMsg} role="status">
              <Check size={12} style={{ marginRight: 6 }} />
              Key validated and saved. Metadata enrichment enabled.
            </p>
          )}
          {rawgStatus === "invalid" && rawgError && (
            <p style={S.errorMsg} role="alert">{rawgError}</p>
          )}

          <p style={S.linkHint}>
            No key yet?{" "}
            <a
              href="https://rawg.io/apidocs"
              target="_blank"
              rel="noopener noreferrer"
              style={S.link}
            >
              Get a free key at rawg.io
            </a>
          </p>

          <div style={S.btnRow}>
            <button
              id="onboarding-rawg-back"
              type="button"
              onClick={back}
              style={S.ghostBtn}
              aria-label="Back"
            >
              <ArrowLeft size={13} />
              Back
            </button>
            <div style={{ display: "flex", gap: 10 }}>
              <button
                id="onboarding-skip-rawg"
                type="button"
                onClick={handleSkipRawg}
                style={S.skipBtn}
              >
                Skip for now
              </button>
              <button
                id="onboarding-rawg-next"
                type="button"
                onClick={next}
                disabled={rawgStatus !== "valid"}
                style={{
                  ...S.primaryBtn,
                  opacity: rawgStatus !== "valid" ? 0.4 : 1,
                }}
                aria-label="Continue with this key"
              >
                Continue
                <ArrowRight size={14} />
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ── Step 2: Find Your Games ────────────────────────────────────────── */}
      {step === 2 && (
        <div className="atlas-enter" style={S.content}>
          <div style={S.heroIcon}>
            <FolderSearch size={28} />
          </div>
          <span style={S.eyebrow}>Step 2 of 3</span>
          <h1 style={S.title}>Find your games.</h1>
          <p style={S.body}>
            Point Pirate Harbor at your games folder. It will scan for
            executables and auto-detect game titles by confidence score.
          </p>

          <div style={S.scanBox}>
            <FilePickerButton
              id="onboarding-folder-picker"
              value={folder}
              onChange={(p: string) => {
                setFolder(p);
                setScanResults([]);
                setScanError(null);
              }}
              directory
              placeholder="Select games folder…"
            />

            <button
              id="onboarding-scan-btn"
              type="button"
              onClick={handleScan}
              disabled={!folder || scanning}
              style={{
                ...S.primaryBtn,
                marginTop: 12,
                opacity: !folder || scanning ? 0.4 : 1,
              }}
            >
              {scanning ? "Scanning…" : "Scan Folder"}
              {!scanning && <Zap size={13} />}
            </button>

            {scanError && (
              <p style={S.errorMsg} role="alert">{scanError}</p>
            )}

            {scanResults.length > 0 && (
              <div style={S.scanPreview}>
                <p style={S.scanPreviewTitle}>
                  Found <strong>{scanResults.length}</strong> candidate{scanResults.length !== 1 ? "s" : ""} ·{" "}
                  <strong style={{ color: "var(--color-text-primary)" }}>{totalSelected}</strong> high-confidence
                </p>
                <div style={S.scanList}>
                  {scanResults.slice(0, 8).map(r => (
                    <div key={r.exe_path} style={S.scanRow}>
                      <span style={{
                        ...S.scanConf,
                        color: r.confidence >= 0.7
                          ? "var(--color-text-secondary)"
                          : "var(--color-text-disabled)",
                      }}>
                        {Math.round(r.confidence * 100)}%
                      </span>
                    <span style={S.scanTitle}>{r.name || r.folder_name}</span>
                    </div>
                  ))}
                  {scanResults.length > 8 && (
                    <p style={S.scanMore}>+{scanResults.length - 8} more…</p>
                  )}
                </div>
              </div>
            )}
          </div>

          <div style={S.btnRow}>
            <button
              id="onboarding-scan-back"
              type="button"
              onClick={back}
              style={S.ghostBtn}
            >
              <ArrowLeft size={13} />
              Back
            </button>
            <div style={{ display: "flex", gap: 10 }}>
              {scanResults.length === 0 && (
                <button
                  id="onboarding-skip-scan"
                  type="button"
                  onClick={next}
                  style={S.skipBtn}
                >
                  Skip scan
                </button>
              )}
              {scanResults.length > 0 && (
                <button
                  id="onboarding-add-games"
                  type="button"
                  onClick={handleAddGames}
                  disabled={adding}
                  style={{ ...S.primaryBtn, opacity: adding ? 0.4 : 1 }}
                >
                  {adding
                    ? "Adding…"
                    : `Add ${totalSelected} game${totalSelected !== 1 ? "s" : ""}`}
                  {!adding && <ArrowRight size={14} />}
                </button>
              )}
            </div>
          </div>
        </div>
      )}

      {/* ── Step 3: Library Ready ──────────────────────────────────────────── */}
      {step === 3 && (
        <div className="atlas-enter" style={S.content}>
          <div style={S.heroIcon} aria-hidden="true">
            <BookOpen size={32} />
          </div>
          <span style={S.eyebrow}>Step 3 of 3</span>
          <h1 style={S.title}>Your library<br />is ready.</h1>

          <div style={S.statGrid}>
            <div style={S.statCard}>
              <span style={S.statNum}>{addedCount}</span>
              <span style={S.statLabel}>
                {addedCount === 1 ? "game" : "games"} imported
              </span>
            </div>
            <div style={S.statCard}>
              <span style={S.statNum}>{rawgStatus === "valid" ? "✓" : "—"}</span>
              <span style={S.statLabel}>metadata enabled</span>
            </div>
          </div>

          {addedCount === 0 && (
            <p style={{ ...S.body, marginTop: 16 }}>
              No games were imported. You can add them manually from your
              library at any time.
            </p>
          )}

          <div style={S.btnRow}>
            <button
              id="onboarding-ready-back"
              type="button"
              onClick={back}
              style={S.ghostBtn}
            >
              <ArrowLeft size={13} />
              Back
            </button>
            <button
              id="onboarding-ready-next"
              type="button"
              onClick={next}
              style={S.primaryBtn}
            >
              Next
              <ArrowRight size={14} />
            </button>
          </div>
        </div>
      )}

      {/* ── Step 4: Let's Go ──────────────────────────────────────────────── */}
      {step === 4 && (
        <div className="atlas-enter" style={S.content}>
          <div style={S.heroIcon} aria-hidden="true">
            <Zap size={36} />
          </div>
          <h1 style={S.title}>Let's go.</h1>
          <p style={S.body}>
            Your library is waiting. Track sessions, write journal entries,
            build collections, and own your gaming history.
          </p>
          <button
            id="onboarding-finish-btn"
            onClick={handleFinish}
            style={S.primaryBtn}
            aria-label="Enter Pirate Harbor"
          >
            Enter Pirate Harbor
            <ArrowRight size={14} aria-hidden="true" />
          </button>
        </div>
      )}
    </div>
  );
}

// ── Styles ────────────────────────────────────────────────────────────────────

const S = {
  page: {
    display:        "flex",
    flexDirection:  "column" as const,
    alignItems:     "center",
    justifyContent: "center",
    minHeight:      "100vh",
    background:     "var(--color-base)",
    padding:        "40px 24px",
  },
  stepNav: {
    display:      "flex",
    gap:          10,
    marginBottom: 56,
  },
  stepDot: {
    width:          10,
    height:         10,
    borderRadius:   "50%",
    background:     "var(--color-elevated)",
    border:         "1px solid var(--color-border)",
    display:        "flex",
    alignItems:     "center",
    justifyContent: "center",
    transition:     "background 200ms, border-color 200ms",
  },
  stepDotActive: {
    background:  "var(--color-text-primary)",
    borderColor: "var(--color-text-primary)",
  },
  stepDotDone: {
    background:  "var(--color-text-secondary)",
    borderColor: "var(--color-text-secondary)",
    color:       "var(--color-base)",
  },
  content: {
    display:       "flex",
    flexDirection: "column" as const,
    alignItems:    "center",
    textAlign:     "center" as const,
    maxWidth:      520,
    width:         "100%",
  },
  heroIcon: {
    width:          64,
    height:         64,
    borderRadius:   "50%",
    background:     "var(--color-elevated)",
    border:         "1px solid var(--color-border)",
    display:        "flex",
    alignItems:     "center",
    justifyContent: "center",
    marginBottom:   28,
    color:          "var(--color-text-secondary)",
  },
  eyebrow: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.12em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    marginBottom:  16,
  },
  title: {
    fontFamily:    "var(--font-display)",
    fontSize:      "clamp(36px, 5vw, 64px)",
    fontWeight:    700,
    letterSpacing: "-0.03em",
    lineHeight:    1.05,
    color:         "var(--color-text-primary)",
    margin:        "0 0 20px",
  },
  body: {
    fontFamily:  "var(--font-body)",
    fontSize:    15,
    lineHeight:  1.65,
    color:       "var(--color-text-muted)",
    maxWidth:    420,
    margin:      "0 0 32px",
  },
  primaryBtn: {
    display:       "inline-flex",
    alignItems:    "center",
    gap:           8,
    background:    "var(--color-text-primary)",
    border:        "none",
    borderRadius:  1,
    padding:       "11px 26px",
    fontSize:      13,
    fontFamily:    "var(--font-mono)",
    fontWeight:    500,
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    color:         "var(--color-base)",
    cursor:        "pointer",
    transition:    "opacity 150ms",
  },
  ghostBtn: {
    display:      "inline-flex",
    alignItems:   "center",
    gap:          6,
    background:   "none",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "9px 18px",
    fontSize:     12,
    fontFamily:   "var(--font-body)",
    color:        "var(--color-text-muted)",
    cursor:       "pointer",
    transition:   "color 150ms",
  },
  skipBtn: {
    background:   "none",
    border:       "none",
    padding:      "9px 14px",
    fontSize:     12,
    fontFamily:   "var(--font-body)",
    color:        "var(--color-text-disabled)",
    cursor:       "pointer",
    transition:   "color 150ms",
  },
  btnRow: {
    display:        "flex",
    justifyContent: "space-between" as const,
    alignItems:     "center",
    width:          "100%",
    marginTop:      32,
    gap:            12,
  },

  // ── Step 1: RAWG ──────────────────────────────────────────────────────────
  keyRow: {
    display:        "flex",
    gap:            8,
    width:          "100%",
    maxWidth:       440,
    marginBottom:   8,
  },
  keyInput: {
    flex:         1,
    background:   "var(--color-elevated)",
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    padding:      "9px 14px",
    fontSize:     13,
    fontFamily:   "var(--font-mono)",
    color:        "var(--color-text-primary)",
    outline:      "none",
    letterSpacing: "0.04em",
    transition:   "border-color 150ms",
  },
  keyInputValid: {
    borderColor: "var(--color-text-secondary)",
  },
  keyInputInvalid: {
    borderColor: "hsl(0 70% 55%)",
  },
  testBtn: {
    flexShrink:  0,
    background:  "var(--color-elevated)",
    border:      "1px solid var(--color-border)",
    borderRadius: 1,
    padding:     "9px 18px",
    fontSize:    12,
    fontFamily:  "var(--font-mono)",
    color:       "var(--color-text-muted)",
    cursor:      "pointer",
    transition:  "opacity 150ms",
  },
  successMsg: {
    display:     "inline-flex",
    alignItems:  "center",
    fontFamily:  "var(--font-body)",
    fontSize:    12,
    color:       "var(--color-text-secondary)",
    marginBottom: 8,
  },
  errorMsg: {
    fontFamily:  "var(--font-body)",
    fontSize:    12,
    color:       "hsl(0 70% 60%)",
    maxWidth:    440,
    textAlign:   "left" as const,
    marginBottom: 8,
  },
  linkHint: {
    fontFamily:  "var(--font-body)",
    fontSize:    12,
    color:       "var(--color-text-disabled)",
    marginBottom: 0,
  },
  link: {
    color:          "var(--color-text-muted)",
    textDecoration: "underline",
  },

  // ── Step 2: Scan ──────────────────────────────────────────────────────────
  scanBox: {
    display:       "flex",
    flexDirection: "column" as const,
    alignItems:    "stretch",
    width:         "100%",
    maxWidth:      440,
    gap:           0,
  },
  scanPreview: {
    marginTop:    16,
    border:       "1px solid var(--color-border)",
    borderRadius: 1,
    overflow:     "hidden",
  },
  scanPreviewTitle: {
    fontFamily:  "var(--font-body)",
    fontSize:    12,
    color:       "var(--color-text-muted)",
    padding:     "10px 14px",
    borderBottom: "1px solid var(--color-border)",
    margin:      0,
    textAlign:   "left" as const,
  },
  scanList: {
    display:       "flex",
    flexDirection: "column" as const,
    maxHeight:     220,
    overflowY:     "auto" as const,
  },
  scanRow: {
    display:     "flex",
    alignItems:  "center",
    gap:         12,
    padding:     "7px 14px",
    borderBottom: "1px solid var(--color-border-sub)",
  },
  scanConf: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.08em",
    flexShrink:    0,
    width:         32,
    textAlign:     "right" as const,
  },
  scanTitle: {
    fontFamily:   "var(--font-body)",
    fontSize:     12,
    color:        "var(--color-text-primary)",
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
    textAlign:    "left" as const,
  },
  scanMore: {
    fontFamily:  "var(--font-mono)",
    fontSize:    10,
    color:       "var(--color-text-disabled)",
    padding:     "6px 14px",
    margin:      0,
    textAlign:   "left" as const,
  },

  // ── Step 3: Stats ─────────────────────────────────────────────────────────
  statGrid: {
    display:             "grid",
    gridTemplateColumns: "1fr 1fr",
    gap:                 12,
    width:               "100%",
    maxWidth:            380,
    marginBottom:        8,
  },
  statCard: {
    display:        "flex",
    flexDirection:  "column" as const,
    alignItems:     "center",
    padding:        "20px 16px",
    border:         "1px solid var(--color-border)",
    borderRadius:   1,
    background:     "var(--color-surface)",
    gap:            6,
  },
  statNum: {
    fontFamily:    "var(--font-display)",
    fontSize:      42,
    fontWeight:    700,
    letterSpacing: "-0.03em",
    color:         "var(--color-text-primary)",
    lineHeight:    1,
  },
  statLabel: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    letterSpacing: "0.10em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
  },
} satisfies Record<string, React.CSSProperties>;

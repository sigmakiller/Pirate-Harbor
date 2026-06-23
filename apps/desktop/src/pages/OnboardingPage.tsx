/**
 * OnboardingPage — first-run experience.
 *
 * Design spec: Design/Pages/onboarding.md
 * Steps: Welcome → How it works (manual add, Phase 1B scanner coming) → Finish
 *
 * Note: Folder scanning is Phase 1B (T10). This onboarding
 * explains the manual-add workflow for Phase 1 MVP.
 */

import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { ArrowRight, Check } from "lucide-react";

import { setSetting } from "@/lib/api";

type Step = 0 | 1 | 2;

const STEPS = [
  { label: "Welcome",      id: "welcome"  },
  { label: "How it works", id: "howitworks" },
  { label: "Ready",        id: "ready"    },
] as const;

export default function OnboardingPage() {
  const navigate  = useNavigate();
  const [step, setStep] = useState<Step>(0);

  const handleFinish = async () => {
    // Mark onboarding complete — persists via SQLite settings
    await setSetting("onboarding_complete", "true");
    navigate("/library");
  };

  return (
    <div style={styles.page}>
      {/* Step indicator */}
      <nav style={styles.stepNav} aria-label="Onboarding steps">
        {STEPS.map((s, i) => (
          <div
            key={s.id}
            style={{
              ...styles.stepDot,
              ...(i <= step ? styles.stepDotActive : {}),
            }}
            aria-current={i === step ? "step" : undefined}
            aria-label={`Step ${i + 1}: ${s.label}${i < step ? " (complete)" : ""}`}
          />
        ))}
      </nav>

      {/* ── Step 0: Welcome ─────────────────────────────────────────────── */}
      {step === 0 && (
        <div className="atlas-enter" style={styles.content}>
          <span style={styles.eyebrow}>Pirate Harbor</span>
          <h1 style={styles.title}>
            A personal OS for<br />gaming history.
          </h1>
          <p style={styles.body}>
            Track every game you play. Preserve your history.
            No accounts, no cloud, no noise — just you and your library.
          </p>
          <button
            id="onboarding-next-btn"
            onClick={() => setStep(1)}
            style={styles.nextBtn}
            aria-label="Continue to how it works"
          >
            Get started
            <ArrowRight size={14} aria-hidden="true" />
          </button>
        </div>
      )}

      {/* ── Step 1: How it works ─────────────────────────────────────────── */}
      {step === 1 && (
        <div className="atlas-enter" style={styles.content}>
          <span style={styles.eyebrow}>How it works</span>
          <h1 style={styles.title}>Manual, precise,<br />intentional.</h1>

          <div style={styles.featureList}>
            <Feature
              title="Add games manually"
              body="Browse to any .exe file and add it to your library in seconds."
            />
            <Feature
              title="Automatic playtime"
              body="When you launch a game through Pirate Harbor, every minute is recorded."
            />
            <Feature
              title="Ambient immersion"
              body="Game detail pages extract color from your cover art for subtle atmosphere."
            />
            <Feature
              title="Folder scanning — coming soon"
              body="Phase 2 will auto-detect installed games from your drives."
            />
          </div>

          <button
            id="onboarding-continue-btn"
            onClick={() => setStep(2)}
            style={styles.nextBtn}
            aria-label="Continue to finish"
          >
            Continue
            <ArrowRight size={14} aria-hidden="true" />
          </button>
        </div>
      )}

      {/* ── Step 2: Ready ────────────────────────────────────────────────── */}
      {step === 2 && (
        <div className="atlas-enter" style={styles.content}>
          <div style={styles.checkCircle} aria-hidden="true">
            <Check size={24} />
          </div>
          <span style={styles.eyebrow}>You're ready</span>
          <h1 style={styles.title}>Start your archive.</h1>
          <p style={styles.body}>
            Head to your library and add your first game.
            Everything you play through Pirate Harbor will be preserved here.
          </p>
          <button
            id="onboarding-finish-btn"
            onClick={handleFinish}
            style={styles.nextBtn}
            aria-label="Go to your library"
          >
            Go to Library
            <ArrowRight size={14} aria-hidden="true" />
          </button>
        </div>
      )}
    </div>
  );
}

// ── Sub-components ────────────────────────────────────────────────────────────

function Feature({ title, body }: { title: string; body: string }) {
  return (
    <div style={featureStyles.item}>
      <p style={featureStyles.title}>{title}</p>
      <p style={featureStyles.body}>{body}</p>
    </div>
  );
}

const featureStyles = {
  item: {
    paddingBottom: 20,
    borderBottom:  "1px solid var(--color-border)",
    display:       "flex",
    flexDirection: "column" as const,
    gap:           4,
  },
  title: {
    fontFamily: "var(--font-body)",
    fontSize:   14,
    fontWeight: 500,
    color:      "var(--color-text-primary)",
    margin:     0,
  },
  body: {
    fontFamily: "var(--font-body)",
    fontSize:   13,
    color:      "var(--color-text-muted)",
    margin:     0,
    lineHeight: 1.6,
  },
};

// ── Styles ────────────────────────────────────────────────────────────────────

const styles = {
  page: {
    display:        "flex",
    flexDirection:  "column" as const,
    alignItems:     "center",
    justifyContent: "center",
    height:         "100%",
    padding:        "40px 56px",
    boxSizing:      "border-box" as const,
    position:       "relative" as const,
  },
  stepNav: {
    position:  "absolute" as const,
    top:       40,
    left:      "50%",
    transform: "translateX(-50%)",
    display:   "flex",
    gap:       8,
  },
  stepDot: {
    width:        6,
    height:       6,
    borderRadius: "50%",
    background:   "var(--color-elevated)",
    border:       "1px solid var(--color-border)",
    transition:   "background 220ms",
  },
  stepDotActive: {
    background: "var(--color-text-muted)",
    border:     "1px solid var(--color-text-disabled)",
  },
  content: {
    maxWidth: 520,
    width:    "100%",
    display:  "flex",
    flexDirection: "column" as const,
    gap:      0,
  },
  eyebrow: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.16em",
    textTransform: "uppercase" as const,
    color:         "var(--color-text-disabled)",
    display:       "block",
    marginBottom:  20,
  },
  title: {
    fontFamily:    "var(--font-display)",
    fontSize:      "clamp(36px, 4vw, 64px)",
    fontWeight:    700,
    letterSpacing: "-0.03em",
    lineHeight:    1.05,
    color:         "var(--color-text-primary)",
    margin:        0,
    marginBottom:  20,
  },
  body: {
    fontFamily:   "var(--font-body)",
    fontSize:     16,
    color:        "var(--color-text-muted)",
    margin:       0,
    lineHeight:   1.65,
    marginBottom: 40,
  },
  featureList: {
    display:       "flex",
    flexDirection: "column" as const,
    gap:           20,
    marginBottom:  40,
  },
  nextBtn: {
    display:       "flex",
    alignItems:    "center",
    gap:           8,
    background:    "var(--color-text-primary)",
    color:         "var(--color-base)",
    border:        "none",
    padding:       "12px 28px",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    fontWeight:    600,
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    cursor:        "pointer",
    borderRadius:  1,
    transition:    "opacity 150ms",
    alignSelf:     "flex-start" as const,
  },
  checkCircle: {
    width:          56,
    height:         56,
    borderRadius:   "50%",
    border:         "1px solid var(--color-border)",
    display:        "flex",
    alignItems:     "center",
    justifyContent: "center",
    color:          "var(--color-text-primary)",
    marginBottom:   24,
  },
} satisfies Record<string, React.CSSProperties>;

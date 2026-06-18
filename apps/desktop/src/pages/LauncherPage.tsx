export default function LauncherPage() {
  return (
    <div className="atlas-enter" style={{ padding: "48px 56px" }}>
      <h1
        style={{
          fontFamily: "var(--font-display)",
          fontSize: "clamp(56px, 7vw, 96px)",
          fontWeight: 700,
          letterSpacing: "-0.03em",
          lineHeight: 1.0,
          color: "var(--color-text-primary)",
          margin: 0,
          marginBottom: 16,
        }}
      >
        Launcher
      </h1>
      <p style={{ fontSize: 16, color: "var(--color-text-muted)", margin: 0 }}>
        Your current journey. Continue where you left off.
      </p>
    </div>
  );
}

export default function LibraryPage() {
  return (
    <div className="atlas-enter" style={{ padding: "48px 56px" }}>
      {/* Editorial title */}
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
        Library
      </h1>
      <p
        style={{
          fontSize: 16,
          color: "var(--color-text-muted)",
          margin: 0,
          marginBottom: 64,
        }}
      >
        Your complete gaming history.
      </p>

      {/* Empty state */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          padding: "96px 0",
          gap: 16,
          borderTop: "1px solid var(--color-border)",
        }}
      >
        <span
          style={{
            fontSize: 13,
            color: "var(--color-text-muted)",
            letterSpacing: "0.1em",
            textTransform: "uppercase",
            fontFamily: "var(--font-mono)",
          }}
        >
          No games yet
        </span>
        <p
          style={{
            fontSize: 14,
            color: "var(--color-text-muted)",
            margin: 0,
            textAlign: "center",
            maxWidth: 320,
            lineHeight: 1.6,
          }}
        >
          Add your first game to begin preserving your gaming history.
        </p>
        <button
          id="add-first-game-btn"
          style={{
            marginTop: 8,
            padding: "10px 24px",
            background: "var(--color-text-primary)",
            color: "var(--color-base)",
            border: "none",
            borderRadius: "var(--radius-md)",
            fontSize: 13,
            fontWeight: 600,
            fontFamily: "var(--font-sans)",
            cursor: "pointer",
            letterSpacing: "0.02em",
            transition: "opacity var(--duration-fast) var(--ease-default)",
          }}
          onMouseEnter={(e) => ((e.currentTarget as HTMLButtonElement).style.opacity = "0.85")}
          onMouseLeave={(e) => ((e.currentTarget as HTMLButtonElement).style.opacity = "1")}
        >
          Add Game
        </button>
      </div>
    </div>
  );
}

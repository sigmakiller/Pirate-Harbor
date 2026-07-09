/**
 * GhostCard — pulsing skeleton placeholder matching GameCard dimensions.
 * GhostGrid — renders N ghost cards in the same grid layout as LibraryPage.
 *
 * T36 UX Polish: replace "Loading…" text with skeleton screens.
 * The pulse animation respects prefers-reduced-motion via CSS.
 */

/** Single ghost card matching GameCard (3:4 cover + title + meta row). */
export function GhostCard() {
  return (
    <div className="ghost-card" aria-hidden="true">
      <div className="skeleton ghost-cover" />
      <div className="skeleton ghost-title" />
      <div className="skeleton ghost-meta" />
    </div>
  );
}

/** Generic skeleton line — useful inside list views. */
export function SkeletonLine({ width = "100%", height = 14, style }: {
  width?: string | number;
  height?: number;
  style?: React.CSSProperties;
}) {
  return (
    <div
      className="skeleton"
      style={{ width, height, borderRadius: 3, ...style }}
      aria-hidden="true"
    />
  );
}

/** Full ghost grid — drop-in replacement for LibraryPage loading state. */
export function GhostGrid({ count = 12 }: { count?: number }) {
  return (
    <div
      style={{
        display:             "grid",
        gridTemplateColumns: "repeat(auto-fill, minmax(160px, 1fr))",
        gap:                 24,
        padding:             "40px 32px",
      }}
      aria-busy="true"
      aria-label="Loading library…"
      role="status"
    >
      {Array.from({ length: count }, (_, i) => (
        <GhostCard key={i} />
      ))}
    </div>
  );
}

/**
 * GhostList — skeleton rows for list-style pages (Milestones, Journal, Identity).
 * Each row is a wide rectangle with a smaller label.
 */
export function GhostList({ count = 8 }: { count?: number }) {
  return (
    <div
      style={{ display: "flex", flexDirection: "column", gap: 12, padding: "32px 56px" }}
      aria-busy="true"
      aria-label="Loading…"
      role="status"
    >
      {Array.from({ length: count }, (_, i) => (
        <div key={i} style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          <SkeletonLine width={`${60 + (i % 3) * 12}%`} height={16} />
          <SkeletonLine width={`${30 + (i % 4) * 8}%`} height={11} />
        </div>
      ))}
    </div>
  );
}

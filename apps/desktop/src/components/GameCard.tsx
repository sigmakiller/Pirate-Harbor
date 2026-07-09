/**
 * GameCard — cover-art card for the library grid view.
 *
 * Atlas OS rules:
 * - Monochrome UI only — no color applied by card itself
 * - Hover: scale 0.98→1 (allowed), no glow/shadow (forbidden)
 * - 300ms transition on cover reveal
 */

import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Star } from "lucide-react";

import { toggleFavorite } from "@/lib/api";
import { formatPlaytime } from "@/lib/utils";
import type { Game } from "@/types";

interface GameCardProps {
  game:     Game;
  onUpdate: (updated: Game) => void;
}

export function GameCard({ game, onUpdate }: GameCardProps) {
  const navigate = useNavigate();
  const [hovered,  setHovered]  = useState(false);
  const [togglingFav, setTogglingFav] = useState(false);

  const handleFavorite = async (e: React.MouseEvent) => {
    e.stopPropagation(); // Don't navigate
    if (togglingFav) return;
    setTogglingFav(true);
    try {
      const updated = await toggleFavorite(game.id);
      onUpdate(updated);
    } finally {
      setTogglingFav(false);
    }
  };

  const coverSrc = game.cover_path
    ? `https://asset.localhost/${encodeURIComponent(game.cover_path.replace(/\\/g, "/"))}`
    : null;

  return (
    <article
      tabIndex={0}
      onClick={() => navigate(`/library/${game.id}`)}
      onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); navigate(`/library/${game.id}`); } }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      aria-label={`${game.title}${game.is_favorite ? ", favorited" : ""}`}
      style={{
        ...styles.card,
        transform: hovered ? "scale(1)" : "scale(0.98)",
        cursor: "pointer",
      }}
    >
      {/* Cover image */}
      <div style={styles.coverWrapper}>
        {coverSrc ? (
          <img
            src={coverSrc}
            alt={`${game.title} cover`}
            style={styles.cover}
            draggable={false}
          />
        ) : (
          <div style={styles.coverPlaceholder}>
            <span style={styles.placeholderInitial}>
              {game.title.charAt(0).toUpperCase()}
            </span>
          </div>
        )}

        {/* Overlay: appears on hover */}
        <div
          style={{
            ...styles.overlay,
            opacity: hovered ? 1 : 0,
          }}
        >
          <span style={styles.playHint}>VIEW</span>
        </div>

        {/* Favorite button */}
        <button
          onClick={handleFavorite}
          style={{
            ...styles.favBtn,
            color: game.is_favorite
              ? "var(--color-text-primary)"
              : "var(--color-text-disabled)",
            opacity: hovered || game.is_favorite ? 1 : 0,
          }}
          aria-label={game.is_favorite ? "Remove from favorites" : "Add to favorites"}
        >
          <Star
            size={14}
            fill={game.is_favorite ? "currentColor" : "none"}
            strokeWidth={1.5}
          />
        </button>
      </div>

      {/* Meta */}
      <div style={styles.meta}>
        <p style={styles.title} title={game.title}>
          {game.title}
        </p>
        <div style={styles.metaRow}>
          <span style={styles.playtime}>
            {formatPlaytime(game.total_playtime_secs)}
          </span>
          <span style={styles.status}>
            {game.status}
          </span>
        </div>
      </div>
    </article>
  );
}

const styles = {
  card: {
    display:    "flex",
    flexDirection: "column" as const,
    transition: "transform 220ms ease",
    userSelect: "none" as const,
  },
  coverWrapper: {
    position:     "relative" as const,
    aspectRatio:  "3 / 4",
    overflow:     "hidden",
    background:   "var(--color-surface)",
    borderRadius: 1,
  },
  cover: {
    width:      "100%",
    height:     "100%",
    objectFit:  "cover" as const,
    display:    "block",
    transition: "opacity 300ms ease",
  },
  coverPlaceholder: {
    width:          "100%",
    height:         "100%",
    display:        "flex",
    alignItems:     "center",
    justifyContent: "center",
    background:     "var(--color-elevated)",
  },
  placeholderInitial: {
    fontFamily:    "var(--font-display)",
    fontSize:      48,
    fontWeight:    700,
    color:         "var(--color-text-disabled)",
    letterSpacing: "-0.02em",
  },
  overlay: {
    position:       "absolute" as const,
    inset:          0,
    background:     "rgba(5, 5, 5, 0.55)",
    display:        "flex",
    alignItems:     "center",
    justifyContent: "center",
    transition:     "opacity 150ms ease",
  },
  playHint: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    letterSpacing: "0.16em",
    color:         "var(--color-text-muted)",
  },
  favBtn: {
    position:    "absolute" as const,
    top:         8,
    right:       8,
    background:  "none",
    border:      "none",
    cursor:      "pointer",
    padding:     4,
    display:     "flex",
    alignItems:  "center",
    transition:  "opacity 150ms ease, color 150ms ease",
    lineHeight:  1,
  },
  meta: {
    padding:    "10px 0 4px",
    display:    "flex",
    flexDirection: "column" as const,
    gap:        4,
  },
  title: {
    fontFamily:    "var(--font-body)",
    fontSize:      13,
    fontWeight:    500,
    color:         "var(--color-text-primary)",
    margin:        0,
    overflow:      "hidden",
    textOverflow:  "ellipsis",
    whiteSpace:    "nowrap" as const,
    letterSpacing: "0.01em",
  },
  metaRow: {
    display:        "flex",
    justifyContent: "space-between",
    alignItems:     "center",
  },
  playtime: {
    fontFamily: "var(--font-mono)",
    fontSize:   11,
    color:      "var(--color-text-disabled)",
    letterSpacing: "0.04em",
  },
  status: {
    fontFamily:    "var(--font-mono)",
    fontSize:      10,
    color:         "var(--color-text-disabled)",
    letterSpacing: "0.08em",
    textTransform: "uppercase" as const,
  },
} satisfies Record<string, React.CSSProperties>;

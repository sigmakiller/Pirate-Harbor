/**
 * GameListRow — table-row alternative for library list view.
 */

import { useNavigate } from "react-router-dom";
import { Star } from "lucide-react";

import { toggleFavorite } from "@/lib/api";
import { formatPlaytime, formatRelativeDate } from "@/lib/utils";
import { STATUS_LABELS } from "@/lib/utils";
import type { Game } from "@/types";

interface GameListRowProps {
  game:     Game;
  onUpdate: (updated: Game) => void;
}

export function GameListRow({ game, onUpdate }: GameListRowProps) {
  const navigate = useNavigate();

  const handleFavorite = async (e: React.MouseEvent) => {
    e.stopPropagation();
    const updated = await toggleFavorite(game.id);
    onUpdate(updated);
  };

  return (
    <tr
      onClick={() => navigate(`/library/${game.id}`)}
      style={styles.row}
      aria-label={game.title}
    >
      {/* Thumbnail */}
      <td style={styles.thumbCell}>
        {game.cover_path ? (
          <img
            src={`https://asset.localhost/${encodeURIComponent(game.cover_path.replace(/\\/g, "/"))}`}
            alt=""
            style={styles.thumb}
            draggable={false}
          />
        ) : (
          <div style={styles.thumbPlaceholder}>
            {game.title.charAt(0).toUpperCase()}
          </div>
        )}
      </td>

      {/* Title */}
      <td style={styles.titleCell}>
        <span style={styles.title}>{game.title}</span>
        {game.genre && (
          <span style={styles.genre}>{game.genre}</span>
        )}
      </td>

      {/* Status */}
      <td style={styles.cell}>
        <span style={styles.mono}>{STATUS_LABELS[game.status]}</span>
      </td>

      {/* Playtime */}
      <td style={styles.cell}>
        <span style={styles.mono}>{formatPlaytime(game.total_playtime_secs)}</span>
      </td>

      {/* Last played */}
      <td style={styles.cell}>
        <span style={styles.muted}>
          {game.last_played ? formatRelativeDate(game.last_played) : "—"}
        </span>
      </td>

      {/* Favorite */}
      <td style={styles.actionCell}>
        <button
          onClick={handleFavorite}
          style={{
            ...styles.favBtn,
            color: game.is_favorite
              ? "var(--color-text-primary)"
              : "var(--color-text-disabled)",
          }}
          aria-label={game.is_favorite ? "Remove from favorites" : "Add to favorites"}
        >
          <Star
            size={13}
            fill={game.is_favorite ? "currentColor" : "none"}
            strokeWidth={1.5}
          />
        </button>
      </td>
    </tr>
  );
}

const styles = {
  row: {
    cursor:         "pointer",
    borderBottom:   "1px solid var(--color-border)",
    transition:     "background 150ms ease",
  } as React.CSSProperties,
  thumbCell: {
    padding:  "8px 16px 8px 0",
    width:    40,
  } as React.CSSProperties,
  thumb: {
    width:        36,
    height:       48,
    objectFit:    "cover" as const,
    display:      "block",
    borderRadius: 1,
    flexShrink:   0,
  } as React.CSSProperties,
  thumbPlaceholder: {
    width:          36,
    height:         48,
    display:        "flex",
    alignItems:     "center",
    justifyContent: "center",
    background:     "var(--color-elevated)",
    borderRadius:   1,
    fontFamily:     "var(--font-display)",
    fontSize:       16,
    fontWeight:     700,
    color:          "var(--color-text-disabled)",
  } as React.CSSProperties,
  titleCell: {
    padding:    "8px 24px 8px 0",
    verticalAlign: "middle" as const,
    display:    "flex",
    flexDirection: "column" as const,
    gap:        2,
    minWidth:   0,
  } as React.CSSProperties,
  title: {
    fontFamily:   "var(--font-body)",
    fontSize:     13,
    fontWeight:   500,
    color:        "var(--color-text-primary)",
    overflow:     "hidden",
    textOverflow: "ellipsis",
    whiteSpace:   "nowrap" as const,
  } as React.CSSProperties,
  genre: {
    fontFamily:    "var(--font-mono)",
    fontSize:      11,
    color:         "var(--color-text-disabled)",
    letterSpacing: "0.04em",
  } as React.CSSProperties,
  cell: {
    padding:       "8px 24px 8px 0",
    verticalAlign: "middle" as const,
    whiteSpace:    "nowrap" as const,
  } as React.CSSProperties,
  actionCell: {
    padding:       "8px 0",
    verticalAlign: "middle" as const,
    textAlign:     "right" as const,
    width:         40,
  } as React.CSSProperties,
  mono: {
    fontFamily:    "var(--font-mono)",
    fontSize:      12,
    color:         "var(--color-text-secondary)",
    letterSpacing: "0.04em",
  } as React.CSSProperties,
  muted: {
    fontFamily: "var(--font-body)",
    fontSize:   13,
    color:      "var(--color-text-muted)",
  } as React.CSSProperties,
  favBtn: {
    background: "none",
    border:     "none",
    cursor:     "pointer",
    padding:    4,
    display:    "flex",
    transition: "color 150ms ease",
  } as React.CSSProperties,
};

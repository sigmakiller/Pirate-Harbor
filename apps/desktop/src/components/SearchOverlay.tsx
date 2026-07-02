/**
 * SearchOverlay — T29
 *
 * A Cmd+K / Ctrl+K modal for global full-text search across games, journal
 * entries, and milestones.  Results are fetched from the `search_global`
 * Tauri command and grouped by category.
 *
 * Design:
 * - Dark glass-morphism overlay, 640px wide, centred
 * - Instant prefix search (debounced 200ms)
 * - Keyboard navigation: ↑/↓ within results, Enter to navigate, Esc to close
 */

import {
  useState,
  useEffect,
  useRef,
  useCallback,
  type KeyboardEvent,
} from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

// ── Types ─────────────────────────────────────────────────────────────────────

interface GameSearchHit {
  id: string;
  title: string;
  developer: string | null;
  genre: string | null;
  status: string;
  cover_path: string | null;
}

interface JournalSearchHit {
  id: string;
  title: string | null;
  body: string;
  entry_type: string;
  game_id: string | null;
  game_title: string | null;
  created_at: string;
}

interface MilestoneSearchHit {
  id: string;
  title: string;
  game_id: string;
  game_title: string | null;
  category: string;
}

interface SearchResults {
  games: GameSearchHit[];
  journal_entries: JournalSearchHit[];
  milestones: MilestoneSearchHit[];
  total: number;
}

// ── Flat result for keyboard navigation ───────────────────────────────────────

type NavItem =
  | { kind: "game";     data: GameSearchHit }
  | { kind: "journal";  data: JournalSearchHit }
  | { kind: "milestone"; data: MilestoneSearchHit };

// ── Component ─────────────────────────────────────────────────────────────────

interface SearchOverlayProps {
  onClose: () => void;
}

export default function SearchOverlay({ onClose }: SearchOverlayProps) {
  const navigate = useNavigate();
  const inputRef  = useRef<HTMLInputElement>(null);

  const [query,    setQuery]   = useState("");
  const [results,  setResults] = useState<SearchResults | null>(null);
  const [loading,  setLoading] = useState(false);
  const [cursor,   setCursor]  = useState(0); // index into flat nav list

  // ── Search ─────────────────────────────────────────────────────────────────

  const doSearch = useCallback(async (q: string) => {
    if (!q.trim()) { setResults(null); return; }
    setLoading(true);
    try {
      const res = await invoke<SearchResults>("search_global", {
        query: q,
        limit: 10,
      });
      setResults(res);
      setCursor(0);
    } catch {
      setResults(null);
    } finally {
      setLoading(false);
    }
  }, []);

  // Debounce 200ms
  useEffect(() => {
    const id = setTimeout(() => doSearch(query), 200);
    return () => clearTimeout(id);
  }, [query, doSearch]);

  // Focus input on mount
  useEffect(() => { inputRef.current?.focus(); }, []);

  // ── Close on backdrop click / Escape ───────────────────────────────────────

  const handleBackdrop = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) onClose();
  };

  // ── Keyboard navigation ────────────────────────────────────────────────────

  const navItems: NavItem[] = results
    ? [
        ...results.games.map((d): NavItem => ({ kind: "game", data: d })),
        ...results.journal_entries.map((d): NavItem => ({ kind: "journal", data: d })),
        ...results.milestones.map((d): NavItem => ({ kind: "milestone", data: d })),
      ]
    : [];

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Escape") { onClose(); return; }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setCursor((c) => Math.min(c + 1, navItems.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setCursor((c) => Math.max(c - 1, 0));
    } else if (e.key === "Enter" && navItems.length > 0) {
      activate(navItems[cursor]);
    }
  };

  const activate = (item: NavItem) => {
    onClose();
    switch (item.kind) {
      case "game":      navigate(`/library`); break;
      case "journal":   navigate(`/journal`); break;
      case "milestone": navigate(`/milestones`); break;
    }
  };

  // ── Render ─────────────────────────────────────────────────────────────────

  return (
    <div
      onClick={handleBackdrop}
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 9999,
        background: "rgba(0,0,0,0.65)",
        backdropFilter: "blur(6px)",
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "center",
        paddingTop: 120,
      }}
    >
      <div
        style={{
          width: 640,
          maxWidth: "calc(100vw - 32px)",
          background: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          borderRadius: 16,
          boxShadow: "0 24px 80px rgba(0,0,0,0.7)",
          overflow: "hidden",
        }}
      >
        {/* Search input */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 12,
            padding: "14px 20px",
            borderBottom: results?.total ? "1px solid var(--color-border)" : "none",
          }}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none"
            stroke="var(--color-text-secondary)" strokeWidth="2"
            strokeLinecap="round" strokeLinejoin="round"
            style={{ flexShrink: 0 }}>
            <circle cx="11" cy="11" r="8"/>
            <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>

          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search games, journal, milestones…"
            style={{
              flex: 1,
              background: "transparent",
              border: "none",
              outline: "none",
              color: "var(--color-text-primary)",
              fontSize: 16,
              fontFamily: "var(--font-sans)",
            }}
          />

          {loading && (
            <span style={{
              width: 14, height: 14, borderRadius: "50%",
              border: "2px solid var(--color-primary)",
              borderTopColor: "transparent",
              display: "inline-block",
              animation: "overlay-spin 0.8s linear infinite",
              flexShrink: 0,
            }}/>
          )}

          <kbd style={{
            fontSize: 11,
            color: "var(--color-text-secondary)",
            background: "var(--color-bg)",
            border: "1px solid var(--color-border)",
            borderRadius: 4,
            padding: "2px 6px",
            flexShrink: 0,
          }}>ESC</kbd>
        </div>

        {/* Results */}
        {results && results.total > 0 && (
          <div style={{ maxHeight: 460, overflowY: "auto" }}>
            {/* Games section */}
            {results.games.length > 0 && (
              <ResultSection
                label="Games"
                items={results.games.map((g): NavItem => ({ kind: "game", data: g }))}
                cursor={cursor}
                navOffset={0}
                onActivate={activate}
                onHover={setCursor}
                renderItem={(item, active) => (
                  <GameRow game={(item as { kind: "game"; data: GameSearchHit }).data} active={active}/>
                )}
              />
            )}

            {/* Journal section */}
            {results.journal_entries.length > 0 && (
              <ResultSection
                label="Journal"
                items={results.journal_entries.map((j): NavItem => ({ kind: "journal", data: j }))}
                cursor={cursor}
                navOffset={results.games.length}
                onActivate={activate}
                onHover={setCursor}
                renderItem={(item, active) => (
                  <JournalRow entry={(item as { kind: "journal"; data: JournalSearchHit }).data} active={active}/>
                )}
              />
            )}

            {/* Milestones section */}
            {results.milestones.length > 0 && (
              <ResultSection
                label="Milestones"
                items={results.milestones.map((m): NavItem => ({ kind: "milestone", data: m }))}
                cursor={cursor}
                navOffset={results.games.length + results.journal_entries.length}
                onActivate={activate}
                onHover={setCursor}
                renderItem={(item, active) => (
                  <MilestoneRow m={(item as { kind: "milestone"; data: MilestoneSearchHit }).data} active={active}/>
                )}
              />
            )}
          </div>
        )}

        {/* Empty state */}
        {results && results.total === 0 && query.trim() && (
          <div style={{
            padding: "32px 20px",
            textAlign: "center",
            color: "var(--color-text-secondary)",
            fontSize: 14,
          }}>
            No results for <em>"{query}"</em>
          </div>
        )}

        {/* Hint when empty */}
        {!results && !loading && (
          <div style={{
            padding: "20px 20px",
            color: "var(--color-text-secondary)",
            fontSize: 12,
            display: "flex",
            gap: 20,
          }}>
            <span>↑↓ navigate</span>
            <span>↵ open</span>
            <span>ESC close</span>
          </div>
        )}
      </div>

      <style>{`
        @keyframes overlay-spin { to { transform: rotate(360deg); } }
      `}</style>
    </div>
  );
}

// ── Sub-components ────────────────────────────────────────────────────────────

interface ResultSectionProps {
  label: string;
  items: NavItem[];
  cursor: number;
  navOffset: number;
  onActivate: (item: NavItem) => void;
  onHover: (idx: number) => void;
  renderItem: (item: NavItem, active: boolean) => React.ReactNode;
}

function ResultSection({
  label, items, cursor, navOffset, onActivate, onHover, renderItem,
}: ResultSectionProps) {
  return (
    <div>
      <div style={{
        padding: "8px 20px 4px",
        fontSize: 11,
        fontWeight: 600,
        letterSpacing: "0.1em",
        textTransform: "uppercase",
        color: "var(--color-text-secondary)",
        fontFamily: "var(--font-mono)",
      }}>
        {label}
      </div>
      {items.map((item, i) => {
        const globalIdx = navOffset + i;
        const active = cursor === globalIdx;
        return (
          <div
            key={globalIdx}
            onClick={() => onActivate(item)}
            onMouseEnter={() => onHover(globalIdx)}
            style={{
              padding: "10px 20px",
              cursor: "pointer",
              background: active ? "rgba(var(--color-primary-rgb, 99,102,241), 0.12)" : "transparent",
              borderLeft: active ? "2px solid var(--color-primary)" : "2px solid transparent",
              transition: "background 0.1s",
            }}
          >
            {renderItem(item, active)}
          </div>
        );
      })}
    </div>
  );
}

function GameRow({ game, active }: { game: GameSearchHit; active: boolean }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
      {game.cover_path ? (
        <img
          src={`asset://${game.cover_path}`}
          alt=""
          style={{ width: 32, height: 32, borderRadius: 4, objectFit: "cover", flexShrink: 0 }}
          onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }}
        />
      ) : (
        <div style={{
          width: 32, height: 32, borderRadius: 4,
          background: "var(--color-border)", flexShrink: 0,
          display: "flex", alignItems: "center", justifyContent: "center",
          fontSize: 14,
        }}>🎮</div>
      )}
      <div>
        <div style={{ fontSize: 14, color: active ? "var(--color-primary)" : "var(--color-text-primary)", fontWeight: 500 }}>
          {game.title}
        </div>
        {game.developer && (
          <div style={{ fontSize: 12, color: "var(--color-text-secondary)" }}>
            {game.developer}{game.genre ? ` · ${game.genre}` : ""}
          </div>
        )}
      </div>
    </div>
  );
}

function JournalRow({ entry, active }: { entry: JournalSearchHit; active: boolean }) {
  const preview = entry.body.length > 80 ? entry.body.slice(0, 80) + "…" : entry.body;
  return (
    <div>
      <div style={{ fontSize: 14, color: active ? "var(--color-primary)" : "var(--color-text-primary)", fontWeight: 500 }}>
        {entry.title ?? preview}
      </div>
      <div style={{ fontSize: 12, color: "var(--color-text-secondary)" }}>
        {entry.game_title ? `${entry.game_title} · ` : ""}{entry.entry_type}
      </div>
    </div>
  );
}

function MilestoneRow({ m, active }: { m: MilestoneSearchHit; active: boolean }) {
  return (
    <div>
      <div style={{ fontSize: 14, color: active ? "var(--color-primary)" : "var(--color-text-primary)", fontWeight: 500 }}>
        🏆 {m.title}
      </div>
      <div style={{ fontSize: 12, color: "var(--color-text-secondary)" }}>
        {m.game_title ? `${m.game_title} · ` : ""}{m.category}
      </div>
    </div>
  );
}

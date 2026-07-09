/**
 * useGridArrowNav — T36 Keyboard Navigation.
 *
 * Attaches arrow-key navigation to a CSS grid container so that
 * keyboard users can move through items with ←/→/↑/↓.
 *
 * Usage:
 *   const gridRef = useGridArrowNav<HTMLDivElement>(columns);
 *   <div ref={gridRef} …>
 *     {games.map(g => <article tabIndex={0} …>…</article>)}
 *   </div>
 *
 * Rules (Atlas OS):
 *   - Only activates when focus is inside the grid.
 *   - PageUp/PageDown jump one visual row up/down.
 *   - Home/End jump to first/last item.
 *   - Enter/Space activates the focused item (click).
 */

import { useCallback, useRef } from "react";

export function useGridArrowNav<T extends HTMLElement>(columns: number) {
  const ref = useRef<T>(null);

  const onKeyDown = useCallback(
    (e: KeyboardEvent) => {
      const container = ref.current;
      if (!container) return;

      const items = Array.from(
        container.querySelectorAll<HTMLElement>(
          "article[tabindex], button[tabindex='0'], [role='gridcell']"
        )
      );
      if (items.length === 0) return;

      const focused = document.activeElement as HTMLElement;
      const idx = items.indexOf(focused);
      if (idx === -1) return; // focus not inside grid

      let next = -1;
      switch (e.key) {
        case "ArrowRight": next = Math.min(idx + 1, items.length - 1); break;
        case "ArrowLeft":  next = Math.max(idx - 1, 0); break;
        case "ArrowDown":  next = Math.min(idx + columns, items.length - 1); break;
        case "ArrowUp":    next = Math.max(idx - columns, 0); break;
        case "Home":       next = 0; break;
        case "End":        next = items.length - 1; break;
        case "PageDown":   next = Math.min(idx + columns * 3, items.length - 1); break;
        case "PageUp":     next = Math.max(idx - columns * 3, 0); break;
        case "Enter":
        case " ":
          focused.click();
          e.preventDefault();
          return;
        default: return;
      }

      e.preventDefault();
      items[next]?.focus();
    },
    [columns]
  );

  const setRef = useCallback(
    (node: T | null) => {
      if (ref.current) {
        ref.current.removeEventListener("keydown", onKeyDown);
      }
      (ref as React.MutableRefObject<T | null>).current = node;
      if (node) {
        node.addEventListener("keydown", onKeyDown);
      }
    },
    [onKeyDown]
  );

  return setRef;
}

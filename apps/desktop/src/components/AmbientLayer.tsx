/**
 * AmbientLayer.tsx — Layer 2 of the Atlas OS three-layer stack.
 *
 * Renders ONLY on Game Detail pages (when coverPath is provided).
 * Sits between Layer 1 (pure black #050505 background) and
 * Layer 3 (the monochrome UI at z-index 2+).
 *
 * Contract:
 * - Never recolors sidebar, navigation, cards, or typography (z-index 2+)
 * - Fades in/out with a 300ms transition (layout duration per MOTION.md)
 * - Returns null when coverPath is null — non-detail pages see no overlay
 * - Opacity fixed at 12% (midpoint of 8–15% spec)
 */

import { useEffect, useRef, useState } from "react";
import {
  type AmbientColor,
  type AmbientConfig,
  extractDominantColor,
  generateAmbientStyle,
  generateClearStyle,
} from "@/engine/ambient";

interface AmbientLayerProps {
  /** Absolute path to the game's cover image. Pass null on non-detail pages. */
  coverPath: string | null;
  /** Optional override for ambient config (opacity, blur, transition). */
  config?: AmbientConfig;
}

export function AmbientLayer({ coverPath, config }: AmbientLayerProps) {
  const [ambientColor, setAmbientColor] = useState<AmbientColor | null>(null);
  const [visible, setVisible] = useState(false);
  const prevCoverPath = useRef<string | null>(null);

  useEffect(() => {
    // No cover → fade out and clear
    if (!coverPath) {
      setVisible(false);
      const t = setTimeout(() => setAmbientColor(null), 300);
      return () => clearTimeout(t);
    }

    // Same cover path — no re-extraction needed
    if (coverPath === prevCoverPath.current) return;
    prevCoverPath.current = coverPath;

    // Fade out before switching colors
    setVisible(false);

    // Convert Tauri asset path to a URL the browser can load
    // On Tauri, local file paths are served via the asset protocol
    const src = coverPath.startsWith("http")
      ? coverPath
      : `https://asset.localhost/${encodeURIComponent(coverPath.replace(/\\/g, "/"))}`;

    let cancelled = false;

    extractDominantColor(src).then((color) => {
      if (cancelled) return;
      if (color) {
        setAmbientColor(color);
        // Small delay to let the previous color fade out first
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            if (!cancelled) setVisible(true);
          });
        });
      }
    });

    return () => {
      cancelled = true;
    };
  }, [coverPath]);

  // Non-detail pages — render nothing
  if (!coverPath && !ambientColor) return null;

  const style = ambientColor && visible
    ? generateAmbientStyle(ambientColor, config)
    : generateClearStyle(config);

  return (
    <div
      aria-hidden="true"
      data-ambient-layer="true"
      style={style}
    />
  );
}

/**
 * ambient.ts — Atlas OS Ambient Engine
 *
 * Generates the contextual immersion layer (Layer 2) for Game Detail pages.
 *
 * Pipeline per Design/AMBIENT_SYSTEM.md:
 *   1. Extract dominant color from game cover artwork (canvas-based sampling)
 *   2. Desaturate by ~70%
 *   3. Darken by ~50%
 *   4. Produce CSS for a blurred radial gradient overlay at 8–15% opacity
 *
 * This module has zero side effects — all functions are pure transforms.
 * The React component (AmbientLayer.tsx) owns the DOM rendering.
 */

// ── Types ─────────────────────────────────────────────────────────────────────

/** Raw RGB sample from the image. */
interface RGB {
  r: number;
  g: number;
  b: number;
}

/** HSL representation for easier saturation/lightness manipulation. */
interface HSL {
  h: number; // 0–360
  s: number; // 0–100
  l: number; // 0–100
}

/** The processed ambient color ready for CSS. */
export interface AmbientColor {
  /** css rgb() string — the processed, desaturated, darkened color */
  css: string;
  /** Raw hex for debugging / testing */
  hex: string;
}

/**
 * Configuration for the ambient overlay style.
 * Callers can override defaults for testing or future design tweaks.
 */
export interface AmbientConfig {
  /** Overlay opacity (0–1). Design spec: 0.08–0.15 */
  opacity?: number;
  /** Blur radius in px applied to the gradient layer */
  blurPx?: number;
  /** CSS transition duration string (e.g. "300ms") */
  transitionDuration?: string;
}

const DEFAULTS: Required<AmbientConfig> = {
  opacity:            0.12,   // midpoint of 8–15% spec
  blurPx:             80,
  transitionDuration: "300ms", // layout duration per MOTION.md
};

// ── Color math ────────────────────────────────────────────────────────────────

function rgbToHsl({ r, g, b }: RGB): HSL {
  const rn = r / 255, gn = g / 255, bn = b / 255;
  const max = Math.max(rn, gn, bn);
  const min = Math.min(rn, gn, bn);
  const l   = (max + min) / 2;
  let h = 0, s = 0;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case rn: h = (gn - bn) / d + (gn < bn ? 6 : 0); break;
      case gn: h = (bn - rn) / d + 2; break;
      case bn: h = (rn - gn) / d + 4; break;
    }
    h /= 6;
  }

  return { h: h * 360, s: s * 100, l: l * 100 };
}

function hslToRgb({ h, s, l }: HSL): RGB {
  const sn = s / 100, ln = l / 100;

  if (s === 0) {
    const v = Math.round(ln * 255);
    return { r: v, g: v, b: v };
  }

  const hue2rgb = (p: number, q: number, t: number): number => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 1 / 2) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
  };

  const q = ln < 0.5 ? ln * (1 + sn) : ln + sn - ln * sn;
  const p = 2 * ln - q;
  const hn = h / 360;

  return {
    r: Math.round(hue2rgb(p, q, hn + 1 / 3) * 255),
    g: Math.round(hue2rgb(p, q, hn) * 255),
    b: Math.round(hue2rgb(p, q, hn - 1 / 3) * 255),
  };
}

/** Clamp a number between min and max. */
const clamp = (v: number, min: number, max: number) =>
  Math.min(Math.max(v, min), max);

/**
 * Desaturate and darken a color per the Atlas OS ambient pipeline.
 * - Desaturate: reduce saturation by 70%
 * - Darken: reduce lightness by 50%
 */
function processColor(rgb: RGB): RGB {
  const hsl = rgbToHsl(rgb);
  const processed: HSL = {
    h: hsl.h,
    s: clamp(hsl.s * 0.30, 0, 100),  // reduce to 30% of original saturation
    l: clamp(hsl.l * 0.50, 0, 100),  // reduce to 50% of original lightness
  };
  return hslToRgb(processed);
}

function toHex({ r, g, b }: RGB): string {
  return `#${r.toString(16).padStart(2, "0")}${g.toString(16).padStart(2, "0")}${b.toString(16).padStart(2, "0")}`;
}

// ── Dominant color extraction ─────────────────────────────────────────────────

/**
 * Extract the dominant color from an image URL using an offscreen canvas.
 *
 * Strategy: sample every Nth pixel from the image to compute a weighted
 * average. This is fast and produces a stable "mood" color rather than
 * the single most common pixel (which is often noise or near-black).
 *
 * Returns `null` if the image fails to load or CORS blocks canvas access.
 */
export async function extractDominantColor(
  imageSrc: string
): Promise<AmbientColor | null> {
  return new Promise((resolve) => {
    const img = new Image();
    img.crossOrigin = "anonymous";

    img.onload = () => {
      try {
        const canvas  = document.createElement("canvas");
        // Downscale to 64×64 for fast sampling — detail is irrelevant
        const SIZE    = 64;
        canvas.width  = SIZE;
        canvas.height = SIZE;

        const ctx = canvas.getContext("2d");
        if (!ctx) { resolve(null); return; }

        ctx.drawImage(img, 0, 0, SIZE, SIZE);
        const { data } = ctx.getImageData(0, 0, SIZE, SIZE);

        // Weighted average skipping near-black / near-white pixels
        let rSum = 0, gSum = 0, bSum = 0, count = 0;
        const STEP = 4 * 4; // sample every 4th pixel

        for (let i = 0; i < data.length; i += STEP) {
          const r = data[i], g = data[i + 1], b = data[i + 2], a = data[i + 3];
          if (a < 128) continue;                            // skip transparent
          const brightness = (r + g + b) / 3;
          if (brightness < 20 || brightness > 235) continue; // skip near-black/white
          rSum += r; gSum += g; bSum += b; count++;
        }

        if (count === 0) { resolve(null); return; }

        const dominant: RGB = {
          r: Math.round(rSum / count),
          g: Math.round(gSum / count),
          b: Math.round(bSum / count),
        };

        const processed = processColor(dominant);

        resolve({
          css: `rgb(${processed.r}, ${processed.g}, ${processed.b})`,
          hex: toHex(processed),
        });
      } catch {
        // Canvas tainted by CORS or other error
        resolve(null);
      }
    };

    img.onerror = () => resolve(null);
    img.src = imageSrc;
  });
}

// ── Style generation ──────────────────────────────────────────────────────────

/**
 * Generate the CSS properties for the ambient overlay div.
 *
 * Produces a radial gradient from the ambient color at the specified
 * opacity over a transparent background, with CSS blur applied.
 */
export function generateAmbientStyle(
  color: AmbientColor,
  config: AmbientConfig = {}
): React.CSSProperties {
  const { opacity, blurPx, transitionDuration } = {
    ...DEFAULTS,
    ...config,
  };

  return {
    position:        "fixed" as const,
    inset:           0,
    zIndex:          1,  // Layer 2: above #050505 bg, below UI content (z-index 2+)
    pointerEvents:   "none",
    backgroundImage: `radial-gradient(ellipse 120% 80% at 60% 30%, ${color.css} ${Math.round(opacity * 100)}%, transparent 70%)`,
    filter:          `blur(${blurPx}px)`,
    opacity:         1,
    transition:      `opacity ${transitionDuration} var(--ease-default)`,
  };
}

/**
 * Generate a "clearing" style — used to fade the ambient layer out
 * when navigating away from a Game Detail page.
 */
export function generateClearStyle(
  config: AmbientConfig = {}
): React.CSSProperties {
  const { transitionDuration } = { ...DEFAULTS, ...config };
  return {
    position:      "fixed" as const,
    inset:         0,
    zIndex:        1,
    pointerEvents: "none",
    opacity:       0,
    transition:    `opacity ${transitionDuration} var(--ease-default)`,
  };
}

/**
 * PageWrapper — T36 UX Polish.
 *
 * Wraps every page in an animated container that:
 *   1. Applies the `atlas-enter` animation (fade + translateY 4px→0 @ 220ms)
 *   2. Respects prefers-reduced-motion (animation is CSS-only, no JS required)
 *
 * Usage:
 *   export default function MyPage() {
 *     return <PageWrapper><…content…></PageWrapper>;
 *   }
 */

interface PageWrapperProps {
  children: React.ReactNode;
  /** Extra inline styles merged onto the wrapper div. */
  style?: React.CSSProperties;
  className?: string;
}

export default function PageWrapper({ children, style, className }: PageWrapperProps) {
  return (
    <div
      className={`atlas-enter${className ? ` ${className}` : ""}`}
      style={{ minHeight: "100%", ...style }}
    >
      {children}
    </div>
  );
}

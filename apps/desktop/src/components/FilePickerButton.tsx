/**
 * FilePickerButton — wraps Tauri's native file dialog.
 *
 * Props:
 *   - value: current path string
 *   - onChange: called with the selected path
 *   - filters: file type filters (e.g. [{ name: "Executables", extensions: ["exe"] }])
 *   - directory: if true, opens a folder picker instead of file picker
 *   - placeholder: displayed when no path is selected
 */

import { open } from "@tauri-apps/plugin-dialog";

interface FileFilter {
  name:       string;
  extensions: string[];
}

interface FilePickerButtonProps {
  value:        string;
  onChange:     (path: string) => void;
  filters?:     FileFilter[];
  directory?:   boolean;
  placeholder?: string;
  id?:          string;
}

export function FilePickerButton({
  value,
  onChange,
  filters,
  directory = false,
  placeholder = "Browse…",
  id,
}: FilePickerButtonProps) {
  const handleClick = async () => {
    const selected = await open({
      multiple:  false,
      directory,
      filters:   filters ?? [],
    });

    if (selected && typeof selected === "string") {
      onChange(selected);
    }
  };

  return (
    <div style={styles.wrapper}>
      {/* Path display */}
      <div
        style={{
          ...styles.pathDisplay,
          color: value ? "var(--color-text-primary)" : "var(--color-text-disabled)",
        }}
        title={value || placeholder}
      >
        {value || placeholder}
      </div>

      {/* Browse trigger */}
      <button
        id={id}
        type="button"
        onClick={handleClick}
        style={styles.browseBtn}
        aria-label="Browse for file"
      >
        Browse
      </button>
    </div>
  );
}

const styles = {
  wrapper: {
    display:       "flex",
    alignItems:    "stretch",
    border:        "1px solid var(--color-border)",
    borderRadius:  1,
    overflow:      "hidden",
  } as React.CSSProperties,

  pathDisplay: {
    flex:         1,
    padding:      "9px 12px",
    fontSize:     13,
    fontFamily:   "var(--font-mono)",
    whiteSpace:   "nowrap",
    overflow:     "hidden",
    textOverflow: "ellipsis",
    background:   "var(--color-surface)",
    minWidth:     0,
  } as React.CSSProperties,

  browseBtn: {
    flexShrink:    0,
    padding:       "9px 16px",
    background:    "var(--color-elevated)",
    border:        "none",
    borderLeft:    "1px solid var(--color-border)",
    color:         "var(--color-text-secondary)",
    fontSize:      12,
    fontFamily:    "var(--font-body)",
    fontWeight:    500,
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
    cursor:        "pointer",
    transition:    "color 150ms",
  } as React.CSSProperties,
};

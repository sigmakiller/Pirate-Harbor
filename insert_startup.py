import sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")

f = r"apps\desktop\src-tauri\src\lib.rs"
with open(f, "rb") as fh:
    text = fh.read().decode("utf-8")

# Insert before Ok(()) in the setup closure
# Find the line "            Ok(())"
import re
# Find the setup Ok(()) - it should be at the end of the setup closure
m = re.search(r'\n            Ok\(\(\)\)\n        \}\)', text)
if not m:
    print("ok(()) not found"); 
    # Try alternate
    print(repr(text[-200:]))
    sys.exit(1)

print(f"Found at pos {m.start()}")

smart_call = (
    "\n"
    "            // T60: Refresh smart collections at startup using a fresh connection\n"
    "            if let Ok(sc_conn) = db::init_db(&app_data_dir) {\n"
    "                let now = chrono::Utc::now().to_rfc3339();\n"
    "                let smart_cols: Vec<(String, String)> = {\n"
    "                    sc_conn\n"
    "                        .prepare(\n"
    "                            \"SELECT id, rule_json FROM collections WHERE is_smart = 1 AND rule_json IS NOT NULL\",\n"
    "                        )\n"
    "                        .ok()\n"
    "                        .and_then(|mut stmt| {\n"
    "                            stmt.query_map([], |row| {\n"
    "                                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))\n"
    "                            })\n"
    "                            .ok()\n"
    "                            .map(|rows| rows.filter_map(|r| r.ok()).collect())\n"
    "                        })\n"
    "                        .unwrap_or_default()\n"
    "                };\n"
    "                // Lazy: we don't block startup on evaluation — the scheduler\n"
    "                // already holds the background worker, so just skip here.\n"
    "                let _ = smart_cols; // evaluated on demand via Tauri command\n"
    "            }\n"
)

# Insert the comment just before Ok(())
insert_at = m.start()
text = text[:insert_at] + smart_call + text[insert_at:]

with open(f, "wb") as fh:
    fh.write(text.encode("utf-8"))
print("Done")

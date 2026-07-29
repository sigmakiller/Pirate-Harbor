import re

f = r"apps\desktop\src-tauri\src\commands\collections.rs"
with open(f, "rb") as fh:
    raw = fh.read()

text = raw.decode("utf-8")

# Fix double CRLF corruption
text = text.replace("\r\r\n", "\r\n")

# Fix missing ? on update_collection map_err
text = text.replace(
    '.map_err(|e| format!("Collection not found: {}", e));\r\n\r\n    let new_name',
    '.map_err(|e| format!("Collection not found: {}", e))?;\r\n\r\n    let new_name',
)

with open(f, "wb") as fh:
    fh.write(text.encode("utf-8"))

count = text.count('.map_err(|e| format!("Collection not found: {}", e));\r\n\r\n    let new_name')
print(f"Done. Remaining without ?: {count}")

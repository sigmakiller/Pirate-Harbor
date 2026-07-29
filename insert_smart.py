import sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

f = r"apps\desktop\src-tauri\src\lib.rs"
with open(f, "rb") as fh:
    raw = fh.read()
text = raw.decode("utf-8")

# Find exact separator
idx = text.find("get_game_collections,\n")
if idx == -1:
    print("get_game_collections not found")
    sys.exit(1)

# The text after the last collections command
after_idx = idx + len("get_game_collections,\n")
print(f"Text after: {repr(text[after_idx:after_idx+80])}")

# Insert smart collections block
smart_block = (
    "            // -- T60: Smart collections\n"
    "            commands::smart_collections::create_smart_collection,\n"
    "            commands::smart_collections::evaluate_smart_collection,\n"
    "            commands::smart_collections::refresh_all_smart_collections,\n"
)

text = text[:after_idx] + smart_block + text[after_idx:]

with open(f, "wb") as fh:
    fh.write(text.encode("utf-8"))
print("Done")

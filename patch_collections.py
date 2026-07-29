"""
patch_collections.py — rewrites collections.rs cleanly for T60.
Run with: python patch_collections.py
"""

import re

FILE = r"apps\desktop\src-tauri\src\commands\collections.rs"

with open(FILE, "rb") as f:
    raw = f.read()

# Work in LF-only internally, write CRLF at end
text = raw.decode("utf-8").replace("\r\n", "\n").replace("\r", "\n")

# ── 1. Update row_to_collection signature ─────────────────────────────────────
text = text.replace(
    "    cover_game_id: Option<String>,\n"
    "    created_at:    String,\n"
    "    updated_at:    String,\n"
    ") -> Collection {",
    "    cover_game_id: Option<String>,\n"
    "    created_at:    String,\n"
    "    updated_at:    String,\n"
    "    is_smart:      bool,\n"
    "    rule_json:     Option<String>,\n"
    ") -> Collection {",
    1  # only first occurrence = the function definition
)

# ── 2. Add is_smart/rule_json to the returned Collection literal ──────────────
text = text.replace(
    "        game_ids,\n"
    "        game_count,\n"
    "    }\n"
    "}\n",
    "        game_ids,\n"
    "        game_count,\n"
    "        is_smart,\n"
    "        rule_json,\n"
    "    }\n"
    "}\n",
    1
)

# ── 3. Helper: replace the SELECT in get_collections ─────────────────────────
text = text.replace(
    '"SELECT id, name, description, cover_path, cover_mode, cover_game_id,\n'
    '                    created_at, updated_at\n'
    '             FROM collections ORDER BY created_at DESC"',
    '"SELECT id, name, description, cover_path, cover_mode, cover_game_id,\n'
    '                    created_at, updated_at, is_smart, rule_json\n'
    '             FROM collections ORDER BY created_at DESC"',
    1
)

# ── 4. Extend get_collections row extraction tuple ────────────────────────────
text = text.replace(
    "                row.get::<_, String>(6)?,\n"
    "                row.get::<_, String>(7)?,\n"
    "            ))\n"
    "        })\n"
    "        .map_err(|e| e.to_string())?\n"
    "        .filter_map(|r| r.ok())\n"
    "        .map(|(id, name, description, cover_path, cover_mode, cover_game_id, created_at, updated_at)| {\n"
    "            row_to_collection(&conn, id, name, description, cover_path, cover_mode, cover_game_id, created_at, updated_at)\n"
    "        })\n"
    "        .collect();",
    "                row.get::<_, String>(6)?,\n"
    "                row.get::<_, String>(7)?,\n"
    "                row.get::<_, i64>(8).map(|v| v != 0).unwrap_or(false),\n"
    "                row.get::<_, Option<String>>(9)?,\n"
    "            ))\n"
    "        })\n"
    "        .map_err(|e| e.to_string())?\n"
    "        .filter_map(|r| r.ok())\n"
    "        .map(|(id, name, description, cover_path, cover_mode, cover_game_id, created_at, updated_at, is_smart, rule_json)| {\n"
    "            row_to_collection(&conn, id, name, description, cover_path, cover_mode, cover_game_id, created_at, updated_at, is_smart, rule_json)\n"
    "        })\n"
    "        .collect();"
)

# ── 5. get_collection SELECT + row + call ─────────────────────────────────────
text = text.replace(
    '"SELECT id, name, description, cover_path, cover_mode, cover_game_id,\n'
    '                    created_at, updated_at\n'
    '             FROM collections WHERE id = ?1"',
    '"SELECT id, name, description, cover_path, cover_mode, cover_game_id,\n'
    '                    created_at, updated_at, is_smart, rule_json\n'
    '             FROM collections WHERE id = ?1"',
    1
)

text = text.replace(
    "                    row.get::<_, String>(6)?,\n"
    "                    row.get::<_, String>(7)?,\n"
    "                ))\n"
    "            },\n"
    "        )\n"
    "        .map_err(|e| format!(\"Collection not found: {}\", e))?;\n"
    "\n"
    "    let (cid, name, description, cover_path, cover_mode, cover_game_id, created_at, updated_at) = result;\n"
    "    Ok(row_to_collection(\n"
    "        &conn, cid, name, description, cover_path, cover_mode, cover_game_id, created_at, updated_at,\n"
    "    ))\n"
    "}\n",
    "                    row.get::<_, String>(6)?,\n"
    "                    row.get::<_, String>(7)?,\n"
    "                    row.get::<_, i64>(8).map(|v| v != 0)?,\n"
    "                    row.get::<_, Option<String>>(9)?,\n"
    "                ))\n"
    "            },\n"
    "        )\n"
    "        .map_err(|e| format!(\"Collection not found: {}\", e))?;\n"
    "\n"
    "    let (cid, name, description, cover_path, cover_mode, cover_game_id, created_at, updated_at, is_smart, rule_json) = result;\n"
    "    Ok(row_to_collection(\n"
    "        &conn, cid, name, description, cover_path, cover_mode, cover_game_id, created_at, updated_at, is_smart, rule_json,\n"
    "    ))\n"
    "}\n",
    1
)

# ── 6. create_collection — add is_smart_i, rule_json to INSERT ────────────────
text = text.replace(
    "    let cover_mode = payload.cover_mode.unwrap_or_else(|| \"auto\".to_string());\n"
    "\n"
    "    conn.execute(\n"
    "        \"INSERT INTO collections\n"
    "             (id, name, description, cover_path, cover_mode, cover_game_id, created_at, updated_at)\n"
    "         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)\",\n"
    "        rusqlite::params![\n"
    "            id,\n"
    "            payload.name.trim(),\n"
    "            payload.description,\n"
    "            payload.cover_path,\n"
    "            cover_mode,\n"
    "            payload.cover_game_id,\n"
    "            now\n"
    "        ],\n"
    "    )\n"
    "    .map_err(|e| e.to_string())?;\n"
    "\n"
    "    Ok(row_to_collection(\n"
    "        &conn,\n"
    "        id,\n"
    "        payload.name.trim().to_string(),\n"
    "        payload.description,\n"
    "        payload.cover_path,\n"
    "        cover_mode,\n"
    "        payload.cover_game_id,\n"
    "        now.clone(),\n"
    "        now,\n"
    "    ))\n"
    "}\n",
    "    let cover_mode = payload.cover_mode.unwrap_or_else(|| \"auto\".to_string());\n"
    "    let is_smart   = payload.is_smart.unwrap_or(false);\n"
    "    let is_smart_i = if is_smart { 1i64 } else { 0i64 };\n"
    "\n"
    "    conn.execute(\n"
    "        \"INSERT INTO collections\n"
    "             (id, name, description, cover_path, cover_mode, cover_game_id,\n"
    "              is_smart, rule_json, created_at, updated_at)\n"
    "         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)\",\n"
    "        rusqlite::params![\n"
    "            id,\n"
    "            payload.name.trim(),\n"
    "            payload.description,\n"
    "            payload.cover_path,\n"
    "            cover_mode,\n"
    "            payload.cover_game_id,\n"
    "            is_smart_i,\n"
    "            payload.rule_json,\n"
    "            now\n"
    "        ],\n"
    "    )\n"
    "    .map_err(|e| e.to_string())?;\n"
    "\n"
    "    Ok(row_to_collection(\n"
    "        &conn,\n"
    "        id,\n"
    "        payload.name.trim().to_string(),\n"
    "        payload.description,\n"
    "        payload.cover_path,\n"
    "        cover_mode,\n"
    "        payload.cover_game_id,\n"
    "        now.clone(),\n"
    "        now,\n"
    "        is_smart,\n"
    "        payload.rule_json,\n"
    "    ))\n"
    "}\n"
)

# ── 7. update_collection — include smart fields in SELECT ─────────────────────
text = text.replace(
    "    let (cur_name, cur_desc, cur_cover_path, cur_cover_mode, cur_cover_game_id) = conn\n"
    "        .query_row(\n"
    '            "SELECT name, description, cover_path, cover_mode, cover_game_id\n'
    '             FROM collections WHERE id = ?1",\n'
    "            rusqlite::params![id],\n"
    "            |row| {\n"
    "                Ok((\n"
    "                    row.get::<_, String>(0)?,\n"
    "                    row.get::<_, Option<String>>(1)?,\n"
    "                    row.get::<_, Option<String>>(2)?,\n"
    "                    row.get::<_, String>(3)?,\n"
    "                    row.get::<_, Option<String>>(4)?,\n"
    "                ))\n"
    "            },\n"
    "        )\n"
    "        .map_err(|e| format!(\"Collection not found: {}\", e))?;",
    "    let (cur_name, cur_desc, cur_cover_path, cur_cover_mode, cur_cover_game_id, cur_is_smart, cur_rule_json) = conn\n"
    "        .query_row(\n"
    '            "SELECT name, description, cover_path, cover_mode, cover_game_id, is_smart, rule_json\n'
    '             FROM collections WHERE id = ?1",\n'
    "            rusqlite::params![id],\n"
    "            |row| {\n"
    "                Ok((\n"
    "                    row.get::<_, String>(0)?,\n"
    "                    row.get::<_, Option<String>>(1)?,\n"
    "                    row.get::<_, Option<String>>(2)?,\n"
    "                    row.get::<_, String>(3)?,\n"
    "                    row.get::<_, Option<String>>(4)?,\n"
    "                    row.get::<_, i64>(5).map(|v| v != 0)?,\n"
    "                    row.get::<_, Option<String>>(6)?,\n"
    "                ))\n"
    "            },\n"
    "        )\n"
    "        .map_err(|e| format!(\"Collection not found: {}\", e));",
)

text = text.replace(
    "    Ok(row_to_collection(\n"
    "        &conn, id, new_name, new_desc, new_cover_path, new_cover_mode, new_cover_game, created_at, now,\n"
    "    ))\n"
    "}\n",
    "    Ok(row_to_collection(\n"
    "        &conn, id, new_name, new_desc, new_cover_path, new_cover_mode, new_cover_game,\n"
    "        created_at, now, cur_is_smart, cur_rule_json,\n"
    "    ))\n"
    "}\n",
    1
)

# ── 8. add_game_to_collection and remove_game_from_collection ─────────────────
COMMON_OLD = (
    '    let (name, desc, cover_path, cover_mode, cover_game_id, created_at, updated_at) = conn\n'
    '        .query_row(\n'
    '            "SELECT name, description, cover_path, cover_mode, cover_game_id,\n'
    '                    created_at, updated_at\n'
    '             FROM collections WHERE id = ?1",\n'
    '            rusqlite::params![collection_id],\n'
    '            |row| {\n'
    '                Ok((\n'
    '                    row.get::<_, String>(0)?,\n'
    '                    row.get::<_, Option<String>>(1)?,\n'
    '                    row.get::<_, Option<String>>(2)?,\n'
    '                    row.get::<_, String>(3)?,\n'
    '                    row.get::<_, Option<String>>(4)?,\n'
    '                    row.get::<_, String>(5)?,\n'
    '                    row.get::<_, String>(6)?,\n'
    '                ))\n'
    '            },\n'
    '        )\n'
    '        .map_err(|e| format!("Collection not found: {}", e))?;\n'
    '\n'
    '    Ok(row_to_collection(\n'
    '        &conn, collection_id, name, desc, cover_path, cover_mode, cover_game_id, created_at, updated_at,\n'
    '    ))\n'
    '}\n'
)

COMMON_NEW = (
    '    let (name, desc, cover_path, cover_mode, cover_game_id, created_at, updated_at, is_smart, rule_json) = conn\n'
    '        .query_row(\n'
    '            "SELECT name, description, cover_path, cover_mode, cover_game_id,\n'
    '                    created_at, updated_at, is_smart, rule_json\n'
    '             FROM collections WHERE id = ?1",\n'
    '            rusqlite::params![collection_id],\n'
    '            |row| {\n'
    '                Ok((\n'
    '                    row.get::<_, String>(0)?,\n'
    '                    row.get::<_, Option<String>>(1)?,\n'
    '                    row.get::<_, Option<String>>(2)?,\n'
    '                    row.get::<_, String>(3)?,\n'
    '                    row.get::<_, Option<String>>(4)?,\n'
    '                    row.get::<_, String>(5)?,\n'
    '                    row.get::<_, String>(6)?,\n'
    '                    row.get::<_, i64>(7).map(|v| v != 0)?,\n'
    '                    row.get::<_, Option<String>>(8)?,\n'
    '                ))\n'
    '            },\n'
    '        )\n'
    '        .map_err(|e| format!("Collection not found: {}", e))?;\n'
    '\n'
    '    Ok(row_to_collection(\n'
    '        &conn, collection_id, name, desc, cover_path, cover_mode, cover_game_id,\n'
    '        created_at, updated_at, is_smart, rule_json,\n'
    '    ))\n'
    '}\n'
)

count = text.count(COMMON_OLD)
print(f"Occurrences of common block: {count}")
text = text.replace(COMMON_OLD, COMMON_NEW)

# Write back as CRLF
with open(FILE, "wb") as f:
    f.write(text.replace("\n", "\r\n").encode("utf-8"))

print("Done.")

#!/usr/bin/env python3
"""Checks the content mod's references: every blockstate/item model and texture exists, every
item a recipe, loot table, tag or tab names is the mod's own or vanilla's (by prefix), and every
declared block/item has a model, a name in both langs and (blocks) a loot table.
usage: check_content.py CONTENT_DIR"""
import json, sys
from pathlib import Path

root = Path(sys.argv[1])
ns = "sushigocrafting"
A, D = root / "assets" / ns, root / "data" / ns
errors = []
blocks = {p.stem: json.loads(p.read_text()) for p in (D / "horizon/block").glob("*.json")}
items = {p.stem for p in (D / "horizon/item").glob("*.json")}
ids = {f"{ns}:{b}" for b, d in blocks.items() if d.get("item", True)} | {f"{ns}:{i}" for i in items}
block_ids = {f"{ns}:{b}" for b in blocks}
lang = {c: json.loads((A / "lang" / f"{c}.json").read_text()) for c in ("en_us", "pt_br")}


def model(ref, seen=set()):
    if ref in seen or not ref.startswith(f"{ns}:"):
        return
    seen.add(ref)
    f = A / "models" / (ref.split(":", 1)[1] + ".json")
    if not f.exists():
        errors.append(f"missing model {ref}")
        return
    m = json.loads(f.read_text())
    for t in m.get("textures", {}).values():
        if t.startswith(f"{ns}:") and not (A / "textures" / (t.split(":", 1)[1] + ".png")).exists():
            errors.append(f"missing texture {t} ({ref})")
    if "parent" in m:
        model(m["parent"])


for p in (A / "blockstates").glob("*.json"):
    for v in json.loads(p.read_text())["variants"].values():
        for e in v if isinstance(v, list) else [v]:
            model(e["model"])
for b, d in blocks.items():
    if not (A / "blockstates" / f"{b}.json").exists():
        errors.append(f"no blockstate {b}")
    if not (D / "loot_table/blocks" / f"{b}.json").exists():
        errors.append(f"no loot {b}")
    if f"block.{ns}.{b}" not in lang["en_us"] or f"block.{ns}.{b}" not in lang["pt_br"]:
        errors.append(f"no lang block {b}")
for i in ids:
    path = i.split(":", 1)[1]
    f = A / "items" / f"{path}.json"
    if not f.exists():
        errors.append(f"no item definition {i}")
    else:
        model(json.loads(f.read_text())["model"]["model"])
for i in items:
    for c in lang:
        if f"item.{ns}.{i}" not in lang[c]:
            errors.append(f"no {c} name for item {i}")


def check_id(x, where, allow_blocks=False):
    if not isinstance(x, str):
        return
    x = x.lstrip("#")
    if x.startswith(f"{ns}:") and x not in ids and not (allow_blocks and x in block_ids):
        errors.append(f"{where}: unknown {x}")


def walk(v, where, keys=("id", "name", "item", "result", "input", "icon")):
    if isinstance(v, dict):
        for k, x in v.items():
            if k in keys:
                check_id(x, where)
            walk(x, where)
    elif isinstance(v, list):
        for x in v:
            walk(x, where)


for p in (D / "recipe").rglob("*.json"):
    r = json.loads(p.read_text())
    walk(r, p.name)
    for x in r.get("key", {}).values():
        check_id(x, p.name)
    if "pattern" in r:
        w = {len(row) for row in r["pattern"]}
        if len(w) != 1:
            errors.append(f"{p.name}: rows of different widths")
        used = set("".join(r["pattern"])) - {" "}
        if used != set(r["key"]):
            errors.append(f"{p.name}: key {sorted(r['key'])} vs pattern {sorted(used)}")
for p in (D / "loot_table").rglob("*.json"):
    walk(json.loads(p.read_text()), p.name, keys=("name",))
for p in root.glob("data/*/tags/**/*.json"):
    kind = p.relative_to(root).parts[3]
    for v in json.loads(p.read_text())["values"]:
        check_id(v, str(p.relative_to(root)), allow_blocks=(kind == "block"))
tab = json.loads((D / "horizon/creative_tab" / f"{ns}.json").read_text())
for i in tab["items"] + [tab["icon"]]:
    check_id(i, "tab")
for p in (D / "horizon/item").glob("*.json"):
    d = json.loads(p.read_text())
    if "minecraft:max_stack_size" in d.get("components", {}):
        errors.append(f"{p.name}: max_stack_size component (use the field)")
print(f"{len(blocks)} blocks, {len(items)} items, {len(ids)} ids with models, "
      f"{len(list((D / 'recipe').rglob('*.json')))} recipes")
if errors:
    print("\n".join(errors))
    sys.exit(1)
print("OK")

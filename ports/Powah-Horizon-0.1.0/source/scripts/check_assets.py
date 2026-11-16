#!/usr/bin/env python3
"""Checks the content mods: every model a blockstate or item names exists, every texture a model
names exists, every declared block/item has a model and a name, and loot tables name known items.
usage: check_assets.py CONTENT_DIR"""
import json, sys
from pathlib import Path
root = Path(sys.argv[1])
mods = {p.name: p for p in root.iterdir() if (p / "horizon-mod.toml").exists()}
errors = []
def res(ref, kind, ext):
    ns, path = ref.split(":", 1) if ":" in ref else ("minecraft", ref)
    if ns == "minecraft":
        return True
    return any((m / "assets" / ns / kind / f"{path}{ext}").exists() for m in mods.values())
def model_textures(ref, seen=()):
    ns, path = ref.split(":", 1) if ":" in ref else ("minecraft", ref)
    if ns == "minecraft":
        return
    for m in mods.values():
        f = m / "assets" / ns / "models" / f"{path}.json"
        if f.exists():
            j = json.loads(f.read_text())
            for t in j.get("textures", {}).values():
                if not t.startswith("#") and not res(t, "textures", ".png"):
                    errors.append(f"{f}: texture {t}")
            if "parent" in j:
                if not res(j["parent"], "models", ".json"):
                    errors.append(f"{f}: parent {j['parent']}")
            return
    errors.append(f"model {ref} missing")
ids = set()
for name, m in mods.items():
    for f in (m / "data" / name / "horizon/item").glob("*.json"):
        ids.add(f"{name}:{f.stem}")
    for f in (m / "data" / name / "horizon/block").glob("*.json"):
        if json.loads(f.read_text()).get("item", True):
            ids.add(f"{name}:{f.stem}")
for name, m in mods.items():
    a = m / "assets" / name
    lang = json.loads((a / "lang/en_us.json").read_text())
    for f in (m / "data" / name / "horizon/block").glob("*.json"):
        bs = a / "blockstates" / f"{f.stem}.json"
        if not bs.exists():
            errors.append(f"blockstate {name}:{f.stem} missing"); continue
        for v in json.loads(bs.read_text()).get("variants", {}).values():
            for vv in (v if isinstance(v, list) else [v]):
                model_textures(vv["model"])
        if f"block.{name}.{f.stem}" not in lang:
            errors.append(f"no name for block {name}:{f.stem}")
    for i in ids:
        ns, p = i.split(":")
        if ns != name: continue
        it = a / "items" / f"{p}.json"
        if not it.exists():
            errors.append(f"item model {i} missing"); continue
        model_textures(json.loads(it.read_text())["model"]["model"])
        if f"item.{name}.{p}" not in lang and f"block.{name}.{p}" not in lang:
            errors.append(f"no name for {i}")
    for f in (m / "data" / name / "loot_table").rglob("*.json"):
        for n in __import__("re").findall(r'"name": "([a-z_]+:[a-z_0-9]+)"', f.read_text()):
            if not n.startswith("minecraft:") and n not in ids:
                errors.append(f"{f}: {n}")
print("\n".join(errors) or f"assets ok ({len(ids)} items)")
sys.exit(1 if errors else 0)

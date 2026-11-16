#!/usr/bin/env python3
"""Confere um conteúdo gerado: JSON válido, modelos/texturas/itens referidos existem.
Uso: python3 -I scripts/check_content.py DIR_MEKANISM DIR_MEKANISMTOOLS"""
import json, sys
from pathlib import Path

roots = [Path(a) for a in sys.argv[1:]]
errors = []
files = {}
for root in roots:
    for p in root.rglob("*"):
        if p.is_file():
            files[(root, p.relative_to(root).as_posix())] = p
def has(rel):
    return any(r == rel for (_, r) in files)
# ids declarados
blocks, items, no_item = set(), set(), set()
for (root, rel) in files:
    parts = rel.split("/")
    if len(parts) >= 5 and parts[0] == "data" and parts[2] == "horizon" and parts[3] in ("block", "item"):
        ns = parts[1]; name = "/".join(parts[4:])[:-5]
        (blocks if parts[3] == "block" else items).add(f"{ns}:{name}")
        if parts[3] == "block" and json.loads(files[(root, rel)].read_text()).get("item") is False:
            no_item.add(f"{ns}:{name}")
known_items = blocks | items
def item_ok(i):
    if i.startswith("#"): return True
    return i.startswith("minecraft:") or i in known_items
def model_ok(m):
    ns, path = (m.split(":", 1) if ":" in m else ("minecraft", m))
    return ns == "minecraft" or has(f"assets/{ns}/models/{path}.json")
def tex_ok(t):
    if t.startswith("#"): return True
    ns, path = (t.split(":", 1) if ":" in t else ("minecraft", t))
    return ns == "minecraft" or has(f"assets/{ns}/textures/{path}.png")
def walk(v, f):
    if isinstance(v, dict):
        for k, x in v.items(): f(k, x); walk(x, f)
    elif isinstance(v, list):
        for x in v: walk(x, f)
for (root, rel), p in files.items():
    if not rel.endswith(".json") and not rel.endswith(".mcmeta"): continue
    try:
        v = json.loads(p.read_text(encoding="utf-8"))
    except Exception as e:
        errors.append(f"{rel}: JSON {e}"); continue
    if rel.endswith(".mcmeta"): continue
    if "/blockstates/" in rel or "/items/" in rel and rel.startswith("assets"):
        walk(v, lambda k, x: k == "model" and isinstance(x, str) and not model_ok(x) and errors.append(f"{rel}: modelo {x}"))
    if "/models/" in rel:
        if "parent" in v and not model_ok(v["parent"]): errors.append(f"{rel}: parent {v['parent']}")
        for k, t in v.get("textures", {}).items():
            if not tex_ok(t): errors.append(f"{rel}: textura {t}")
    if "/recipe/" in rel:
        def chk(k, x):
            if k in ("id", "input", "ingredient", "result") and isinstance(x, str) and ":" in x and not item_ok(x):
                errors.append(f"{rel}: item {x}")
        walk(v, chk)
        for x in v.get("key", {}).values():
            if isinstance(x, str) and not item_ok(x): errors.append(f"{rel}: chave {x}")
        for x in v.get("ingredients", []):
            if isinstance(x, str) and not item_ok(x): errors.append(f"{rel}: ingrediente {x}")
    if "/loot_table/" in rel:
        def loot(v):
            if isinstance(v, dict):
                if v.get("type") == "minecraft:item" and not item_ok(v.get("name", "")):
                    errors.append(f"{rel}: loot {v.get('name')}")
                for x in v.values(): loot(x)
            elif isinstance(v, list):
                for x in v: loot(x)
        loot(v)
    if "/tags/" in rel:
        for x in v.get("values", []):
            if isinstance(x, str) and not x.startswith("#") and not x.startswith("minecraft:") and x not in known_items:
                errors.append(f"{rel}: tag {x}")
    if "/equipment/" in rel:
        for kind, layers in v["layers"].items():
            for l in layers:
                ns, n = l["texture"].split(":")
                if not has(f"assets/{ns}/textures/entity/equipment/{kind}/{n}.png"): errors.append(f"{rel}: {kind} {n}")
# todo bloco/item declarado tem modelo de item e nome
for i in sorted(known_items):
    ns, n = i.split(":")
    if i not in no_item and not has(f"assets/{ns}/items/{n}.json"): errors.append(f"{i}: sem assets/{ns}/items/{n}.json")
    kind = "block" if i in blocks else "item"
    for lang in ("en_us", "pt_br"):
        lp = [p for (r, rel), p in files.items() if rel == f"assets/{ns}/lang/{lang}.json"]
        if not lp or f"{kind}.{ns}.{n}" not in json.loads(lp[0].read_text(encoding="utf-8")):
            errors.append(f"{i}: sem nome em {lang}")
    if i in blocks:
        if not has(f"assets/{ns}/blockstates/{n}.json"): errors.append(f"{i}: sem blockstate")
        if not has(f"data/{ns}/loot_table/blocks/{n}.json"): errors.append(f"{i}: sem loot")
print(f"{len(blocks)} blocos, {len(items)} itens, {len(files)} arquivos, {len(errors)} erros")
for e in errors[:50]: print(" ", e)
sys.exit(1 if errors else 0)

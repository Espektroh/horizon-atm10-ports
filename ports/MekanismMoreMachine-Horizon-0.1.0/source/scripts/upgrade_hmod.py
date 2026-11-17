#!/usr/bin/env python3
"""Bring a content mod (.hmod) written with 1.21.1 data up to Horizon 26.3's shapes.

Port of Horizon 66f3981's horizon-port-pack converters (crates/horizon-port-pack/src):
- carry.rs upgrade_loot / upgrade_condition / type_number / type_function_numbers /
  named_tool_condition / upgrade_entity_predicate, for data/*/loot_table(s)/**.json
- worldgen.rs block_states ({"Name","Properties"} -> {"id","properties"}), for data/*/worldgen/**.json

usage: upgrade_hmod.py IN.hmod OUT.hmod   (or a directory: upgrade_hmod.py --dir CONTENT_DIR)
Prints how many files changed. Deterministic zip (fixed dates), like the packagers.
"""
import json
import sys
import zipfile
from pathlib import Path

ENTITY_FIELDS = ["distance", "movement", "location", "stepping_on", "movement_affected_by", "effects",
                 "nbt", "flags", "equipment", "periodic_tick", "team", "slots", "components",
                 "predicates", "type_specific"]
ENTITY_NESTED = ["vehicle", "passenger", "targeted_entity"]
VARIANT_KINDS = ["axolotl", "cat", "chicken", "cow", "fox", "frog", "horse", "llama", "mooshroom",
                 "painting", "parrot", "pig", "rabbit", "wolf"]


def type_number(v):
    if not isinstance(v, dict):
        return
    if "type" not in v and "min" in v and "max" in v:
        v["type"] = "minecraft:uniform"
    kind = v.get("type") or ""
    kind = kind[len("minecraft:"):] if kind.startswith("minecraft:") else kind
    for key in {"uniform": ["min", "max"], "binomial": ["n", "p"]}.get(kind, []):
        if key in v:
            type_number(v[key])


def type_function_numbers(f):
    if not isinstance(f, dict):
        return
    kind = f.get("type") or ""
    kind = kind[len("minecraft:"):] if kind.startswith("minecraft:") else kind

    def each(lst, key):
        for x in f.get(lst) or []:
            if isinstance(x, dict) and key in x:
                type_number(x[key])

    if kind == "set_stew_effect":
        each("effects", "duration")
    elif kind == "set_attributes":
        each("modifiers", "amount")
    elif kind == "set_enchantments":
        for k, lvl in (f.get("enchantments") or {}).items():
            type_number(lvl)
    elif kind == "limit_count":
        lim = f.get("limit")
        if isinstance(lim, dict):
            for key in ("min", "max"):
                if key in lim:
                    type_number(lim[key])
    else:
        key = {"set_count": "count", "enchanted_count_increase": "count", "set_damage": "damage",
               "enchant_with_levels": "levels", "set_ominous_bottle_amplifier": "amplifier"}.get(kind)
        if key and key in f:
            type_number(f[key])


def rename_kind(v, key):
    if isinstance(v, dict) and "type" not in v and key in v:
        v["type"] = v.pop(key)


def named_tool_condition(p):
    if not isinstance(p, dict):
        return None
    keys = list(p.keys())
    if keys == ["predicates"]:
        lst = (p["predicates"] or {}).get("minecraft:enchantments")
        if not isinstance(lst, list) or len(lst) != 1:
            return None
        e = lst[0]
        lv = e.get("levels") or {}
        silk = (e.get("enchantments") == "minecraft:silk_touch" and (lv.get("min", 1) if isinstance(lv, dict) else 1) <= 1
                and not (isinstance(lv, dict) and "max" in lv) and len(p["predicates"]) == 1)
        return "minecraft:tool/can_silk_touch" if silk else None
    if keys == ["items"]:
        it = p["items"]
        return "minecraft:tool/can_shear" if it in ("minecraft:shears", ["minecraft:shears"]) else None
    return None


def upgrade_entity_predicate(p):
    if not isinstance(p, dict):
        return
    key = lambda f: f"minecraft:{f}"
    if "type" in p:
        p[key("entity_type")] = p.pop("type")
    for f in ENTITY_NESTED:
        if f in p:
            inner = p.pop(f)
            upgrade_entity_predicate(inner)
            p[key(f)] = inner
    for f in ENTITY_FIELDS:
        if f in p:
            p[key(f)] = p.pop(f)
    ts = p.pop("minecraft:type_specific", None)
    if not isinstance(ts, dict):
        if ts is not None:
            p["minecraft:type_specific"] = ts
        return
    kind = ts.pop("type", None) or "any"
    kind = kind[len("minecraft:"):] if kind.startswith("minecraft:") else kind
    kind = "cube_mob" if kind == "slime" else kind
    comp = None
    if kind in VARIANT_KINDS and "variant" in ts:
        comp = (f"{kind}/variant", ts.pop("variant"))
    elif kind == "sheep" and "color" in ts:
        comp = ("sheep/color", ts.pop("color"))
    if comp:
        p.setdefault(key("components"), {})[key(comp[0])] = comp[1]
    for f in ("looking_at", "entity_struck"):
        if f in ts:
            upgrade_entity_predicate(ts[f])
    if ts or not (kind == "any" or kind in VARIANT_KINDS):
        p[key(f"type_specific/{kind}")] = ts


def upgrade_condition(c):
    rename_kind(c, "condition")
    if not isinstance(c, dict):
        return c
    if "term" in c:
        c["term"] = upgrade_condition(c["term"])
    if isinstance(c.get("terms"), list):
        c["terms"] = [upgrade_condition(t) for t in c["terms"]]
    if c.get("type") == "minecraft:match_tool":
        named = named_tool_condition(c.get("predicate"))
        if named:
            return named
    t = c.get("type")
    if t == "minecraft:entity_properties" and "predicate" in c:
        upgrade_entity_predicate(c["predicate"])
    elif t == "minecraft:damage_source_properties":
        for k in ("direct_entity", "source_entity"):
            if isinstance(c.get("predicate"), dict) and k in c["predicate"]:
                upgrade_entity_predicate(c["predicate"][k])
    return c


def upgrade_loot(v):
    if isinstance(v, list):
        for x in v:
            upgrade_loot(x)
    elif isinstance(v, dict):
        if isinstance(v.get("pools"), list):
            for pool in v["pools"]:
                for k in ("rolls", "bonus_rolls"):
                    if isinstance(pool, dict) and k in pool:
                        type_number(pool[k])
        if isinstance(v.get("conditions"), list):
            lst = [upgrade_condition(c) for c in v.pop("conditions")]
            v["condition"] = lst[0] if len(lst) == 1 else {"type": "minecraft:all_of", "terms": lst}
        if isinstance(v.get("functions"), list):
            lst = v.pop("functions")
            for f in lst:
                rename_kind(f, "function")
                type_function_numbers(f)
                upgrade_loot(f)
            v["modifier"] = lst
        for k, x in v.items():
            if k not in ("condition", "modifier"):
                upgrade_loot(x)


def block_states(v):
    changed = False
    if isinstance(v, dict):
        if isinstance(v.get("Name"), str) and all(k in ("Name", "Properties") for k in v):
            v["id"] = v.pop("Name")
            if "Properties" in v:
                v["properties"] = v.pop("Properties")
            return True
        for x in v.values():
            changed |= block_states(x)
    elif isinstance(v, list):
        for x in v:
            changed |= block_states(x)
    return changed


def flatten_config(v):
    """26.3 dropped configured features' `config` wrapper: {"type", "config": {...}} -> {"type", ...}."""
    changed = False
    if isinstance(v, dict):
        if isinstance(v.get("type"), str) and isinstance(v.get("config"), dict) and \
                not any(k in v for k in v["config"] if k != "type"):
            cfg = v.pop("config")
            v.update(cfg)
            changed = True
        for x in list(v.values()):
            changed |= flatten_config(x)
    elif isinstance(v, list):
        for x in v:
            changed |= flatten_config(x)
    return changed


PROVIDERS = {"simple_state_provider": "simple_state_provider", "weighted_state_provider": "weighted",
             "rotated_block_provider": "rotated", "randomized_int_state_provider": "randomized_int",
             "rule_based_state_provider": "rule_based", "noise_provider": "noise",
             "dual_noise_provider": "dual_noise", "noise_threshold_provider": "noise_threshold"}


def short(t):
    return t[len("minecraft:"):] if isinstance(t, str) and t.startswith("minecraft:") else t


def upgrade_features(v):
    """port-pack upgrade/features.rs: providers renamed (simple_state_provider -> its state),
    random_offset -> offset (x and z from xz_spread)."""
    if isinstance(v, list):
        return [upgrade_features(x) for x in v]
    if not isinstance(v, dict):
        return v
    t = short(v.get("type"))
    if t == "simple_state_provider" and "state" in v:
        return upgrade_features(v["state"])
    if t in PROVIDERS:
        v["type"] = "minecraft:" + PROVIDERS[t]
    if t == "random_offset":
        xz = v.get("xz_spread", 0)
        return {"type": "minecraft:offset", "x": xz, "y": v.get("y_spread", 0), "z": xz}
    return {k: upgrade_features(x) for k, x in v.items()}


def kind_of(name):
    parts = name.split("/")
    if len(parts) >= 3 and parts[0] == "data" and name.endswith(".json"):
        if parts[2] in ("loot_table", "loot_tables"):
            return "loot"
        if parts[2] == "worldgen":
            return "worldgen"
    return None


def upgrade_bytes(name, data):
    kind = kind_of(name)
    if not kind:
        return data, False
    try:
        v = json.loads(data)
    except Exception:
        return data, False
    before = json.dumps(v, sort_keys=True)
    if kind == "loot":
        upgrade_loot(v)
    else:
        block_states(v)
        if "/worldgen/configured_feature/" in name or "/worldgen/placed_feature/" in name:
            flatten_config(v)
            v = upgrade_features(v)
    if json.dumps(v, sort_keys=True) == before:
        return data, False
    return (json.dumps(v, indent=2, ensure_ascii=False) + "\n").encode(), True


def upgrade_zip(src, dst):
    changed = {"loot": 0, "worldgen": 0}
    with zipfile.ZipFile(src) as zin:
        entries = [(i, zin.read(i.filename)) for i in zin.infolist()]
    with zipfile.ZipFile(dst, "w", compression=zipfile.ZIP_DEFLATED) as zout:
        for info, data in entries:
            new, did = upgrade_bytes(info.filename, data)
            if did:
                changed[kind_of(info.filename)] += 1
            zi = zipfile.ZipInfo(info.filename, info.date_time)
            zi.compress_type = zipfile.ZIP_DEFLATED
            zi.external_attr = info.external_attr
            zout.writestr(zi, new)
    return changed


def upgrade_dir(root):
    changed = {"loot": 0, "worldgen": 0}
    root = Path(root)
    for p in root.rglob("*.json"):
        rel = p.relative_to(root).as_posix()
        i = rel.find("data/")
        if i < 0:
            continue
        name = rel[i:]
        new, did = upgrade_bytes(name, p.read_bytes())
        if did:
            p.write_bytes(new)
            changed[kind_of(name)] += 1
    return changed


if __name__ == "__main__":
    if sys.argv[1] == "--dir":
        print(json.dumps(upgrade_dir(sys.argv[2])))
    else:
        print(json.dumps(upgrade_zip(sys.argv[1], sys.argv[2])))

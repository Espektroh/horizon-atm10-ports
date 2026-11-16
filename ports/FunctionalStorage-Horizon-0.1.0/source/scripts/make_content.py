#!/usr/bin/env python3
"""Builds the Functional Storage content mod (`functionalstorage.hmod` source tree).

Functional Storage (c) Buuz135, Rid: MIT. The textures, models and translations are copied
from the mod's jar (extracted to SRC); block and item declarations, blockstates, item
definitions, loot tables and recipes are written here in Horizon 26.3's shapes.

usage: make_content.py SRC OUT
  SRC: the extracted jar (holds assets/functionalstorage and data/functionalstorage)
  OUT: the content tree to write (source/content/functionalstorage); then run
       upgrade_hmod.py --dir OUT and package.py.

0.1.0: the ten vanilla woods' 1x1, 1x2 and 2x2 drawers, the storage upgrades (iron downgrade,
copper, gold, diamond, netherite) and the void upgrade. Not yet: compacting, fluid, framed and
ender drawers, the controllers, the armory cabinet, the tools and the other utility upgrades.
"""
import json
import shutil
import sys
from pathlib import Path

NS = "functionalstorage"
VERSION = "1.5.8-horizon.1"
WOODS = ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "crimson", "warped", "mangrove", "cherry"]
SIZES = [("1", "1x1"), ("2", "1x2"), ("4", "2x2")]
UPGRADES = ["iron_downgrade", "copper_upgrade", "gold_upgrade", "diamond_upgrade", "netherite_upgrade",
            "void_upgrade"]
# Each drawer copies a vanilla block with a horizontal `facing` that it takes from where the player
# looks (front toward the player) and no use of its own (glazed terracotta: no menu, no item use).
TEMPLATE = "minecraft:white_glazed_terracotta"
# Functional Storage copies its wood's planks: hardness 2, blast resistance 3, the planks' sounds.
# requires_tool false: a drawer drops by hand, as planks do (the template's terracotta needs a pickaxe).
# The `c:` tags the recipes name, as the vanilla items each stands for here.
CHESTS = ["minecraft:chest", "minecraft:trapped_chest"]
YAW = {"north": 0, "east": 90, "south": 180, "west": 270}
# Each wood's planks' MapColor id (MapColor.WOOD, PODZOL, SAND...), as the drawer copies its planks.
MAP_COLOR = {"oak": 13, "spruce": 34, "birch": 2, "jungle": 10, "acacia": 15, "dark_oak": 26, "crimson": 53,
             "warped": 56, "mangrove": 28, "cherry": 36}


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(data, (dict, list)):
        data = json.dumps(data, indent=2, ensure_ascii=False) + "\n"
    path.write_text(data, encoding="utf-8")


def drawer_ids():
    return [f"{w}_{n}" for w in WOODS for n, _ in SIZES]


def main():
    src, out = Path(sys.argv[1]), Path(sys.argv[2])
    if out.exists():
        shutil.rmtree(out)
    jar_assets = src / "assets" / NS
    jar_data = src / "data" / NS
    write(out / "horizon-mod.toml", f'''id = "{NS}"
name = "Functional Storage"
version = "{VERSION}"
description = "Gavetas 1x1, 1x2 e 2x2 das 10 madeiras, upgrades de armazenamento (ferro, cobre, ouro, diamante, netherita) e de vazio. Precisa do mod nativo functionalstorage_native 0.1.0."
authors = ["Buuz135, Rid (original, MIT)", "port Horizon"]
api = "0.2"
side = "both"
capabilities = ["registries"]
''')
    data, assets = out / "data" / NS, out / "assets" / NS

    # Models and textures, as the jar has them (front facing north).
    (assets / "models/block").mkdir(parents=True, exist_ok=True)
    for base in ("base_x_1", "base_x_2", "base_x_4"):
        shutil.copy(jar_assets / "models/block" / f"{base}.json", assets / "models/block" / f"{base}.json")
    tex = assets / "textures/block"
    tex.mkdir(parents=True, exist_ok=True)
    for w in WOODS:
        shutil.copy(jar_assets / "textures/block" / f"{w}_side.png", tex / f"{w}_side.png")
        for n, _ in SIZES:
            shutil.copy(jar_assets / "textures/block" / f"{w}_front_{n}.png", tex / f"{w}_front_{n}.png")

    for w in WOODS:
        planks = f"minecraft:{w}_planks"
        for n, label in SIZES:
            name = f"{w}_{n}"
            write(data / "horizon/block" / f"{name}.json", {
                "template": TEMPLATE,
                "properties": {"hardness": 2.0, "explosion_resistance": 3.0, "sounds": planks,
                               "requires_tool": False, "map_color": MAP_COLOR[w]},
                "item": True,
            })
            model = json.loads((jar_assets / "models/block" / f"{name}.json").read_text())
            write(assets / "models/block" / f"{name}.json", model)
            write(assets / "blockstates" / f"{name}.json", {"variants": {
                f"facing={f}": ({"model": f"{NS}:block/{name}", "y": y} if y else {"model": f"{NS}:block/{name}"})
                for f, y in YAW.items()}})
            write(assets / "models/item" / f"{name}.json", {"parent": f"{NS}:block/{name}"})
            write(assets / "items" / f"{name}.json",
                  {"model": {"type": "minecraft:model", "model": f"{NS}:item/{name}"}})
            write(data / "loot_table/blocks" / f"{name}.json", {
                "type": "minecraft:block",
                "pools": [{"rolls": 1, "bonus_rolls": 0,
                           "entries": [{"type": "minecraft:item", "name": f"{NS}:{name}"}],
                           "conditions": [{"condition": "minecraft:survives_explosion"}]}],
                "random_sequence": f"{NS}:blocks/{name}",
            })
            # DrawerType recipes: 1x1 PPP/PCP/PPP, 1x2 PCP/PPP/PCP (2), 2x2 CPC/PPP/CPC (4).
            pattern, count = {"1": (["PPP", "PCP", "PPP"], 1), "2": (["PCP", "PPP", "PCP"], 2),
                              "4": (["CPC", "PPP", "CPC"], 4)}[n]
            write(data / "recipe" / f"{name}.json", {
                "type": "minecraft:crafting_shaped", "category": "misc", "pattern": pattern,
                "key": {"P": planks, "C": CHESTS},
                "result": {"id": f"{NS}:{name}", "count": count}})

    itex = assets / "textures/item"
    itex.mkdir(parents=True, exist_ok=True)
    for up in UPGRADES:
        shutil.copy(jar_assets / "textures/item" / f"{up}.png", itex / f"{up}.png")
        write(data / "horizon/item" / f"{up}.json", {"template": "minecraft:paper"})
        write(assets / "models/item" / f"{up}.json",
              {"parent": "minecraft:item/generated", "textures": {"layer0": f"{NS}:item/{up}"}})
        write(assets / "items" / f"{up}.json",
              {"model": {"type": "minecraft:model", "model": f"{NS}:item/{up}"}})

    def shaped(name, pattern, key):
        write(data / "recipe" / f"{name}.json", {
            "type": "minecraft:crafting_shaped", "category": "misc", "pattern": pattern, "key": key,
            "result": {"id": f"{NS}:{name}", "count": 1}})

    drawer = f"#{NS}:drawer"
    shaped("copper_upgrade", ["IBI", "CDC", "IBI"],
           {"I": "minecraft:copper_ingot", "B": "minecraft:copper_block", "C": CHESTS, "D": drawer})
    shaped("gold_upgrade", ["IBI", "CDC", "BIB"],
           {"I": "minecraft:gold_ingot", "B": "minecraft:gold_block", "C": CHESTS, "D": f"{NS}:copper_upgrade"})
    shaped("diamond_upgrade", ["IBI", "CDC", "IBI"],
           {"I": "minecraft:diamond", "B": "minecraft:diamond_block", "C": CHESTS, "D": f"{NS}:gold_upgrade"})
    shaped("iron_downgrade", ["III", "IDI", "III"], {"I": "minecraft:iron_ingot", "D": drawer})
    shaped("void_upgrade", ["III", "IDI", "III"], {"I": "minecraft:obsidian", "D": drawer})
    write(data / "recipe/netherite_upgrade.json", {
        "type": "minecraft:smithing_transform",
        "template": "minecraft:netherite_upgrade_smithing_template",
        "base": f"{NS}:diamond_upgrade", "addition": "minecraft:netherite_ingot",
        "result": {"id": f"{NS}:netherite_upgrade", "count": 1}})

    ids = [f"{NS}:{d}" for d in drawer_ids()]
    write(data / "tags/item/drawer.json", {"values": ids})
    for n, label in SIZES:
        write(data / "tags/item" / f"drawer_{label}.json", {"values": [f"{NS}:{w}_{n}" for w in WOODS]})
    write(out / "data/minecraft/tags/block/mineable/axe.json", {"values": ids})
    write(data / "horizon/creative_tab/functionalstorage.json", {
        "title": "itemGroup.functionalstorage", "icon": f"{NS}:oak_1",
        "items": ids + [f"{NS}:{u}" for u in UPGRADES]})

    en = json.loads((jar_assets / "lang/en_us.json").read_text(encoding="utf-8"))
    for lang in ("en_us", "pt_br"):
        names = json.loads((jar_assets / "lang" / f"{lang}.json").read_text(encoding="utf-8"))
        keys = [f"block.{NS}.{d}" for d in drawer_ids()] + [f"item.{NS}.{u}" for u in UPGRADES] + \
               ["itemGroup.functionalstorage"]
        write(assets / "lang" / f"{lang}.json", {k: names.get(k, en[k]) for k in keys})
    print(f"{len(ids)} drawers, {len(UPGRADES)} upgrades -> {out}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    main()

#!/usr/bin/env python3
"""Build the Torchmaster content mod for Horizon (API 0.2): torchmaster.hmod,
folder first, then zipped.

Usage: make_content.py <torchmaster jar> <output folder>

Only data files are read from the jars (textures, models, lang, loot); the jars' code is
never run. Torchmaster is MIT (Xalcon).
Recipes are rewritten with vanilla items (26.3 has no c: tags).
"""
import json
import shutil
import sys
import zipfile
from pathlib import Path


def write(path: Path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(data, (dict, list)):
        path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    elif isinstance(data, bytes):
        path.write_bytes(data)
    else:
        path.write_text(data, encoding="utf-8")


def shaped(ns, pattern, key, result):
    return {"type": "minecraft:crafting_shaped", "category": "misc", "pattern": pattern, "key": key,
            "result": {"id": f"{ns}:{result}"}}


def loot(ns, block):
    return {"type": "minecraft:block", "pools": [{
        "rolls": 1.0, "bonus_rolls": 0.0,
        "conditions": [{"condition": "minecraft:survives_explosion"}],
        "entries": [{"type": "minecraft:item", "name": f"{ns}:{block}"}]}],
        "random_sequence": f"{ns}:blocks/{block}"}


def torchmaster(jar: Path, out: Path):
    ns = "torchmaster"
    a, d = out / "assets" / ns, out / "data" / ns
    write(out / "horizon-mod.toml", f'''id = "{ns}"
name = "Torchmaster"
version = "21.1.12-horizon.1"
description = "Mega Tocha, Lampada do Medo e Lanterna Feroz, portadas do Torchmaster. Precisa do mod nativo torchmaster_native para funcionar."
authors = ["Xalcon (original, MIT)", "port Horizon (Utilidades ATM)"]
# A content mod: blocks and items from data/ and assets/, no code (API 0.2).
api = "0.2"
side = "both"
capabilities = ["registries"]
''')
    with zipfile.ZipFile(jar) as z:
        def copy(src, dst=None):
            write(out / (dst or src), z.read(src))
        for t in ["megatorch", "megatorch_top", "feral_flare_lantern"]:
            copy(f"assets/{ns}/textures/block/{t}.png")
        for m in ["megatorch", "dreadlamp", "feral_flare_lantern", "feral_flare_lantern_stand", "invisible_light"]:
            copy(f"assets/{ns}/models/block/{m}.json")
        for m in ["megatorch", "dreadlamp", "feral_flare_lantern"]:
            copy(f"assets/{ns}/models/item/{m}.json")
        for lang in ["en_us", "pt_br"]:
            copy(f"assets/{ns}/lang/{lang}.json")
    # Blocks: each from a vanilla template (docs/mods.md, "Content mods").
    # Mega torch: torch-shaped post, light 15, wood sounds (Torchmaster: strength 1, SoundType.WOOD).
    write(d / "horizon" / "block" / "megatorch.json", {
        "template": "minecraft:stone",
        "properties": {"hardness": 1.0, "explosion_resistance": 1.0, "light": 15,
                       "sounds": "minecraft:oak_planks", "requires_tool": False, "occludes": False},
        "shapes": {"default": [[0.375, 0.0, 0.375, 0.625, 1.0, 0.625]]},
        "item": True})
    # Dread lamp: obsidian and glass, cutout like glass.
    write(d / "horizon" / "block" / "dreadlamp.json", {
        "template": "minecraft:glass",
        "properties": {"hardness": 1.0, "explosion_resistance": 1.0, "light": 15,
                       "sounds": "minecraft:oak_planks"},
        "item": True})
    # Feral flare lantern: the lantern template (floor or ceiling, hanging), light 15.
    write(d / "horizon" / "block" / "feral_flare_lantern.json", {
        "template": "minecraft:lantern",
        "properties": {"hardness": 1.0, "explosion_resistance": 1.0, "light": 15},
        "shapes": {"default": [[0.25, 0.0, 0.25, 0.75, 1.0, 0.75]]},
        "item": True})
    # The lantern's lights: vanilla's light block (invisible, no collision, replaceable, level 15).
    write(d / "horizon" / "block" / "invisible_light.json", {"template": "minecraft:light", "item": False})
    write(a / "blockstates" / "megatorch.json", {"variants": {"": {"model": f"{ns}:block/megatorch"}}})
    write(a / "blockstates" / "dreadlamp.json", {"variants": {"": {"model": f"{ns}:block/dreadlamp"}}})
    write(a / "blockstates" / "invisible_light.json", {"variants": {"": {"model": f"{ns}:block/invisible_light"}}})
    # The original's stand points at the block it was placed on (facing = clicked face's opposite):
    # on the floor that is "down" (no rotation), under a ceiling "up" (x 180).
    write(a / "blockstates" / "feral_flare_lantern.json", {"multipart": [
        {"apply": {"model": f"{ns}:block/feral_flare_lantern"}},
        {"apply": {"model": f"{ns}:block/feral_flare_lantern_stand"}, "when": {"hanging": "false"}},
        {"apply": {"model": f"{ns}:block/feral_flare_lantern_stand", "x": 180}, "when": {"hanging": "true"}},
    ]})
    for b in ["megatorch", "dreadlamp", "feral_flare_lantern"]:
        write(a / "items" / f"{b}.json", {"model": {"type": "minecraft:model", "model": f"{ns}:item/{b}"}})
        write(d / "loot_table" / "blocks" / f"{b}.json", loot(ns, b))
    write(d / "recipe" / "megatorch.json", shaped(ns, ["TTT", "DLD", "GLG"], {
        "T": "minecraft:torch", "D": "minecraft:diamond", "G": "minecraft:gold_block", "L": "#minecraft:logs"},
        "megatorch"))
    write(d / "recipe" / "dreadlamp.json", shaped(ns, ["OOO", "PGP", "OBO"], {
        "O": "minecraft:obsidian", "P": "minecraft:glass_pane", "G": "minecraft:glowstone",
        "B": "minecraft:black_dye"}, "dreadlamp"))
    write(d / "recipe" / "feral_flare_lantern.json", shaped(ns, [" I ", "GgG", " I "], {
        "I": "minecraft:gold_ingot", "g": "minecraft:glowstone_dust", "G": "minecraft:glass"},
        "feral_flare_lantern"))
    tags = out / "data" / "minecraft" / "tags" / "block" / "mineable"
    write(tags / "axe.json", {"values": [f"{ns}:megatorch"]})
    write(tags / "pickaxe.json", {"values": [f"{ns}:feral_flare_lantern", f"{ns}:dreadlamp"]})
    write(out / "LICENSE-torchmaster.txt", "Textures, models and lang: Torchmaster by Xalcon, MIT License.\n"
          "https://github.com/xalcon/TorchMaster\nModified: blockstates, recipes and loot tables rewritten "
          "for Horizon.\n")


def pack(folder: Path, dest: Path):
    """A .hmod is a zip of the manifest, assets/ and data/ (the only paths Horizon accepts,
    horizon-mods content.rs check_path), files sorted, fixed timestamps. The license
    notes stay in the source folder."""
    def kept(p: Path) -> bool:
        rel = p.relative_to(folder).as_posix()
        return rel == "horizon-mod.toml" or rel.startswith(("assets/", "data/"))
    with zipfile.ZipFile(dest, "w", zipfile.ZIP_DEFLATED) as z:
        for f in sorted(p for p in folder.rglob("*") if p.is_file() and kept(p)):
            info = zipfile.ZipInfo(f.relative_to(folder).as_posix(), (1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(info, f.read_bytes())


def main(tm_jar: Path, out: Path):
    for name, fn, jar in [("torchmaster", torchmaster, tm_jar)]:
        folder = out / name
        if folder.exists():
            shutil.rmtree(folder)
        fn(jar, folder)
        pack(folder, out / f"{name}.hmod")
        print(f"{out / (name + '.hmod')}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    main(Path(sys.argv[1]), Path(sys.argv[2]))

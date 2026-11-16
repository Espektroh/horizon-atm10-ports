#!/usr/bin/env python3
"""Builds the Iron Jetpacks content mod (`ironjetpacks.hmod` source tree).

Iron Jetpacks (BlakeBr0) is MIT: its textures, models and translations come from the
public repository (branch 1.21, the 8.0.x line that ATM10 ships), here read from SRC.

The original has ONE jetpack item and one item per component, told apart by a data
component (`ironjetpacks:jetpack_id`) and tinted at runtime. Horizon's content mods have
no runtime tint by component, so this port has one item per tier and part
(`ironjetpacks:iron_jetpack`, `iron_cell`, `iron_thruster`, `iron_capacitor`), with the
tint baked into each texture (vanilla's tint: the texture's RGB times the tier's colour).

The tiers and their numbers are ATM10's (config/ironjetpacks/jetpacks/*.json of the
AllTheMods/ATM-10 repository), limited to the materials the user's game has: vanilla,
Mekanism's steel (mekanism:ingot_steel) and All the Modium's three ingots. Bronze, silver,
electrum, invar and platinum (All the Ores, not ported) are left out. Recipes are the
mod's dynamic ones (DynamicRecipeManager), written out as files; recipes that need
Mekanism or All the Modium are gated with `neoforge:conditions` (mod_loaded).

usage: make_content.py SRC OUT
  SRC: the repository's src/main/resources
  OUT: the content tree (horizon-mod.toml, assets/, data/)
"""
import json
import shutil
import sys
from pathlib import Path

from PIL import Image

NS = "ironjetpacks"
VERSION = "8.0.11-horizon.1"

# name, tier, colour, armour, material ingredient, English, Portuguese, mod needed
TIERS = [
    ("wood", 0, 0x83663C, 1, "#minecraft:planks", "Wood", "Madeira", None),
    ("stone", 1, 0x7F7F7F, 2, ["minecraft:stone", "minecraft:andesite", "minecraft:diorite",
                                "minecraft:granite", "minecraft:tuff", "minecraft:deepslate"],
     "Stone", "Pedra", None),
    ("copper", 1, 0xEE825B, 2, "minecraft:copper_ingot", "Copper", "Cobre", None),
    ("iron", 2, 0xD8D8D8, 3, "minecraft:iron_ingot", "Iron", "Ferro", None),
    ("gold", 3, 0xF9EB59, 2, "minecraft:gold_ingot", "Gold", "Ouro", None),
    ("steel", 3, 0x565656, 3, "mekanism:ingot_steel", "Steel", "Aço", "mekanism"),
    ("diamond", 4, 0x4CF4E0, 4, "minecraft:diamond", "Diamond", "Diamante", None),
    ("emerald", 5, 0x4DD979, 4, "minecraft:emerald", "Emerald", "Esmeralda", None),
    ("allthemodium", 6, 0xFFEF0E, 9, "allthemodium:allthemodium_ingot", "Allthemodium", "Allthemodium", "allthemodium"),
    ("vibranium", 7, 0x26DE88, 12, "allthemodium:vibranium_ingot", "Vibranium", "Vibranium", "allthemodium"),
    ("unobtainium", 8, 0xD152E3, 15, "allthemodium:unobtainium_ingot", "Unobtainium", "Unobtainium", "allthemodium"),
]
CREATIVE = ("creative", -1, 0xCF1AE9, 8, None, "Creative", "Criativo", None)
COILS = ["basic_coil", "advanced_coil", "elite_coil", "ultimate_coil"]
COIL_MATERIAL = {"basic_coil": "minecraft:iron_ingot", "advanced_coil": "minecraft:gold_ingot",
                 "elite_coil": "minecraft:diamond", "ultimate_coil": "minecraft:emerald"}
PARTS = ["cell", "thruster", "capacitor"]


def coil_for(tier, tiers):
    """JetpackRegistry.getCoilForTier over the tiers ATM10 has (0..8)."""
    index = tiers.index(tier) / len(tiers)
    if index > 0.75:
        return "ultimate_coil"
    if index > 0.5:
        return "elite_coil"
    if index > 0.25:
        return "advanced_coil"
    return "basic_coil"


def tint(img, color):
    r, g, b = (color >> 16) & 255, (color >> 8) & 255, color & 255
    img = img.convert("RGBA")
    px = img.load()
    for y in range(img.height):
        for x in range(img.width):
            pr, pg, pb, pa = px[x, y]
            px[x, y] = (pr * r // 255, pg * g // 255, pb * b // 255, pa)
    return img


def write_json(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def item_model(out, name, texture):
    write_json(out / f"assets/{NS}/models/item/{name}.json",
               {"parent": "minecraft:item/generated", "textures": {"layer0": f"{NS}:item/{texture}"}})
    write_json(out / f"assets/{NS}/items/{name}.json",
               {"model": {"type": "minecraft:model", "model": f"{NS}:item/{name}"}})


def jetpack_item(armor, rarity):
    return {
        "template": "minecraft:iron_chestplate",
        "max_stack_size": 1,
        "rarity": rarity,
        "components": {
            # Worn in the chest slot; no asset_id: nothing is drawn on the body
            # (the original draws its own model).
            "minecraft:equippable": {"slot": "chest", "equip_sound": "minecraft:item.armor.equip_iron"},
            "minecraft:attribute_modifiers": [{
                "type": "minecraft:armor", "id": "ironjetpacks:armor.jetpack", "amount": float(armor),
                "operation": "add_value", "slot": "chest"}],
            # Unbreakable in the original; not enchantable (enchantableJetpacks = false).
            "!minecraft:max_damage": {},
            "!minecraft:damage": {},
            "!minecraft:repairable": {},
            "!minecraft:enchantable": {},
        },
    }


def recipe(pattern, key, result, category="misc", needs=()):
    r = {}
    needs = [m for m in needs if m]
    if needs:
        r["neoforge:conditions"] = [{"type": "neoforge:mod_loaded", "modid": m} for m in dict.fromkeys(needs)]
    r.update({"type": "minecraft:crafting_shaped", "category": category, "pattern": pattern, "key": key,
              "result": {"id": result, "count": 1}})
    return r


def main():
    src, out = Path(sys.argv[1]), Path(sys.argv[2])
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)
    tex = src / f"assets/{NS}/textures/item"
    (out / f"assets/{NS}/textures/item").mkdir(parents=True)
    (out / "horizon-mod.toml").write_text(
        f'id = "{NS}"\n'
        'name = "Iron Jetpacks"\n'
        f'version = "{VERSION}"\n'
        'description = "Jetpacks do Iron Jetpacks (tiers e numeros do ATM10) para Horizon. Precisa do mod nativo ironjetpacks_native."\n'
        'authors = ["BlakeBr0 (original, MIT)", "port Horizon (uso pessoal)"]\n'
        'api = "0.2"\n'
        'side = "both"\n'
        'capabilities = ["registries"]\n', encoding="utf-8")

    en = json.loads((src / f"assets/{NS}/lang/en_us.json").read_text(encoding="utf-8"))
    pt = json.loads((src / f"assets/{NS}/lang/pt_br.json").read_text(encoding="utf-8"))
    lang = {"en_us": {}, "pt_br": {}}
    for code, src_lang in (("en_us", en), ("pt_br", pt)):
        lang[code]["itemGroup.ironjetpacks"] = src_lang["itemGroup.ironjetpacks"]
        for k in ["strap"] + COILS:
            lang[code][f"item.{NS}.{k}"] = src_lang[f"item.{NS}.{k}"]

    items = {}
    # Strap and coils: the original's textures and models.
    for name in ["strap"] + COILS:
        shutil.copy(tex / f"{name}.png", out / f"assets/{NS}/textures/item/{name}.png")
        item_model(out, name, name)
        items[name] = {"template": "minecraft:iron_ingot", "max_stack_size": 64, "rarity": "common"}

    strap = Image.open(tex / "jetpack_strap.png").convert("RGBA")
    body = Image.open(tex / "jetpack.png").convert("RGBA")
    tier_numbers = sorted({t[1] for t in TIERS})
    order = []
    for name, tier, color, armor, material, en_name, pt_name, needs in TIERS + [CREATIVE]:
        jet = f"{name}_jetpack"
        img = strap.copy()
        img.alpha_composite(tint(body, color))
        img.save(out / f"assets/{NS}/textures/item/{jet}.png")
        item_model(out, jet, jet)
        items[jet] = jetpack_item(armor, "epic" if name == "creative" else "common")
        lang["en_us"][f"item.{NS}.{jet}"] = en["item.ironjetpacks.jetpack"].replace("%s", en_name)
        lang["pt_br"][f"item.{NS}.{jet}"] = pt["item.ironjetpacks.jetpack"].replace("%s", pt_name)
        order.append(jet)
        if name == "creative":
            continue
        for part in PARTS:
            pid = f"{name}_{part}"
            tint(Image.open(tex / f"{part}.png"), color).save(out / f"assets/{NS}/textures/item/{pid}.png")
            item_model(out, pid, pid)
            items[pid] = {"template": "minecraft:iron_ingot", "max_stack_size": 64, "rarity": "common"}
            lang["en_us"][f"item.{NS}.{pid}"] = en[f"item.ironjetpacks.{part}"].replace("%s", en_name)
            lang["pt_br"][f"item.{NS}.{pid}"] = pt[f"item.ironjetpacks.{part}"].replace("%s", pt_name)
            order.append(pid)

    for name, data in items.items():
        write_json(out / f"data/{NS}/horizon/item/{name}.json", data)
    for code, table in lang.items():
        write_json(out / f"assets/{NS}/lang/{code}.json", table)
    tab = ["strap"] + COILS + order
    write_json(out / f"data/{NS}/horizon/creative_tab/jetpacks.json",
               {"title": "itemGroup.ironjetpacks", "icon": f"{NS}:iron_jetpack", "items": [f"{NS}:{i}" for i in tab]})

    # Recipes: the jar's static ones (tags made vanilla items) and the dynamic ones.
    rec = out / f"data/{NS}/recipe"
    write_json(rec / "strap.json", recipe([" I ", "LLL", " I "],
               {"I": "minecraft:iron_nugget", "L": "minecraft:leather"}, f"{NS}:strap"))
    for coil in COILS:
        write_json(rec / f"{coil}.json", recipe([" DR", "DSD", "RD "],
                   {"R": "minecraft:redstone", "D": COIL_MATERIAL[coil], "S": "minecraft:stick"}, f"{NS}:{coil}"))
    lowest = min(tier_numbers)
    for name, tier, color, armor, material, en_name, pt_name, needs in TIERS:
        coil = f"{NS}:{coil_for(tier, tier_numbers)}"
        cell, thruster, capacitor = (f"{NS}:{name}_{p}" for p in PARTS)
        write_json(rec / f"{name}_cell.json", recipe([" R ", "MCM", " R "],
                   {"R": "minecraft:redstone", "M": material, "C": coil}, cell, needs=[needs]))
        write_json(rec / f"{name}_thruster.json", recipe(["MCM", "CXC", "MFM"],
                   {"M": material, "C": coil, "X": cell, "F": "minecraft:furnace"}, thruster, needs=[needs]))
        write_json(rec / f"{name}_capacitor.json", recipe(["MXM", "MXM", "MXM"],
                   {"M": material, "X": cell}, capacitor, needs=[needs]))
        jet = f"{NS}:{name}_jetpack"
        if tier == lowest:
            write_json(rec / f"{name}_jetpack.json", recipe(["MPM", "MSM", "T T"],
                       {"M": material, "P": capacitor, "S": f"{NS}:strap", "T": thruster}, jet, "equipment", [needs]))
        else:
            below = [f"{NS}:{t[0]}_jetpack" for t in TIERS if t[1] == tier - 1]
            write_json(rec / f"{name}_jetpack.json", recipe(["MPM", "MJM", "T T"],
                       {"M": material, "P": capacitor, "J": below[0] if len(below) == 1 else below, "T": thruster},
                       jet, "equipment", [needs]))
    print(f"{len(items)} items, {len(list(rec.glob('*.json')))} recipes -> {out}")


if __name__ == "__main__":
    main()

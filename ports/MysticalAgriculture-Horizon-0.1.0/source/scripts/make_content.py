#!/usr/bin/env python3
"""Mystical Agriculture + Mystical Agradditions for Horizon 26.3: the two content mods.

Everything (textures, models, lang, the original recipes and loot) comes from the two jars of the
player's ATM10 (MIT, BlakeBr0), converted to 26.3's formats. The ids stay the original's
(`mysticalagriculture:`, `mysticalagradditions:`), so a JEI lookup or a quest reads the same.

usage: make_content.py MA_DIR MAA_DIR OUT_DIR
  MA_DIR / MAA_DIR: the jars unzipped (their assets/ and data/).
  OUT_DIR: gets mysticalagriculture/ and mysticalagradditions/ (each a content mod folder).

Also checks that the native mod's tables (runtime.rs and the two mod_content helpers) name the
same crops, farmlands and essences as the content, so the two halves stay in step.
"""
import json
import re
import shutil
import sys
from pathlib import Path

from PIL import Image

MA, MAA = "mysticalagriculture", "mysticalagradditions"
VERSION = "8.0.28-horizon.1"
VERSION_MAA = "8.0.14-horizon.1"

TIERS = ["inferium", "prudentium", "tertium", "imperium", "supremium"]  # 1..5 (MA)
# Tier 6 is Agradditions' insanium.

# The crops: name -> (tier, material ingredient). Material: an item id or "#tag".
# Inferium is special (its seed recipe is its own, and its drops grow with the farmland).
CROPS = {
    "inferium": (1, None),
    "stone": (1, "minecraft:stone"),
    "dirt": (1, "minecraft:dirt"),
    "wood": (1, "#minecraft:logs"),
    "ice": (1, "minecraft:ice"),
    "deepslate": (1, "minecraft:deepslate"),
    "nature": (2, f"{MA}:nature_agglomeratio"),
    "dye": (2, f"{MA}:dye_agglomeratio"),
    "nether": (2, f"{MA}:nether_agglomeratio"),
    "coal": (2, "minecraft:coal"),
    "coral": (2, f"{MA}:coral_agglomeratio"),
    "honey": (2, f"{MA}:honey_agglomeratio"),
    "amethyst": (2, "minecraft:amethyst_shard"),
    "iron": (3, "minecraft:iron_ingot"),
    "copper": (3, "minecraft:copper_ingot"),
    "nether_quartz": (3, "minecraft:quartz"),
    "glowstone": (3, "minecraft:glowstone_dust"),
    "redstone": (3, "minecraft:redstone"),
    "obsidian": (3, "minecraft:obsidian"),
    "prismarine": (3, f"{MA}:prismarine_agglomeratio"),
    "sculk": (3, "minecraft:sculk"),
    "gold": (4, "minecraft:gold_ingot"),
    "lapis_lazuli": (4, "minecraft:lapis_lazuli"),
    "end": (4, f"{MA}:end_agglomeratio"),
    "osmium": (4, "mekanism:ingot_osmium"),
    "steel": (4, "mekanism:ingot_steel"),
    "diamond": (5, "minecraft:diamond"),
    "emerald": (5, "minecraft:emerald"),
    "netherite": (5, "minecraft:netherite_ingot"),
    # Agradditions (tier 6): their crops live in mysticalagriculture's namespace, as in the original.
    "nether_star": (6, "minecraft:nether_star"),
    "dragon_egg": (6, f"{MAA}:dragon_scale"),
}
CRUX = {"nether_star": f"{MAA}:nether_star_crux", "dragon_egg": f"{MAA}:dragon_egg_crux"}
AGGLOMERATIOS = ["nature", "dye", "nether", "coral", "honey", "prismarine", "end"]

# Items of other mods the user has ported (Mekanism for Horizon): recipes may name them.
OTHER_ITEMS = {"mekanism:ingot_osmium", "mekanism:ingot_steel"}
# `c:` tags no Horizon content defines: the vanilla (or ported) item each one means here.
C_TAGS = {
    "c:gems/diamond": "minecraft:diamond", "c:gems/emerald": "minecraft:emerald",
    "c:gems/quartz": "minecraft:quartz", "c:gems/lapis": "minecraft:lapis_lazuli",
    "c:gems/amethyst": "minecraft:amethyst_shard",
    "c:ingots/iron": "minecraft:iron_ingot", "c:ingots/gold": "minecraft:gold_ingot",
    "c:ingots/copper": "minecraft:copper_ingot", "c:ingots/netherite": "minecraft:netherite_ingot",
    "c:ingots/osmium": "mekanism:ingot_osmium", "c:ingots/steel": "mekanism:ingot_steel",
    "c:dusts/redstone": "minecraft:redstone", "c:dusts/glowstone": "minecraft:glowstone_dust",
    "c:storage_blocks/diamond": "minecraft:diamond_block",
    "c:storage_blocks/iron": "minecraft:iron_block", "c:storage_blocks/gold": "minecraft:gold_block",
    "c:nether_stars": "minecraft:nether_star", "c:rods/blaze": "minecraft:blaze_rod",
    "c:ender_pearls": "minecraft:ender_pearl", "c:gunpowders": "minecraft:gunpowder",
    "c:slime_balls": "minecraft:slime_ball", "c:leathers": "minecraft:leather",
    "c:strings": "minecraft:string", "c:feathers": "minecraft:feather",
    "c:obsidians": "minecraft:obsidian", "c:bones": "minecraft:bone",
    "c:eggs": "minecraft:egg", "c:crops/wheat": "minecraft:wheat",
    "c:glass_blocks/colorless": "minecraft:glass", "c:chests/wooden": "minecraft:chest",
    "c:stones": "minecraft:stone", "c:cobblestones": "minecraft:cobblestone",
}
for _dye in ["white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
             "light_gray", "cyan", "purple", "blue", "brown", "green", "red", "black"]:
    C_TAGS[f"c:dyes/{_dye}"] = f"minecraft:{_dye}_dye"
VANILLA_TAGS = {"minecraft:logs", "minecraft:planks", "minecraft:wool", "minecraft:saplings",
                "minecraft:leaves", "minecraft:hoes"}

LOG = []


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(data, (dict, list)):
        data = json.dumps(data, indent=2, ensure_ascii=False) + "\n"
    path.write_text(data, encoding="utf-8")


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


class Mod:
    def __init__(self, ns, out):
        self.ns, self.out = ns, out / ns
        self.data, self.assets = self.out / "data" / ns, self.out / "assets" / ns
        self.blocks, self.items = [], []  # declared ids (without namespace), in tab order
        self.tags = {}  # ("block"|"item", "ns:path") -> [values]
        self.lang = {"en_us": {}, "pt_br": {}}

    def tag(self, kind, name, value):
        self.tags.setdefault((kind, name), []).append(value)

    def block(self, name, decl, loot=None, item=True):
        if not item:
            decl["item"] = False
        write(self.data / "horizon/block" / f"{name}.json", decl)
        if loot is not None:
            write(self.data / "loot_table/blocks" / f"{name}.json", loot)
        self.blocks.append((name, item))

    def item(self, name, decl):
        write(self.data / "horizon/item" / f"{name}.json", decl)
        self.items.append(name)

    def ids(self):
        return {f"{self.ns}:{b}" for b, it in self.blocks if it} | {f"{self.ns}:{i}" for i in self.items}

    def block_ids(self):
        return {f"{self.ns}:{b}" for b, _ in self.blocks}


def copy_tex(src_root, rel, dst_mod, dst_rel=None):
    """Copies textures/<rel>.png (and its .mcmeta) into the mod's textures/<dst_rel>."""
    dst_rel = dst_rel or rel
    for ext in (".png", ".png.mcmeta"):
        f = src_root / "textures" / f"{rel}{ext}"
        if f.exists():
            d = dst_mod.assets / "textures" / f"{dst_rel}{ext}"
            d.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(f, d)
        elif ext == ".png":
            raise SystemExit(f"missing texture {f}")


def item_model(mod, name, texture):
    write(mod.assets / "models/item" / f"{name}.json",
          {"parent": "minecraft:item/generated", "textures": {"layer0": texture}})
    write(mod.assets / "items" / f"{name}.json",
          {"model": {"type": "minecraft:model", "model": f"{mod.ns}:item/{name}"}})


def block_item_model(mod, name, model=None):
    write(mod.assets / "items" / f"{name}.json",
          {"model": {"type": "minecraft:model", "model": model or f"{mod.ns}:block/{name}"}})


def self_loot(ns, block):
    return {"type": "minecraft:block", "random_sequence": f"{ns}:blocks/{block}", "pools": [{
        "rolls": 1.0, "bonus_rolls": 0.0, "conditions": [{"condition": "minecraft:survives_explosion"}],
        "entries": [{"type": "minecraft:item", "name": f"{ns}:{block}"}]}]}


def cube_block(mod, src_assets, name, template, props=None):
    """A full block whose blockstate and model come from the jar."""
    for kind in ("blockstates", "models/block"):
        f = src_assets / kind / f"{name}.json"
        write(mod.assets / kind / f"{name}.json", strip_model(load(f)))
    for tex in textures_of(load(src_assets / "models/block" / f"{name}.json")):
        copy_texture_ref(tex, mod)
    decl = {"template": template}
    if props:
        decl["properties"] = props
    mod.block(name, decl, self_loot(mod.ns, name))
    block_item_model(mod, name)


def strip_model(m):
    return m


def textures_of(model):
    return [t for t in model.get("textures", {}).values() if not t.startswith("#")]


SRC = {}


def copy_texture_ref(ref, mod):
    """`ns:block/x` (or `block/x`, vanilla) used by a copied model: copied when it is a mod's."""
    if ":" not in ref or ref.startswith("minecraft:"):
        return
    ns, rel = ref.split(":", 1)
    src = SRC[ns]
    target = mod if ns == mod.ns else MODS[ns]
    copy_tex(src, rel, target)


MODS = {}


# ---- recipes -----------------------------------------------------------------------------

def known_item(i):
    return i.startswith("minecraft:") or i in OTHER_ITEMS or any(i in m.ids() for m in MODS.values())


def known_tag(t):
    return t in VANILLA_TAGS or any(("item", t) in m.tags for m in MODS.values())


def ingredient(v):
    """A 1.21.1 ingredient as 26.3 writes it: "ns:item" or "#ns:tag" (or a list of them)."""
    if isinstance(v, list):
        out = [ingredient(x) for x in v]
        return out if all(x is not None for x in out) else None
    if isinstance(v, str):
        return ingredient({"tag": v[1:]} if v.startswith("#") else {"item": v})
    if set(v) == {"item"}:
        return v["item"] if known_item(v["item"]) else None
    if set(v) == {"tag"}:
        t = v["tag"]
        if t in C_TAGS:
            return C_TAGS[t] if known_item(C_TAGS[t]) else None
        return f"#{t}" if known_tag(t) else None
    return None


def result(v):
    if "tag" in v:
        item = C_TAGS.get(v["tag"])
    else:
        item = v["id"]
    if item is None or not known_item(item):
        return None
    out = {"id": item}
    if v.get("count", 1) != 1:
        out["count"] = v["count"]
    return out


def convert_recipe(r):
    """A 1.21.1 recipe in 26.3's format, or None (and why) when it names what Horizon lacks."""
    t = r.get("type", "")
    t = t if ":" in t else f"minecraft:{t}"
    if t in ("cucumber:shaped_no_mirror", "cucumber:shaped_tag"):
        t = "minecraft:crafting_shaped"
    if t == "minecraft:crafting_shaped":
        key = {}
        for k, v in r["key"].items():
            key[k] = ingredient(v)
            if key[k] is None:
                return None, f"ingredient {v}"
        out = {"type": t, "category": r.get("category", "misc"), "pattern": r["pattern"], "key": key}
    elif t == "minecraft:crafting_shapeless":
        ings = [ingredient(v) for v in r["ingredients"]]
        if any(i is None for i in ings):
            return None, f"ingredients {r['ingredients']}"
        out = {"type": t, "category": r.get("category", "misc"), "ingredients": ings}
    elif t in ("minecraft:smelting", "minecraft:blasting"):
        ing = ingredient(r["ingredient"])
        if ing is None:
            return None, f"ingredient {r['ingredient']}"
        out = {"type": t, "category": "misc", "ingredient": ing, "experience": r.get("experience", 0.1),
               "cookingtime": r.get("cookingtime", 200 if t.endswith("smelting") else 100)}
    else:
        return None, f"type {t}"
    if "group" in r:
        out["group"] = r["group"]
    res = result(r["result"])
    if res is None:
        return None, f"result {r['result']}"
    out["result"] = res
    return out, None


def add_recipes(mod, src_dir, names=None, prefix=""):
    """Converts every recipe file under src_dir (recursively); those naming missing items are left out."""
    files = sorted(src_dir.rglob("*.json")) if names is None else [src_dir / f"{n}.json" for n in names]
    kept = 0
    for f in files:
        r, why = convert_recipe(load(f))
        rel = f.relative_to(src_dir).with_suffix("").as_posix()
        if r is None:
            LOG.append(f"recipe {mod.ns}:{prefix}{rel} left out: {why}")
            continue
        write(mod.data / "recipe" / f"{prefix}{rel}.json", r)
        kept += 1
    return kept


# ---- the mods ----------------------------------------------------------------------------

def farmland_textures(mod, name, base_png):
    """Essence farmland: the original tints vanilla's farmland in code (no tint hook in Horizon),
    so the top is drawn here from the tier's block texture, furrowed like farmland."""
    base = Image.open(base_png).convert("RGBA").resize((16, 16), Image.NEAREST)
    for suffix, k in (("", 0.85), ("_moist", 0.55)):
        img = Image.new("RGBA", (16, 16))
        for y in range(16):
            row = 0.6 if y % 4 == 3 else (1.1 if y % 4 == 0 else 1.0)
            for x in range(16):
                r, g, b, _ = base.getpixel((x, y))
                f = k * row
                img.putpixel((x, y), (min(255, int(r * f)), min(255, int(g * f)), min(255, int(b * f)), 255))
        d = mod.assets / "textures/block" / f"{name}_top{suffix}.png"
        d.parent.mkdir(parents=True, exist_ok=True)
        img.save(d)


def farmland(mod, name, ma_assets, essence, block_png):
    farmland_textures(mod, name, block_png)
    template = load(ma_assets / "models/block/infused_farmland.json")
    for suffix in ("", "_moist"):
        m = json.loads(json.dumps(template))
        m["textures"] = {"particle": "minecraft:block/dirt", "dirt": "minecraft:block/dirt",
                         "top": f"{mod.ns}:block/{name}_top{suffix}"}
        for face in m["elements"][0]["faces"].values():
            face.pop("tintindex", None)
        write(mod.assets / "models/block" / f"{name}{suffix}.json", m)
    write(mod.assets / "blockstates" / f"{name}.json", {"variants": {
        f"moisture={i}": {"model": f"{mod.ns}:block/{name}{'_moist' if i == 7 else ''}"} for i in range(8)}})
    block_item_model(mod, name)
    silk = {"condition": "minecraft:match_tool", "predicate": {"predicates": {
        "minecraft:enchantments": [{"enchantments": "minecraft:silk_touch", "levels": {"min": 1}}]}}}
    loot = {"type": "minecraft:block", "random_sequence": f"{mod.ns}:blocks/{name}", "pools": [
        {"rolls": 1.0, "bonus_rolls": 0.0, "entries": [{"type": "minecraft:alternatives", "children": [
            {"type": "minecraft:item", "conditions": [silk], "name": f"{mod.ns}:{name}"},
            {"type": "minecraft:item", "name": "minecraft:dirt"}]}]},
        {"rolls": 1.0, "bonus_rolls": 0.0,
         "conditions": [{"condition": "minecraft:inverted", "term": silk},
                        {"condition": "minecraft:random_chance", "chance": 0.25}],
         "entries": [{"type": "minecraft:item", "name": essence}]}]}
    mod.block(name, {"template": "minecraft:farmland"}, loot)
    mod.tag("block", "minecraft:supports_crops", f"{mod.ns}:{name}")
    # `CropBlock.getGrowthSpeed` (Horizon: level/tick.rs crop_growth_speed) counts only
    # `#minecraft:grows_crops` under the crop: without it a crop on essence farmland grows as on
    # dirt-level speed (1 instead of 3 when moist), much slower than the original.
    mod.tag("block", "minecraft:grows_crops", f"{mod.ns}:{name}")
    mod.tag("block", "minecraft:mineable/shovel", f"{mod.ns}:{name}")
    MODS[MA].tag("item", f"{MA}:farmland", f"{mod.ns}:{name}")


def ore_loot(ns, block, src_loot):
    loot = load(src_loot)
    loot.setdefault("random_sequence", f"{ns}:blocks/{block}")
    for pool in loot["pools"]:
        pool.pop("name", None)
    return loot


def ore(mod, src_assets, src_data, name, template):
    for kind in ("blockstates", "models/block"):
        write(mod.assets / kind / f"{name}.json", load(src_assets / kind / f"{name}.json"))
    for tex in textures_of(load(src_assets / "models/block" / f"{name}.json")):
        copy_texture_ref(tex, mod)
    mod.block(name, {"template": template, "properties": {"hardness": 3.0}},
              ore_loot(mod.ns, name, src_data / "loot_table/blocks" / f"{name}.json"))
    block_item_model(mod, name)


def crop(ma, maa, name, tier, ma_src, maa_src):
    """The crop block (wheat's class and states), its seeds, its essence."""
    agr = tier == 6
    src_assets = (maa_src if agr else ma_src) / "assets" / MA
    tex_root = (maa_src / "assets" / MAA) if agr else (ma_src / "assets" / MA)
    SRC_TEX = {"inferium": ("item/inferium_essence", "item/seeds/inferium_seeds")}
    essence = f"{MA}:inferium_essence" if name == "inferium" else f"{MA}:{name}_essence"
    # Block: states and drops; its blockstate and age-7 model are the jar's.
    write(ma.assets / "blockstates" / f"{name}_crop.json", load(src_assets / "blockstates" / f"{name}_crop.json"))
    model = src_assets / "models/block" / f"{name}_crop.json"
    m = load(model) if model.exists() else {"parent": f"{MA}:block/mystical_resource_crop_7",
                                             "textures": {"flower": f"{MA}:block/flower/{name}_flower"}}
    m["textures"]["flower"] = f"{MA}:block/flower/{name}_flower"
    write(ma.assets / "models/block" / f"{name}_crop.json", m)
    copy_tex(tex_root, f"block/flower/{name}_flower", ma)
    loot = {"type": "minecraft:block", "random_sequence": f"{MA}:blocks/{name}_crop", "pools": [
        {"rolls": 1.0, "bonus_rolls": 0.0, "entries": [{"type": "minecraft:item", "name": f"{MA}:{name}_seeds"}]},
        {"rolls": 1.0, "bonus_rolls": 0.0,
         "conditions": [{"condition": "minecraft:block_state_property", "block": f"{MA}:{name}_crop",
                         "properties": {"age": "7"}}],
         "entries": [{"type": "minecraft:item", "name": essence}]}]}
    ma.block(f"{name}_crop", {"template": "minecraft:wheat"}, loot, item=False)
    ma.tag("block", "minecraft:crops", f"{MA}:{name}_crop")
    ma.tag("block", "minecraft:maintains_farmland", f"{MA}:{name}_crop")
    ma.tag("block", f"{MA}:crops", f"{MA}:{name}_crop")
    # Seeds: wheat seeds' item; the native mod points them at the crop.
    ma.item(f"{name}_seeds", {"template": "minecraft:wheat_seeds"})
    seeds_tex = SRC_TEX.get(name, (None, f"item/seeds/{name}_seeds"))[1]
    copy_tex(tex_root, seeds_tex, ma, f"item/seeds/{name}_seeds")
    item_model(ma, f"{name}_seeds", f"{MA}:item/seeds/{name}_seeds")
    ma.tag("item", f"{MA}:seeds", f"{MA}:{name}_seeds")
    # Essence (inferium's is the tier's own, declared with the tiers).
    if name != "inferium":
        ma.item(f"{name}_essence", {"template": "minecraft:flint"})
        copy_tex(tex_root, f"item/essence/{name}_essence", ma, f"item/essence/{name}_essence")
        item_model(ma, f"{name}_essence", f"{MA}:item/essence/{name}_essence")
        ma.tag("item", f"{MA}:essences", f"{MA}:{name}_essence")


def lang(ma, maa, ma_src, maa_src):
    for code in ("en_us", "pt_br"):
        src = load(ma_src / f"assets/{MA}/lang/{code}.json")
        srca = load(maa_src / f"assets/{MAA}/lang/{code}.json")
        for mod, table in ((ma, src), (maa, srca)):
            out = mod.lang[code]
            for b, _ in mod.blocks:
                k = f"block.{mod.ns}.{b}"
                if k in table:
                    out[k] = table[k]
            for i in mod.items:
                k = f"item.{mod.ns}.{i}"
                if k in table:
                    out[k] = table[k]
        for name, (tier, _) in CROPS.items():
            cname = (srca if tier == 6 else src).get(f"crop.{MAA if tier == 6 else MA}.{name}")
            if cname is None:
                cname = load(ma_src / f"assets/{MA}/lang/en_us.json").get(f"crop.{MA}.{name}") or \
                    load(maa_src / f"assets/{MAA}/lang/en_us.json")[f"crop.{MAA}.{name}"]
            out = ma.lang[code]
            out[f"block.{MA}.{name}_crop"] = src["block.mysticalagriculture.mystical_crop"].replace("%s", cname)
            out[f"item.{MA}.{name}_seeds"] = src["item.mysticalagriculture.mystical_seeds"].replace("%s", cname)
            if name != "inferium":
                out[f"item.{MA}.{name}_essence"] = src["item.mysticalagriculture.mystical_essence"].replace("%s", cname)
        ma.lang[code]["itemGroup.mysticalagriculture.mysticalagriculture"] = "Mystical Agriculture"
        maa.lang[code]["itemGroup.mysticalagradditions.mysticalagradditions"] = "Mystical Agradditions"


def main():
    ma_src, maa_src, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    for ns in (MA, MAA):
        if (out / ns).exists():
            shutil.rmtree(out / ns)
    ma, maa = Mod(MA, out), Mod(MAA, out)
    MODS.update({MA: ma, MAA: maa})
    SRC.update({MA: ma_src / "assets" / MA, MAA: maa_src / "assets" / MAA})
    ma_assets, maa_assets = ma_src / "assets" / MA, maa_src / "assets" / MAA
    ma_data, maa_data = ma_src / "data" / MA, maa_src / "data" / MAA

    write(ma.out / "horizon-mod.toml", f'''id = "{MA}"
name = "Mystical Agriculture"
version = "{VERSION}"
description = "Sementes que crescem recursos: essencias de Inferium a Supremium, terras de essencia, sementes de recurso. Port do Mystical Agriculture (BlakeBr0, MIT). Precisa do mod nativo mysticalagriculture_native."
authors = ["BlakeBr0 (original, MIT)", "port Horizon"]
api = "0.2"
side = "both"
capabilities = ["registries"]
''')
    write(maa.out / "horizon-mod.toml", f'''id = "{MAA}"
name = "Mystical Agradditions"
version = "{VERSION_MAA}"
description = "Tier 6 (Insanium) do Mystical Agriculture: essencia, lingote, terra, cruxes, minerios no Nether e no End. Port do Mystical Agradditions (BlakeBr0, MIT). Precisa do mysticalagriculture."
authors = ["BlakeBr0 (original, MIT)", "port Horizon"]
api = "0.2"
side = "both"
capabilities = ["registries"]
''')

    # Tiers 1-5: essence, ingot, nugget, gemstone, their blocks, farmland.
    for t in TIERS:
        for kind, template in (("essence", "minecraft:flint"), ("ingot", "minecraft:brick"),
                               ("nugget", "minecraft:iron_nugget"), ("gemstone", "minecraft:flint")):
            ma.item(f"{t}_{kind}", {"template": template})
            copy_tex(ma_assets, f"item/{t}_{kind}", ma)
            item_model(ma, f"{t}_{kind}", f"{MA}:item/{t}_{kind}")
        ma.tag("item", f"{MA}:essences", f"{MA}:{t}_essence")
        for b, template in ((f"{t}_block", "minecraft:iron_block"), (f"{t}_ingot_block", "minecraft:iron_block"),
                            (f"{t}_gemstone_block", "minecraft:diamond_block")):
            cube_block(ma, ma_assets, b, template)
            ma.tag("block", "minecraft:mineable/pickaxe", f"{MA}:{b}")
        farmland(ma, f"{t}_farmland", ma_assets, f"{MA}:{t}_essence", ma_assets / f"textures/block/{t}_block.png")
    # Prosperity.
    for kind, template in (("shard", "minecraft:flint"), ("ingot", "minecraft:brick"),
                           ("nugget", "minecraft:iron_nugget"), ("gemstone", "minecraft:flint"),
                           ("seed_base", "minecraft:flint")):
        ma.item(f"prosperity_{kind}", {"template": template})
        copy_tex(ma_assets, f"item/prosperity_{kind}", ma)
        item_model(ma, f"prosperity_{kind}", f"{MA}:item/prosperity_{kind}")
    for b, template in (("prosperity_block", "minecraft:iron_block"), ("prosperity_ingot_block", "minecraft:iron_block"),
                        ("prosperity_gemstone_block", "minecraft:diamond_block")):
        cube_block(ma, ma_assets, b, template)
        ma.tag("block", "minecraft:mineable/pickaxe", f"{MA}:{b}")
    # Ores (overworld).
    for name, template in (("inferium_ore", "minecraft:coal_ore"), ("deepslate_inferium_ore", "minecraft:deepslate_coal_ore"),
                           ("prosperity_ore", "minecraft:coal_ore"), ("deepslate_prosperity_ore", "minecraft:deepslate_coal_ore")):
        ore(ma, ma_assets, ma_data, name, template)
        ma.tag("block", "minecraft:mineable/pickaxe", f"{MA}:{name}")
        metal = "inferium" if "inferium" in name else "prosperity"
        ma.tag("block", f"c:ores/{metal}", f"{MA}:{name}")
        ma.tag("item", f"c:ores/{metal}", f"{MA}:{name}")
    # Crystals, fertilizers, agglomeratios.
    ma.item("infusion_crystal", {"template": "minecraft:flint",
                                 "max_stack_size": 1, "components": {"minecraft:max_damage": 1000}})
    ma.item("master_infusion_crystal", {"template": "minecraft:flint", "max_stack_size": 1, "rarity": "epic",
                                        "crafting_remainder": f"{MA}:master_infusion_crystal"})
    for c in ("infusion_crystal", "master_infusion_crystal"):
        copy_tex(ma_assets, f"item/{c}", ma)
        item_model(ma, c, f"{MA}:item/{c}")
        ma.tag("item", f"{MA}:infusion_crystals", f"{MA}:{c}")
    for f in ("fertilized_essence", "mystical_fertilizer"):
        ma.item(f, {"template": "minecraft:bone_meal"})
        copy_tex(ma_assets, f"item/{f}", ma)
        item_model(ma, f, f"{MA}:item/{f}")
    for a in AGGLOMERATIOS:
        ma.item(f"{a}_agglomeratio", {"template": "minecraft:flint"})
        copy_tex(ma_assets, f"item/{a}_agglomeratio", ma)
        item_model(ma, f"{a}_agglomeratio", f"{MA}:item/{a}_agglomeratio")
    # Crop models shared by every crop.
    for i in range(8):
        for kind in ("resource",):
            n = f"mystical_{kind}_crop_{i}"
            write(ma.assets / "models/block" / f"{n}.json", load(ma_assets / "models/block" / f"{n}.json"))
            copy_tex(ma_assets, f"block/{n}", ma)

    # Agradditions: insanium, ores, cruxes, boss drops.
    for kind, template in (("essence", "minecraft:flint"), ("ingot", "minecraft:brick"),
                           ("nugget", "minecraft:iron_nugget"), ("gemstone", "minecraft:flint")):
        maa.item(f"insanium_{kind}", {"template": template})
        copy_tex(maa_assets, f"item/insanium_{kind}", maa)
        item_model(maa, f"insanium_{kind}", f"{MAA}:item/insanium_{kind}")
    ma.tag("item", f"{MA}:essences", f"{MAA}:insanium_essence")  # the fence: only ma writes its own tags
    for b, template in (("insanium_block", "minecraft:iron_block"), ("insanium_ingot_block", "minecraft:iron_block"),
                        ("insanium_gemstone_block", "minecraft:diamond_block")):
        cube_block(maa, maa_assets, b, template)
        maa.tag("block", "minecraft:mineable/pickaxe", f"{MAA}:{b}")
    farmland(maa, "insanium_farmland", ma_assets, f"{MAA}:insanium_essence", maa_assets / "textures/block/insanium_block.png")
    for name, template in (("nether_inferium_ore", "minecraft:nether_gold_ore"), ("nether_prosperity_ore", "minecraft:nether_gold_ore"),
                           ("end_inferium_ore", "minecraft:coal_ore"), ("end_prosperity_ore", "minecraft:coal_ore")):
        ore(maa, maa_assets, maa_data, name, template)
        maa.tag("block", "minecraft:mineable/pickaxe", f"{MAA}:{name}")
        metal = "inferium" if "inferium" in name else "prosperity"
        maa.tag("block", f"c:ores/{metal}", f"{MAA}:{name}")
        maa.tag("item", f"c:ores/{metal}", f"{MAA}:{name}")
    for name in ("nether_star_crux", "dragon_egg_crux"):
        cube_block(maa, maa_assets, name, "minecraft:obsidian", {"hardness": 5.0, "explosion_resistance": 1200.0})
        maa.tag("block", "minecraft:mineable/pickaxe", f"{MAA}:{name}")
    for name in ("withering_soul", "dragon_scale", "nether_star_shard", "dragon_egg_chunk"):
        maa.item(name, {"template": "minecraft:flint"})
        copy_tex(maa_assets, f"item/{name}", maa)
        item_model(maa, name, f"{MAA}:item/{name}")

    # The crops.
    for name, (tier, _) in CROPS.items():
        crop(ma, maa, name, tier, ma_src, maa_src)

    # Tool tiers, from the jars' own tags (blocks this port has).
    for mod, src in ((ma, ma_src), (maa, maa_src)):
        for need in ("needs_stone_tool", "needs_iron_tool"):
            f = src / "data/minecraft/tags/block" / f"{need}.json"
            for v in load(f)["values"]:
                if v in mod.block_ids():
                    mod.tag("block", f"minecraft:{need}", v)
        maa_needs = maa_src / "data/minecraft/tags/block"
    for v in load(maa_src / "data/minecraft/tags/block/needs_iron_tool.json")["values"] + \
            load(maa_src / "data/minecraft/tags/block/needs_stone_tool.json")["values"]:
        pass

    # Recipes: the jars' own, converted; those naming what Horizon lacks stay out (logged).
    root = [f.stem for f in sorted((ma_data / "recipe").glob("*.json"))]
    n = add_recipes(ma, ma_data / "recipe", root)
    # Seeds: the original's "seedCraftingRecipes" option (MEM/ESE/MEM on a crafting table)
    # stands in for the Infusion Altar, which this version doesn't have (LEIA-ME).
    for name, (tier, material) in CROPS.items():
        if name == "inferium":
            continue
        essence = f"{MAA}:insanium_essence" if tier == 6 else f"{MA}:{TIERS[tier - 1]}_essence"
        mat = ingredient(material)
        if mat is None:
            LOG.append(f"seed recipe {name}: material {material} unknown")
            continue
        write(ma.data / "recipe/seed" / f"{name}.json", {
            "type": "minecraft:crafting_shaped", "category": "misc", "group": f"{MA}:seeds",
            "pattern": ["MEM", "ESE", "MEM"],
            "key": {"M": mat, "E": essence, "S": f"{MA}:prosperity_seed_base"},
            "result": {"id": f"{MA}:{name}_seeds"}})
        n += 1
    # Essence -> resources: every recipe whose essences are crops this port has.
    crops_here = {f"{MA}:{c}_essence" for c in CROPS} | {f"{MA}:inferium_essence"}
    for f in sorted((ma_data / "recipe/essence").rglob("*.json")) + sorted((maa_data / "recipe/essence").rglob("*.json")):
        text = f.read_text(encoding="utf-8")
        used = set(re.findall(r'"(mysticalagriculture:[a-z_]+_essence)"', text))
        if not used or not used <= crops_here:
            continue
        r, why = convert_recipe(load(f))
        mod = maa if MAA in f.parts else ma
        rel = f.relative_to(ma_data / "recipe" if mod is ma else maa_data / "recipe").with_suffix("").as_posix()
        if r is None:
            LOG.append(f"recipe {mod.ns}:{rel} left out: {why}")
            continue
        write(mod.data / "recipe" / f"{rel}.json", r)
        n += 1
    nmaa = add_recipes(maa, maa_data / "recipe", [f.stem for f in sorted((maa_data / "recipe").glob("*.json"))])

    # Worldgen: the originals' features, in the vanilla dimensions (new chunks only).
    for mod, src, names, biomes in (
            (ma, ma_data, ("inferium_ore", "prosperity_ore"), "#minecraft:is_overworld"),
            (maa, maa_data, ("nether_inferium_ore", "nether_prosperity_ore"), "#minecraft:is_nether"),
            (maa, maa_data, ("end_inferium_ore", "end_prosperity_ore"), "#minecraft:is_end")):
        for name in names:
            for kind in ("configured_feature", "placed_feature"):
                write(mod.data / "worldgen" / kind / f"{name}.json", load(src / "worldgen" / kind / f"{name}.json"))
            write(mod.data / "neoforge/biome_modifier" / f"{name}.json", {
                "type": "neoforge:add_features", "biomes": biomes,
                "features": f"{mod.ns}:{name}", "step": "underground_ores"})

    # Tags.
    for mod in (ma, maa):
        for (kind, name), values in mod.tags.items():
            ns, path = name.split(":", 1)
            write(mod.out / "data" / ns / "tags" / kind / f"{path}.json",
                  {"replace": False, "values": sorted(set(values), key=values.index)})

    lang(ma, maa, ma_src, maa_src)
    for mod in (ma, maa):
        for code, table in mod.lang.items():
            write(mod.assets / "lang" / f"{code}.json", table)
        order = [f"{mod.ns}:{i}" for i in mod.items] + [f"{mod.ns}:{b}" for b, it in mod.blocks if it]
        write(mod.data / "horizon/creative_tab" / f"{mod.ns}.json", {
            "title": f"itemGroup.{mod.ns}", "icon": f"{mod.ns}:{'inferium_essence' if mod is ma else 'insanium_essence'}",
            "items": order})
        mod.lang["en_us"][f"itemGroup.{mod.ns}"] = mod.lang["en_us"].pop(f"itemGroup.{mod.ns}.{mod.ns}")
        mod.lang["pt_br"][f"itemGroup.{mod.ns}"] = mod.lang["pt_br"].pop(f"itemGroup.{mod.ns}.{mod.ns}")
        for code, table in mod.lang.items():
            write(mod.assets / "lang" / f"{code}.json", table)
        for f in ("LICENSE",):
            pass
        write(mod.out / "LICENSE-original.txt", MIT)

    check_native(Path(__file__).resolve().parent.parent / "native/mixins/mysticalagriculture_native", ma, maa)
    report = out / "content-report.txt"
    write(report, "\n".join(LOG) + "\n")
    print(f"{MA}: {len(ma.blocks)} blocks, {len(ma.items)} items, {n} recipes")
    print(f"{MAA}: {len(maa.blocks)} blocks, {len(maa.items)} items, {nmaa} recipes")
    print(f"{len(LOG)} notes in {report}")


def check_native(native, ma, maa):
    """The native tables name only what the content declares, and every crop."""
    if not (native / "runtime.rs").exists():
        print("native folder not found: skipped the cross-check")
        return
    ids = ma.ids() | maa.ids() | ma.block_ids() | maa.block_ids()
    text = "".join((native / f).read_text(encoding="utf-8") for f in ("runtime.rs", "crop_seed.rs", "seed_crop.rs"))
    named = set(re.findall(r'"((?:mysticalagriculture|mysticalagradditions):[a-z0-9_]+)"', text))
    missing = sorted(n for n in named if n not in ids)
    if missing:
        raise SystemExit(f"native names ids the content lacks: {missing}")
    for c in CROPS:
        for i in (f"{MA}:{c}_crop", f"{MA}:{c}_seeds"):
            for f in ("runtime.rs", "crop_seed.rs", "seed_crop.rs"):
                if f'"{i}"' not in (native / f).read_text(encoding="utf-8") and not (f == "runtime.rs" and i.endswith("_seeds")):
                    raise SystemExit(f"{f} lacks {i}")
    print(f"native tables checked: {len(named)} ids, all declared")


MIT = """MIT License

Copyright (c) 2018 BlakeBr0

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"""

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Sushi Go Crafting for Horizon 26.3: the content mod (sushigocrafting).

Everything comes from the official source of Sushi Go Crafting (Buuz135, MIT), branch 1.21
(commit c3b296b, version 0.6.7): textures, models, blockstates, lang, loot tables, recipes,
worldgen and the food numbers of `FoodIngredient` / `FoodType` / `FoodItem.Info`, converted to
26.3's formats. The ids stay the original's (`sushigocrafting:`).

usage: make_content.py SRC_DIR OUT_DIR
  SRC_DIR: the repository's src/ (main/ and generated/).
  OUT_DIR: gets sushigocrafting/ (a content mod folder). Run upgrade_hmod.py --dir on it after.

It also checks that the native mod's tables (runtime.rs, seed_crop.rs, crop_seed.rs) name the
same crops, seeds and portion items as the content.
"""
import json
import re
import shutil
import sys
from pathlib import Path

NS = "sushigocrafting"
VERSION = "0.6.7-horizon.1"

SRC = Path(sys.argv[1])
OUT = Path(sys.argv[2]) / NS
MAIN = SRC / "main/resources"
GEN = SRC / "generated/resources"
A_MAIN, A_GEN = MAIN / f"assets/{NS}", GEN / f"assets/{NS}"
D_MAIN, D_GEN = MAIN / f"data/{NS}", GEN / f"data/{NS}"
NATIVE = Path(__file__).resolve().parent.parent / "native/mixins/sushigocrafting_native"

DATA, ASSETS = OUT / "data" / NS, OUT / "assets" / NS
LOG = []


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(data, (dict, list)):
        data = json.dumps(data, indent=2, ensure_ascii=False) + "\n"
    path.write_text(data, encoding="utf-8")


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def find(rel):
    """A file of the original: generated/ first, then main/."""
    for root in (A_GEN, A_MAIN):
        f = root / rel
        if f.exists():
            return f
    raise SystemExit(f"missing {rel}")


# ---------------------------------------------------------------- food (FoodIngredient.java)
# name: (item, defaultAmount, hunger, saturation, effect)
# effect: ("add", effect id, seconds, level) | ("mod", time multiplier, level added) | None
ING = {
    "empty": (None, 0, 0, 0, None),
    "rice": (f"{NS}:cooked_rice", 30, 0, 2, ("mod", 1.75, 0)),
    "nori": (f"{NS}:nori_sheets", 1, 1, 1, None),
    "tuna": (f"{NS}:tuna_fillet", 30, 3, 1, ("add", f"{NS}:acquired_taste", 30, 0)),
    "salmon": (f"{NS}:salmon_fillet", 30, 3, 1, ("add", f"{NS}:small_bites", 30, 0)),
    "avocado": (f"{NS}:avocado_slices", 45, 2, 2, ("mod", 1.20, 1)),
    "cucumber": (f"{NS}:cucumber_slices", 15, 2, 2, ("mod", 1.25, 0)),
    "sesame": (f"{NS}:sesame_seed", 6, 1, 1, ("mod", 2.0, 0)),
    "crab": (f"{NS}:imitation_crab", 30, 2, 1, ("add", "minecraft:speed", 3, 0)),
    "wakame": ("minecraft:seagrass", 1, 0, 1, None),
    "tobiko": (f"{NS}:tobiko", 12, 1, 2, ("mod", 1.50, 0)),
    "cheese": (f"{NS}:cheese", 12, 1, 2, ("add", "minecraft:saturation", 15, 0)),
    "shrimp": (f"{NS}:shrimp", 30, 2, 1, ("add", f"{NS}:steady_hands", 60, 0)),
    "chicken": ("minecraft:cooked_chicken", 30, 2, 1, ("add", "minecraft:slow_falling", 10, 0)),
    "soy_sauce": (f"{NS}:soy_sauce", 4, 0, 0, ("mod", 1.0, 1)),
    "wasabi": (f"{NS}:wasabi_paste", 4, 0, 0, ("add", "minecraft:fire_resistance", 10, 0)),
}

# FoodType.java: name -> (name indexes, ingredient choices per slot)
TYPES = {
    "maki": ([2], [["nori"], ["rice"], ["salmon", "tuna", "avocado", "cucumber", "crab"]]),
    "gunkan": ([2], [["nori"], ["rice"], ["salmon", "tuna", "wakame"]]),
    "california": ([5, 2, 4], [["nori"], ["rice"], ["tuna", "salmon", "crab"], ["avocado"],
                               ["cucumber", "empty", "cheese"], ["tobiko", "empty"]]),
    "nigiri": ([2], [["nori"], ["rice"], ["salmon", "tuna", "shrimp"]]),
    "onigiri": ([], [["nori"], ["rice"]]),
    "temaki": ([2], [["nori"], ["rice"], ["salmon", "tuna", "shrimp", "chicken"], ["avocado"]]),
}

# Shaped patterns of the fixed recipes (the Roller stays in the grid: native remainders hook).
# N nori, C cooked rice, F the slot-2 filling, A avocado, X slot 4, T slot 5, R roller.
PATTERNS = {
    "maki": ["NCF", " R "],
    "gunkan": ["F ", "C ", "NR"],
    "nigiri": ["FN", "CR"],
    "onigiri": ["N", "C", "R"],
    "temaki": ["NCF", "AR "],
    "california": ["NCF", "AXT", " R "],
}


def combos(slots):
    if len(slots) == 1:
        return [[x] for x in slots[0]]
    return [[x] + rest for x in slots[0] for rest in combos(slots[1:])]


def sushi_name(kind, idx, ings):
    names = [ings[i] for i in idx if ings[i] != "empty"]
    return "_".join(names + [kind])


def food_values(ings):
    """FoodItem.Info for a stack without weights (negative = positive = 0: modifier 1.25), and
    1.21's FoodData.eat(int, float): saturation = nutrition * modifier * 2."""
    mod = 1.25
    hunger = int(sum(ING[i][2] for i in ings) * mod)
    sat_mod = sum(ING[i][3] for i in ings) * mod / (len(ings) + 2)
    saturation = round(hunger * sat_mod * 2.0, 4)
    effects = []  # [effect id, ticks, amplifier]
    adds = [ING[i][4] for i in ings if ING[i][4] and ING[i][4][0] == "add"]
    mods = [ING[i][4] for i in ings if ING[i][4] and ING[i][4][0] == "mod"]
    for _, eid, secs, lvl in adds:
        effects.append([eid, secs * 20, lvl])
    for _, t, lvl in mods:
        effects = [[e, int(d * t), a + lvl] for e, d, a in effects]
    return hunger, saturation, effects


SUSHI = []  # (name, kind, ings, hunger, saturation, effects)
for kind, (idx, slots) in TYPES.items():
    for ings in combos(slots):
        h, s, e = food_values(ings)
        SUSHI.append((sushi_name(kind, idx, ings), kind, ings, h, s, e))
assert len(SUSHI) == 34, len(SUSHI)

# ---------------------------------------------------------------- portions (AmountItem)
# item -> (min grams, max grams, max combine grams, hurts): SushiContent.Items.amountItem(...).
# One portion here is what the Roller takes at the highest weight (FoodIngredient.defaultAmount).
AMOUNT = {
    "avocado_slices": (100, 500, 1000, False, "avocado"),
    "tuna_fillet": (1000, 3000, 6000, False, "tuna"),
    "salmon_fillet": (500, 2000, 4000, False, "salmon"),
    "cooked_rice": (50, 500, 2000, False, "rice"),
    "cucumber_slices": (50, 200, 400, False, "cucumber"),
    "imitation_crab": (50, 200, 400, False, "crab"),
    "sesame_seed": (10, 100, 200, False, "sesame"),
    "tobiko": (10, 50, 100, False, "tobiko"),
    "cheese": (50, 250, 500, False, "cheese"),
    "shrimp": (20, 50, 100, False, "shrimp"),
    "soy_sauce": (10, 50, 100, True, "soy_sauce"),
    "wasabi_paste": (10, 50, 100, True, "wasabi"),
}


def portions(grams, default):
    return max(1, min(64, int(grams / default + 0.5)))


CROPS = {  # crop block -> (seed item, produce, template)
    "rice_crop": ("rice_seeds", "rice", "minecraft:seagrass"),
    "cucumber_crop": ("cucumber_seeds", "cucumber", "minecraft:beetroots"),
    "soy_crop": ("soy_seeds", "soy_bean", "minecraft:beetroots"),
    "wasabi_crop": ("wasabi_seeds", "wasabi_root", "minecraft:beetroots"),
    "sesame_crop": ("sesame_seeds", "sesame_seed", "minecraft:beetroots"),
}

LANG_EN = load(A_GEN / "lang/en_us.json")
PT = {
    "block.sushigocrafting.avocado_leaves": "Folhas de Abacateiro",
    "block.sushigocrafting.avocado_leaves_logged": "Tronco com Folhas de Abacateiro",
    "block.sushigocrafting.avocado_log": "Tronco de Abacateiro",
    "block.sushigocrafting.avocado_sapling": "Muda de Abacateiro",
    "block.sushigocrafting.cucumber_crop": "Sementes de Pepino",
    "block.sushigocrafting.cutting_board": "Tábua de Corte",
    "block.sushigocrafting.fermentation_barrel": "Barril de Fermentação",
    "block.sushigocrafting.rice_cooker": "Panela de Arroz",
    "block.sushigocrafting.rice_crop": "Sementes de Arroz",
    "block.sushigocrafting.roller": "Esteira de Sushi",
    "block.sushigocrafting.sesame_crop": "Sementes de Gergelim",
    "block.sushigocrafting.soy_crop": "Sementes de Soja",
    "block.sushigocrafting.wasabi_crop": "Sementes de Wasabi",
    "effect.sushigocrafting.acquired_taste": "Gosto Adquirido",
    "effect.sushigocrafting.acquired_taste.description": "Mais nutrição e saturação ao comer",
    "effect.sushigocrafting.small_bites": "Mordidinhas",
    "effect.sushigocrafting.small_bites.description": "Chance de devolver a comida que você come",
    "effect.sushigocrafting.steady_hands": "Mãos Firmes",
    "effect.sushigocrafting.steady_hands.description": "Mais rendimento ao cortar na tábua",
    "item.sushigocrafting.avocado": "Abacate",
    "item.sushigocrafting.avocado_slices": "Fatias de Abacate",
    "item.sushigocrafting.cheese": "Queijo",
    "item.sushigocrafting.cleaver_knife": "Cutelo",
    "item.sushigocrafting.cooked_rice": "Arroz Cozido",
    "item.sushigocrafting.cucumber": "Pepino",
    "item.sushigocrafting.cucumber_seeds": "Sementes de Pepino",
    "item.sushigocrafting.cucumber_slices": "Fatias de Pepino",
    "item.sushigocrafting.imitation_crab": "Kani (Imitação de Caranguejo)",
    "item.sushigocrafting.nori_sheets": "Folhas de Nori",
    "item.sushigocrafting.raw_tuna": "Atum Cru",
    "item.sushigocrafting.rice": "Arroz",
    "item.sushigocrafting.rice_seeds": "Sementes de Arroz",
    "item.sushigocrafting.salmon_fillet": "Filé de Salmão",
    "item.sushigocrafting.sesame_seed": "Gergelim",
    "item.sushigocrafting.sesame_seeds": "Sementes de Gergelim",
    "item.sushigocrafting.shrimp": "Camarão",
    "item.sushigocrafting.soy_bean": "Grão de Soja",
    "item.sushigocrafting.soy_sauce": "Molho de Soja",
    "item.sushigocrafting.soy_seeds": "Sementes de Soja",
    "item.sushigocrafting.tobiko": "Tobiko",
    "item.sushigocrafting.tuna_fillet": "Filé de Atum",
    "item.sushigocrafting.wasabi_paste": "Pasta de Wasabi",
    "item.sushigocrafting.wasabi_root": "Raiz de Wasabi",
    "item.sushigocrafting.wasabi_seeds": "Sementes de Wasabi",
    "itemGroup.sushigocrafting": "Sushi Go Crafting",
}
PT_WORDS = {"salmon": "Salmão", "tuna": "Atum", "avocado": "Abacate", "cucumber": "Pepino",
            "crab": "Kani", "wakame": "Wakame", "shrimp": "Camarão", "chicken": "Frango",
            "tobiko": "Tobiko", "cheese": "Queijo"}
PT_KIND = {"maki": "Maki", "gunkan": "Gunkan", "california": "Califórnia", "nigiri": "Nigiri",
           "onigiri": "Onigiri", "temaki": "Temaki"}


def pt_sushi(name, kind):
    parts = name[: -len(kind) - 1].split("_") if name != kind else []
    words = [PT_WORDS[p] for p in parts]
    return " ".join([PT_KIND[kind]] + ([] if not words else ["de " + " com ".join(words)]))


class Mod:
    def __init__(self):
        self.blocks, self.items, self.tab = [], [], []
        self.tags = {}
        self.lang = {"en_us": {}, "pt_br": {}}

    def tag(self, kind, name, *values):
        self.tags.setdefault((kind, name), []).extend(values)

    def block(self, name, decl, loot, tab=True):
        write(DATA / "horizon/block" / f"{name}.json", decl)
        if loot is not None:
            write(DATA / "loot_table/blocks" / f"{name}.json", loot)
        self.blocks.append(name)
        if decl.get("item", True) and tab:
            self.tab.append(f"{NS}:{name}")

    def item(self, name, decl, tab=True):
        write(DATA / "horizon/item" / f"{name}.json", decl)
        self.items.append(name)
        if tab:
            self.tab.append(f"{NS}:{name}")


M = Mod()


def copy_tex(rel):
    for root in (A_MAIN, A_GEN):
        f = root / "textures" / f"{rel}.png"
        if f.exists():
            d = ASSETS / "textures" / f"{rel}.png"
            d.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(f, d)
            meta = f.with_name(f.name + ".mcmeta")
            if meta.exists():
                shutil.copy(meta, d.with_name(d.name + ".mcmeta"))
            return
    raise SystemExit(f"missing texture {rel}")


def model_textures(model):
    out = []
    for t in model.get("textures", {}).values():
        if t.startswith("#"):
            continue
        if t.startswith(f"{NS}:"):
            out.append(t.split(":", 1)[1])
    return out


def copy_model(rel, seen=None):
    """Copies models/<rel>.json and its textures (and its parent, when it is the mod's)."""
    seen = seen if seen is not None else set()
    if rel in seen:
        return
    seen.add(rel)
    m = load(find(f"models/{rel}.json"))
    m.pop("credit", None)
    write(ASSETS / "models" / f"{rel}.json", m)
    for t in model_textures(m):
        copy_tex(t)
    parent = m.get("parent", "")
    if parent.startswith(f"{NS}:"):
        copy_model(parent.split(":", 1)[1], seen)


def item_def(name, model=None):
    write(ASSETS / "items" / f"{name}.json",
          {"model": {"type": "minecraft:model", "model": model or f"{NS}:item/{name}"}})


def item_with_model(name):
    copy_model(f"item/{name}")
    item_def(name)


def blockstate(name, rewrite=None):
    bs = load(find(f"blockstates/{name}.json"))
    if rewrite:
        bs = rewrite(bs)
    write(ASSETS / "blockstates" / f"{name}.json", bs)
    for v in bs.get("variants", {}).values():
        for e in (v if isinstance(v, list) else [v]):
            copy_model(e["model"].split(":", 1)[1])


def self_loot(name):
    return {"type": "minecraft:block", "random_sequence": f"{NS}:blocks/{name}", "pools": [{
        "rolls": 1.0, "bonus_rolls": 0.0, "conditions": [{"condition": "minecraft:survives_explosion"}],
        "entries": [{"type": "minecraft:item", "name": f"{NS}:{name}"}]}]}


def orig_loot(name):
    return load(D_GEN / "loot_table/blocks" / f"{name}.json")


def lang(key, en=None, pt=None):
    M.lang["en_us"][key] = en if en is not None else LANG_EN[key]
    M.lang["pt_br"][key] = pt if pt is not None else PT.get(key, M.lang["en_us"][key])


# ---------------------------------------------------------------- crops and seeds
for crop, (seed, produce, template) in CROPS.items():
    decl = {"template": template, "item": False}
    if template == "minecraft:seagrass":
        # Always in water, as WaterCropBlock (getFluidState = water source); an age 0..3 of its own,
        # ticking until ripe (the native mod grows it, CropBlock.randomTick's numbers).
        decl["state_properties"] = {"age": {"type": "enum", "values": ["0", "1", "2", "3"]}}
        decl["default_state"] = {"age": "0"}
        decl["properties"] = {"random_ticks": {"property": "age", "values": ["0", "1", "2"]}}
    M.block(crop, decl, orig_loot(crop))
    blockstate(crop)
    lang(f"block.{NS}.{crop}")
    seed_template = "minecraft:wheat_seeds" if template == "minecraft:seagrass" else "minecraft:beetroot_seeds"
    M.item(seed, {"template": seed_template})
    item_with_model(seed)
    lang(f"item.{NS}.{seed}", LANG_EN[f"block.{NS}.{crop}"], PT[f"item.{NS}.{seed}"])
    M.tag("item", "c:seeds", f"{NS}:{seed}")
    M.tag("item", f"c:seeds/{seed.removesuffix('_seeds')}", f"{NS}:{seed}")
    if template != "minecraft:seagrass":
        M.tag("block", "minecraft:crops", f"{NS}:{crop}")
        M.tag("block", "minecraft:bee_growables", f"{NS}:{crop}")
        M.tag("block", "minecraft:maintains_farmland", f"{NS}:{crop}")

# Produce (SushiItem "plant": not food). Wheat's template composts at 0.65 as the data map does.
for name, template in [("rice", "minecraft:flint"), ("cucumber", "minecraft:wheat"),
                       ("soy_bean", "minecraft:wheat"), ("wasabi_root", "minecraft:wheat"),
                       ("avocado", "minecraft:wheat")]:
    M.item(name, {"template": template})
    item_with_model(name)
    lang(f"item.{NS}.{name}")
for n in ["rice", "cucumber", "soy_bean", "wasabi_root", "sesame_seed"]:
    M.tag("item", "c:crops", f"{NS}:{n}")
for tag, n in [("rice", "rice"), ("cucumber", "cucumber"), ("soybean", "soy_bean"),
               ("wasabi_root", "wasabi_root"), ("sesame_seed", "sesame_seed")]:
    M.tag("item", f"c:crops/{tag}", f"{NS}:{n}")
M.tag("item", "c:fruits", f"{NS}:avocado")
M.tag("item", "c:fruits/avocado", f"{NS}:avocado")

# ---------------------------------------------------------------- avocado tree
leaves_loot = orig_loot("avocado_leaves")
M.block("avocado_leaves", {
    "template": "minecraft:oak_leaves",
    "state_properties": {"stage": {"type": "enum", "values": ["0", "1", "2"]}},
    "default_state": {"stage": "0"},
    # Every state: the template's decay (distance 7) and stage 1 ripening (native random tick).
    "properties": {"random_ticks": True},
}, leaves_loot)
blockstate("avocado_leaves")
item_def("avocado_leaves")
copy_model("item/avocado_leaves")
lang(f"block.{NS}.avocado_leaves")

px = 1 / 16
M.block("avocado_log", {
    "template": "minecraft:oak_log",
    "properties": {"occludes": False},
    "shapes": {"property": "axis", "values": {
        "y": [[3 * px, 0, 3 * px, 13 * px, 1, 13 * px]],
        "z": [[3 * px, 3 * px, 0, 13 * px, 13 * px, 1]],
        "x": [[0, 3 * px, 3 * px, 1, 13 * px, 13 * px]]}},
}, orig_loot("avocado_log"))
blockstate("avocado_log")
item_def("avocado_log")
copy_model("item/avocado_log")
lang(f"block.{NS}.avocado_log")

M.block("avocado_leaves_logged", {"template": "minecraft:oak_log", "properties": {"occludes": False}},
        orig_loot("avocado_leaves_logged"))
blockstate("avocado_leaves_logged")
item_def("avocado_leaves_logged")
copy_model("item/avocado_leaves_logged")
lang(f"block.{NS}.avocado_leaves_logged")

M.block("avocado_sapling", {"template": "minecraft:oak_sapling"}, orig_loot("avocado_sapling"))
blockstate("avocado_sapling")
item_with_model("avocado_sapling")
lang(f"block.{NS}.avocado_sapling")

M.tag("block", "minecraft:logs", f"{NS}:avocado_log", f"{NS}:avocado_leaves_logged")
M.tag("block", "minecraft:logs_that_burn", f"{NS}:avocado_log", f"{NS}:avocado_leaves_logged")
M.tag("item", "minecraft:logs", f"{NS}:avocado_log", f"{NS}:avocado_leaves_logged")
M.tag("item", "minecraft:logs_that_burn", f"{NS}:avocado_log", f"{NS}:avocado_leaves_logged")
M.tag("block", "minecraft:leaves", f"{NS}:avocado_leaves")
M.tag("item", "minecraft:leaves", f"{NS}:avocado_leaves")
M.tag("block", "minecraft:mineable/hoe", f"{NS}:avocado_leaves")
M.tag("block", "minecraft:saplings", f"{NS}:avocado_sapling")
M.tag("item", "minecraft:saplings", f"{NS}:avocado_sapling")
M.tag("block", "minecraft:mineable/axe", f"{NS}:avocado_log", f"{NS}:avocado_leaves_logged")

# Worldgen: the original's tree, placed feature and biome modifier (plains), 1.21 shapes.
for kind in ("configured_feature", "placed_feature"):
    for f in (D_MAIN / "worldgen" / kind).glob("*.json"):
        write(DATA / "worldgen" / kind / f.name, load(f))
write(DATA / "neoforge/biome_modifier/avocado.json", load(D_MAIN / "neoforge/biome_modifier/avocado.json"))

# ---------------------------------------------------------------- machines
def facing(bs):
    """Titanium's `subfacing` (FOUR_WAY: the opposite of the placer's look, as the furnace's
    `facing`) as the furnace template's `facing`."""
    return {"variants": {k.replace("subfacing=", "facing="): v for k, v in bs["variants"].items()}}


def boxes(n, s, e, w):
    return {"property": "facing", "values": {"north": [n], "south": [s], "east": [e], "west": [w]}}


board_ns = [px, 0, 3 * px, 15 * px, px, 13 * px]
board_ew = [3 * px, 0, px, 13 * px, px, 15 * px]
MACHINES = {
    # name: (properties, shapes, has facing variants)
    "cutting_board": ({"hardness": 3.0, "explosion_resistance": 3.0, "sounds": "minecraft:oak_trapdoor",
                       "requires_tool": False, "occludes": False}, boxes(board_ns, board_ns, board_ew, board_ew)),
    "roller": ({"hardness": 2.0, "explosion_resistance": 3.0, "sounds": "minecraft:oak_planks",
                "requires_tool": False, "occludes": False}, None),
    "rice_cooker": ({"hardness": 2.0, "explosion_resistance": 6.0, "sounds": "minecraft:iron_block",
                     "requires_tool": False, "occludes": False}, None),
    "fermentation_barrel": ({"hardness": 2.0, "explosion_resistance": 3.0, "sounds": "minecraft:oak_planks",
                             "requires_tool": False, "occludes": False}, None),
}
for name, (props, shapes) in MACHINES.items():
    decl = {"template": "minecraft:furnace", "properties": props}
    if shapes:
        decl["shapes"] = shapes
    M.block(name, decl, self_loot(name))
    bs = load(find(f"blockstates/{name}.json"))
    if any(k.startswith("subfacing=") for k in bs["variants"]):
        blockstate(name, facing)
    else:
        blockstate(name)
    item_def(name)
    copy_model(f"item/{name}")
    lang(f"block.{NS}.{name}")
M.tag("block", "minecraft:mineable/axe", f"{NS}:cutting_board", f"{NS}:roller", f"{NS}:fermentation_barrel")
M.tag("block", "minecraft:mineable/pickaxe", f"{NS}:rice_cooker")

# ---------------------------------------------------------------- items
def food(nutrition, saturation, effects, probability=1.0):
    c = {"minecraft:food": {"nutrition": nutrition, "saturation": saturation}}
    if effects:
        c["minecraft:consumable"] = {
            "consume_seconds": 1.6, "animation": "eat", "sound": "minecraft:entity.generic.eat",
            "has_consume_particles": True,
            "on_consume_effects": [{"type": "minecraft:apply_effects", "probability": probability, "effects": [
                {"id": e, "duration": d, "amplifier": a, "ambient": False, "show_particles": False,
                 "show_icon": True} for e, d, a in effects]}]}
    return c


# AmountItem: food (2, 0.3 -> 1.2 saturation), poison 5 s at 60% (soy sauce, wasabi) or 1%.
for name, (mn, mx, comb, hurts, ing) in AMOUNT.items():
    M.item(name, {"template": "minecraft:bread", "max_stack_size": 64,
                  "components": food(2, 1.2, [["minecraft:poison", 100, 0]], 0.6 if hurts else 0.01)})
    item_with_model(name)
    lang(f"item.{NS}.{name}")
for name, template in [("nori_sheets", "minecraft:paper"), ("raw_tuna", "minecraft:flint"),
                       ("cleaver_knife", "minecraft:flint")]:
    M.item(name, {"template": template})
    item_with_model(name)
    lang(f"item.{NS}.{name}")
M.tag("item", "c:tools/knives", f"{NS}:cleaver_knife")
M.tag("item", "c:tools/knife", f"{NS}:cleaver_knife")
M.tag("item", "c:raw_fishes", "minecraft:salmon", f"{NS}:shrimp", f"{NS}:raw_tuna")
M.tag("item", "c:raw_fishes/salmon", "minecraft:salmon")
M.tag("item", "c:raw_fishes/tuna", f"{NS}:raw_tuna")
M.tag("item", "c:raw_fishes/shrimp", f"{NS}:shrimp")
M.tag("item", "minecraft:fishes", f"{NS}:raw_tuna", f"{NS}:shrimp")

# Sushi: 34 combinations, as FoodHelper.generateFood makes them.
for name, kind, ings, h, s, effects in SUSHI:
    M.item(name, {"template": "minecraft:bread", "components": food(h, s, effects)})
    item_with_model(name)
    lang(f"item.{NS}.{name}", pt=pt_sushi(name, kind))

# Mob effects (category, colour): AcquiredTaste CYAN, SmallBites GREEN, SteadyHands ORANGE.
for name, color in [("acquired_taste", 0x00FFFF), ("small_bites", 0x00FF00), ("steady_hands", 0xFFC800)]:
    write(DATA / "horizon/mob_effect" / f"{name}.json", {"category": "beneficial", "color": color})
    copy_tex(f"mob_effect/{name}")
    lang(f"effect.{NS}.{name}")
    lang(f"effect.{NS}.{name}.description")
lang("itemGroup.sushigocrafting")

# ---------------------------------------------------------------- recipes
def shaped(name, pattern, key, result, count=1):
    write(DATA / "recipe" / f"{name}.json", {
        "type": "minecraft:crafting_shaped", "category": "misc", "pattern": pattern, "key": key,
        "result": {"id": result, "count": count}})


C_TAGS = {"c:ingots/iron": "minecraft:iron_ingot"}


def ingredient(v):
    if "item" in v:
        return v["item"]
    t = v["tag"]
    return C_TAGS.get(t) or "#" + t


for name in ["roller", "cleaver_knife", "rice_cooker", "cutting_board", "fermentation_barrel"]:
    r = load(D_GEN / "recipe" / f"{name}.json")
    pattern = r["pattern"]
    # ShapedRecipePattern drops empty rows; 26.3 refuses a blank row only at the edges as 1.21 does.
    pattern = [row for row in pattern if row.strip()] if name == "cutting_board" else pattern
    shaped(name, pattern, {k: ingredient(v) for k, v in r["key"].items()}, r["result"]["id"])

for name, kind, ings, *_ in SUSHI:
    pat = [row for row in PATTERNS[kind]]
    key = {"R": f"{NS}:roller"}
    letters = {0: "N", 1: "C", 2: "F", 3: "A", 4: "X", 5: "T"}
    for i, ing in enumerate(ings):
        ch = letters[i]
        if ing == "empty":
            pat = [row.replace(ch, " ") for row in pat]
        else:
            key[ch] = ING[ing][0]
    used = set("".join(pat)) - {" "}
    key = {k: v for k, v in key.items() if k in used}
    shaped(f"sushi/{name}", pat, key, f"{NS}:{name}")

# Cutting board (CuttingBoardRecipe: input + ingredient name) and, here, nori from a dried kelp
# block (the original presses it with a piston against an iron block: 5 to 8 sheets per block).
CUT = {"salmon": "#c:raw_fishes/salmon", "tuna": "#c:raw_fishes/tuna", "crab": "minecraft:cod",
       "avocado": "#c:fruits/avocado", "cucumber": "#c:crops/cucumber", "wasabi": "#c:crops/wasabi_root"}
for ing, inp in CUT.items():
    write(DATA / "recipe/cutting_board" / f"{ing}.json", {
        "type": f"{NS}:cutting_board", "ingredient": ing, "input": inp, "result": ING[ing][0]})
write(DATA / "recipe/cutting_board/nori.json", {
    "type": f"{NS}:cutting_board", "ingredient": "nori", "input": "minecraft:dried_kelp_block",
    "result": f"{NS}:nori_sheets", "count": {"min": 5, "max": 8}})
# Fermenting barrel (FermentingBarrelRecipe), from the original's generated files.
for f in sorted((D_GEN / "recipe/fermenting_barrel").glob("*.json")):
    r = load(f)
    inp = r.get("input")
    out = {"type": f"{NS}:fermenting_barrel", "fluid": r["fluid"], "output": r["output"]}
    if isinstance(inp, dict) and inp:
        out["input"] = ingredient(inp)
    write(DATA / "recipe/fermenting_barrel" / f.name, out)

# ---------------------------------------------------------------- tabs, tags, lang, manifest
write(DATA / "horizon/creative_tab" / f"{NS}.json",
      {"title": "itemGroup.sushigocrafting", "icon": f"{NS}:salmon_maki", "items": M.tab})
for (kind, name), values in sorted(M.tags.items()):
    ns, path = name.split(":", 1)
    seen = []
    for v in values:
        if v not in seen:
            seen.append(v)
    write(OUT / "data" / ns / "tags" / kind / f"{path}.json", {"replace": False, "values": seen})
for code, entries in M.lang.items():
    write(ASSETS / "lang" / f"{code}.json", dict(sorted(entries.items())))
shutil.copy(SRC.parent / "LICENSE", OUT / "LICENSE-original.txt")
write(OUT / "horizon-mod.toml", f'''id = "{NS}"
name = "Sushi Go Crafting"
version = "{VERSION}"
description = "Arroz, pepino, soja, wasabi, gergelim, abacateiro, peixes, ingredientes, 34 sushis, tabua de corte, panela de arroz e barril de fermentacao. Port do Sushi Go Crafting (Buuz135, MIT). Precisa do mod nativo sushigocrafting_native."
authors = ["Buuz135 (original, MIT)", "port Horizon"]
api = "0.2"
side = "both"
capabilities = ["registries"]
''')

# ---------------------------------------------------------------- native tables in step
rt = (NATIVE / "runtime.rs").read_text()
for name, (mn, mx, comb, hurts, ing) in AMOUNT.items():
    d = ING[ing][1]
    row = f'("{NS}:{name}", {mn}, {mx}, {comb}, {d})'
    if row not in rt:
        raise SystemExit(f"runtime.rs AMOUNTS lacks {row}")
for crop, (seed, *_rest) in CROPS.items():
    for f in ("seed_crop.rs", "crop_seed.rs"):
        txt = (NATIVE / f).read_text()
        if f'"{NS}:{crop}"' not in txt or f'"{NS}:{seed}"' not in txt:
            raise SystemExit(f"{f} lacks {crop}/{seed}")

# A report: the sushi numbers (LEIA-ME's table) and what was written.
rep = ["sushi | nutrição | saturação | efeitos (ticks, nível 0 = I)"]
for name, kind, ings, h, s, effects in SUSHI:
    eff = ", ".join(f"{e.split(':')[1]} {d} t nv {a}" for e, d, a in effects) or "-"
    rep.append(f"{name} | {h} | {s} | {eff}")
rep.append("")
rep.append("porções (min-max por corte/drop): " + ", ".join(
    f"{n} {portions(mn, ING[i][1])}-{portions(mx - 1, ING[i][1])}" for n, (mn, mx, c, h, i) in AMOUNT.items()))
rep.append(f"blocks {len(M.blocks)}, items {len(M.items)}, tab {len(M.tab)}")
write(OUT.parent / "content-report.txt", "\n".join(rep) + "\n")
print("\n".join(rep[-3:]))

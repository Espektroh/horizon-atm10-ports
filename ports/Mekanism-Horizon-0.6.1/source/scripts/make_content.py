#!/usr/bin/env python3
"""Gera o conteúdo do Mekanism 0.6.0 para Horizon (mekanism.hmod e mekanismtools.hmod).

Uso (de dentro de source/):
  python3 -I scripts/make_content.py BASE_0.5.0.hmod Mekanism.jar MekanismTools.jar OUT_DIR

- BASE_0.5.0.hmod: o mekanism.hmod 0.5.0 instalado. Tudo o que ele tem fica igual (mesmos ids,
  mesmos arquivos); este script só acrescenta arquivos e acrescenta entradas a tags e lang.
- Os jars são do ATM10 (Mekanism 10.7.19, MIT): só lê texturas, modelos e lang deles.
Escreve OUT_DIR/content/ (mekanism) e OUT_DIR/content-tools/ (mekanismtools) e os dois .hmod.
"""
import json
import shutil
import sys
import zipfile
from pathlib import Path

VERSION_MEK = "10.7.19-horizon.6"
VERSION_TOOLS = "10.7.19-horizon.1"


def dump(path: Path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


class Jar:
    def __init__(self, path):
        self.zip = zipfile.ZipFile(path)
        self.names = set(self.zip.namelist())

    def json(self, name):
        return json.loads(self.zip.read(name))

    def copy(self, name, dest: Path, required=True):
        if name not in self.names:
            if required:
                raise SystemExit(f"falta no jar: {name}")
            return False
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(self.zip.read(name))
        meta = name + ".mcmeta"
        if meta in self.names:
            Path(str(dest) + ".mcmeta").write_bytes(self.zip.read(meta))
        return True


def item_stack(id_, count=1):
    return {"id": id_, "count": count}


def shaped(pattern, key, result, count=1, category="misc"):
    return {"type": "minecraft:crafting_shaped", "category": category, "pattern": pattern, "key": key,
            "result": item_stack(result, count)}


def loot(block):
    ns, name = block.split(":")
    return {"type": "minecraft:block", "pools": [{"rolls": 1.0, "bonus_rolls": 0.0,
            "conditions": [{"condition": "minecraft:survives_explosion"}],
            "entries": [{"type": "minecraft:item", "name": block}]}],
            "random_sequence": f"{ns}:blocks/{name}"}


def facing_lit_states(model, active):
    variants = {}
    for facing, y in (("north", 0), ("east", 90), ("south", 180), ("west", 270)):
        for lit, m in (("false", model), ("true", active)):
            v = {"model": m}
            if y:
                v["y"] = y
            variants[f"facing={facing},lit={lit}"] = v
    return {"variants": variants}


# ------------------------------------------------------------------ mekanism

TIERS = ["advanced", "elite", "ultimate"]

NEW_ITEMS = [
    "advanced_control_circuit", "elite_control_circuit", "ultimate_control_circuit",
    "alloy_reinforced", "alloy_atomic",
    "dust_diamond", "enriched_diamond", "dust_obsidian", "dust_refined_obsidian", "enriched_refined_obsidian",
    "ingot_refined_obsidian", "ingot_refined_glowstone",
]

STORAGE = {  # bloco: (lingote, dureza, luz)
    "block_steel": ("mekanism:ingot_steel", 5.0, 0),
    "block_refined_obsidian": ("mekanism:ingot_refined_obsidian", 50.0, 0),
    "block_refined_glowstone": ("mekanism:ingot_refined_glowstone", 5.0, 15),
}

# Nomes pt-BR onde o lang do jar não serve (ou não tem).
PT_OVERRIDES = {
    "block.mekanism.block_refined_obsidian": "Bloco de Obsidiana Refinada",
    "block.mekanism.block_refined_glowstone": "Bloco de Pedra Luminosa Refinada",
}


def mekanism(base_hmod: Path, jar: Jar, out: Path):
    if out.exists():
        shutil.rmtree(out)
    with zipfile.ZipFile(base_hmod) as z:
        z.extractall(out)
    data, assets = out / "data/mekanism", out / "assets/mekanism"
    new_blocks = []

    # Compressor de Ósmio
    dump(data / "horizon/block/osmium_compressor.json", {"template": "minecraft:furnace", "properties": {"hardness": 3.5}})
    for m in ("osmium_compressor", "osmium_compressor_active"):
        dump(assets / f"models/block/{m}.json", jar.json(f"assets/mekanism/models/block/{m}.json"))
    for t in ("back", "back_active", "bottom", "front", "front_active", "left", "left_active", "right",
              "right_active", "top", "top_active"):
        jar.copy(f"assets/mekanism/textures/block/osmium_compressor/{t}.png",
                 assets / f"textures/block/osmium_compressor/{t}.png")
    dump(assets / "blockstates/osmium_compressor.json",
         facing_lit_states("mekanism:block/osmium_compressor", "mekanism:block/osmium_compressor_active"))
    dump(assets / "items/osmium_compressor.json", {"model": {"type": "minecraft:model", "model": "mekanism:block/osmium_compressor"}})
    new_blocks.append("osmium_compressor")

    # Cubos de energia avançado, elite e definitivo: o modelo do básico com o canto do tier.
    basic_cube = json.loads((assets / "models/block/basic_energy_cube.json").read_text())
    basic_cube_state = json.loads((assets / "blockstates/basic_energy_cube.json").read_text())
    for tier in TIERS:
        name = f"{tier}_energy_cube"
        dump(data / f"horizon/block/{name}.json", {"template": "minecraft:furnace", "properties": {"hardness": 2.0}})
        model = json.loads(json.dumps(basic_cube))
        model["textures"]["core"] = f"mekanism:block/models/energy_cube_{tier}_corner"
        dump(assets / f"models/block/{name}.json", model)
        jar.copy(f"assets/mekanism/textures/block/models/energy_cube_{tier}_corner.png",
                 assets / f"textures/block/models/energy_cube_{tier}_corner.png")
        dump(assets / f"blockstates/{name}.json",
             json.loads(json.dumps(basic_cube_state).replace("mekanism:block/basic_energy_cube", f"mekanism:block/{name}")))
        dump(assets / f"items/{name}.json", {"model": {"type": "minecraft:model", "model": f"mekanism:block/{name}"}})
        new_blocks.append(name)

    # Cabos universais avançado, elite e definitivo: blocos de cano (pipe) como o básico.
    for tier in TIERS:
        name = f"{tier}_universal_cable"
        dump(data / f"horizon/block/{name}.json", {"template": "minecraft:chorus_plant", "properties": {"hardness": 1.0},
                                                   "connects_to": "#mekanism:cable_connects", "thickness": 6})
        for part in ("center", "arm", "item"):
            text = (assets / f"models/block/basic_universal_cable_{part}.json").read_text()
            text = text.replace("multipart/basic_universal_cable", f"multipart/{name}")
            dump(assets / f"models/block/{name}_{part}.json", json.loads(text))
        for t in (name, f"{name}_vertical"):
            jar.copy(f"assets/mekanism/textures/block/models/multipart/{t}.png",
                     assets / f"textures/block/models/multipart/{t}.png")
        state = (assets / "blockstates/basic_universal_cable.json").read_text()
        dump(assets / f"blockstates/{name}.json",
             json.loads(state.replace("mekanism:block/basic_universal_cable_", f"mekanism:block/{name}_")))
        dump(assets / f"items/{name}.json", {"model": {"type": "minecraft:model", "model": f"mekanism:block/{name}_item"}})
        new_blocks.append(name)

    # Blocos de armazenamento
    for name, (ingot, hardness, light) in STORAGE.items():
        props = {"hardness": hardness}
        if light:
            props["light"] = light
        dump(data / f"horizon/block/{name}.json", {"template": "minecraft:iron_block", "properties": props})
        short = name.removeprefix("block_")
        dump(assets / f"models/block/storage/{short}.json",
             {"parent": "minecraft:block/cube_all", "textures": {"all": f"mekanism:block/{name}"}})
        jar.copy(f"assets/mekanism/textures/block/{name}.png", assets / f"textures/block/{name}.png")
        dump(assets / f"blockstates/{name}.json", {"variants": {"": {"model": f"mekanism:block/storage/{short}"}}})
        dump(assets / f"items/{name}.json", {"model": {"type": "minecraft:model", "model": f"mekanism:block/storage/{short}"}})
        new_blocks.append(name)
        dump(data / f"recipe/storage/{name}.json",
             shaped(["###", "###", "###"], {"#": ingot}, f"mekanism:{name}"))
        dump(data / f"recipe/storage/{short}_ingot_from_block.json",
             {"type": "minecraft:crafting_shapeless", "category": "misc", "ingredients": [f"mekanism:{name}"],
              "result": item_stack(ingot, 9)})

    for b in new_blocks:
        dump(data / f"loot_table/blocks/{b}.json", loot(f"mekanism:{b}"))

    # Itens
    for name in NEW_ITEMS:
        dump(data / f"horizon/item/{name}.json", {"template": "minecraft:iron_ingot"})
        jar.copy(f"assets/mekanism/textures/item/{name}.png", assets / f"textures/item/{name}.png")
        dump(assets / f"models/item/{name}.json",
             {"parent": "minecraft:item/generated", "textures": {"layer0": f"mekanism:item/{name}"}})
        dump(assets / f"items/{name}.json", {"model": {"type": "minecraft:model", "model": f"mekanism:item/{name}"}})

    # Tags: o que os cabos juntam e o que a picareta minera (só acrescenta).
    tag = data / "tags/block/cable_connects.json"
    values = json.loads(tag.read_text())["values"]
    for b in ["mekanism:osmium_compressor", "mekanism:basic_universal_cable"] + \
             [f"mekanism:{t}_energy_cube" for t in TIERS] + [f"mekanism:{t}_universal_cable" for t in TIERS]:
        if b not in values:
            values.append(b)
    dump(tag, {"values": values})
    tag = out / "data/minecraft/tags/block/mineable/pickaxe.json"
    values = json.loads(tag.read_text())["values"]
    values += [f"mekanism:{b}" for b in new_blocks if f"mekanism:{b}" not in values]
    dump(tag, {"values": values})
    tag = out / "data/minecraft/tags/block/needs_iron_tool.json"
    dump(tag, {"values": ["mekanism:block_refined_obsidian"]})

    recipes(data)

    # Lang: os nomes do jar para o que é novo.
    for lang in ("en_us", "pt_br"):
        path = assets / f"lang/{lang}.json"
        have = json.loads(path.read_text(encoding="utf-8"))
        src = jar.json(f"assets/mekanism/lang/{lang}.json")
        for b in new_blocks:
            key = f"block.mekanism.{b}"
            have.setdefault(key, (PT_OVERRIDES if lang == "pt_br" else {}).get(key) or src[key])
        for i in NEW_ITEMS:
            key = f"item.mekanism.{i}"
            have.setdefault(key, src[key])
        dump(path, have)

    manifest = (out / "horizon-mod.toml").read_text()
    manifest = manifest.replace('version = "10.7.19-horizon.5"', f'version = "{VERSION_MEK}"')
    manifest = manifest.replace(
        'description = "Mekanism para Horizon, energia, maquinas, infusor, aco, osmio e cabos. Precisa do mod nativo mekanism_native."',
        'description = "Mekanism para Horizon: energia, maquinas, infusor, compressor de osmio, aco, obsidiana refinada, '
        'cubos e cabos de todos os tiers. Precisa do mod nativo mekanism_native 0.6.0."')
    assert VERSION_MEK in manifest and "0.6.0" in manifest, "horizon-mod.toml da 0.5.0 mudou?"
    (out / "horizon-mod.toml").write_text(manifest)
    return new_blocks


def recipes(data: Path):
    r = data / "recipe"
    # circuitos
    dump(r / "advanced_control_circuit.json",
         shaped(["ACA"], {"A": "mekanism:alloy_infused", "C": "mekanism:basic_control_circuit"}, "mekanism:advanced_control_circuit"))
    dump(r / "elite_control_circuit.json",
         shaped(["ACA"], {"A": "mekanism:alloy_reinforced", "C": "mekanism:advanced_control_circuit"}, "mekanism:elite_control_circuit"))
    dump(r / "ultimate_control_circuit.json",
         shaped(["ACA"], {"A": "mekanism:alloy_atomic", "C": "mekanism:elite_control_circuit"}, "mekanism:ultimate_control_circuit"))

    def infusing(name, input_, type_, amount, output, count=1, input_count=1):
        dump(r / f"metallurgic_infusing/{name}.json",
             {"type": "mekanism:metallurgic_infusing", "input": input_, "input_count": input_count,
              "infuse_type": type_, "infuse_amount": amount, "output": item_stack(output, count)})
    infusing("advanced_control_circuit", "mekanism:basic_control_circuit", "redstone", 60, "mekanism:advanced_control_circuit")
    infusing("elite_control_circuit", "mekanism:advanced_control_circuit", "diamond", 120, "mekanism:elite_control_circuit")
    infusing("ultimate_control_circuit", "mekanism:elite_control_circuit", "refined_obsidian", 240, "mekanism:ultimate_control_circuit")
    infusing("alloy_reinforced", "mekanism:alloy_infused", "diamond", 20, "mekanism:alloy_reinforced")
    infusing("alloy_atomic", "mekanism:alloy_reinforced", "refined_obsidian", 40, "mekanism:alloy_atomic")
    infusing("dust_refined_obsidian", "mekanism:dust_obsidian", "diamond", 10, "mekanism:dust_refined_obsidian")

    def conversion(name, input_, type_, amount):
        dump(r / f"infuse_conversion/{name}.json",
             {"type": "mekanism:infuse_conversion", "input": input_, "infuse_type": type_, "amount": amount})
    conversion("diamond_from_dust", "mekanism:dust_diamond", "diamond", 10)
    conversion("diamond_from_enriched", "mekanism:enriched_diamond", "diamond", 80)
    conversion("refined_obsidian_from_dust", "mekanism:dust_refined_obsidian", "refined_obsidian", 10)
    conversion("refined_obsidian_from_enriched", "mekanism:enriched_refined_obsidian", "refined_obsidian", 80)

    def machine(kind, name, input_, output, count=1, input_count=1):
        dump(r / f"{kind}/{name}.json", {"type": f"mekanism:{kind}", "input": input_, "input_count": input_count,
                                         "output": item_stack(output, count)})
    machine("crushing", "diamond_to_dust", "minecraft:diamond", "mekanism:dust_diamond")
    machine("crushing", "obsidian_to_dust", "minecraft:obsidian", "mekanism:dust_obsidian", 4)
    machine("crushing", "refined_obsidian_ingot_to_dust", "mekanism:ingot_refined_obsidian", "mekanism:dust_refined_obsidian")
    machine("enriching", "diamond_to_enriched", "minecraft:diamond", "mekanism:enriched_diamond")
    machine("enriching", "refined_obsidian_to_enriched", "mekanism:dust_refined_obsidian", "mekanism:enriched_refined_obsidian")
    machine("enriching", "glowstone_to_dust", "minecraft:glowstone", "minecraft:glowstone_dust", 4)

    # Compressor de Ósmio: item + ósmio líquido (1 mB por tick de trabalho)
    def compressing(name, input_, output):
        dump(r / f"compressing/{name}.json", {"type": "mekanism:compressing", "input": input_, "input_count": 1,
                                              "chemical": "osmium", "chemical_per_tick": 1, "output": item_stack(output)})
    compressing("refined_obsidian_ingot", "mekanism:dust_refined_obsidian", "mekanism:ingot_refined_obsidian")
    compressing("refined_glowstone_ingot", "minecraft:glowstone_dust", "mekanism:ingot_refined_glowstone")
    dump(r / "chemical_conversion/osmium_from_ingot.json",
         {"type": "mekanism:chemical_conversion", "input": "mekanism:ingot_osmium", "chemical": "osmium", "amount": 200})

    # máquina, cubos, cabos (o tablete de energia do original segue trocado por bloco de redstone, como no cubo básico)
    dump(r / "osmium_compressor.json", shaped(["ACA", "BXB", "ACA"], {
        "A": "mekanism:alloy_infused", "C": "mekanism:advanced_control_circuit", "B": "minecraft:bucket",
        "X": "mekanism:steel_casing"}, "mekanism:osmium_compressor"))
    for tier, alloy, ingot, prev in (("advanced", "mekanism:alloy_infused", "mekanism:ingot_osmium", "basic"),
                                     ("elite", "mekanism:alloy_reinforced", "minecraft:gold_ingot", "advanced"),
                                     ("ultimate", "mekanism:alloy_atomic", "minecraft:diamond", "elite")):
        dump(r / f"{tier}_energy_cube.json", shaped(["AEA", "IPI", "AEA"], {
            "A": alloy, "E": "minecraft:redstone_block", "I": ingot, "P": f"mekanism:{prev}_energy_cube"},
            f"mekanism:{tier}_energy_cube"))
        dump(r / f"{tier}_universal_cable.json", shaped(["PPP", "PAP", "PPP"], {
            "A": alloy, "P": f"mekanism:{prev}_universal_cable"}, f"mekanism:{tier}_universal_cable", 8))


# ------------------------------------------------------------------ mekanismtools

# Números do Mekanism Tools 10.7 (MaterialDefaults), MIT.
MATERIALS = {
    # nome: usos, velocidade, bônus de dano, enchant, dano e vel. do machado, armadura (capacete, peito, pernas, botas),
    #       durabilidade da armadura, dureza, resistência a repulsão, som, reparo, tier vanilla ("incorrect_for")
    "osmium": dict(uses=1024, speed=4.0, bonus=4.0, ench=14, axe=(8.0, -3.3), defense=(4, 8, 6, 3),
                   durability=(330, 480, 450, 390), toughness=3.0, kb=0.1, sound="minecraft:item.armor.equip_iron",
                   repair="mekanism:ingot_osmium", incorrect="#minecraft:incorrect_for_iron_tool", fire=False),
    "steel": dict(uses=500, speed=8.0, bonus=3.0, ench=16, axe=(7.0, -3.0), defense=(3, 8, 6, 3),
                  durability=(220, 320, 300, 260), toughness=2.0, kb=0.0, sound="minecraft:item.armor.equip_iron",
                  repair="mekanism:ingot_steel", incorrect="#minecraft:incorrect_for_iron_tool", fire=False),
    "lapis_lazuli": dict(uses=128, speed=9.0, bonus=1.0, ench=32, axe=(4.0, -2.9), defense=(1, 4, 3, 1),
                         durability=(110, 160, 150, 130), toughness=0.0, kb=0.0, sound="minecraft:item.armor.equip_diamond",
                         repair="minecraft:lapis_lazuli", incorrect="#minecraft:incorrect_for_stone_tool", fire=False),
    "refined_glowstone": dict(uses=384, speed=15.0, bonus=2.0, ench=20, axe=(6.0, -2.9), defense=(3, 8, 6, 3),
                              durability=(187, 272, 255, 221), toughness=0.0, kb=0.0, sound="minecraft:item.armor.equip_iron",
                              repair="mekanism:ingot_refined_glowstone", incorrect="#minecraft:incorrect_for_diamond_tool", fire=False),
    "refined_obsidian": dict(uses=4096, speed=12.0, bonus=8.0, ench=18, axe=(7.0, -2.9), defense=(6, 12, 8, 5),
                             durability=(825, 1200, 1125, 975), toughness=5.0, kb=0.2, sound="minecraft:item.armor.equip_iron",
                             repair="mekanism:ingot_refined_obsidian", incorrect="#minecraft:incorrect_for_netherite_tool", fire=True),
}

TOOLS = {  # tipo: (dano base, velocidade de ataque, blocos minerados, tag de item vanilla)
    "sword": (3.0, -2.4, None, ["minecraft:swords"]),
    "pickaxe": (1.0, -2.8, ["#minecraft:mineable/pickaxe"], ["minecraft:pickaxes"]),
    "axe": (None, None, ["#minecraft:mineable/axe"], ["minecraft:axes"]),
    "shovel": (1.5, -3.0, ["#minecraft:mineable/shovel"], ["minecraft:shovels"]),
    "hoe": (None, None, ["#minecraft:mineable/hoe"], ["minecraft:hoes"]),
    "paxel": (None, -2.4, ["#minecraft:mineable/pickaxe", "#minecraft:mineable/axe", "#minecraft:mineable/shovel"],
              ["minecraft:pickaxes", "minecraft:axes", "minecraft:shovels"]),
}
ARMOR = [("helmet", "head", "armor.helmet", "minecraft:head_armor"), ("chestplate", "chest", "armor.chestplate", "minecraft:chest_armor"),
         ("leggings", "legs", "armor.leggings", "minecraft:leg_armor"), ("boots", "feet", "armor.boots", "minecraft:foot_armor")]


def tool_components(mat, kind):
    m = MATERIALS[mat]
    damage, speed, blocks, _ = TOOLS[kind]
    uses = m["uses"]
    if kind == "axe":
        damage, speed = m["axe"]
    elif kind == "hoe":
        damage, speed = -m["bonus"], m["bonus"] - 3.0
    elif kind == "paxel":
        damage, uses = m["axe"][0] + 1.0, 2 * m["uses"]
    attack = damage + m["bonus"]
    c = {
        "minecraft:max_damage": uses,
        "minecraft:attribute_modifiers": [
            {"type": "minecraft:attack_damage", "id": "minecraft:base_attack_damage", "amount": float(attack),
             "operation": "add_value", "slot": "mainhand"},
            {"type": "minecraft:attack_speed", "id": "minecraft:base_attack_speed", "amount": float(speed),
             "operation": "add_value", "slot": "mainhand"},
        ],
        "minecraft:enchantable": {"value": m["ench"]},
        "minecraft:repairable": {"items": m["repair"]},
    }
    if blocks:
        rules = [{"blocks": m["incorrect"], "correct_for_drops": False}]
        rules += [{"blocks": b, "correct_for_drops": True, "speed": float(m["speed"])} for b in blocks]
        c["minecraft:tool"] = {"rules": rules, "default_mining_speed": 1.0, "damage_per_block": 1}
    return c


def armor_components(mat, i):
    m = MATERIALS[mat]
    piece, slot, mod_id, _ = ARMOR[i]
    mods = [{"type": "minecraft:armor", "id": f"minecraft:{mod_id}", "amount": float(m["defense"][i]),
             "operation": "add_value", "slot": slot},
            {"type": "minecraft:armor_toughness", "id": f"minecraft:{mod_id}", "amount": float(m["toughness"]),
             "operation": "add_value", "slot": slot}]
    if m["kb"] > 0:
        mods.append({"type": "minecraft:knockback_resistance", "id": f"minecraft:{mod_id}", "amount": float(m["kb"]),
                     "operation": "add_value", "slot": slot})
    return {
        "minecraft:max_damage": m["durability"][i],
        "minecraft:attribute_modifiers": mods,
        "minecraft:equippable": {"slot": slot, "asset_id": f"mekanismtools:{mat}", "equip_sound": m["sound"]},
        "minecraft:enchantable": {"value": m["ench"]},
        "minecraft:repairable": {"items": m["repair"]},
    }


def mekanismtools(jar: Jar, out: Path):
    if out.exists():
        shutil.rmtree(out)
    data, assets = out / "data/mekanismtools", out / "assets/mekanismtools"
    items, tags = [], {}
    for mat, m in MATERIALS.items():
        vanilla = "netherite" if m["fire"] else "iron"
        for kind in TOOLS:
            name = f"{mat}_{kind}"
            template = f"minecraft:{vanilla}_{'pickaxe' if kind == 'paxel' else kind}"
            dump(data.parent / "mekanismtools/horizon/item" / f"{name}.json",
                 {"template": template, "components": tool_components(mat, kind)})
            jar.copy(f"assets/mekanismtools/textures/item/{mat}/{kind}.png", assets / f"textures/item/{mat}/{kind}.png")
            dump(assets / f"models/item/{name}.json",
                 {"parent": "minecraft:item/handheld", "textures": {"layer0": f"mekanismtools:item/{mat}/{kind}"}})
            items.append(name)
            for t in TOOLS[kind][3]:
                tags.setdefault(t, []).append(f"mekanismtools:{name}")
        for i, (piece, *_rest) in enumerate(ARMOR):
            name = f"{mat}_{piece}"
            dump(data.parent / "mekanismtools/horizon/item" / f"{name}.json",
                 {"template": f"minecraft:{vanilla}_{piece}", "components": armor_components(mat, i)})
            jar.copy(f"assets/mekanismtools/textures/item/{mat}/{piece}.png", assets / f"textures/item/{mat}/{piece}.png")
            dump(assets / f"models/item/{name}.json",
                 {"parent": "minecraft:item/generated", "textures": {"layer0": f"mekanismtools:item/{mat}/{piece}"}})
            items.append(name)
            tags.setdefault(ARMOR[i][3], []).append(f"mekanismtools:{name}")
        # o que se vê vestido (26.3: equipment asset + texturas em entity/equipment)
        dump(assets / f"equipment/{mat}.json", {"layers": {"humanoid": [{"texture": f"mekanismtools:{mat}"}],
                                                           "humanoid_leggings": [{"texture": f"mekanismtools:{mat}"}]}})
        jar.copy(f"assets/mekanismtools/textures/models/armor/{mat}_layer_1.png",
                 assets / f"textures/entity/equipment/humanoid/{mat}.png")
        jar.copy(f"assets/mekanismtools/textures/models/armor/{mat}_layer_2.png",
                 assets / f"textures/entity/equipment/humanoid_leggings/{mat}.png")
        tool_recipes(data, mat, m["repair"])
    for name in items:
        dump(assets / f"items/{name}.json", {"model": {"type": "minecraft:model", "model": f"mekanismtools:item/{name}"}})
    for tag, values in tags.items():
        ns, path = tag.split(":")
        dump(out / f"data/{ns}/tags/item/{path}.json", {"values": values})
    for lang in ("en_us", "pt_br"):
        src = jar.json(f"assets/mekanismtools/lang/{lang}.json")
        dump(assets / f"lang/{lang}.json", {f"item.mekanismtools.{n}": src[f"item.mekanismtools.{n}"] for n in items})
    (out / "horizon-mod.toml").write_text(
        'id = "mekanismtools"\n'
        'name = "Mekanism Tools"\n'
        f'version = "{VERSION_TOOLS}"\n'
        'description = "Ferramentas, paxels e armaduras de osmio, aco, lapis-lazuli, pedra luminosa refinada e obsidiana '
        'refinada. Precisa do mod de conteudo mekanism (os lingotes)."\n'
        'authors = ["Mekanism team (original, MIT)", "Espektroh (port Horizon)"]\n'
        'api = "0.2"\n'
        'side = "both"\n'
        'capabilities = ["registries"]\n')
    return items


def tool_recipes(data: Path, mat, ingot):
    r = data / "recipe" / mat
    stick = "minecraft:stick"
    t = lambda kind: f"mekanismtools:{mat}_{kind}"
    for kind, pattern, key in (
        ("sword", ["I", "I", "R"], {"I": ingot, "R": stick}),
        ("pickaxe", ["III", " R ", " R "], {"I": ingot, "R": stick}),
        ("axe", ["II", "IR", " R"], {"I": ingot, "R": stick}),
        ("shovel", ["I", "R", "R"], {"I": ingot, "R": stick}),
        ("hoe", ["II", " R", " R"], {"I": ingot, "R": stick}),
        # o original usa o tipo mekanismtools:paxel só para levar encantamentos; aqui é uma receita normal
        ("paxel", ["APS", " R ", " R "], {"A": t("axe"), "P": t("pickaxe"), "S": t("shovel"), "R": stick}),
        ("helmet", ["III", "I I"], {"I": ingot}),
        ("chestplate", ["I I", "III", "III"], {"I": ingot}),
        ("leggings", ["III", "I I", "I I"], {"I": ingot}),
        ("boots", ["I I", "I I"], {"I": ingot}),
    ):
        dump(r / f"{kind}.json", shaped(pattern, key, t(kind), category="equipment"))


def pack(src: Path, dest: Path):
    """Zip com datas e modos fixos: mesmo conteúdo, mesmo SHA-256."""
    dest.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(dest, "w", compression=zipfile.ZIP_DEFLATED) as z:
        for path in sorted(p for p in src.rglob("*") if p.is_file()):
            info = zipfile.ZipInfo(path.relative_to(src).as_posix(), (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            z.writestr(info, path.read_bytes())


def main():
    if len(sys.argv) != 5:
        raise SystemExit(__doc__)
    base, mek_jar, tools_jar, out = Path(sys.argv[1]), Jar(sys.argv[2]), Jar(sys.argv[3]), Path(sys.argv[4])
    blocks = mekanism(base, mek_jar, out / "content")
    items = mekanismtools(tools_jar, out / "content-tools")
    pack(out / "content", out / "mekanism.hmod")
    pack(out / "content-tools", out / "mekanismtools.hmod")
    print(json.dumps({"mekanism_new_blocks": len(blocks), "mekanism_new_items": len(NEW_ITEMS),
                      "mekanismtools_items": len(items)}))


if __name__ == "__main__":
    main()

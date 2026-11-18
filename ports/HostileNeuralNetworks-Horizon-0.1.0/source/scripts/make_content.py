#!/usr/bin/env python3
"""Gera o conteúdo do Hostile Neural Networks 0.1.0 para Horizon (hostilenetworks.hmod) a partir do
repositório-fonte (ramo 1.21, MIT): texturas, modelos, lang, receitas e dados dos Data Models.

Uso (de dentro de source/):
  python3 -I scripts/make_content.py /home/claude/ext/src/hostilenetworks content
  python3 -I scripts/upgrade_hmod.py --dir content/hostilenetworks
  python3 -I scripts/check_assets.py content

Os Data Models e as Predictions viram um item por mob (`data_model_zombie`, `prediction_zombie`); as
texturas por mob são as do original tingidas com o `name_color` do Data Model (PIL).
"""
import json
import shutil
import sys
from pathlib import Path

from PIL import Image

NS = "hostilenetworks"
VERSION = "6.5.1-horizon.1"
MOBS = ("zombie skeleton creeper spider enderman blaze slime wither_skeleton witch ghast magma_cube guardian "
        "drowned zombified_piglin shulker phantom evoker vindicator hoglin breeze elder_guardian wither "
        "ender_dragon warden").split()
PT = {"zombie": "Zumbi", "skeleton": "Esqueleto", "creeper": "Creeper", "spider": "Aranha", "enderman": "Enderman",
      "blaze": "Blaze", "slime": "Slime", "wither_skeleton": "Esqueleto Wither", "witch": "Bruxa", "ghast": "Ghast",
      "magma_cube": "Cubo de Magma", "guardian": "Guardião", "drowned": "Afogado",
      "zombified_piglin": "Piglin Zumbificado", "shulker": "Shulker", "phantom": "Phantom", "evoker": "Invocador",
      "vindicator": "Vingador", "hoglin": "Hoglin", "breeze": "Breeze", "elder_guardian": "Guardião Ancião",
      "wither": "Wither", "ender_dragon": "Dragão do Ender", "warden": "Warden"}
EN = {m: m.replace("_", " ").title() for m in MOBS}
SIMPLE = ["deep_learner", "blank_data_model", "prediction_matrix", "overworld_prediction", "nether_prediction",
          "end_prediction", "polymer_clay"]
SINGLE = {"deep_learner", "blank_data_model"}
BLOCKS = ["sim_chamber", "loot_fabricator"]
C_TAGS = {"c:dusts/redstone": "minecraft:redstone", "c:glass_panes": "minecraft:glass_pane",
          "c:obsidians": "minecraft:obsidian", "c:ingots/gold": "minecraft:gold_ingot",
          "c:ingots/iron": "minecraft:iron_ingot", "c:gems/diamond": "minecraft:diamond",
          "c:ingots/netherite": "minecraft:netherite_ingot", "c:gems/lapis": "minecraft:lapis_lazuli"}
RECIPES = ["deep_learner", "framework", "prediction_matrix", "sim_chamber", "loot_fabricator"]
LOG = []


def dump(path: Path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def copy(src: Path, dest: Path):
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(src, dest)
    meta = Path(str(src) + ".mcmeta")
    if meta.exists():
        shutil.copyfile(meta, Path(str(dest) + ".mcmeta"))


def tint(src: Path, dest: Path, rgb: int, strength: float):
    img = Image.open(src).convert("RGBA")
    r0, g0, b0 = (rgb >> 16) & 255, (rgb >> 8) & 255, rgb & 255
    px = img.load()
    for y in range(img.height):
        for x in range(img.width):
            r, g, b, a = px[x, y]
            if a == 0:
                continue
            lum = (r * 299 + g * 587 + b * 114) / 255000
            tr, tg, tb = (int(min(255, c * (0.35 + lum))) for c in (r0, g0, b0))
            px[x, y] = (int(r + (tr - r) * strength), int(g + (tg - g) * strength), int(b + (tb - b) * strength), a)
    dest.parent.mkdir(parents=True, exist_ok=True)
    img.save(dest)


def ingredient(v):
    if isinstance(v, list):
        return [ingredient(x) for x in v]
    if "item" in v:
        return v["item"]
    if v.get("tag", "").startswith("minecraft:"):
        return "#" + v["tag"]
    if v.get("tag") in C_TAGS:
        return C_TAGS[v["tag"]]
    raise SystemExit(f"ingrediente desconhecido {v}")


def convert(r):
    out = {"type": r["type"], "category": r.get("category", "misc")}
    if r["type"] == "minecraft:crafting_shaped":
        out["pattern"] = r["pattern"]
        out["key"] = {k: ingredient(v) for k, v in r["key"].items()}
    else:
        out["ingredients"] = [ingredient(v) for v in r["ingredients"]]
    res = {"id": r["result"]["id"]}
    if r["result"].get("count", 1) != 1:
        res["count"] = r["result"]["count"]
    out["result"] = res
    return out


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    repo, content = Path(sys.argv[1]), Path(sys.argv[2])
    res = repo / "src/main/resources"
    gen = repo / "src/generated/resources/data" / NS
    src_assets = res / "assets" / NS
    root = content / NS
    if root.exists():
        shutil.rmtree(root)
    assets, data = root / "assets" / NS, root / "data" / NS
    models = {m: json.loads((res / f"data/{NS}/data_models/{m}.json").read_text()) for m in MOBS}

    # Blocks: furnace template (facing + lit), the original models (not full cubes).
    for b in BLOCKS:
        dump(data / "horizon/block" / f"{b}.json",
             {"template": "minecraft:furnace", "properties": {"hardness": 4.0, "occludes": False}})
        m = json.loads((src_assets / f"models/block/{b}.json").read_text())
        m.pop("credit", None)
        m["parent"] = "minecraft:block/block"
        for t in m["textures"].values():
            p = t.split(":", 1)[1]
            copy(src_assets / f"textures/{p}.png", assets / f"textures/{p}.png")
        dump(assets / f"models/block/{b}.json", m)
        src = json.loads((src_assets / f"blockstates/{b}.json").read_text())["variants"]
        dump(assets / f"blockstates/{b}.json",
             {"variants": {f"{k},lit={lit}": v for k, v in src.items() for lit in ("false", "true")}})
        dump(assets / f"items/{b}.json", {"model": {"type": "minecraft:model", "model": f"{NS}:block/{b}"}})
        dump(data / f"loot_table/blocks/{b}.json",
             {"type": "minecraft:block", "pools": [{"rolls": 1.0, "bonus_rolls": 0.0,
              "conditions": [{"condition": "minecraft:survives_explosion"}],
              "entries": [{"type": "minecraft:item", "name": f"{NS}:{b}"}]}], "random_sequence": f"{NS}:blocks/{b}"})
    dump(root / "data/minecraft/tags/block/mineable/pickaxe.json", {"values": [f"{NS}:{b}" for b in BLOCKS]})
    dump(data / "horizon/menu/sim_chamber.json", {"slots": 4, "data_slots": 7, "results": [2, 3]})
    dump(data / "horizon/menu/loot_fabricator.json", {"slots": 17, "data_slots": 5, "results": list(range(1, 17))})

    def item(name, texture_src=None, single=False):
        decl = {"template": "minecraft:iron_ingot", "max_stack_size": 1 if single else 64}
        dump(data / "horizon/item" / f"{name}.json", decl)
        dump(assets / f"models/item/{name}.json",
             {"parent": "minecraft:item/generated", "textures": {"layer0": f"{NS}:item/{name}"}})
        dump(assets / f"items/{name}.json", {"model": {"type": "minecraft:model", "model": f"{NS}:item/{name}"}})
        if texture_src:
            copy(texture_src, assets / f"textures/item/{name}.png")

    for name in SIMPLE:
        item(name, src_assets / f"textures/item/{name}.png", name in SINGLE)
    for m in MOBS:
        color = int(models[m]["name_color"].lstrip("#"), 16)
        item(f"data_model_{m}", single=True)
        tint(src_assets / "textures/item/blank_data_model.png", assets / f"textures/item/data_model_{m}.png", color, 0.45)
        item(f"prediction_{m}")
        tint(src_assets / "textures/item/prediction.png", assets / f"textures/item/prediction_{m}.png", color, 0.6)

    tab = ([f"{NS}:{b}" for b in BLOCKS] + [f"{NS}:{n}" for n in SIMPLE]
           + [f"{NS}:data_model_{m}" for m in MOBS] + [f"{NS}:prediction_{m}" for m in MOBS])
    dump(data / "horizon/creative_tab/hostilenetworks.json",
         {"title": "itemGroup.hostilenetworks", "icon": f"{NS}:sim_chamber", "items": tab})

    n = 0
    for r in RECIPES:
        dump(data / f"recipe/{r}.json", convert(json.loads((gen / f"recipe/{r}.json").read_text())))
        n += 1
    for f in sorted((gen / "recipe/living_matter").rglob("*.json")):
        rel = f.relative_to(gen / "recipe")
        dump(data / "recipe" / rel, convert(json.loads(f.read_text())))
        n += 1
    # Polymer Clay: não existe na 6.5.1 (o original trocou por clay_ball); receita do port.
    dump(data / "recipe/polymer_clay.json",
         {"type": "minecraft:crafting_shapeless", "category": "misc",
          "ingredients": ["minecraft:clay_ball", "minecraft:clay_ball", "minecraft:iron_nugget", "minecraft:gold_nugget"],
          "result": {"id": f"{NS}:polymer_clay", "count": 2}})
    n += 1
    LOG.append("fora: data_center, data_center_io_port, fab_directive, twilight_prediction (e receitas deles)")

    en_src = json.loads((src_assets / "lang/en_us.json").read_text())
    # O pt_br.json do original não é JSON válido (linha 131): nomes em português escritos aqui.
    pt_src = {f"block.{NS}.sim_chamber": "Câmara de Simulação", f"block.{NS}.loot_fabricator": "Fabricador de Loot",
              f"item.{NS}.deep_learner": "Deep Learner", f"item.{NS}.blank_data_model": "Estrutura de Modelo",
              f"item.{NS}.prediction_matrix": "Matriz de Predição",
              f"item.{NS}.overworld_prediction": "Predição Generalizada da Superfície",
              f"item.{NS}.nether_prediction": "Predição Generalizada do Nether",
              f"item.{NS}.end_prediction": "Predição Generalizada do End",
              f"item.{NS}.data_model": "Data Model de %s", f"item.{NS}.prediction": "Predição de %s"}
    for code, src in (("en_us", en_src), ("pt_br", pt_src)):
        lang = {}
        for b in BLOCKS:
            lang[f"block.{NS}.{b}"] = src.get(f"block.{NS}.{b}", en_src[f"block.{NS}.{b}"])
        for s in SIMPLE:
            k = f"item.{NS}.{s}"
            lang[k] = src.get(k, en_src.get(k, "Polymer Clay" if code == "en_us" else "Argila Polimérica"))
        if code == "pt_br":
            lang[f"item.{NS}.polymer_clay"] = "Argila Polimérica"
        dm = src.get(f"item.{NS}.data_model", en_src[f"item.{NS}.data_model"])
        pr = src.get(f"item.{NS}.prediction", en_src[f"item.{NS}.prediction"])
        for m in MOBS:
            name = PT[m] if code == "pt_br" else EN[m]
            lang[f"item.{NS}.data_model_{m}"] = dm.replace("%s", name).replace("%1$s", name)
            lang[f"item.{NS}.prediction_{m}"] = pr.replace("%s", name).replace("%1$s", name)
        lang["itemGroup.hostilenetworks"] = "Hostile Neural Networks"
        lang[f"container.{NS}.sim_chamber"] = lang[f"block.{NS}.sim_chamber"]
        lang[f"container.{NS}.loot_fabricator"] = lang[f"block.{NS}.loot_fabricator"]
        dump(assets / f"lang/{code}.json", lang)

    shutil.copyfile(repo / "LICENSE", root / "LICENSE-original.txt")
    (root / "horizon-mod.toml").write_text(
        f'id = "{NS}"\n'
        'name = "Hostile Neural Networks"\n'
        f'version = "{VERSION}"\n'
        'description = "Hostile Neural Networks para Horizon: Deep Learner, Data Models (24 mobs vanilla, 5 tiers), '
        'Camara de Simulacao e Fabricador de Loot. Port do Hostile Neural Networks (Shadows_of_Fire, MIT). '
        'Precisa do mod nativo hostilenetworks_native."\n'
        'authors = ["Shadows_of_Fire (original, MIT)", "port Horizon"]\n'
        'api = "0.2"\n'
        'side = "both"\n'
        'capabilities = ["registries"]\n')
    items = len(SIMPLE) + 2 * len(MOBS)
    (content / "content-report.txt").write_text(
        "\n".join([f"blocos: {len(BLOCKS)}", f"itens: {items}", f"receitas: {n}"] + LOG) + "\n", encoding="utf-8")
    print(json.dumps({"blocks": len(BLOCKS), "items": items, "recipes": n}))


if __name__ == "__main__":
    main()

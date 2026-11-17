#!/usr/bin/env python3
"""Gera o mod de conteúdo `bibliocraft` (Horizon, API 0.2) a partir do código-fonte do
Bibliocraft Legacy 1.6.6 (MinecraftschurliMods/Bibliocraft-Legacy, branch version/1.21.1, MIT).

Blocos (11 madeiras vanilla do original: oak spruce birch jungle acacia dark_oak crimson warped
mangrove bamboo cherry):
  <madeira>_bookcase   estante, 16 livros, tela (menu bibliocraft:bookcase)
  <madeira>_shelf      prateleira, 4 itens, tela (menu bibliocraft:shelf)
  <madeira>_table      mesa, 1 item em cima (sem tela, como o original)
  red_<madeira>_seat   assento vermelho (só bloco: sem sentar nesta versão)
  sword_pedestal       pedestal de espada verde (a cor padrão do original)

uso: make_content.py <clone do Bibliocraft-Legacy> <pasta de saída>
Depois rode upgrade_hmod.py --dir <pasta de saída> (loot para o 26.3) e check_content.py.
"""
import json
import shutil
import sys
from pathlib import Path

NS = "bibliocraft"
WOODS = ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "crimson", "warped", "mangrove",
         "bamboo", "cherry"]
WOOD_PT = {"oak": "Carvalho", "spruce": "Abeto", "birch": "Bétula", "jungle": "Selva", "acacia": "Acácia",
           "dark_oak": "Carvalho Escuro", "crimson": "Carmesim", "warped": "Distorcida",
           "mangrove": "Mangue", "bamboo": "Bambu", "cherry": "Cerejeira"}
FACINGS = {"north": 0, "east": 90, "south": 180, "west": 270}
GREEN = (0x5E, 0x7C, 0x16)  # DyeColor.GREEN.getTextureDiffuseColor(): a cor padrão do pedestal
DROP_KEYS = ("credit", "render_type", "loader", "groups")


def planks(w):
    return f"minecraft:{w}_planks"


def slab(w):
    return f"minecraft:{w}_slab"


def wjson(path: Path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def clean_model(d):
    for k in DROP_KEYS:
        d.pop(k, None)
    for k in [k for k in d if k.startswith("book_") or k == "pride_books"]:
        d.pop(k)
    return d


# ---- formas (outline/colisão), as do código do original em blocos (0..1) ----
def rot_cw(box):
    """ShapeUtil.rotate(..., CLOCKWISE_90): norte -> leste, visto de cima."""
    x0, y0, z0, x1, y1, z1 = box
    return [round(1 - z1, 6), y0, x0, round(1 - z0, 6), y1, x1]


def by_facing(north):
    out, cur = {}, north
    for f in ["north", "east", "south", "west"]:
        out[f] = [list(b) for b in cur]
        cur = [rot_cw(b) for b in cur]
    return out


BOOKCASE_N = [[0.0625, 0, 0.5, 0.9375, 0.0625, 0.9375], [0, 0, 0.9375, 1, 1, 1],
              [0.0625, 0.9375, 0.5, 0.9375, 1, 0.9375], [0.0625, 0.4375, 0.5, 0.9375, 0.5625, 0.9375],
              [0.9375, 0, 0.5, 1, 1, 0.9375], [0, 0, 0.5, 0.0625, 1, 0.9375]]
SHELF_N = [[0, 0, 0.5, 1, 1, 1]]
TABLE_NONE = [[0.0625, 0.875, 0, 0.9375, 1, 1], [0, 0.875, 0.0625, 0.0625, 1, 0.9375],
              [0.9375, 0.875, 0.0625, 1, 1, 0.9375], [0.4375, 0.0625, 0.4375, 0.5625, 0.875, 0.5625],
              [0.0625, 0, 0.4375, 0.3125, 0.09375, 0.5625], [0.1875, 0.09375, 0.4375, 0.4375, 0.1875, 0.5625],
              [0.6875, 0, 0.4375, 0.9375, 0.09375, 0.5625], [0.5625, 0.09375, 0.4375, 0.8125, 0.1875, 0.5625],
              [0.4375, 0, 0.0625, 0.5625, 0.09375, 0.3125], [0.4375, 0.09375, 0.1875, 0.5625, 0.1875, 0.4375],
              [0.4375, 0, 0.6875, 0.5625, 0.09375, 0.9375], [0.4375, 0.09375, 0.5625, 0.5625, 0.1875, 0.8125]]
SEAT = [[0.0625, 0.625, 0.0625, 0.9375, 0.8125, 0.9375], [0.125, 0.8125, 0.125, 0.875, 0.875, 0.875],
        [0.1875, 0, 0.1875, 0.3125, 0.625, 0.3125], [0.6875, 0, 0.1875, 0.8125, 0.625, 0.3125],
        [0.6875, 0, 0.6875, 0.8125, 0.625, 0.8125], [0.1875, 0, 0.6875, 0.3125, 0.625, 0.8125],
        [0.21875, 0.28125, 0.3125, 0.28125, 0.34375, 0.6875], [0.71875, 0.28125, 0.3125, 0.78125, 0.34375, 0.6875],
        [0.3125, 0.28125, 0.21875, 0.6875, 0.34375, 0.28125], [0.3125, 0.28125, 0.71875, 0.6875, 0.34375, 0.78125]]
PEDESTAL_Z = [[0, 0, 0.25, 1, 0.0625, 0.75], [0.0625, 0.0625, 0.25, 0.9375, 0.125, 0.75],
              [0.125, 0.125, 0.25, 0.875, 0.1875, 0.75], [0.1875, 0.1875, 0.25, 0.8125, 0.25, 0.75]]


def facing_shapes(north):
    return {"properties": ["facing"], "values": by_facing(north)}


def main(src: Path, out: Path):
    res = src / "src/main/resources/assets/bibliocraft"
    gen = src / "src/main/generated"
    if out.exists():
        shutil.rmtree(out)
    a, d = out / "assets" / NS, out / "data" / NS
    en_src = json.loads((gen / "assets/bibliocraft/lang/en_us.json").read_text(encoding="utf-8"))
    en, pt = {}, {}
    blocks = []

    # ---- modelos-modelo (templates) sem os loaders do NeoForge ----
    tpl = res / "models/block/template"
    for rel in ["bookcase/bookcase.json", "shelf.json", "table/none.json", "seat/seat.json"]:
        wjson(a / "models/block/template" / rel, clean_model(json.loads((tpl / rel).read_text())))
    # Os 16 livros: modelos avulsos (native_standalone_models), desenhados pelo nativo.
    for i in range(16):
        rel = f"bookcase/book_{i}.json"
        wjson(a / "models/block/template" / rel, clean_model(json.loads((tpl / rel).read_text())))
        (a / "textures/block/bookcase").mkdir(parents=True, exist_ok=True)
        shutil.copy(res / f"textures/block/bookcase/book_{i}.png", a / f"textures/block/bookcase/book_{i}.png")

    # ---- pedestal: a faixa colorida já pintada de verde (sem tintindex) ----
    ped = clean_model(json.loads((res / "models/block/sword_pedestal.json").read_text()))
    ped["textures"]["2"] = f"{NS}:block/sword_pedestal_overlay_green"
    for e in ped["elements"]:
        for f in e["faces"].values():
            f.pop("tintindex", None)
    wjson(a / "models/block/sword_pedestal.json", ped)
    tex = a / "textures/block"
    tex.mkdir(parents=True, exist_ok=True)
    for n in ["sword_pedestal_base", "sword_pedestal_side"]:
        shutil.copy(res / f"textures/block/{n}.png", tex / f"{n}.png")
    from PIL import Image  # só aqui: tingir a faixa
    img = Image.open(res / "textures/block/sword_pedestal_overlay.png").convert("RGBA")
    px = img.load()
    for y in range(img.height):
        for x in range(img.width):
            r, g, b, al = px[x, y]
            px[x, y] = (r * GREEN[0] // 255, g * GREEN[1] // 255, b * GREEN[2] // 255, al)
    img.save(tex / "sword_pedestal_overlay_green.png")

    def block(name, data, model, states, item_model, recipe, pt_name):
        blocks.append(name)
        wjson(d / f"horizon/block/{name}.json", data)
        wjson(a / f"blockstates/{name}.json", {"variants": states})
        wjson(a / f"models/item/{name}.json", {"parent": item_model})
        wjson(a / f"items/{name}.json", {"model": {"type": "minecraft:model", "model": f"{NS}:item/{name}"}})
        wjson(d / f"recipe/{name}.json", recipe)
        wjson(d / f"loot_table/blocks/{name}.json", {
            "type": "minecraft:block",
            "pools": [{"rolls": 1.0, "bonus_rolls": 0.0,
                       "conditions": [{"condition": "minecraft:survives_explosion"}],
                       "entries": [{"type": "minecraft:item", "name": f"{NS}:{name}"}]}],
            "random_sequence": f"{NS}:blocks/{name}"})
        key = f"block.{NS}.{name}"
        en[key] = en_src[key]
        pt[key] = pt_name

    def facing_states(model, lit):
        out = {}
        for f, y in FACINGS.items():
            v = {"model": model, "uvlock": True}
            if y:
                v["y"] = y
            for l in (["false", "true"] if lit else [None]):
                out[f"facing={f}" + (f",lit={l}" if l else "")] = v
        return out

    def shaped(name, pattern, key, group):
        return {"type": "minecraft:crafting_shaped", "category": "misc", "group": f"{NS}:{group}",
                "pattern": pattern, "key": key, "result": {"id": f"{NS}:{name}", "count": 1}}

    def wood_props(w):
        return {"hardness": 2.0, "explosion_resistance": 3.0, "sounds": planks(w), "requires_tool": False,
                "occludes": False}

    for w in WOODS:
        P, S, tx = planks(w), slab(w), f"minecraft:block/{w}_planks"
        wp = WOOD_PT[w]
        # Estante: modelo de fornalha (abre a tela), 16 casas.
        m = f"{NS}:block/wood/{w}/bookcase"
        wjson(a / f"models/block/wood/{w}/bookcase.json",
              {"parent": f"{NS}:block/template/bookcase/bookcase", "textures": {"texture": tx}})
        block(f"{w}_bookcase",
              {"template": "minecraft:furnace", "properties": wood_props(w), "item": True,
               "shapes": facing_shapes(BOOKCASE_N)},
              m, facing_states(m, True), m,
              shaped(f"{w}_bookcase", ["PSP", "PSP", "PSP"], {"P": P, "S": S}, "bookcases"),
              f"Estante de {wp}")
        m = f"{NS}:block/wood/{w}/shelf"
        wjson(a / f"models/block/wood/{w}/shelf.json",
              {"parent": f"{NS}:block/template/shelf", "textures": {"texture": tx}})
        block(f"{w}_shelf",
              {"template": "minecraft:furnace", "properties": wood_props(w), "item": True,
               "shapes": facing_shapes(SHELF_N)},
              m, facing_states(m, True), m,
              shaped(f"{w}_shelf", ["SSS", " P ", "SSS"], {"P": P, "S": S}, "shelves"),
              f"Prateleira de {wp}")
        m = f"{NS}:block/wood/{w}/table"
        wjson(a / f"models/block/wood/{w}/table.json",
              {"parent": f"{NS}:block/template/table/none", "textures": {"texture": tx}})
        block(f"{w}_table",
              {"template": P, "properties": {"occludes": False}, "item": True,
               "shapes": {"default": TABLE_NONE}},
              m, {"": {"model": m}}, m,
              shaped(f"{w}_table", ["SSS", " P ", " P "], {"P": P, "S": S}, "tables"),
              f"Mesa de {wp}")
        m = f"{NS}:block/color/red/wood/{w}/seat"
        wjson(a / f"models/block/color/red/wood/{w}/seat.json",
              {"parent": f"{NS}:block/template/seat/seat",
               "textures": {"color": "minecraft:block/red_wool", "texture": tx}})
        block(f"red_{w}_seat",
              {"template": P, "properties": {"occludes": False}, "item": True, "shapes": {"default": SEAT}},
              m, {"": {"model": m}}, m,
              shaped(f"red_{w}_seat", [" W ", " S ", "RSR"],
                     {"R": "minecraft:stick", "S": S, "W": "minecraft:red_wool"}, "seats"),
              f"Assento Vermelho de {wp}")

    m = f"{NS}:block/sword_pedestal"
    block("sword_pedestal",
          {"template": "minecraft:white_glazed_terracotta",
           "properties": {"hardness": 2.0, "explosion_resistance": 6.0, "sounds": "minecraft:smooth_stone",
                          "requires_tool": True, "occludes": False},
           "item": True,
           "shapes": {"properties": ["facing"],
                      "values": {"north": PEDESTAL_Z, "south": PEDESTAL_Z,
                                 "east": [rot_cw(b) for b in PEDESTAL_Z],
                                 "west": [rot_cw(b) for b in PEDESTAL_Z]}}},
          m, {f"facing={f}": ({"model": m, "y": y} if y else {"model": m}) for f, y in FACINGS.items()}, m,
          shaped("sword_pedestal", [" S ", "SWS"],
                 {"S": "minecraft:smooth_stone_slab", "W": "minecraft:green_wool"}, "sword_pedestal"),
          "Pedestal de Espada")

    # ---- telas (tipos de menu do próprio mod) ----
    wjson(d / "horizon/menu/bookcase.json", {"slots": 16})
    wjson(d / "horizon/menu/shelf.json", {"slots": 4})

    # ---- aba criativa, tags, nomes ----
    order = [f"{NS}:{w}_{k}" for k in ["bookcase", "shelf", "table"] for w in WOODS]
    order += [f"{NS}:red_{w}_seat" for w in WOODS] + [f"{NS}:sword_pedestal"]
    wjson(d / "horizon/creative_tab/bibliocraft.json",
          {"title": "itemGroup.bibliocraft", "icon": f"{NS}:oak_bookcase", "items": order})
    wood_blocks = [f"{NS}:{b}" for b in blocks if b != "sword_pedestal"]
    wjson(out / "data/minecraft/tags/block/mineable/axe.json", {"replace": False, "values": wood_blocks})
    wjson(out / "data/minecraft/tags/block/mineable/pickaxe.json",
          {"replace": False, "values": [f"{NS}:sword_pedestal"]})
    for kind, tag in [("bookcase", "bookcases"), ("shelf", "shelves"), ("table", "tables")]:
        vals = [f"{NS}:{w}_{kind}" for w in WOODS]
        wjson(d / f"tags/item/{tag}.json", {"values": vals})
        wjson(d / f"tags/block/{tag}.json", {"values": vals})
    wjson(d / "tags/item/seats.json", {"values": [f"{NS}:red_{w}_seat" for w in WOODS]})
    # Os livros que a estante aceita (a mesma lista que o nativo usa: BOOKS em runtime.rs).
    wjson(d / "tags/item/bookcase_books.json",
          {"values": ["#minecraft:bookshelf_books"]})
    wjson(d / "tags/item/sword_pedestal_swords.json", {"values": ["#minecraft:swords"]})
    for k in ["itemGroup.bibliocraft", "container.bibliocraft.bookcase", "container.bibliocraft.shelf"]:
        en[k] = en_src[k]
    pt.update({"itemGroup.bibliocraft": "Bibliocraft", "container.bibliocraft.bookcase": "Estante",
               "container.bibliocraft.shelf": "Prateleira"})
    wjson(a / "lang/en_us.json", dict(sorted(en.items())))
    wjson(a / "lang/pt_br.json", dict(sorted(pt.items())))

    (out / "horizon-mod.toml").write_text(
        'id = "bibliocraft"\n'
        'name = "Bibliocraft Legacy"\n'
        'version = "1.6.6-horizon.1"\n'
        'description = "Estantes, prateleiras, mesas e assentos das 11 madeiras do original e o pedestal de espada. '
        'Precisa do mod nativo bibliocraft_native 0.1.0."\n'
        'authors = ["IchHabeHunger54, Minecraftschurli (Bibliocraft Legacy, MIT)", "Nuchaz (BiblioCraft original)", '
        '"port Horizon"]\n'
        'api = "0.2"\n'
        'side = "both"\n'
        'capabilities = ["registries"]\n', encoding="utf-8")
    print(f"{len(blocks)} blocos em {out}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    main(Path(sys.argv[1]), Path(sys.argv[2]))

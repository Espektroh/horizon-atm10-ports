#!/usr/bin/env python3
"""Gera o conteúdo do Mekanism Generators 0.1.0 para Horizon e o mekanism.hmod corrigido.

Uso (de dentro de source/):
  python3 -I scripts/make_content.py MekanismGenerators.jar MEKANISM_0.6.1.hmod OUT_DIR

- MekanismGenerators.jar: o do ATM10 (10.7.19, MIT). Só lê texturas, modelos e lang.
- MEKANISM_0.6.1.hmod: o mekanism.hmod do pacote Mekanism-Horizon-0.6.1. Sai como 10.7.19-horizon.7:
  * deixa de reclamar o namespace "mekanismgenerators" (senão o Horizon desliga um dos dois mods);
    os 3 modelos e 3 texturas dos geradores antigos (mekanism:heat_generator e mekanism:solar_generator)
    passam para assets/mekanism/.../legacy_generators/ e as referências são reescritas;
  * a tag mekanism:cable_connects ganha os 4 geradores novos (o cabo se liga a eles na tela);
  * os nomes dos geradores antigos ganham "(antigo)". Nada mais muda (mesmos ids, receitas, NBT).
Escreve OUT_DIR/content/ (mekanismgenerators), OUT_DIR/mekanism-content/ e os dois .hmod.
"""
import json
import shutil
import sys
import zipfile
from pathlib import Path

VERSION = "10.7.19-horizon.1"
VERSION_MEK = "10.7.19-horizon.7"
NS = "mekanismgenerators"
BLOCKS = ["solar_generator", "advanced_solar_generator", "heat_generator", "wind_generator"]
LEGACY_NAMES = {
    "en_us": {"block.mekanism.heat_generator": "Heat Generator (old)",
              "block.mekanism.solar_generator": "Solar Generator (old)"},
    "pt_br": {"block.mekanism.heat_generator": "Gerador a Calor (antigo)",
              "block.mekanism.solar_generator": "Gerador Solar (antigo)"},
}


def dump(path: Path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


class Jar:
    def __init__(self, path):
        self.zip = zipfile.ZipFile(path)
        self.names = set(self.zip.namelist())

    def json(self, name):
        return json.loads(self.zip.read(name))

    def copy(self, name, dest: Path):
        if name not in self.names:
            raise SystemExit(f"falta no jar: {name}")
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(self.zip.read(name))


def shaped(pattern, key, result, count=1):
    return {"type": "minecraft:crafting_shaped", "category": "misc", "pattern": pattern, "key": key,
            "result": {"id": result, "count": count}}


def shapeless(ingredients, result, count=1):
    return {"type": "minecraft:crafting_shapeless", "category": "misc", "ingredients": ingredients,
            "result": {"id": result, "count": count}}


def loot(block):
    ns, name = block.split(":")
    return {"type": "minecraft:block", "pools": [{"rolls": 1.0, "bonus_rolls": 0.0,
            "conditions": [{"condition": "minecraft:survives_explosion"}],
            "entries": [{"type": "minecraft:item", "name": block}]}],
            "random_sequence": f"{ns}:blocks/{name}"}


def states(model, active=None, rotate=True):
    """facing x lit (o template de fornalha): lit=true usa `active` quando houver."""
    variants = {}
    for facing, y in (("north", 0), ("east", 90), ("south", 180), ("west", 270)):
        for lit in ("false", "true"):
            v = {"model": active if (lit == "true" and active) else model}
            if rotate and y:
                v["y"] = y
            variants[f"facing={facing},lit={lit}"] = v
    return {"variants": variants}


def advanced_solar_model(m):
    """O modelo do original vai de y=-16 a 32 (é desenhado a partir do bloco do meio da estrutura 3x3x3).
    Aqui é um bloco só: a base sobe 16 px e o mastro encurta, para tudo caber em 0..32 (limite do vanilla)."""
    for e in m["elements"]:
        f, t = e["from"], e["to"]
        if t[1] <= -4:          # base, porta, conector
            f[1] += 16
            t[1] += 16
        elif f[1] < -4 < t[1] or f[1] == -4:  # mastro: começa no topo da base
            f[1] = 12
    lo = min(e["from"][1] for e in m["elements"])
    hi = max(e["to"][1] for e in m["elements"])
    assert 0 <= lo and hi <= 32, (lo, hi)
    return m


def box_uv(u, v, w, h, d):
    """A planificação de um cubo de modelo de entidade (texOffs) numa textura 128x128, em unidades 0..16."""
    s = 8.0
    r = lambda a, b, c, e: [a / s, b / s, c / s, e / s]
    return {
        "up": r(u + d, v, u + d + w, v + d),
        "down": r(u + d + w, v, u + d + 2 * w, v + d),
        "west": r(u, v + d, u + d, v + d + h),
        "north": r(u + d, v + d, u + d + w, v + d + h),
        "east": r(u + d + w, v + d, u + 2 * d + w, v + d + h),
        "south": r(u + 2 * d + w, v + d, u + 2 * d + 2 * w, v + d + h),
    }


def wind_model():
    """Gerador eólico compacto (2 blocos de altura visual): base, aro, mastro, cabeça e pás paradas, com a
    textura do original (render/wind_generator.png). O original tem 5 blocos e pás girando (renderizador Java)."""
    parts = [
        # nome, from, to, texOffs+tamanho do cubo do original (para a UV)
        ("base", [0, 0, 0], [16, 2, 16], (10, 64, 16, 2, 16)),
        ("base_rim", [2, 2, 2], [14, 4, 14], (26, 50, 12, 2, 12)),
        ("plate", [4, 4, 0], [12, 12, 1], (42, 25, 8, 8, 1)),
        ("plate_connector", [5, 4, 1], [11, 10, 7], (42, 34, 6, 10, 6)),
        ("mast", [7, 4, 7], [9, 26, 9], (74, 0, 2, 65, 2)),
        ("head", [4.5, 20.5, 3], [11.5, 27.5, 12], (20, 0, 7, 7, 9)),
        ("rear_plate", [5.5, 21, 12], [10.5, 27, 15], (20, 16, 5, 6, 3)),
        ("blade_center", [6, 22, 0], [10, 26, 3], (20, 25, 4, 4, 3)),
        ("blade_cap", [7, 23, -1], [9, 25, 0], (22, 0, 2, 2, 1)),
    ]
    elements = []
    for name, f, t, (u, v, w, h, d) in parts:
        uv = box_uv(u, v, w, h, d)
        elements.append({"name": name, "from": f, "to": t,
                         "faces": {k: {"uv": uv[k], "texture": "#0"} for k in uv}})
    # três pás de 2 px x 32 px: uma de pé e duas a 45 graus (vanilla só aceita 0, ±22,5 e ±45)
    blade_uv = box_uv(20, 32, 2, 32, 1)
    for name, angle in (("blade_1", 0), ("blade_2", 45), ("blade_3", -45)):
        e = {"name": name, "from": [7, 8, 0.5], "to": [9, 32, 1.5],
             "faces": {k: {"uv": blade_uv[k], "texture": "#0"} for k in blade_uv}}
        if angle:
            e["rotation"] = {"angle": angle, "axis": "z", "origin": [8, 24, 1]}
        elements.append(e)
    tex = f"{NS}:block/model/wind_generator"
    return {"credit": "Simplificado do ModelWindGenerator do Mekanism (MIT)", "texture_size": [128, 128],
            "textures": {"0": tex, "particle": tex}, "render_type": "minecraft:cutout", "elements": elements}


def generators(jar: Jar, out: Path):
    if out.exists():
        shutil.rmtree(out)
    data, assets = out / f"data/{NS}", out / f"assets/{NS}"
    src = f"assets/{NS}"
    # blocos
    for b in BLOCKS:
        dump(data / f"horizon/block/{b}.json", {"template": "minecraft:furnace", "properties": {"hardness": 3.5}})
        dump(data / f"loot_table/blocks/{b}.json", loot(f"{NS}:{b}"))
    # modelos e texturas
    for m in ("heat_generator", "heat_generator_active", "solar_generator"):
        dump(assets / f"models/block/{m}.json", jar.json(f"{src}/models/block/{m}.json"))
    dump(assets / "models/block/advanced_solar_generator.json",
         advanced_solar_model(jar.json(f"{src}/models/block/advanced_solar_generator.json")))
    dump(assets / "models/block/wind_generator.json", wind_model())
    for t in ("heat_generator", "heat_generator_led", "solar_generator", "advanced_solar_generator"):
        jar.copy(f"{src}/textures/block/model/{t}.png", assets / f"textures/block/model/{t}.png")
    jar.copy(f"{src}/render/wind_generator.png", assets / "textures/block/model/wind_generator.png")
    dump(assets / "blockstates/heat_generator.json",
         states(f"{NS}:block/heat_generator", f"{NS}:block/heat_generator_active"))
    dump(assets / "blockstates/solar_generator.json", states(f"{NS}:block/solar_generator", rotate=False))
    # o original gira o modelo avançado e o eólico pela frente, como aqui
    dump(assets / "blockstates/advanced_solar_generator.json", states(f"{NS}:block/advanced_solar_generator"))
    dump(assets / "blockstates/wind_generator.json", states(f"{NS}:block/wind_generator"))
    for b in BLOCKS:
        dump(assets / f"items/{b}.json", {"model": {"type": "minecraft:model", "model": f"{NS}:block/{b}"}})
    # item: painel solar
    dump(data / "horizon/item/solar_panel.json", {"template": "minecraft:iron_ingot"})
    jar.copy(f"{src}/textures/item/solar_panel.png", assets / "textures/item/solar_panel.png")
    dump(assets / "models/item/solar_panel.json",
         {"parent": "minecraft:item/generated", "textures": {"layer0": f"{NS}:item/solar_panel"}})
    dump(assets / "items/solar_panel.json", {"model": {"type": "minecraft:model", "model": f"{NS}:item/solar_panel"}})
    # receitas (as do original; tablete de energia -> bloco de redstone, como o port do Mekanism faz)
    r = data / "recipe"
    alloy, osmium, iron = "mekanism:alloy_infused", "mekanism:ingot_osmium", "minecraft:iron_ingot"
    dump(r / "solar_panel.json", shaped(["GGG", "RAR", "OOO"], {
        "G": "minecraft:glass_pane", "R": "minecraft:redstone", "A": alloy, "O": osmium}, f"{NS}:solar_panel"))
    dump(r / "generator/solar.json", shaped(["###", "AIA", "OEO"], {
        "#": f"{NS}:solar_panel", "A": alloy, "I": iron, "O": osmium, "E": "minecraft:redstone_block"},
        f"{NS}:solar_generator"))
    dump(r / "generator/advanced_solar.json", shaped(["PAP", "PAP", "III"], {
        "P": f"{NS}:solar_generator", "A": alloy, "I": iron}, f"{NS}:advanced_solar_generator"))
    dump(r / "generator/wind.json", shaped([" O ", "OAO", "ECE"], {
        "O": osmium, "A": alloy, "E": "minecraft:redstone_block", "C": "mekanism:basic_control_circuit"},
        f"{NS}:wind_generator"))
    # O padrão do original (III/WOW/CFC) já faz o mekanism:heat_generator do port; aqui ele é trocado.
    dump(r / "generator/heat_from_mekanism.json", shapeless(["mekanism:heat_generator"], f"{NS}:heat_generator"))
    dump(r / "generator/solar_from_mekanism.json", shapeless(["mekanism:solar_generator"], f"{NS}:solar_generator"))
    # tags vanilla: picareta
    dump(out / "data/minecraft/tags/block/mineable/pickaxe.json", {"values": [f"{NS}:{b}" for b in BLOCKS]})
    # lang
    for lang in ("en_us", "pt_br"):
        s = jar.json(f"{src}/lang/{lang}.json")
        keys = [f"block.{NS}.{b}" for b in BLOCKS] + [f"item.{NS}.solar_panel"]
        dump(assets / f"lang/{lang}.json", {k: s[k] for k in keys})
    (out / "horizon-mod.toml").write_text(
        f'id = "{NS}"\n'
        'name = "Mekanism Generators"\n'
        f'version = "{VERSION}"\n'
        'description = "Geradores do Mekanism: solar, solar avancado, a calor e eolico. Precisa do mod nativo '
        'mekanismgenerators_native 0.1.0 e do mekanism (10.7.19-horizon.7 ou mais novo)."\n'
        'authors = ["Mekanism team (original, MIT)", "Espektroh (port Horizon)"]\n'
        'api = "0.2"\n'
        'side = "both"\n'
        'capabilities = ["registries"]\n')


def mekanism_patch(base: Path, out: Path):
    if out.exists():
        shutil.rmtree(out)
    with zipfile.ZipFile(base) as z:
        z.extractall(out)
    old = out / f"assets/{NS}"
    assert old.exists(), "o mekanism.hmod 0.6.1 já não tem assets/mekanismgenerators?"
    mek = out / "assets/mekanism"
    moves = {}
    for p in sorted(old.rglob("*")):
        if p.is_file():
            rel = p.relative_to(old)  # models/block/x.json ou textures/block/model/x.png
            kind, rest = rel.parts[0], Path(*rel.parts[1:])
            dest = mek / kind / "block/legacy_generators" / rest.name
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.move(str(p), dest)
            stem = rest.with_suffix("").as_posix()
            moves[f"{NS}:{stem}"] = f"mekanism:block/legacy_generators/{rest.stem}"
    shutil.rmtree(old)

    def rewrite(path: Path):
        text = path.read_text(encoding="utf-8")
        for a, b in sorted(moves.items(), key=lambda kv: -len(kv[0])):
            text = text.replace(f'"{a}"', f'"{b}"')
        assert f"{NS}:" not in text, (path, text[:200])
        path.write_text(text, encoding="utf-8")

    for p in list(mek.rglob("*.json")):
        if NS in p.read_text(encoding="utf-8"):
            rewrite(p)
    tag = out / "data/mekanism/tags/block/cable_connects.json"
    values = json.loads(tag.read_text())["values"]
    values += [f"{NS}:{b}" for b in BLOCKS if f"{NS}:{b}" not in values]
    dump(tag, {"values": values})
    for lang, names in LEGACY_NAMES.items():
        path = mek / f"lang/{lang}.json"
        have = json.loads(path.read_text(encoding="utf-8"))
        have.update(names)
        dump(path, have)
    manifest = (out / "horizon-mod.toml").read_text()
    assert 'version = "10.7.19-horizon.6"' in manifest and 'namespaces = ["mekanismgenerators"]' in manifest
    manifest = manifest.replace('version = "10.7.19-horizon.6"', f'version = "{VERSION_MEK}"')
    manifest = manifest.replace('namespaces = ["mekanismgenerators"]\n', "")
    manifest = manifest.replace("Precisa do mod nativo mekanism_native 0.6.0.",
                                "Precisa do mod nativo mekanism_native 0.6.x. Sem o namespace mekanismgenerators "
                                "(fica para o mod mekanismgenerators).")
    (out / "horizon-mod.toml").write_text(manifest)


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
    if len(sys.argv) != 4:
        raise SystemExit(__doc__)
    jar, base, out = Jar(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    generators(jar, out / "content")
    mekanism_patch(base, out / "mekanism-content")
    print(json.dumps({"blocks": BLOCKS, "items": ["solar_panel"]}))


if __name__ == "__main__":
    main()

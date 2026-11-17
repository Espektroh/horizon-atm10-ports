#!/usr/bin/env python3
"""Gera o conteúdo do Mekanism: MoreMachine 0.1.0 para Horizon (mod de conteúdo `mekmm`).

Uso (de dentro de source/):
  python3 -I scripts/make_content.py MEKMM_SRC MEKANISM_CONTENT OUT_DIR

- MEKMM_SRC: o clone do código oficial (github.com/lostmyself8/Mekanism-MoreMachine, branch 1.21.1, MIT).
  Só lê arquivos de dados: texturas, modelos, blockstates, lang e as receitas JSON.
- MEKANISM_CONTENT: a pasta source/content do Mekanism-Horizon-0.6.1 (só para copiar a textura
  `mekanism:block/bottom`, que os modelos do MoreMachine usam, para dentro do namespace mekmm).
Escreve OUT_DIR/ (o conteúdo, ainda no formato de loot 1.21.1: rode upgrade_hmod.py depois).
"""
import json
import shutil
import sys
from pathlib import Path

VERSION = "1.4.3-horizon.1"
NS = "mekmm"
# bloco -> (tipo de receita do original, nome dos modelos)
MACHINES = ["recycler", "cnc_stamper", "cnc_lathe", "cnc_rolling_mill", "presser"]
ITEMS = ["scrap"]
PT_BR_NAMES = {"block.mekmm.presser": "Prensa"}

ALLOY_BASIC, ALLOY_ADVANCED, ALLOY_ELITE = "mekanism:alloy_infused", "mekanism:alloy_reinforced", "mekanism:alloy_atomic"
CIRCUIT_BASIC = "mekanism:basic_control_circuit"
CIRCUIT_ADVANCED = "mekanism:advanced_control_circuit"
CIRCUIT_ELITE = "mekanism:elite_control_circuit"
CASING = "mekanism:steel_casing"
# o original aceita pistão ou pistão grudento; aqui só o pistão (chave de texto simples)
PISTONS = "minecraft:piston"
# c:stones da NeoForge (o 26.3 não traz as tags c:), por extenso
STONES = ["minecraft:stone", "minecraft:andesite", "minecraft:diorite", "minecraft:granite",
          "minecraft:deepslate", "minecraft:tuff"]


def dump(path: Path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def shaped(pattern, key, result, count=1):
    return {"type": "minecraft:crafting_shaped", "category": "misc", "pattern": pattern, "key": key,
            "result": {"id": result, "count": count}}


def loot(block):
    ns, name = block.split(":")
    return {"type": "minecraft:block", "pools": [{"rolls": 1.0, "bonus_rolls": 0.0,
            "conditions": [{"condition": "minecraft:survives_explosion"}],
            "entries": [{"type": "minecraft:item", "name": block}]}],
            "random_sequence": f"{ns}:blocks/{name}"}


def states(model, active):
    """facing x lit (template de fornalha), com os ângulos do original (west = -90 -> 270)."""
    variants = {}
    for facing, y in (("north", 0), ("east", 90), ("south", 180), ("west", 270)):
        for lit in ("false", "true"):
            v = {"model": active if lit == "true" else model}
            if y:
                v["y"] = y
            variants[f"facing={facing},lit={lit}"] = v
    return {"variants": variants}


def fix_model(m):
    m = dict(m)
    m.pop("format_version", None)
    if m.get("parent") == "block/block":
        m["parent"] = "minecraft:block/block"
    tex = m.get("textures", {})
    for k, t in list(tex.items()):
        if t == "mekanism:block/bottom":
            tex[k] = f"{NS}:block/bottom"
    return m


def machine_recipe(kind, inputs, result, count=1):
    """Receita de máquina no formato que o glue lê: ingrediente como texto ou lista de textos
    (formato de `native_recipe_ingredient`), quantidade em *_count."""
    out = {"type": f"{NS}:{kind}"}
    out.update(inputs)
    out["output"] = {"id": result, "count": count}
    return out


def processing(r: Path):
    # --- Reciclador (receitas do original, data/mekmm/recipe/recycler/*.json) ---
    dump(r / "recycler/from_stone.json", {"type": f"{NS}:recycler", "input": STONES, "input_count": 1,
                                          "chance": 0.17, "output": {"id": f"{NS}:scrap", "count": 1}})
    dump(r / "recycler/from_dirt.json", {"type": f"{NS}:recycler",
                                         "input": ["minecraft:dirt", "minecraft:grass_block"], "input_count": 1,
                                         "chance": 0.17, "output": {"id": f"{NS}:scrap", "count": 1}})
    # from_substrate (mekanism:substrate, 0.43) fica de fora: o substrato não foi portado.

    # --- Estampador CNC: compat AE2 do original (o molde/prensa não é gasto) ---
    ae2_stamp = {
        "printed_silicon": ("ae2:silicon", "ae2:silicon_press"),
        "printed_logic_processor": ("minecraft:gold_ingot", "ae2:logic_processor_press"),
        "printed_engineering_processor": ("minecraft:diamond", "ae2:engineering_processor_press"),
        "printed_calculation_processor": (["ae2:certus_quartz_crystal", "ae2:charged_certus_quartz_crystal"],
                                          "ae2:calculation_processor_press"),
    }
    for out, (inp, mold) in ae2_stamp.items():
        dump(r / f"compat/ae2/stamper/{out}.json", machine_recipe("stamper", {
            "input": inp, "input_count": 1, "mold": mold}, f"ae2:{out}"))
    # receitas do port (vanilla), para dar para usar e testar sem AE2/IE: placas de pressão como molde
    dump(r / "stamper/heavy_plate.json", machine_recipe("stamper", {
        "input": "minecraft:iron_ingot", "input_count": 1, "mold": "minecraft:heavy_weighted_pressure_plate"},
        "minecraft:heavy_weighted_pressure_plate"))
    dump(r / "stamper/light_plate.json", machine_recipe("stamper", {
        "input": "minecraft:gold_ingot", "input_count": 1, "mold": "minecraft:light_weighted_pressure_plate"},
        "minecraft:light_weighted_pressure_plate"))

    # --- Prensa: compat AE2 do original (as três entradas são gastas) ---
    for proc in ("logic", "engineering", "calculation"):
        dump(r / f"compat/ae2/presser/{proc}_processor.json", machine_recipe("pressing", {
            "primary_input": f"ae2:printed_{proc}_processor", "primary_count": 1,
            "secondary_input": "minecraft:redstone", "secondary_count": 1,
            "tertiary_input": "ae2:printed_silicon", "tertiary_count": 1}, f"ae2:{proc}_processor"))
    # receita do port (vanilla): livro + tinta + pena -> livro e pena
    dump(r / "presser/writable_book.json", machine_recipe("pressing", {
        "primary_input": "minecraft:book", "primary_count": 1,
        "secondary_input": "minecraft:ink_sac", "secondary_count": 1,
        "tertiary_input": "minecraft:feather", "tertiary_count": 1}, "minecraft:writable_book"))
    # O refletor (3 painéis de vidro + 3 lingotes de prata + 3 folhas de PEAD) fica de fora: sem prata/PEAD.

    # --- Torno CNC e Laminador CNC: o original só tem receitas de compat (IE/Create, não portados) ---
    dump(r / "lathe/stick_from_planks.json", machine_recipe("lathe", {
        "input": "#minecraft:planks", "input_count": 1}, "minecraft:stick", 4))
    dump(r / "lathe/lightning_rod.json", machine_recipe("lathe", {
        "input": "minecraft:copper_ingot", "input_count": 3}, "minecraft:lightning_rod", 1))
    dump(r / "rolling_mill/paper.json", machine_recipe("rolling_mill", {
        "input": "minecraft:sugar_cane", "input_count": 1}, "minecraft:paper", 2))
    dump(r / "rolling_mill/iron_bars.json", machine_recipe("rolling_mill", {
        "input": "minecraft:iron_ingot", "input_count": 3}, "minecraft:iron_bars", 8))


def generate(src: Path, mek_content: Path, out: Path):
    if out.exists():
        shutil.rmtree(out)
    res = src / "src/main/resources/assets/mekmm"
    data, assets = out / f"data/{NS}", out / f"assets/{NS}"
    for b in MACHINES:
        dump(data / f"horizon/block/{b}.json", {"template": "minecraft:furnace", "properties": {"hardness": 3.5}})
        dump(data / f"loot_table/blocks/{b}.json", loot(f"{NS}:{b}"))
        for m in (b, f"{b}_active"):
            dump(assets / f"models/block/{m}.json", fix_model(json.loads((res / f"models/block/{m}.json").read_text())))
        tex = res / f"textures/block/{b}"
        for p in sorted(tex.iterdir()):
            dest = assets / f"textures/block/{b}/{p.name}"
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(p, dest)
        dump(assets / f"blockstates/{b}.json", states(f"{NS}:block/{b}", f"{NS}:block/{b}_active"))
        dump(assets / f"items/{b}.json", {"model": {"type": "minecraft:model", "model": f"{NS}:block/{b}"}})
    shutil.copyfile(mek_content / "assets/mekanism/textures/block/bottom.png", assets / "textures/block/bottom.png")
    # o menu próprio da prensa: 3 entradas + saída (o de fornalha só tem 3 slots)
    dump(data / "horizon/menu/presser.json", {"slots": 4, "data_slots": 4, "results": [3]})
    # item: sucata
    dump(data / "horizon/item/scrap.json", {"template": "minecraft:iron_nugget"})
    (assets / "textures/item").mkdir(parents=True, exist_ok=True)
    shutil.copyfile(res / "textures/item/scrap.png", assets / "textures/item/scrap.png")
    dump(assets / "models/item/scrap.json", {"parent": "minecraft:item/generated",
                                             "textures": {"layer0": f"{NS}:item/scrap"}})
    dump(assets / "items/scrap.json", {"model": {"type": "minecraft:model", "model": f"{NS}:item/scrap"}})

    # receitas das máquinas (as do original, com o que o port do Mekanism tem)
    r = data / "recipe"
    dump(r / "recycler.json", shaped(["ACA", "O#O", "ACA"], {
        "A": ALLOY_ADVANCED, "C": CIRCUIT_ADVANCED, "O": "mekanism:ingot_osmium", "#": "mekanism:crusher"},
        f"{NS}:recycler"))
    dump(r / "cnc_stamper.json", shaped(["ACA", "#X#", "ACA"], {
        "A": ALLOY_BASIC, "C": CIRCUIT_BASIC, "#": PISTONS, "X": CASING}, f"{NS}:cnc_stamper"))
    # o Robit (não portado) vira rebolo (grindstone)
    dump(r / "cnc_lathe.json", shaped(["ACA", "#X#", "ACA"], {
        "A": ALLOY_BASIC, "C": CIRCUIT_BASIC, "#": "minecraft:grindstone", "X": CASING}, f"{NS}:cnc_lathe"))
    dump(r / "cnc_rolling_mill.json", shaped(["ACA", "SXS", "ACA"], {
        "A": ALLOY_BASIC, "C": CIRCUIT_BASIC, "S": "mekanism:ingot_steel", "X": CASING}, f"{NS}:cnc_rolling_mill"))
    dump(r / "presser.json", shaped(["A#A", "CMC", "A#A"], {
        "A": ALLOY_ELITE, "#": PISTONS, "C": CIRCUIT_ELITE, "M": f"{NS}:cnc_stamper"}, f"{NS}:presser"))
    processing(r)

    dump(out / "data/minecraft/tags/block/mineable/pickaxe.json", {"values": [f"{NS}:{b}" for b in MACHINES]})
    for lang in ("en_us", "pt_br"):
        s = json.loads((res / f"lang/{lang}.json").read_text(encoding="utf-8"))
        keys = [f"block.{NS}.{b}" for b in MACHINES] + [f"item.{NS}.{i}" for i in ITEMS]
        names = {k: s[k] for k in keys}
        if lang == "pt_br":
            names.update(PT_BR_NAMES)
        names[f"container.{NS}.presser"] = "Prensa" if lang == "pt_br" else "Presser"
        dump(assets / f"lang/{lang}.json", names)
    (out / "horizon-mod.toml").write_text(
        f'id = "{NS}"\n'
        'name = "Mekanism: MoreMachine"\n'
        f'version = "{VERSION}"\n'
        'description = "Mekanism MoreMachine: Reciclador, Estampador CNC, Torno CNC, Laminador CNC e Prensa. '
        'Precisa do mod nativo mekmm_native 0.1.0, do mekanism e do mekanism_native 0.6.x."\n'
        'authors = ["lostmyself8 / Jerry (original, MIT)", "Espektroh (port Horizon)"]\n'
        'api = "0.2"\n'
        'side = "both"\n'
        'capabilities = ["registries"]\n')


def pack(src: Path, dest: Path):
    """Zip com datas e modos fixos: mesmo conteúdo, mesmo SHA-256."""
    import zipfile
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
    generate(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]))
    print(json.dumps({"blocks": MACHINES, "items": ITEMS}))


if __name__ == "__main__":
    main()

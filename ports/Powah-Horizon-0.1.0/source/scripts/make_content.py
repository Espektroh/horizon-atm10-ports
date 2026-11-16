#!/usr/bin/env python3
"""Gera o conteúdo do Powah 0.1.0 para Horizon (powah.hmod) a partir do jar do ATM10.

Uso (de dentro de source/):
  python3 -I scripts/make_content.py Powah-6.2.10.jar content
  python3 -I scripts/upgrade_hmod.py --dir content/powah      (loot no formato do 26.3)
  python3 -I scripts/check_assets.py content

Powah (owmii, Technici4n, shartte) é LGPLv3: este script só lê do jar texturas, modelos, lang e receitas.
O que entra (começo da progressão do ATM10): Orbe Energizante, Hastes Energizantes inicial/básica, Furnator e
Magmator inicial/básico, Célula de Energia inicial/básica, Cabo de Energia inicial/básico, cristais, aço
energizado, os itens dielétricos e os capacitores básicos. Receitas que pedem algo fora disso ficam de fora
(listadas em content-report.txt).
"""
import json
import shutil
import sys
import zipfile
from pathlib import Path

NS = "powah"
VERSION = "6.2.10-horizon.1"
TIERS = ["starter", "basic"]

# bloco: (template, tipo de modelo)
BLOCKS = {}
for t in TIERS:
    BLOCKS[f"furnator_{t}"] = "furnator"
    BLOCKS[f"magmator_{t}"] = "magmator"
    BLOCKS[f"energy_cell_{t}"] = "cell"
    BLOCKS[f"energizing_rod_{t}"] = "rod"
    BLOCKS[f"energy_cable_{t}"] = "cable"
BLOCKS["energizing_orb"] = "orb"
STORAGE = ["energized_steel_block", "blazing_crystal_block", "niotic_crystal_block", "spirited_crystal_block",
           "nitro_crystal_block"]
for b in STORAGE:
    BLOCKS[b] = "storage"

ITEMS = ["steel_energized", "crystal_blazing", "crystal_niotic", "crystal_spirited", "crystal_nitro",
         "dielectric_paste", "dielectric_rod", "dielectric_rod_horizontal", "dielectric_casing",
         "capacitor_basic_tiny", "capacitor_basic"]

# Ordem da aba criativa.
TAB = (["energizing_orb"] + [f"energizing_rod_{t}" for t in TIERS] + [f"furnator_{t}" for t in TIERS]
       + [f"magmator_{t}" for t in TIERS] + [f"energy_cell_{t}" for t in TIERS]
       + [f"energy_cable_{t}" for t in TIERS] + ITEMS + STORAGE)

# Tags `c:` que o Horizon não define: o item vanilla que cada uma quer dizer aqui.
C_TAGS = {
    "c:ingots/iron": "minecraft:iron_ingot", "c:ingots/gold": "minecraft:gold_ingot",
    "c:gems/diamond": "minecraft:diamond", "c:gems/emerald": "minecraft:emerald",
    "c:nether_stars": "minecraft:nether_star", "c:rods/blaze": "minecraft:blaze_rod",
    "c:storage_blocks/redstone": "minecraft:redstone_block", "c:storage_blocks/quartz": "minecraft:quartz_block",
    "c:glass_blocks": "minecraft:glass",
}
VANILLA_TAGS = {"minecraft:coals"}

# Receitas de energização do jar que entram (as outras pedem itens não portados).
ENERGIZING = ["energized_steel", "blazing_crystal", "blazing_crystal_2", "niotic_crystal", "spirited_crystal",
              "nitro_crystal"]

PT_OVERRIDES = {}
LOG = []


def dump(path: Path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


class Jar:
    def __init__(self, path):
        self.zip = zipfile.ZipFile(path)
        self.names = set(self.zip.namelist())

    def json(self, name):
        return json.loads(self.zip.read(name))

    def has(self, name):
        return name in self.names

    def copy(self, name, dest: Path):
        if name not in self.names:
            raise SystemExit(f"falta no jar: {name}")
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(self.zip.read(name))
        if name + ".mcmeta" in self.names:
            Path(str(dest) + ".mcmeta").write_bytes(self.zip.read(name + ".mcmeta"))


class Out:
    def __init__(self, jar: Jar, root: Path):
        self.jar, self.root = jar, root
        self.assets, self.data = root / "assets" / NS, root / "data" / NS
        self.models = set()

    def ref(self, r):
        ns, path = r.split(":", 1) if ":" in r else ("minecraft", r)
        return ns, path

    def texture(self, r):
        ns, path = self.ref(r)
        if ns != NS:
            return
        dest = self.assets / "textures" / f"{path}.png"
        if not dest.exists():
            self.jar.copy(f"assets/{NS}/textures/{path}.png", dest)

    def model(self, r):
        """Copia o modelo `r` do jar e, em cadeia, o pai e as texturas dele."""
        ns, path = self.ref(r)
        if ns != NS or r in self.models:
            return
        self.models.add(r)
        m = self.jar.json(f"assets/{NS}/models/{path}.json")
        m.pop("credit", None)
        parent = m.get("parent")
        if parent == "block":  # o energy_cell do jar: o pai vanilla é block/block
            m["parent"] = "minecraft:block/block"
        elif parent:
            self.model(parent)
        for t in m.get("textures", {}).values():
            if not t.startswith("#"):
                self.texture(t)
        dump(self.assets / "models" / f"{path}.json", m)

    def own_model(self, path, value):
        self.models.add(f"{NS}:{path}")
        dump(self.assets / "models" / f"{path}.json", value)
        if value.get("parent"):
            self.model(value["parent"])
        for t in value.get("textures", {}).values():
            if not t.startswith("#"):
                self.texture(t)

    def item_def(self, name, model):
        self.model(model)
        dump(self.assets / "items" / f"{name}.json", {"model": {"type": "minecraft:model", "model": model}})


def loot(name):
    return {"type": "minecraft:block", "pools": [{"rolls": 1.0, "bonus_rolls": 0.0,
            "conditions": [{"condition": "minecraft:survives_explosion"}],
            "entries": [{"type": "minecraft:item", "name": f"{NS}:{name}"}]}],
            "random_sequence": f"{NS}:blocks/{name}"}


HORIZONTAL = (("north", 0), ("east", 90), ("south", 180), ("west", 270))
OPPOSITE = {"up": "down", "down": "up", "north": "south", "south": "north", "east": "west", "west": "east"}


def furnace_states(model, lit_model, rotate=True):
    variants = {}
    for facing, y in HORIZONTAL:
        for lit, m in (("false", model), ("true", lit_model)):
            v = {"model": m}
            if rotate and y:
                v["y"] = y
            variants[f"facing={facing},lit={lit}"] = v
    return {"variants": variants}


def blocks(out: Out):
    jar = out.jar
    for name, kind in BLOCKS.items():
        decl = {"template": "minecraft:furnace", "properties": {"hardness": 2.0}}
        if kind in ("furnator", "magmator"):
            tier = name.split("_")[1]
            if kind == "furnator":
                # O jar não tem modelo aceso do Furnator (o original troca a textura em código):
                # o "_on" é o mesmo modelo com a face acesa (furnator_lit, animada).
                base = jar.json(f"assets/{NS}/models/block/{name}.json")
                on = json.loads(json.dumps(base))
                on["textures"]["lit"] = f"{NS}:block/furnator_lit"
                out.model(f"{NS}:block/{name}")
                out.own_model(f"block/{name}_on", on)
            else:
                out.model(f"{NS}:block/{name}")
                out.model(f"{NS}:block/{name}_on")
            states = furnace_states(f"{NS}:block/{name}", f"{NS}:block/{name}_on")
        elif kind == "cell":
            decl["properties"]["occludes"] = False
            out.model(f"{NS}:block/{name}")
            states = furnace_states(f"{NS}:block/{name}", f"{NS}:block/{name}", rotate=False)
        elif kind == "orb":
            decl["properties"]["occludes"] = False
            # Contorno do orbe (base + vidro), como o EnergizingOrbBlock com facing=down.
            decl["shapes"] = {"default": [[0.15625, 0.0, 0.15625, 0.84375, 0.0625, 0.84375],
                                          [0.21875, 0.0625, 0.21875, 0.78125, 0.89, 0.78125]]}
            out.model(f"{NS}:block/{name}")
            states = furnace_states(f"{NS}:block/{name}", f"{NS}:block/{name}", rotate=False)
        elif kind == "rod":
            # Template da end rod: facing nas 6 direções, apontando para fora do bloco onde foi posta.
            # O facing do Powah é o lado da base (o oposto): end rod facing=up = Powah facing=down.
            decl = {"template": "minecraft:end_rod", "properties": {"hardness": 2.0, "light": 0}}
            out.model(f"{NS}:block/{name}")
            src = jar.json(f"assets/{NS}/blockstates/{name}.json")["variants"]
            states = {"variants": {f"facing={f}": src[f"facing={OPPOSITE[f]}"] for f in OPPOSITE}}
        elif kind == "cable":
            decl = {"template": "minecraft:chorus_plant", "properties": {"hardness": 2.0},
                    "connects_to": f"#{NS}:cable_connects", "thickness": 4}
            out.model(f"{NS}:block/{name}")
            out.model(f"{NS}:block/{name}_multipart")
            states = jar.json(f"assets/{NS}/blockstates/{name}.json")
            for part in states["multipart"]:
                if "when" in part:  # o jar escreve true (JSON); propriedades de estado são texto
                    part["when"] = {k: str(v).lower() for k, v in part["when"].items()}
        else:  # storage
            decl = {"template": "minecraft:iron_block", "properties": {"hardness": 2.0}}
            out.model(f"{NS}:block/{name}")
            states = jar.json(f"assets/{NS}/blockstates/{name}.json")
        dump(out.data / "horizon/block" / f"{name}.json", decl)
        dump(out.assets / "blockstates" / f"{name}.json", states)
        dump(out.data / "loot_table/blocks" / f"{name}.json", loot(name))
        item_model = jar.json(f"assets/{NS}/models/item/{name}.json")
        if set(item_model) == {"parent"}:
            out.item_def(name, item_model["parent"])
        else:
            out.own_model(f"item/{name}", item_model)
            out.item_def(name, f"{NS}:item/{name}")

    for name in ITEMS:
        dump(out.data / "horizon/item" / f"{name}.json", {"template": "minecraft:iron_ingot"})
        m = jar.json(f"assets/{NS}/models/item/{name}.json")
        m.pop("credit", None)
        out.own_model(f"item/{name}", m)
        out.item_def(name, f"{NS}:item/{name}")

    # O menu do orbe: 7 slots (0 = saída, 1..6 = entradas) e 5 números para a tela.
    dump(out.data / "horizon/menu/energizing_orb.json", {"slots": 7, "data_slots": 5, "results": [0]})
    dump(out.data / "horizon/creative_tab/powah.json",
         {"title": "itemGroup.powah.tab", "icon": f"{NS}:energizing_orb", "items": [f"{NS}:{n}" for n in TAB]})

    # Tags: onde os cabos ligam, o que a picareta quebra, os grupos do original.
    joins = [f"{NS}:{n}" for n, k in BLOCKS.items() if k not in ("storage",)]
    dump(out.data / "tags/block/cable_connects.json", {"values": joins})
    dump(out.root / "data/minecraft/tags/block/mineable/pickaxe.json",
         {"values": [f"{NS}:{n}" for n in BLOCKS]})
    for group, prefix in (("energy_cells", "energy_cell_"), ("energy_cables", "energy_cable_"),
                          ("energizing_rods", "energizing_rod_"), ("furnators", "furnator_"),
                          ("magmators", "magmator_")):
        dump(out.data / f"tags/block/{group}.json", {"values": [f"{NS}:{prefix}{t}" for t in TIERS]})


def ids():
    return {f"{NS}:{n}" for n in list(BLOCKS) + ITEMS}


def known_item(i):
    return i.startswith("minecraft:") or i in ids()


def ingredient(v):
    """Um ingrediente do 1.21.1 como o 26.3 escreve: "ns:item" ou "#ns:tag"; None se não existe aqui."""
    if isinstance(v, list):
        out = [ingredient(x) for x in v]
        return out if all(x is not None for x in out) else None
    if set(v) == {"item"}:
        return v["item"] if known_item(v["item"]) else None
    if set(v) == {"tag"}:
        t = v["tag"]
        if t in C_TAGS:
            return C_TAGS[t]
        return f"#{t}" if t in VANILLA_TAGS else None
    return None


def result(v):
    if not known_item(v["id"]):
        return None
    out = {"id": v["id"]}
    if v.get("count", 1) != 1:
        out["count"] = v["count"]
    return out


def convert(r):
    t = r["type"]
    res = result(r["result"])
    if res is None:
        return None, f"resultado {r['result']['id']}"
    if t == "minecraft:crafting_shaped":
        key = {}
        for k, v in r["key"].items():
            key[k] = ingredient(v)
            if key[k] is None:
                return None, f"ingrediente {v}"
        out = {"type": t, "category": r.get("category", "misc"), "pattern": r["pattern"], "key": key}
    elif t == "minecraft:crafting_shapeless":
        ings = [ingredient(v) for v in r["ingredients"]]
        if any(i is None for i in ings):
            return None, f"ingredientes {r['ingredients']}"
        out = {"type": t, "category": r.get("category", "misc"), "ingredients": ings}
    elif t == "powah:energizing":
        ings = [ingredient(v) for v in r["ingredients"]]
        if any(i is None for i in ings):
            return None, f"ingredientes {r['ingredients']}"
        # Lido pelo mod nativo (native_recipes_of_type): energy em FE, até 6 ingredientes.
        out = {"type": t, "energy": int(r["energy"]), "ingredients": ings}
    else:
        return None, f"tipo {t}"
    out["result"] = res
    return out, None


def recipes(out: Out):
    jar = out.jar
    kept = 0
    for name in sorted(n for n in jar.names if n.startswith(f"data/{NS}/recipe/crafting/") and n.endswith(".json")):
        stem = Path(name).stem
        r, why = convert(jar.json(name))
        if r is None:
            LOG.append(f"receita crafting/{stem} fora: {why}")
            continue
        dump(out.data / "recipe" / "crafting" / f"{stem}.json", r)
        kept += 1
    for stem in ENERGIZING:
        r, why = convert(jar.json(f"data/{NS}/recipe/energizing/{stem}.json"))
        if r is None:
            raise SystemExit(f"energizing/{stem}: {why}")
        dump(out.data / "recipe" / "energizing" / f"{stem}.json", r)
        kept += 1
    for name in sorted(n for n in jar.names if n.startswith(f"data/{NS}/recipe/energizing/") and n.endswith(".json")):
        if Path(name).stem not in ENERGIZING:
            LOG.append(f"receita energizing/{Path(name).stem} fora: pede ou dá item não portado")
    return kept


def lang(out: Out):
    for code in ("en_us", "pt_br"):
        src = out.jar.json(f"assets/{NS}/lang/{code}.json")
        en = out.jar.json(f"assets/{NS}/lang/en_us.json")
        have = {}
        for n in BLOCKS:
            k = f"block.{NS}.{n}"
            have[k] = PT_OVERRIDES.get(k) if code == "pt_br" and k in PT_OVERRIDES else src.get(k, en[k])
        for n in ITEMS:
            k = f"item.{NS}.{n}"
            have[k] = src.get(k, en[k])
        have["itemGroup.powah.tab"] = "Powah"
        have["container.powah.energizing_orb"] = src.get(f"block.{NS}.energizing_orb", en[f"block.{NS}.energizing_orb"])
        dump(out.assets / "lang" / f"{code}.json", have)


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    jar, content = Jar(sys.argv[1]), Path(sys.argv[2])
    root = content / NS
    if root.exists():
        shutil.rmtree(root)
    out = Out(jar, root)
    blocks(out)
    n = recipes(out)
    lang(out)
    (root / "LICENSE-original.txt").write_bytes(jar.zip.read("LICENSE_Powah"))
    (root / "horizon-mod.toml").write_text(
        f'id = "{NS}"\n'
        'name = "Powah"\n'
        f'version = "{VERSION}"\n'
        'description = "Powah para Horizon: Orbe Energizante e hastes (aco energizado, cristais), Furnator, Magmator, '
        'celulas e cabos de energia (inicial e basico). Port do Powah (owmii, LGPLv3). Precisa do mod nativo powah_native."\n'
        'authors = ["owmii, Technici4n, shartte (original, LGPLv3)", "port Horizon"]\n'
        'api = "0.2"\n'
        'side = "both"\n'
        'capabilities = ["registries"]\n')
    report = [f"blocos: {len(BLOCKS)}", f"itens: {len(ITEMS)}", f"receitas: {n}"] + LOG
    (content / "content-report.txt").write_text("\n".join(report) + "\n", encoding="utf-8")
    print(json.dumps({"blocks": len(BLOCKS), "items": len(ITEMS), "recipes": n, "left_out": len(LOG)}))


if __name__ == "__main__":
    main()

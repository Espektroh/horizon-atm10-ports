#!/usr/bin/env python3
"""Package Bibliocraft: bibliocraft.hmod (from source/content/bibliocraft, made by
make_content.py and upgrade_hmod.py), bibliocraft_native-0.1.0.hnmod (from
source/native/mixins), SHA256SUMS.txt. Deterministic zips (fixed dates, sorted names).

usage: package.py <package folder>   (the folder holding source/)
"""
import hashlib
import json
import sys
import zipfile
from pathlib import Path

HORIZON = "66f3981f522b24c250b21523a70080965b940424"
MIXINS = "5a64af938ec86b00d53d16a408aabc0038fb3e57"
NATIVE_ID, NATIVE_VERSION = "bibliocraft_native", "0.1.0"
CONTENT = "bibliocraft"
DATE = (2026, 1, 1, 0, 0, 0)


def add(z, name, data):
    info = zipfile.ZipInfo(name, DATE)
    info.compress_type = zipfile.ZIP_DEFLATED
    z.writestr(info, data)


def hmod(src: Path, dest: Path):
    with zipfile.ZipFile(dest, "w", zipfile.ZIP_DEFLATED) as z:
        for f in sorted(p for p in src.rglob("*") if p.is_file()):
            add(z, f.relative_to(src).as_posix(), f.read_bytes())


def hnmod(src: Path, dest: Path):
    meta = {"game": "26.3", "horizon": HORIZON, "id": NATIVE_ID, "mixins": MIXINS, "schema": 1,
            "version": NATIVE_VERSION}
    with zipfile.ZipFile(dest, "w", zipfile.ZIP_DEFLATED) as z:
        add(z, "horizon-native.json", json.dumps(meta, indent=2) + "\n")
        for f in sorted(src.iterdir()):
            if f.is_file() and (f.suffix == ".rs" or f.name == "horizon-mixin.toml"):
                add(z, f"mixins/{NATIVE_ID}/{f.name}", f.read_bytes())


def main(pkg: Path):
    names = [f"{CONTENT}.hmod", f"{NATIVE_ID}-{NATIVE_VERSION}.hnmod"]
    hmod(pkg / "source" / "content" / CONTENT, pkg / names[0])
    hnmod(pkg / "source" / "native" / "mixins" / NATIVE_ID, pkg / names[1])
    lines = [f"{hashlib.sha256((pkg / n).read_bytes()).hexdigest()}  {n}" for n in names]
    (pkg / "SHA256SUMS.txt").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    main(Path(sys.argv[1]))

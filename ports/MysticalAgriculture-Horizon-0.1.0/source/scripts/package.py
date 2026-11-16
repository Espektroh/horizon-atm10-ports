#!/usr/bin/env python3
"""Packs the two content mods (.hmod) and the native mod (.hnmod) into the package folder, with
fixed timestamps and sorted entries (the same bytes each time), and writes SHA256SUMS.txt.
usage: package.py SOURCE_DIR PACKAGE_DIR"""
import hashlib, json, sys, zipfile
from pathlib import Path

src, pkg = Path(sys.argv[1]), Path(sys.argv[2])
HORIZON = "66f3981f522b24c250b21523a70080965b940424"
MIXINS = "5a64af938ec86b00d53d16a408aabc0038fb3e57"
NATIVE_ID, NATIVE_VERSION = "mysticalagriculture_native", "0.1.0"


def pack(out, entries, date):
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        for name, data in sorted(entries):
            info = zipfile.ZipInfo(name, date_time=date)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            z.writestr(info, data)


def content(mod):
    root = src / "content" / mod
    files = [p for p in root.rglob("*") if p.is_file()]
    entries = [(p.relative_to(root).as_posix(), p.read_bytes()) for p in files
               if p.relative_to(root).parts[0] in ("assets", "data") or p.name in ("horizon-mod.toml", "LICENSE-original.txt")]
    pack(pkg / f"{mod}.hmod", entries, (1980, 1, 1, 0, 0, 0))


content("mysticalagriculture")
content("mysticalagradditions")
manifest = {"game": "26.3", "horizon": HORIZON, "id": NATIVE_ID, "mixins": MIXINS, "schema": 1,
            "version": NATIVE_VERSION}
entries = [("horizon-native.json", json.dumps(manifest, indent=2) + "\n")]
mix = src / "native/mixins" / NATIVE_ID
for p in sorted(mix.iterdir()):
    if p.is_file():
        entries.append((f"mixins/{NATIVE_ID}/{p.name}", p.read_bytes()))
pack(pkg / f"{NATIVE_ID}-{NATIVE_VERSION}.hnmod", entries, (2026, 1, 1, 0, 0, 0))

names = ["mysticalagriculture.hmod", "mysticalagradditions.hmod", f"{NATIVE_ID}-{NATIVE_VERSION}.hnmod"]
lines = [f"{hashlib.sha256((pkg / n).read_bytes()).hexdigest()}  {n}" for n in names]
(pkg / "SHA256SUMS.txt").write_text("\n".join(lines) + "\n")
print("\n".join(lines))

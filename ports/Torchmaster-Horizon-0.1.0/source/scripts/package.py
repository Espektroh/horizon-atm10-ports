#!/usr/bin/env python3
"""Package Torchmaster: the .hnmod native bundle, the .hmod content mod
(made by make_content.py into source/content), SHA256SUMS.txt.

Usage: package.py <package folder>   (the folder holding source/)
"""
import hashlib
import json
import shutil
import sys
import zipfile
from pathlib import Path

HORIZON = "66f3981f522b24c250b21523a70080965b940424"
MIXINS = "5a64af938ec86b00d53d16a408aabc0038fb3e57"
NATIVE = [("torchmaster_native", "0.1.0")]
CONTENT = ["torchmaster.hmod"]


def hnmod(src: Path, mod_id: str, version: str, dest: Path):
    meta = {"game": "26.3", "horizon": HORIZON, "id": mod_id, "mixins": MIXINS, "schema": 1,
            "version": version}
    with zipfile.ZipFile(dest, "w", zipfile.ZIP_DEFLATED) as z:
        info = zipfile.ZipInfo("horizon-native.json", (2026, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        z.writestr(info, json.dumps(meta, indent=2) + "\n")
        folder = src / "mixins" / mod_id
        for f in sorted(folder.iterdir()):
            if f.is_file() and (f.suffix == ".rs" or f.name == "horizon-mixin.toml"):
                info = zipfile.ZipInfo(f"mixins/{mod_id}/{f.name}", (2026, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                z.writestr(info, f.read_bytes())


def main(pkg: Path):
    names = []
    for mod_id, version in NATIVE:
        name = f"{mod_id}-{version}.hnmod"
        hnmod(pkg / "source" / "native" / mod_id, mod_id, version, pkg / name)
        names.append(name)
    for name in CONTENT:
        shutil.copyfile(pkg / "source" / "content" / name, pkg / name)
        names.append(name)
    lines = []
    for name in CONTENT + [n for n in names if n.endswith(".hnmod")]:
        digest = hashlib.sha256((pkg / name).read_bytes()).hexdigest()
        lines.append(f"{digest}  {name}")
    (pkg / "SHA256SUMS.txt").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    main(Path(sys.argv[1]))

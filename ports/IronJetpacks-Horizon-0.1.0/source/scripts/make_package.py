#!/usr/bin/env python3
"""Builds the two installable files of the package from source/.

usage: make_package.py CONTENT OUT
  CONTENT: the tree make_content.py wrote (horizon-mod.toml, assets/, data/)
  OUT: the package folder (gets ironjetpacks.hmod, ironjetpacks_native-<version>.hnmod
       and SHA256SUMS.txt)
The zips are deterministic (fixed dates, sorted names), so the hashes repeat.
"""
import hashlib
import json
import sys
import zipfile
from pathlib import Path

VERSION = "0.1.0"
ID = "ironjetpacks_native"
HERE = Path(__file__).resolve().parent.parent
MIXINS = HERE / "native" / "mixins" / ID
SHIPPED = ["glue.rs", "horizon-mixin.toml", "hud.rs", "item_bar.rs", "key.rs", "list.rs", "movement_input.rs",
           "net_from_client.rs", "player_tick.rs", "right_click.rs", "runtime.rs", "server_tick.rs", "tick.rs",
           "tooltip.rs"]


def zip_files(path, entries, date):
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for name, data in sorted(entries):
            info = zipfile.ZipInfo(name, date_time=date)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            z.writestr(info, data)


def main():
    content, out = Path(sys.argv[1]), Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    present = sorted(p.name for p in MIXINS.glob("*") if p.is_file())
    assert present == sorted(SHIPPED), f"mixin files differ from SHIPPED: {present}"
    hmod = out / "ironjetpacks.hmod"
    zip_files(hmod, [(p.relative_to(content).as_posix(), p.read_bytes())
                     for p in content.rglob("*") if p.is_file()], (1980, 1, 1, 0, 0, 0))
    manifest = {
        "game": "26.3",
        "horizon": "66f3981f522b24c250b21523a70080965b940424",
        "id": ID,
        "mixins": "5a64af938ec86b00d53d16a408aabc0038fb3e57",
        "schema": 1,
        "version": VERSION,
    }
    entries = [("horizon-native.json", (json.dumps(manifest, indent=2) + "\n").encode())]
    entries += [(f"mixins/{ID}/{n}", (MIXINS / n).read_bytes()) for n in SHIPPED]
    hnmod = out / f"{ID}-{VERSION}.hnmod"
    zip_files(hnmod, entries, (2026, 1, 1, 0, 0, 0))
    lines = [f"{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}" for p in (hmod, hnmod)]
    (out / "SHA256SUMS.txt").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    main()

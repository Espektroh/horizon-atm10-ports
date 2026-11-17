#!/usr/bin/env bash
# Gera tudo de novo (de dentro de source/): conteúdo, .hmod, .hnmod e SHA256SUMS.
#   scripts/build.sh /caminho/Mekanism-MoreMachine (clone 1.21.1) /caminho/Mekanism-Horizon-0.6.1/source/content
set -euo pipefail
cd "$(dirname "$0")/.."
SRC="$1"; MEK="$2"; OUT=build
python3 -I scripts/make_content.py "$SRC" "$MEK" "$OUT/content"
python3 -I scripts/upgrade_hmod.py --dir "$OUT/content"
rm -rf content && mv "$OUT/content" content && rm -rf "$OUT"
python3 -I - <<'PY'
import sys; sys.path.insert(0, "scripts")
from pathlib import Path
from make_content import pack
pack(Path("content"), Path("../mekmm.hmod"))
PY
(cd native && cargo test --offline -q && python3 -I tools/package.py >/dev/null && cp target/mekmm_native-0.1.0.hnmod ../../)
cd .. && sha256sum mekmm.hmod mekmm_native-0.1.0.hnmod > SHA256SUMS.txt && cat SHA256SUMS.txt

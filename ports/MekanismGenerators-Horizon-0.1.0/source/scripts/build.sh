#!/usr/bin/env bash
# Gera tudo de novo (de dentro de source/): conteúdo, mekanism.hmod corrigido, .hnmod e SHA256SUMS.
#   scripts/build.sh /caminho/MekanismGenerators-1.21.1-10.7.19.85.jar /caminho/mekanism.hmod(0.6.1)
set -euo pipefail
cd "$(dirname "$0")/.."
JAR="$1"; BASE="$2"; OUT=build
python3 -I scripts/make_content.py "$JAR" "$BASE" "$OUT"
python3 -I scripts/upgrade_hmod.py --dir "$OUT/content"
rm -rf content mekanism-content && mv "$OUT/content" content && mv "$OUT/mekanism-content" mekanism-content && rm -rf "$OUT"
python3 -I - <<'PY'
import sys; sys.path.insert(0, "scripts")
from pathlib import Path
from make_content import pack
pack(Path("content"), Path("../mekanismgenerators.hmod"))
pack(Path("mekanism-content"), Path("../mekanism.hmod"))
PY
(cd native && cargo test --offline -q && python3 -I tools/package.py >/dev/null && cp target/mekanismgenerators_native-0.1.0.hnmod ../../)
cd .. && sha256sum mekanismgenerators.hmod mekanism.hmod mekanismgenerators_native-0.1.0.hnmod > SHA256SUMS.txt && cat SHA256SUMS.txt

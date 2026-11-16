#!/usr/bin/env bash
# Compila TODOS os mods nativos do repositório juntos, como o Build do launcher faz:
# Horizon 66f3981 + horizon-mixins 5a64af9 -> apply -> cargo check -p horizon-client.
# Uso local: tools/ci_check.sh [pasta-de-trabalho]   (no CI roda sozinho)
set -euo pipefail
HORIZON_REV=66f3981f522b24c250b21523a70080965b940424
MIXINS_REV=5a64af938ec86b00d53d16a408aabc0038fb3e57
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="${1:-$ROOT/../ci-work}"
mkdir -p "$WORK"; cd "$WORK"

if [ ! -d horizon/.git ]; then git clone --filter=blob:none https://github.com/horizon-rust/horizon horizon; fi
git -C horizon fetch -q --filter=blob:none origin "$HORIZON_REV" || true
git -C horizon checkout -q "$HORIZON_REV"
# O horizon-mixins recusa symlinks (ex.: CLAUDE.md -> AGENTS.md). Troca cada um por uma cópia do alvo
# (ou apaga, se o alvo não for um arquivo). Não muda nada que o cargo compila.
while IFS= read -r -d '' l; do
  t=$(readlink -f "$l" || true)
  if [ -n "$t" ] && [ -f "$t" ]; then cp --remove-destination "$t" "$l"; else rm -f "$l"; fi
  echo "symlink resolvido: ${l#horizon/}"
done < <(find horizon -path horizon/.git -prune -o -type l -print0)

if [ ! -d horizon-mixins/.git ]; then git clone https://github.com/horizon-rust/horizon-mixins horizon-mixins; fi
git -C horizon-mixins fetch -q origin "$MIXINS_REV" || true
git -C horizon-mixins checkout -q "$MIXINS_REV"
(cd horizon-mixins && cargo build --locked -q -p horizon-mixins-cli)
MIXINS="$WORK/horizon-mixins/target/debug/horizon-mixins"

# Um bundle por mod nativo: o conteúdo de mixins/<id>/ de cada .hnmod em ports/
rm -rf mods && mkdir mods
shopt -s nullglob
for f in "$ROOT"/ports/*/*.hnmod; do
  tmp=$(mktemp -d); unzip -q "$f" -d "$tmp"
  for d in "$tmp"/mixins/*/; do cp -r "$d" "mods/$(basename "$d")"; done
  rm -rf "$tmp"
done
echo "Mods: $(ls mods | tr '\n' ' ')"

rm -rf staged
"$MIXINS" apply --workspace horizon --mods mods --out staged > apply.json
cd staged
cargo check --locked -p horizon-client --no-default-features 2>&1 | tee ../check.log
grep -q "^error" ../check.log && exit 1 || echo "OK: todos os mods compilam juntos"

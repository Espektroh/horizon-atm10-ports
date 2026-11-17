#!/usr/bin/env bash
# Compila TODOS os mods nativos do repositório juntos, como o Build do launcher faz:
# Horizon 66f3981 + horizon-mixins 5a64af9 -> apply -> cargo check -p horizon-client.
# Uso local: tools/ci_check.sh [pasta-de-trabalho]   (no CI roda sozinho)
set -euo pipefail
HORIZON_REV=${HORIZON_REV:-66f3981f522b24c250b21523a70080965b940424}
MIXINS_REV=5a64af938ec86b00d53d16a408aabc0038fb3e57
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="${1:-$ROOT/../ci-work}"
mkdir -p "$WORK"; cd "$WORK"

# git init + fetch (e não clone): o cache do CI recria horizon-mixins/target antes, e o clone recusaria a pasta.
fetch_rev() { # pasta url rev
  if [ ! -d "$1/.git" ]; then git init -q "$1"; git -C "$1" remote add origin "$2"; fi
  git -C "$1" fetch -q --depth 1 origin "$3"
  git -C "$1" checkout -q -f FETCH_HEAD
}
fetch_rev horizon https://github.com/horizon-rust/horizon "$HORIZON_REV"
# O horizon-mixins recusa symlinks (ex.: CLAUDE.md -> AGENTS.md). Troca cada um por uma cópia do alvo
# (ou apaga, se o alvo não for um arquivo). Não muda nada que o cargo compila.
while IFS= read -r -d '' l; do
  t=$(readlink -f "$l" || true)
  if [ -n "$t" ] && [ -f "$t" ]; then cp --remove-destination "$t" "$l"; else rm -f "$l"; fi
  echo "symlink resolvido: ${l#horizon/}"
done < <(find horizon -path horizon/.git -prune -o -type l -print0)

fetch_rev horizon-mixins https://github.com/horizon-rust/horizon-mixins "$MIXINS_REV"
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

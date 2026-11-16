# Mekanism para Horizon 0.6.0 — fontes

- `content/` → `mekanism.hmod` e `content-tools/` → `mekanismtools.hmod`: gerados por `scripts/make_content.py`
  a partir do `mekanism.hmod` 0.5.0 instalado + jars do ATM10 (Mekanism 10.7.19, MekanismTools 10.7.19).
  `scripts/check_content.py` confere JSON, modelos, texturas, itens das receitas, loot, lang.
- `native/` → `mekanism_native-0.6.0.hnmod`: `mixins/mekanism_native/` (runtime.rs = lógica pura com testes,
  glue.rs = ponte para o Horizon, helpers dos hooks, horizon-mixin.toml).

```sh
# conteúdo
python3 -I scripts/make_content.py mekanism-0.5.0.hmod Mekanism-1.21.1-10.7.19.85.jar MekanismTools-1.21.1-10.7.19.85.jar out
python3 -I scripts/check_content.py out/content out/content-tools
# nativo
cd native && cargo test --offline            # 28 testes da lógica
(cd tools/glue_mockcheck && CARGO_TARGET_DIR=/tmp/mc cargo build --offline)   # checagem de tipos do glue.rs
python3 -I tools/package.py                  # target/mekanism_native-0.6.0.hnmod
```

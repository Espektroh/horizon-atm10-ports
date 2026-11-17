# Fontes do Mekanism: MoreMachine 0.1.0 para Horizon

- `content/`: o conteúdo do `mekmm.hmod` (gerado; não edite à mão).
- `native/`: o mod nativo. `mixins/mekmm_native/runtime.rs` (lógica, só std, com testes), `glue.rs` (Horizon),
  os helpers dos ganchos e `horizon-mixin.toml`. `cargo test --offline` testa o runtime;
  `python3 -I tools/package.py` gera o `.hnmod`.
- `scripts/make_content.py`: gera o conteúdo a partir do clone do código oficial (branch 1.21.1, MIT; só lê
  texturas, modelos, blockstates e lang) e do conteúdo do Mekanism-Horizon-0.6.1 (textura `bottom`).
  `scripts/upgrade_hmod.py`: conversor de loot/worldgen 1.21.1 -> 26.3. `scripts/check_content.py`: confere
  referências. `scripts/build.sh SRC MEKANISM_CONTENT`: faz tudo e regrava `SHA256SUMS.txt`.
- `LICENSE-MoreMachine.txt` (MIT, código e assets do MoreMachine), `LICENSE-Mekanism.txt` (MIT, a textura `bottom`).

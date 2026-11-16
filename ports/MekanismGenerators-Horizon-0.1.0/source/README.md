# Fontes do Mekanism Generators 0.1.0 para Horizon

- `content/`: o conteúdo do `mekanismgenerators.hmod` (gerado).
- `mekanism-content/`: o `mekanism.hmod` 10.7.19-horizon.7 corrigido (gerado a partir do 0.6.1; ver LEIA-ME).
- `native/`: o mod nativo. `mixins/mekanismgenerators_native/runtime.rs` (lógica, só std, com testes),
  `glue.rs` (Horizon), os helpers dos ganchos e `horizon-mixin.toml`. `cargo test --offline` testa o runtime;
  `python3 -I tools/package.py` gera o `.hnmod`.
- `scripts/make_content.py`: gera os dois conteúdos a partir do jar do Mekanism Generators e do mekanism.hmod 0.6.1.
  `scripts/upgrade_hmod.py`: conversor de loot/worldgen 1.21.1 -> 26.3. `scripts/check_content.py`: confere
  referências. `scripts/build.sh JAR MEKANISM_HMOD`: faz tudo e regrava `SHA256SUMS.txt`.

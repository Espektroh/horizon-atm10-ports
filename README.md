# Ports do All the Mods 10 para o Horizon

Ports de mods do ATM10 para o [Horizon](https://github.com/horizon-rust/horizon) (Minecraft 26.3 em Rust),
fixados no Horizon `66f3981` + horizon-mixins `5a64af9`. Só entram aqui mods com licença que permite
redistribuição.

| Pacote | Mod original | Licença | Estado |
|---|---|---|---|
| `ports/Mekanism-Horizon-0.6.1` | Mekanism + Mekanism Tools | MIT | Testado no jogo (0.6.0); 0.6.1 aguarda Build |
| `ports/MysticalAgriculture-Horizon-0.1.0` | Mystical Agriculture | MIT | Testado no jogo |
| `ports/Torchmaster-Horizon-0.1.0` | Torchmaster | MIT | Testado no jogo |
| `ports/Powah-Horizon-0.1.0` | Powah! | MIT | Aguarda Build e teste no jogo |
| `ports/FunctionalStorage-Horizon-0.1.0` | Functional Storage | LGPL-3.0 | Aguarda Build e teste no jogo |

Cada pacote tem o `.hmod` de conteúdo, o `.hnmod` nativo, `instalar.ps1` (Windows, com backup), `LEIA-ME.md`
(o que funciona, as limitações e um roteiro de teste), `APIS.md` (cada API do Horizon usada, com arquivo:linha),
`SHA256SUMS.txt` e `source/`.

## Instalar
Na pasta do pacote: `powershell -ExecutionPolicy Bypass -File .\instalar.ps1`. Depois clique em **Build** na
página Mods do launcher. Todos os mods nativos compilam juntos, então um erro de compilação em um deles quebra o
build inteiro. Se isso acontecer, tire o `.hnmod` do pacote novo de `horizon-native-mods\` e abra uma issue com o
`build.log`.

## Ferramentas
- `tools/upgrade_hmod.py`: converte dados de jar 1.21.1 para o formato do 26.3 (loot tables com
  `condition`/`modifier`, worldgen sem o invólucro `config`, `{Name, Properties}` → `{id, properties}`,
  `random_offset` → `offset`). Uso: `upgrade_hmod.py IN.hmod OUT.hmod` ou `--dir PASTA`.
- `tools/ci_check.sh`: aplica todos os `.hnmod` juntos no Horizon e roda `cargo check -p horizon-client`
  (o mesmo que o Build do launcher). Roda no GitHub Actions a cada push.
- `status-atm10.csv`: os mods do ATM10 e o status de port de cada um.

## Outros ports do ATM10 para o Horizon
Para não duplicar trabalho, confira também:
- AdLf0208: pipeline automático de ports do ATM10.
- Joeypc6: Iron Furnaces, [Elevator](https://github.com/Joeypc6/Elevator-Horizon) e outros.
- Fraskas: Farmer's Delight.

## Licenças
Texturas, modelos e textos vêm dos mods originais e mantêm a licença de cada um (o aviso fica em
`source/content/<mod>/`). O código nativo em Rust foi reescrito a partir das regras dos mods originais e segue a
mesma licença do mod original de cada pacote.

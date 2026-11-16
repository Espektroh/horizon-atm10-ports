# Iron Jetpacks para Horizon 66f3981 (port 0.1.0)

Port do [Iron Jetpacks](https://github.com/BlakeBr0/IronJetpacks) 8.0.11 (BlakeBr0, **MIT**), a versão do ATM10,
para o Horizon (Minecraft 26.3 em Rust). Texturas, nomes (en_us e pt_br) e receitas vêm do repositório público
(branch `1.21`); os **números de cada jetpack são os do ATM10** (`config/ironjetpacks/jetpacks/*.json` do
repositório AllTheMods/ATM-10), que substituem os padrões do mod.

## São dois mods, os dois são necessários

- **`ironjetpacks.hmod`** (conteúdo, id `ironjetpacks`, versão 8.0.11-horizon.1): itens, texturas, nomes, receitas e a
  aba criativa "Iron Jetpacks".
- **`ironjetpacks_native-0.1.0.hnmod`** (nativo, id `ironjetpacks_native`): voo, energia, teclas, HUD, recarga.

Sem o nativo os jetpacks são só uma peça de peito com armadura.

## Diferença do original: um item por tier

O original tem **um** item `ironjetpacks:jetpack` (e um `cell`, `thruster`, `capacitor`) diferenciado por um
componente (`ironjetpacks:jetpack_id`) e pintado em tempo real. Aqui cada tier tem seus próprios itens, com a cor já
aplicada na textura: `ironjetpacks:<tier>_jetpack`, `<tier>_cell`, `<tier>_thruster`, `<tier>_capacitor`
(ex.: `ironjetpacks:iron_jetpack`). Mais `ironjetpacks:strap` e `basic_coil`, `advanced_coil`, `elite_coil`,
`ultimate_coil`.

## Jetpacks (números do ATM10)

| Tier | Id | Nível | Armadura | Energia (FE) | Uso FE/t (correndo) | Vel. vertical | Acel. | Lateral | Pairar (subir / descer / parado) |
|---|---|---|---|---|---|---|---|---|---|
| Madeira | `wood_jetpack` | 0 | 1 | 20.000 | 32 (32) | 0,15 | 0,10 | 0,05 | 0,15 / 0,25 / 0,10 |
| Pedra | `stone_jetpack` | 1 | 2 | 50.000 | 64 (64) | 0,15 | 0,11 | 0,06 | 0,20 / 0,25 / 0,10 |
| Cobre | `copper_jetpack` | 1 | 2 | 75.000 | 80 (96) | 0,16 | 0,11 | 0,07 | 0,21 / 0,25 / 0,10 |
| Ferro | `iron_jetpack` | 2 | 3 | 120.000 | 120 (240) | 0,20 | 0,12 | 0,10 | 0,25 / 0,25 / 0,08 |
| Ouro | `gold_jetpack` | 3 | 2 | 400.000 | 300 (900) | 0,30 | 0,13 | 0,14 | 0,30 / 0,25 / 0,03 |
| Aço (Mekanism) | `steel_jetpack` | 3 | 3 | 500.000 | 310 (992) | 0,31 | 0,13 | 0,14 | 0,32 / 0,25 / 0,03 |
| Diamante | `diamond_jetpack` | 4 | 4 | 5.000.000 | 600 (2.100) | 0,35 | 0,15 | 0,16 | 0,35 / 0,25 / 0,02 |
| Esmeralda | `emerald_jetpack` | 5 | 4 | 12.500.000 | 900 (3.600) | 0,40 | 0,16 | 0,17 | 0,40 / 0,25 / 0,01 |
| Allthemodium | `allthemodium_jetpack` | 6 | 9 | 60.000.000 | 1.600 (7.200) | 0,45 | 0,17 | 0,18 | 0,45 / 0,35 / 0,0025 |
| Vibranium | `vibranium_jetpack` | 7 | 12 | 120.000.000 | 2.700 (13.500) | 0,50 | 0,18 | 0,19 | 0,50 / 0,40 / 0 |
| Unobtainium | `unobtainium_jetpack` | 8 | 15 | 180.000.000 | 4.000 (22.000) | 0,55 | 0,20 | 0,21 | 0,55 / 0,45 / 0 |
| Criativo | `creative_jetpack` | C | 8 | infinita | 0 | 0,6 | 0,2 | 0,25 | 0,7 / 0,6 / 0 |

Velocidades em blocos por tick (0,2 = 4 blocos/s). Correndo: lateral × multiplicador de corrida (1 a 1,7),
subida × multiplicador vertical (1 a 1,6) e consumo × multiplicador de combustível (1 a 5,5). Na água a velocidade
vertical cai para 40%. Agachado (sem pairar) a velocidade lateral cai pela metade.

Ficaram de fora os tiers do ATM10 de materiais ainda não portados (All the Ores): bronze, prata, electrum, invar e
platina. O ATM10 **não tem jetpack de netherita** (nem o mod). O jetpack não tem durabilidade (inquebrável no original)
e não é encantável (`enchantableJetpacks = false`, padrão).

## Como usar

- **Vista no peito** (clique direito com ele na mão, ou arraste para o slot de peitoral).
- **V** liga/desliga o motor (vem **desligado**, como no original). **H** liga/desliga o modo pairar.
  **.** e **,** aumentam/diminuem a potência (20% a 100%, de 20 em 20). Troque em Options > Mods > Iron Jetpacks.
  A barra de ação confirma: "Motor: LIGADO", "Modo pairar: LIGADO", "Potência definida para 80%".
- **Segure pular** para subir (acelera até a velocidade vertical do tier). W/A/S/D empurram na direção que você olha.
- **Pairar**: no ar, sem tecla nenhuma, você desce devagar (velocidade "parado"); agachar desce mais rápido; pular sobe.
  Pairar gasta energia enquanto você estiver no ar.
- **Sem dano de queda** enquanto o jetpack estiver voando (o servidor zera a distância de queda a cada tick, como o
  original). Desligar o motor no alto = queda normal.
- **Energia** fica no próprio item (`minecraft:custom_data`, inteiro `horizon_energy`, o mesmo formato das baterias de
  item do Horizon). Gasta só no servidor. Sem energia o jetpack para (no modo criativo do jogo voa sem gastar).
- **Recarregar**: com o jetpack na **mão principal**, **clique direito** num **Cubo de Energia do Mekanism** (qualquer
  tier) ou numa **Célula de Energia do Powah**: ele puxa de uma vez o que couber. Cubo do Mekanism: 2 FE para cada 5 J
  (a conversão padrão do Mekanism); célula do Powah: FE por FE. Com o jetpack na mão o clique não abre a tela do bloco.
  (No original o jetpack carrega no slot de carga do cubo; o port do Mekanism não carrega itens, por isso o clique.)
- **HUD**: com o jetpack vestido aparece uma caixa no canto superior esquerdo (ou inferior, se o F3 ocupar o de cima):
  "Jetpack de Ferro / Energia: 85% (102k FE) / Motor LIG | Pairar DESL | 100%".
- **Tooltip**: energia "102.000 / 120.000 FE", "NÍVEL 2 | MOTOR | PAIRAR" (verde = ligado), potência, consumo e
  velocidades. Barra de energia embaixo do ícone (verde → vermelho).

## Receitas (as dinâmicas do mod, escritas como arquivos)

`M` = material do tier, `C` = bobina do tier, `X` = célula do tier, `P` = capacitor, `T` = propulsor.

| Item | Receita |
|---|---|
| Tira de Couro (`strap`) | `_I_ / LLL / _I_` (pepita de ferro, couro) |
| Bobinas | `_DR / DSD / RD_` (D = ferro / ouro / diamante / esmeralda para básica / avançada / elite / suprema; S graveto; R redstone) |
| Célula de Energia | `_R_ / MCM / _R_` |
| Propulsor | `MCM / CXC / MFM` (F = fornalha) |
| Capacitor | `MXM / MXM / MXM` |
| Jetpack de Madeira | `MPM / MSM / T_T` (S = tira de couro) |
| Os outros jetpacks | `MPM / MJM / T_T` (J = um jetpack do nível anterior: pedra/cobre ← madeira; ferro ← pedra ou cobre; ouro/aço ← ferro; diamante ← ouro ou aço; esmeralda ← diamante; allthemodium ← esmeralda; vibranium ← allthemodium; unobtainium ← vibranium) |

Bobina por tier (regra `getCoilForTier` com os 9 níveis do ATM10): básica para madeira, pedra, cobre e ferro; avançada
para ouro, aço e diamante; elite para esmeralda e allthemodium; suprema para vibranium e unobtainium.

Materiais: madeira `#minecraft:planks`; pedra = pedra, andesito, diorito, granito, tufo ou ardósia (no lugar de
`#c:stones`); cobre/ferro/ouro/diamante/esmeralda vanilla; aço `mekanism:ingot_steel`; allthemodium/vibranium/
unobtainium `allthemodium:*_ingot`. As receitas do aço só existem com o Mekanism ligado, as do allthemodium com o
All the Modium ligado (`neoforge:conditions`). O jetpack criativo não tem receita (no ATM10 ele vem de itens do Powah
avançado e da ATM Star): só por `/give` ou pela aba criativa.

Atenção: na melhoria de tier pela bancada a **energia e os ajustes do jetpack antigo se perdem** (o original copia os
componentes; o Horizon não copia componentes de ingredientes para o resultado). Gaste ou não carregue antes.

## Roteiro de teste (para executar no jogo)

Mundo de teste, single player. Posição de exemplo do cubo: `X Y Z` = um bloco ao seu lado.

1. **Carregou**: Options > Mods mostra "Iron Jetpacks (Horizon) 0.1.0" com 4 teclas (V, H, ., ,). A aba criativa
   "Iron Jetpacks" tem 50 itens. No log não aparece "content not loaded" para `ironjetpacks`.
2. **Teclas e HUD**: `/gamemode survival`, `/give @s ironjetpacks:iron_jetpack[minecraft:custom_data={horizon_energy:120000}]`,
   vista. HUD: "Jetpack de Ferro", "Energia: 100% (120k FE)", "Motor DESL | Pairar DESL | 100%". Aperte **V**: barra de
   ação "Motor: LIGADO", HUD "Motor LIG". Armadura do HUD vanilla: +3 (1,5 peitoral).
3. **Subida e consumo**: no chão, segure **espaço por 5 s** (100 ticks). Resultado: subiu ~19-20 blocos
   (0,2 bloco/tick; `/data get entity @s Pos[1]` antes e depois) e o tooltip mostra **~108.000 / 120.000 FE**
   (120 FE/tick; ±1 tick de diferença). Correndo (Ctrl + W) o consumo dobra: 240 FE/tick.
4. **Pairar**: no ar, aperte **H** ("Modo pairar: LIGADO") e solte tudo: desce a ~1,6 bloco/s (0,08/tick) e a energia
   continua caindo 120/tick. Agachado: ~5 blocos/s (0,25/tick). No chão, pairando, não gasta.
5. **Sem dano de queda**: suba 30 blocos, desça pairando com agachar até o chão: vida continua 20. Controle: suba 20
   blocos, aperte **V** (motor desligado) e caia: leva dano (~17 de vida perdida em 20 blocos, sem armadura de pés).
6. **Fim da energia**: `/give @s ironjetpacks:wood_jetpack[minecraft:custom_data={horizon_energy:3200,ironjetpacks_engine:1b}]`,
   vista, segure espaço: voa **~99 ticks** (32 FE/tick) e para; tooltip "0 / 20.000 FE", barra vazia vermelha.
7. **Recarga no cubo**: `/give @s mekanism:basic_energy_cube`, coloque em `X Y Z`,
   `/data merge block X Y Z {energy:4000000L}`. Com um jetpack de ferro **vazio** na mão principal, clique direito no
   cubo: barra de ação "Jetpack de Ferro: +120.000 FE (100%)", a tela do cubo **não** abre, e
   `/data get block X Y Z energy` = **3700000L** (300.000 J = 120.000 FE). Clicar de novo (cheio): nada muda.
   Mão vazia: a tela do cubo abre normalmente.
8. **Potência**: com o jetpack vestido aperte **,** duas vezes: "Potência definida para 60%"; segurando espaço sobe a
   0,12 bloco/tick (60% de 0,2) e o HUD mostra "| 60%".
9. **Receitas** (bancada): `/give @s minecraft:iron_nugget 2`, `minecraft:leather 3` → Tira de Couro.
   `minecraft:iron_ingot 4`, `minecraft:redstone 3`, `minecraft:stick` → Bobina Básica.
   Com `#minecraft:planks`: Célula de Madeira (`_R_/MCM/_R_`), Propulsor de Madeira (precisa de fornalha), Capacitor
   de Madeira (3 células) e Jetpack de Madeira (`MPM/MSM/T_T`). Jetpack de Ferro: 6 lingotes de ferro + capacitor de
   ferro + jetpack de pedra (ou de cobre) + 2 propulsores de ferro.
10. **Criativo**: `/give @s ironjetpacks:creative_jetpack`, em survival, V: voa sem energia, sem barra embaixo do ícone,
    tooltip "Infinito FE", sobe a 0,6 bloco/tick.

## Não tem (ainda) e por quê

- **Modelo 3D nas costas**: o jetpack vestido não aparece no corpo (o original desenha um modelo próprio; o Horizon só
  desenha armaduras vanilla). Sem partículas nem som de jato.
- **Curios** (slot de costas): não portado; só no peito.
- **Carregar no slot de carga** dos cubos/geradores: o port do Mekanism não carrega itens; use o clique direito.
- **Tecla do HUD** e teclas próprias de subir/descer (no original vêm sem tecla): aqui subir = pular, descer = agachar,
  HUD sempre visível com o jetpack vestido.
- Tiers de bronze, prata, electrum, invar e platina (All the Ores ainda sem lingotes no Horizon).
- Num servidor **vanilla** (sem Horizon) o mod não faz nada no servidor; em LAN/dedicado Horizon funciona, mas o
  anticheat de movimento pode registrar "moved wrongly" no log do servidor (não puxa de volta por padrão).

## Instalar

`powershell -ExecutionPolicy Bypass -File .\instalar.ps1` (confere o SHA-256, faz backup de `horizon-mods.txt`, copia
para `horizon-mods\` e `horizon-native-mods\` e liga os dois). Depois **Build** no launcher. Conferir:
`.\instalar.ps1 -Verificar`.

## Para quem mexer no código

- `source/native/mixins/ironjetpacks_native/`: `runtime.rs` = regras puras (tiers, voo, consumo, pairar, recarga, textos);
  `glue.rs` = Horizon; os outros `.rs` são os helpers de cada hook.
- `source/native`: `cargo test --offline` (11 testes da lógica pura: consumo, aceleração/velocidade por tier, pairar,
  movimento lateral, barra, recarga).
- `source/native/glue_check`: `python3 gen.py && cargo test --offline -- --test-threads=1` (18 testes: os 11 + 7 que
  compilam e exercitam o glue sobre stubs com as assinaturas do Horizon 66f3981).
- `source/scripts/make_content.py SRC OUT` gera o conteúdo (SRC = `src/main/resources` do repositório do mod);
  `make_package.py CONTENT OUT` gera os zips e o SHA256SUMS. Depois de gerar, rode
  `python3 -I upgrade_hmod.py --dir <conteúdo>` (aqui não há loot nem worldgen; não muda nada).
- `APIS.md`: cada API do Horizon usada, com arquivo:linha, e os riscos.

## Licença

Iron Jetpacks é MIT (Copyright (c) 2018 BlakeBr0); texturas, nomes e receitas vêm do repositório do mod. Os números são
da configuração do ATM10. O código deste port (`source/native`, `source/scripts`) foi escrito do zero para o Horizon e
segue a mesma licença MIT.

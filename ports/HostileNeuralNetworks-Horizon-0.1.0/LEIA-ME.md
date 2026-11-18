# Hostile Neural Networks para Horizon — 0.1.0

Port do **Hostile Neural Networks 6.5.1** (Shadows_of_Fire, NeoForge 1.21.1, o do ATM10) para o Horizon (Minecraft 26.3
em Rust), fixado no Horizon `66f3981` + horizon-mixins `5a64af9`.

- `hostilenetworks.hmod` — conteúdo (blocos, itens, texturas, receitas, lang en_us/pt_br, menus).
- `hostilenetworks_native-0.1.0.hnmod` — mod nativo (treino por abate, Câmara de Simulação, Fabricador de Loot).
- Instalar: `powershell -ExecutionPolicy Bypass -File instalar.ps1` (confere SHA-256, faz backup do
  `horizon-mods.txt`, copia e liga os dois). `-Verificar` só confere.

## O que tem
| Item / bloco | id | Uso |
|---|---|---|
| Deep Learner | `hostilenetworks:deep_learner` | Leva até 4 Data Models; abates treinam os modelos dentro dele |
| Estrutura de Modelo (Model Framework) | `hostilenetworks:blank_data_model` | Clique direito num mob → Data Model daquele mob |
| Data Model (por mob) | `hostilenetworks:data_model_<mob>` | Dados em `minecraft:custom_data` (`data`, `iterations`) |
| Matriz de Predição | `hostilenetworks:prediction_matrix` | Consumida a cada simulação |
| Predição (por mob) | `hostilenetworks:prediction_<mob>` | Vai para o Fabricador de Loot |
| Predições generalizadas | `overworld_prediction`, `nether_prediction`, `end_prediction` | Saída fixa da câmara; receitas "living matter" do original |
| Argila Polimérica | `hostilenetworks:polymer_clay` | Só item/compat (não existe mais na 6.5.1; receita do port: 2 clay_ball + iron_nugget + gold_nugget → 2) |
| Câmara de Simulação | `hostilenetworks:sim_chamber` | Data Model + Matrix + FE → predição generalizada + Predição do mob |
| Fabricador de Loot | `hostilenetworks:loot_fabricator` | Predição do mob + FE → o drop escolhido |

Mobs (24, vanilla): zombie (+husk, zombie_villager), skeleton (+stray, bogged), creeper, spider (+cave_spider),
enderman, blaze, slime, wither_skeleton, witch, ghast, magma_cube, guardian, drowned, zombified_piglin (+piglin),
shulker, phantom, evoker, vindicator, hoglin (+zoglin), breeze, elder_guardian, wither, ender_dragon, warden.

## Números (os do original)
| Tier | Dados p/ chegar | Dados por abate | Precisão | Abates até o próximo |
|---|---|---|---|---|
| Defeituoso (Faulty) | 0 | 1 | 0% (não simula) | 6 |
| Básico | 6 | 4 | 5% | 12 |
| Avançado | 54 | 10 | 22% | 30 |
| Superior | 354 | 18 | 65% | 50 |
| Autoconsciente | 1254 | 0 | 100% | — |

Precisão contínua (sobe linearmente dentro do tier). Guardião Ancião e Dragão: 3/12/30/45 dados por abate.
- Câmara: 2.000.000 FE; simulação de 240 ticks (modo Inferência), custo `sim_cost` FE/t (zumbi 128, blaze 256,
  enderman 512, esqueleto wither 768, wither/warden 2560, dragão 4096). Sempre sai 1 predição generalizada; a Predição
  do mob sai com a chance da precisão. Gasta 1 Matrix no início.
- Fabricador: 1.000.000 FE, 256 FE/t, 60 ticks por Predição; sai o drop escolhido (ex.: zumbi 64 carne podre / 8 ferro /
  16 cenoura / 16 batata). Escolha por mob, clicando no ícone na tela (fica salva no bloco); "Limpar" desfaz.
- Energia: `energy` (FE, long) no NBT do bloco. Puxa de um **cubo de energia do Mekanism** com a frente virada para a
  máquina (2 FE por 5 J, até a saída do cubo) e de uma **Célula de Energia do Powah** encostada (até 1.000/4.000 FE/t).
- Funis: Matrix entra por cima/lados na câmara, saídas por baixo; Predição entra no Fabricador por cima/lados, 16 saídas por baixo.
- Deep Learner (sem tela própria): com ele numa mão e um Data Model na outra, clique direito → o modelo entra. Com a outra
  mão vazia → devolve o último modelo. A dica (tooltip) mostra tier, dados, precisão e abates que faltam.

## Receitas (do original, tags `c:` trocadas por vanilla)
Deep Learner, Model Framework, Prediction Matrix (16), Câmara (obsidiana, pérolas, lápis, comparador, vidro), Fabricador
(netherite, diamante, ouro, obsidiana, comparador) e as 22 receitas "living matter" das predições generalizadas.

## Roteiro de teste (MCP)
Em criativo, mundo plano, posição X Y Z livre.
1. **Atunar**: `/give @s hostilenetworks:blank_data_model`, `/summon minecraft:zombie ~3 ~ ~` (com `NoAI:1b`), clique
   direito no zumbi com a Estrutura na mão → a mão passa a ter 1 `hostilenetworks:data_model_zombie`; barra de ação
   "Data Model de Zumbi construído". Num cow: "Nenhum Data Model para este mob" e a estrutura fica.
2. **Treino por abate**: `/give @s hostilenetworks:deep_learner`; Deep Learner na mão principal, o Data Model na mão
   secundária, clique direito → mão secundária vazia, tooltip do Deep Learner: "Defeituoso Zumbi: 0/6 dados ...
   faltam 6 abates". Em sobrevivência, mate 6 zumbis (espada ou arco) com o Deep Learner no inventário → tooltip
   "Básico Zumbi: 6/54 dados, precisão 5.0%, faltam 12 abates". Mate 1 husk → 10/54.
3. **Câmara**: `/setblock X Y Z hostilenetworks:sim_chamber`, `/setblock X+1 Y Z powah:energy_cell_starter`,
   `/setblock X+2 Y Z powah:furnator_starter` com 16 carvão no Furnator (ou um `mekanism:basic_energy_cube[facing=west]`
   carregado em X+1). Dê `/give @s hostilenetworks:data_model_zombie[minecraft:custom_data={data:1254}]` e
   `/give @s hostilenetworks:prediction_matrix 4`. Abra a câmara: modelo no slot de cima à esquerda, Matrix no de
   baixo. Tela: "Zumbi / Autoconsciente / Precisão 100.0% / 128 FE/t". Após 241 ticks com energia: saída de cima
   = 1 `hostilenetworks:overworld_prediction`, de baixo = 1 `hostilenetworks:prediction_zombie`; Matrix 4 → 3;
   energia gasta 239 × 128 = 30.592 FE. Modelo com `data:0` → "Defeituoso", nada acontece.
4. **Fabricador**: `/setblock X Y Z+2 hostilenetworks:loot_fabricator` com uma célula Powah carregada encostada;
   ponha a `prediction_zombie` no slot da esquerda, clique na carne podre (fica com fundo verde) → após 60 ticks
   aparecem 64 `minecraft:rotten_flesh` nas saídas e a Predição some (15.360 FE gastos). Sem escolha: não anda.
5. Shift-clique de um Data Model do inventário na câmara vai para o slot 0; de uma Matrix, para o slot 1; nada entra
   nas saídas.

## Limitações (fora da 0.1.0)
- Data Center, IO Port, Fabrication Directive, Twilight Prediction e modelos de bloco; modelos de mods (Twilight
  Forest, Allthemodium, Thermal...) e drops opcionais de mods (Reliquary etc.).
- Modo Treinamento da câmara, controle por redstone, fila do Fabricador (só modo "fixo").
- Deep Learner sem tela e sem HUD; curios.
- Data Model e Predição são **um item por mob** (no original é um item com componente): não são intercambiáveis
  com mundos do NeoForge. Texturas por mob = as do original tingidas pela cor do mob (não renderiza o mob).
- Treino por abate só no jogador "atuante" (single player / host).
- Shift-clique dentro da tela usa as regras de slot do menu (sem `native_menu_quick_move`).

## Licença
Hostile Neural Networks é **MIT** (Copyright (c) 2021 Brennan Ward), código e assets no mesmo repositório
(https://github.com/Shadows-of-Fire/Hostile-Neural-Networks, ramo 1.21, `LICENSE`). Pode publicar. O texto vai em
`LICENSE-original.txt` dentro do `.hmod`; o código Rust do port é MIT também (`NOTICE`).

## Testes
`cd source/native && cargo test --offline` — 13 testes da lógica (tiers, abates por tier, precisão, simulação,
fabricador, plano de saída, energia). Gerar de novo: `source/scripts/make_content.py` → `upgrade_hmod.py` →
`check_assets.py` → `package.py . ..`.

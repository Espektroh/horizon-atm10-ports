# Mekanism: MoreMachine para Horizon 0.1.0

Complemento do port do Mekanism (Mekanism-Horizon-0.6.1): cinco máquinas de itens do **Mekanism: MoreMachine**
(mod id `mekmm`, 1.4.3 para 1.21.1, de lostmyself8/Jerry) com os ids do original (`mekmm:*`). Elas recebem energia
dos **cabos universais, cubos de energia e geradores** do port do Mekanism e dos geradores do Mekanism Generators,
com a mesma chave `energy` (Long) no NBT e as mesmas regras de cabo (até 512 cabos, limitado pelo cabo mais lento).

São dois arquivos, os dois necessários (além do `mekanism.hmod` e do `mekanism_native-0.6.1.hnmod`, que já devem
estar instalados):

- `mekmm.hmod` (conteúdo, id `mekmm`, versão 1.4.3-horizon.1): blocos, item, receitas, loot, modelos, texturas,
  nomes e o menu da Prensa.
- `mekmm_native-0.1.0.hnmod` (nativo, id `mekmm_native`, versão 0.1.0): a lógica.

Não mexe no `mekanism.hmod` nem no `mekanismgenerators.hmod` (namespace só `mekmm`). O `instalar.ps1` confere os
SHA-256, guarda o `horizon-mods.txt` num backup, copia os dois arquivos e liga `mekmm` e `mekmm_native`. Depois:
**Build** no launcher.

## Por que estas cinco
O MoreMachine tem ~20 máquinas. A maioria usa gás ou fluido (Estação de Plantio com solução nutritiva, Replicadores
com matéria UU, Coletor de Gás, as fábricas químicas e as "large machines"), que o port do Mekanism ainda não tem.
As que só trabalham com itens são estas cinco, e são também as mais usadas no ATM10: a **Prensa** e o **Estampador
CNC** automatizam os processadores do AE2 (substituem o Inscriber), e o **Reciclador** é o começo da cadeia de matéria
UU (sucata). Torno e Laminador vêm junto porque são a mesma máquina por dentro (1 entrada, 1 saída).

## O que tem

Números do original (`MoreMachineUsageConfig`, `MoreMachineStorageConfig`, `BASE_TICKS_REQUIRED`), iguais para as cinco:
**50 J/t, guarda 20 kJ, 200 ticks por operação** (10.000 J por operação). Aceitam as melhorias do port
(`mekanism:upgrade_speed` / `mekanism:upgrade_energy`, até 8 de cada, clique direito com a melhoria na mão; voltam
para o inventário ao quebrar a máquina). 8 de velocidade: 20 ticks e 5.000 J/t; 8 de energia: 5 J/t e 200 kJ.

| Bloco | Slots | Gasta | Original |
|---|---|---|---|
| Reciclador (`mekmm:recycler`) | entrada → saída | a entrada | `TileEntityRecycler` |
| Estampador CNC (`mekmm:cnc_stamper`) | entrada + molde → saída | só a entrada (o molde fica) | `TileEntityStamper` |
| Torno CNC (`mekmm:cnc_lathe`) | entrada → saída | a entrada | `TileEntityLathe` |
| Laminador CNC (`mekmm:cnc_rolling_mill`) | entrada → saída | a entrada | `TileEntityRollingMill` |
| Prensa (`mekmm:presser`) | 3 entradas → saída | as três | `TileEntityPresser` |

Item novo: **Sucata** (`mekmm:scrap`), que sai do Reciclador.

### Energia
- As máquinas **puxam** a energia: a cada tick, se têm espaço, seguem os cabos universais (todos os tiers, até 512
  cabos) a partir das 6 faces e tiram das fontes que acham, a mais perto primeiro, no máximo a saída por tick da fonte
  e o limite do cabo mais lento no caminho (encostada direto na fonte, sem limite de cabo).
- Fontes e por onde elas dão: cubos de energia básico/avançado/elite/definitivo (4/16/64/256 kJ/t, **só pela frente**,
  como no port); Gerador a Calor e Gerador Solar antigos do port (400 e 100 J/t, todas as faces); Mekanism Generators:
  Solar (100 J/t, por baixo), Solar Avançado (600, frente e baixo), a Calor (1.020, frente), Eólico (960, frente e baixo).
- Todas as máquinas do MoreMachine juntas tiram de uma fonte no máximo a saída dela por tick (chaves
  `mekmm_pull_tick`/`mekmm_pull_amount` no NBT da fonte).
- Uma máquina do Mekanism ligada na mesma fonte continua recebendo do jeito dela (a fonte empurra para ela); as duas
  dividem o que a fonte tem.

### Telas
Fundo e slots do Horizon, entradas numa coluna em x 64 (como no original), saída em (116, 35), barra de energia
verde à direita, barra de progresso azul no meio, e à esquerda: "50 J/t", a energia guardada e a capacidade
("/ 20.000 J"). Com melhorias: "V3 E8". No Reciclador: "Sorte N/1000" (ver abaixo).

### Funis (hopper)
Saída por baixo. Torno, Laminador e Reciclador: entrada por cima e pelos lados. Estampador: entrada por cima, molde
pelos lados. Prensa: 1ª entrada por cima, 2ª pelo norte/sul, 3ª pelo oeste/leste.

### Shift-clique
Estampador: o que é molde de alguma receita vai para o slot de baixo (molde), o resto para o de cima. Prensa: cada
item vai para a entrada em que ele aparece nas receitas (1ª, 2ª, 3ª). Nas outras três, o shift-clique normal do menu
de fornalha (só há um slot de entrada).

## Receitas (formato 26.3)

### Das máquinas (as do original; troca documentada)
| Resultado | Receita |
|---|---|
| Reciclador | `ACA / O#O / ACA`: A liga reforçada, C circuito avançado, O lingote de ósmio, # Triturador (`mekanism:crusher`) |
| Estampador CNC | `ACA / #X# / ACA`: A liga infundida, C circuito básico, # pistão, X revestimento de aço |
| Torno CNC | `ACA / #X# / ACA`: A liga infundida, C circuito básico, # **rebolo** (`minecraft:grindstone`), X revestimento de aço |
| Laminador CNC | `ACA / SXS / ACA`: A liga infundida, C circuito básico, S lingote de aço, X revestimento de aço |
| Prensa | `A#A / CMC / A#A`: A liga atômica, # pistão, C circuito elite, M Estampador CNC |

Trocas: o Torno pede um **Robit** no original (não portado) → rebolo. O original aceita pistão **ou** pistão grudento;
aqui só pistão.

### De processamento
| Máquina | Entrada | Saída | Origem |
|---|---|---|---|
| Reciclador | 1 pedra, andesito, diorito, granito, ardósia (deepslate) ou tufo | 1 Sucata com 17% | original (`c:stones`, por extenso) |
| Reciclador | 1 terra ou bloco de grama | 1 Sucata com 17% | original |
| Estampador | silício do AE2 + Prensa de Silício | Silício Impresso | original (compat AE2) |
| Estampador | lingote de ouro + Prensa de Lógica | Circuito de Lógica Impresso | original (compat AE2) |
| Estampador | diamante + Prensa de Engenharia | Circuito de Engenharia Impresso | original (compat AE2) |
| Estampador | cristal de quartzo certus (ou carregado) + Prensa de Cálculo | Circuito de Cálculo Impresso | original (compat AE2) |
| Prensa | circuito impresso (lógica/engenharia/cálculo) + redstone + silício impresso | o processador | original (compat AE2) |
| Estampador | 1 lingote de ferro + molde placa de pressão pesada | 1 placa de pressão pesada | **do port** (vanilla) |
| Estampador | 1 lingote de ouro + molde placa de pressão leve | 1 placa de pressão leve | **do port** (vanilla) |
| Prensa | livro + bolsa de tinta + pena | livro e pena | **do port** (vanilla) |
| Torno | 1 tábua (qualquer, `#minecraft:planks`) | 4 gravetos | **do port** (vanilla) |
| Torno | 3 lingotes de cobre | 1 para-raios | **do port** (vanilla) |
| Laminador | 1 cana-de-açúcar | 2 papéis | **do port** (vanilla) |
| Laminador | 3 lingotes de ferro | 8 barras de ferro | **do port** (vanilla) |

As receitas do AE2 usam os ids do AE2 (`ae2:silicon_press`, `ae2:printed_silicon`...) e só aparecem se um port do
AE2 com esses ids estiver instalado (o `ae2` do registro do Horizon); sem ele, o Horizon as ignora sem erro. O Torno e
o Laminador do original só têm receitas de compat (Immersive Engineering, Create Additions), mods não portados: por isso
as receitas vanilla do port, para as máquinas servirem para alguma coisa e poderem ser testadas.

**Chance do Reciclador (diferença do original):** o original sorteia a cada operação; aqui a chance vai acumulando
(17% = 170/1000 por operação, sai 1 sucata cada vez que chega a 1000). Dá a mesma média, sem sorte: **6 pedras dão
1 sucata (sobram 20/1000), 100 pedras dão 17**. O que sobrou fica salvo na máquina ("Sorte N/1000" na tela).

## Roteiro de teste (MCP, criativo, mundo plano novo)

Preparação: `/gamemode creative`. As posições abaixo usam Y = -60 (chão do mundo plano); troque se for outro.
Medida: o conteúdo dos slots (abrir a tela ou ler o bloco) e a energia na tela da máquina (e no NBT, chave `energy`,
Long). Os ticks contam a partir de quando o combustível/entrada estão no lugar; os números valem com ±5 ticks
(o primeiro tick de energia do gerador).

1. **Torno ligado direto num gerador.** `/setblock 0 -60 0 mekanism:heat_generator` e
   `/setblock 1 -60 0 mekmm:cnc_lathe`. No gerador, 1 `minecraft:coal`. No torno, 2 `minecraft:oak_planks`.
   Esperado: a tela do torno mostra "50 J/t" e a energia sobe; **em até 205 ticks a saída tem 4 `minecraft:stick` e a
   entrada 1 tábua; em até 405 ticks, 8 gravetos e a entrada vazia**. Depois disso a energia do torno chega a
   "20.000 J" e fica (gerador a 200 J/t, torno parado).
2. **Cubo pela frente, por cabos, até o Reciclador.** `/setblock 0 -60 3 mekanism:heat_generator`,
   `/setblock 1 -60 3 mekanism:basic_energy_cube[facing=east]`, `/setblock 2 -60 3 mekanism:basic_universal_cable`,
   `/setblock 3 -60 3 mekanism:basic_universal_cable`, `/setblock 4 -60 3 mekmm:recycler`. No gerador 1
   `minecraft:coal_block`. No reciclador 6 `minecraft:stone`. Esperado: a energia do reciclador sobe (até 20.000 J);
   **depois de 6 operações (~1.200 ticks) a entrada fica vazia, a saída tem 1 `mekmm:scrap` e a tela mostra
   "Sorte 20/1000"**. Com 5 pedras só: 0 sucata e "Sorte 850/1000".
3. **Cubo virado para o lado errado não dá.** `/setblock 0 -60 6 mekanism:heat_generator`,
   `/setblock 1 -60 6 mekanism:basic_energy_cube[facing=north]`, `/setblock 2 -60 6 mekmm:cnc_rolling_mill` (encostado
   no lado leste do cubo). 1 `minecraft:coal` no gerador. Esperado: **depois de 100 ticks o cubo tem ~20.000 J ("x /
   4,0 MJ" na tela dele) e o laminador continua com 0 J** (o cubo só dá pela frente, que aponta para o norte).
   `/setblock 2 -60 6 minecraft:air` e `/setblock 1 -60 5 mekmm:cnc_rolling_mill` (na frente do cubo, lado norte):
   **no tick seguinte o laminador recebe 4.000 J e enche (20.000 J) em 5 ticks** (4 kJ/t, a saída do cubo básico).
4. **Laminador com contagem de entrada.** Laminador alimentado como no passo 1 (gerador a calor ao lado + 1 carvão).
   7 `minecraft:iron_ingot` na entrada. Esperado: **em até 205 ticks a saída tem 8 `minecraft:iron_bars` e sobram 4
   lingotes; em até 405 ticks, 16 barras e sobra 1 lingote, e ele para** (1 não basta para a receita de 3).
   1 `minecraft:sugar_cane`: 2 `minecraft:paper` em até 205 ticks.
5. **Estampador e molde.** Estampador alimentado como no passo 1. Com a tela aberta, dê shift-clique em 1
   `minecraft:heavy_weighted_pressure_plate` do inventário: **vai para o slot de baixo (molde)**; shift-clique em 3
   `minecraft:iron_ingot`: **vão para o de cima**. Esperado: **em até 205 ticks a saída tem 1 placa pesada, a entrada 2
   lingotes e o molde continua 1**. Sem molde, não trabalha (a barra azul não anda).
6. **Prensa.** `/setblock 1 -60 9 mekmm:presser` com um gerador a calor + 1 carvão ao lado. Shift-clique em 1
   `minecraft:book`, 1 `minecraft:ink_sac` e 1 `minecraft:feather`: **cada um vai para a sua entrada (de cima para
   baixo: livro, tinta, pena)**. Esperado: **em até 205 ticks a saída tem 1 `minecraft:writable_book` e as três
   entradas ficam vazias**.
7. **Melhorias.** Com 8 `mekanism:upgrade_speed` e 8 `mekanism:upgrade_energy` na mão, clique direito no torno do
   passo 1 (um por clique): a tela mostra **"V8 E8", "500 J/t" e "/ 200,0 kJ"**; com 1 tábua e energia, sai 4 gravetos
   em 20 ticks. Quebre o torno: **voltam 8 + 8 melhorias** para o inventário e o próprio torno cai.
8. **Funil.** Funil (`minecraft:hopper`) em cima do torno com 4 tábuas: as tábuas entram na entrada. Funil embaixo do
   torno: os gravetos saem para ele.
9. **Receitas e loot.** Dê `mekanism:alloy_infused`, `mekanism:basic_control_circuit`, `minecraft:piston`,
   `mekanism:steel_casing` e faça o Estampador CNC na bancada (`ACA / #X# / ACA`). Em sobrevivência, quebre uma
   máquina com picareta de pedra: cai o próprio bloco.

## Ficou de fora nesta versão (documentado)
- Todas as máquinas com gás/fluido: Estação de Plantio, Replicadores (itens, fluidos, químicos), Coletor de Gás
  Ambiente, fábricas avançadas, máquinas grandes (Large Rotary Condensentrator, etc.), estações sem fio, tanques
  químicos médios/máximos, e os itens da cadeia (matéria UU, solução nutritiva, caixa de sucata).
- **Fábricas** (tiers básico a definitivo das cinco máquinas): fora.
- Receitas que dependem de itens não portados: Refletor da Prensa (prata, folha de PEAD), Reciclador com substrato,
  todas as de IE, Create Additions e dos addons do AE2 (ExtendedAE, MEGA Cells, Advanced AE...).
- **Configuração de lados e ejetor** do Mekanism (não há no port); os funis seguem as faces fixas de cima.
- **Slot de energia** (carregar a máquina por item) não existe: só por cabo/cubo/gerador.
- **Cabo na tela:** o cabo universal do port só "encosta" visualmente nos blocos da tag `mekanism:cable_connects`, que
  é do `mekanism.hmod` (um mod não pode escrever no namespace de outro). A energia passa normalmente (o caminho é pelos
  nomes dos blocos), mas o cabo aparece sem o braço até a máquina. Para ligar visualmente, a próxima versão do
  `mekanism.hmod` precisa pôr `mekmm:recycler`, `mekmm:cnc_stamper`, `mekmm:cnc_lathe`, `mekmm:cnc_rolling_mill` e
  `mekmm:presser` nessa tag (o Horizon descarta ids que não existem, então não quebra sem este mod).
- A chance do Reciclador é acumulada, não sorteada (ver acima).
- A fonte pode dar, no mesmo tick, a saída inteira para as máquinas do Mekanism (ela empurra) e de novo a saída
  inteira para as do MoreMachine (elas puxam). No original a saída é uma só para a rede.
- Sem tooltip/JEI das receitas novas e sem som das máquinas.

## Licença
Mekanism: MoreMachine é **MIT** (lostmyself8, 2025; `source/LICENSE-MoreMachine.txt`), código **e** assets: o
repositório oficial (branch 1.21.1, `gradle.properties`: `mod_license = MIT`) não tem licença à parte para texturas,
modelos ou lang. Números e regras lidos no código público. Texturas, modelos e nomes vêm do repositório oficial; a
textura `bottom` vem do Mekanism (MIT, `source/LICENSE-Mekanism.txt`). Port para Horizon também MIT. Pode publicar
(mantendo os créditos e as licenças).

## Testes feitos aqui
- `cargo test` do runtime (`source/native`): **11 testes, todos passam** (números do original, as 5 máquinas, molde
  que não é gasto, as 3 entradas da prensa, contagem de entrada, sorte do reciclador 6→1 e 100→17, melhorias, cabos e
  faces das fontes, busca pelos cabos, limite por tick das fontes, 20 kJ em 5 ticks de um cubo básico).
- Compilação de verdade no CI (Horizon 66f3981 e Nightly 2ebf00a + horizon-mixins 5a64af9 + todos os outros mods
  nativos do repositório, inclusive `mekanism_native` 0.6.1 e `mekanismgenerators_native` 0.1.0): **verde nas duas**.
- `check_content.py` (com o conteúdo do Mekanism): JSON válido, modelos, texturas, nomes, loot e blockstates
  conferidos; os únicos avisos são os itens `ae2:*` das receitas de compat (esperado: vêm de outro mod).
- `upgrade_hmod.py` rodado no conteúdo (5 tabelas de loot convertidas para o 26.3).
- Ainda **não** testado dentro do jogo.

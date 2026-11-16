# Mekanism Generators para Horizon 0.1.0

Complemento do port do Mekanism (Mekanism-Horizon-0.6.1): quatro geradores com os ids do original
(`mekanismgenerators:*`) que mandam energia para os **cabos universais, cubos de energia e máquinas do port**, do
mesmo jeito que os geradores do próprio port (chave `energy` no NBT do bloco, pelos cabos, até 512 cabos de alcance,
limitado pelo cabo mais lento no caminho).

São três arquivos, os três necessários (além do `mekanism_native-0.6.1.hnmod`, que já deve estar instalado):

- `mekanismgenerators.hmod` (conteúdo, id `mekanismgenerators`, versão 10.7.19-horizon.1): blocos, item, receitas,
  loot, modelos, texturas, nomes.
- `mekanismgenerators_native-0.1.0.hnmod` (nativo, id `mekanismgenerators_native`, versão 0.1.0): a lógica.
- `mekanism.hmod` **corrigido** (10.7.19-horizon.7), que substitui o 10.7.19-horizon.6 da 0.6.1. Motivo: o antigo
  reclamava o namespace `mekanismgenerators` (para as texturas dos geradores antigos), e o Horizon não deixa dois mods
  ligados no mesmo namespace — ele carregaria o `mekanism` e deixaria o `mekanismgenerators` de fora ("mods mekanism
  and mekanismgenerators both use the namespace mekanismgenerators"). O corrigido só: (1) tira essa linha e move os
  3 modelos + 3 texturas para `assets/mekanism/.../legacy_generators/`; (2) põe os 4 geradores novos na tag
  `mekanism:cable_connects` (o cabo se liga a eles na tela); (3) renomeia os antigos para "Gerador a Calor (antigo)" e
  "Gerador Solar (antigo)". Mesmos ids, receitas e NBT; o `mekanism_native` 0.6.1 não muda.

O `instalar.ps1` confere os SHA-256, guarda o `mekanism.hmod` antigo e o `horizon-mods.txt` num backup, copia tudo e
liga os três em `horizon-mods.txt`. Depois: **Build** no launcher.

## O que tem

| Bloco | Gera | Guarda | Sai por | Saída máx. |
|---|---|---|---|---|
| Gerador a Energia Solar (`mekanismgenerators:solar_generator`) | até 50 J/t | 96 kJ | baixo | 100 J/t |
| Gerador a Energia Solar Avançado (`mekanismgenerators:advanced_solar_generator`) | até 300 J/t | 200 kJ | frente e baixo | 600 J/t |
| Gerador a Calor (`mekanismgenerators:heat_generator`) | 200 J/t queimando + 30 J/t por lado com lava + 100 J/t no Nether | 160 kJ | frente | 1.020 J/t |
| Gerador Eólico (`mekanismgenerators:wind_generator`) | 60 a 480 J/t pela altura | 200 kJ | frente e baixo | 960 J/t |

"Frente" é o lado para onde o bloco aponta (`facing`; colocado à mão, aponta para o jogador). Os lados de saída e a
saída máxima (2x a geração máxima) são os do original. Os geradores não recebem energia de nada.

Item novo: **Painel Solar** (`mekanismgenerators:solar_panel`), ingrediente dos solares.

### Solares (números do `TileEntitySolarGenerator` / `SolarCheck`)
- Só geram com **sol à vista**: dimensão com luz do céu e céu claro o bastante (`skyDarken < 4`, ou seja, de dia;
  tempestade com trovão escurece demais e para), e **céu aberto** acima: todos os blocos do bloco de cima até o topo
  do mundo deixam passar a luz do céu (ar, vidro...). Folhas, água e blocos sólidos tapam.
- Geração = pico x brilho do sol x multiplicador do bioma. Brilho: 1,0 entre ~730 e ~11.270 de tempo do dia (meio
  do dia), caindo até 0,2 à noite; chuva e trovão diminuem.
- Bioma: multiplicador = 1 + 0,3 x (0,8 − temperatura) − 0,3 x umidade (a umidade só conta onde chove).
  Planícies: 0,88 (44 J/t); deserto: 0,64 (32 J/t); bioma com temperatura 0,8 e sem chuva: 1,0 (50 J/t).
- **Chuva**: onde o bioma tem chuva, chovendo gera só 20% (e o sol fica mais fraco). É a regra do original (ele não
  para na chuva, só perde 80%). Em bioma sem chuva (deserto) a chuva não muda nada.
- Avançado: confere 9 pontos (um 3x3 no nível acima do bloco); precisa de 5 ou mais com céu aberto; a geração é a
  média dos 9 (os tapados contam 0). Planícies, meio-dia, céu limpo: 264 J/t.
- O céu é olhado uma vez por segundo (como o port faz no gerador solar antigo).

### Gerador a Calor (`TileEntityHeatGenerator`)
- Tanque interno de lava de 24.000 mB. O slot de combustível aceita **qualquer combustível de fornalha**: cada item
  vira tempo de queima / 20 mB de lava, um por vez, se couber (carvão = 80 mB, bloco de carvão = 800 mB, balde de lava
  = 1.000 mB e devolve o balde vazio). É a conta do original (`FluidFuelInventorySlot`).
- Queima 10 mB/t para 200 J/t enquanto tiver espaço para energia (cheio, guarda a lava). Ou seja: **um carvão dá
  1.600 J** (8 ticks), um balde de lava 20.000 J (100 ticks). Bem menos que o gerador antigo do port (que dava 200 J/t
  pelo tempo de queima inteiro); é o número do original, que foi feito para lava.
- Bônus passivo, sempre (mesmo sem combustível): 30 J/t por lado (dos 6) encostado em lava (fonte ou corrente),
  +100 J/t no Nether.
- Funil/hopper põe combustível por qualquer lado. Shift-clique leva o combustível ao slot certo (até tronco, que a
  fornalha mandaria para a entrada escondida).

### Gerador Eólico (`TileEntityWindGenerator`)
- Precisa de céu aberto acima do bloco (do bloco de cima até o topo do mundo, como os solares) e dimensão com luz do
  céu (no Nether e no End não gira).
- Geração pela altura do topo (Y do bloco + 4, como no original, que tem 5 blocos de altura): 60 J/t com o topo em
  Y ≤ 24, subindo em linha reta até 480 J/t no topo do mundo lógico (Y 319 no Overworld).
  Exemplos: bloco em Y 64 → 122 J/t; Y 100 → 173 J/t; Y 200 → 316 J/t; Y 315 → 480 J/t.
- Recalcula uma vez por segundo. Não depende de chuva nem de hora (como o original).

## Receitas (formato 26.3)

| Resultado | Receita |
|---|---|
| Painel Solar | `GGG / RAR / OOO`: G painel de vidro, R pó de redstone, A liga infundida, O lingote de ósmio |
| Gerador a Energia Solar | `PPP / AIA / OEO`: P painel solar, A liga infundida, I lingote de ferro, O lingote de ósmio, E bloco de redstone |
| Gerador a Energia Solar Avançado | `SAS / SAS / III`: S gerador solar, A liga infundida, I lingote de ferro |
| Gerador Eólico | ` O  / OAO / ECE`: O lingote de ósmio, A liga infundida, E bloco de redstone, C circuito básico |
| Gerador a Calor | sem forma: 1 Gerador a Calor (antigo) do port (`mekanism:heat_generator`) |
| Gerador a Energia Solar | também sem forma: 1 Gerador Solar (antigo) do port |

Trocas em relação ao original: o **Tablete de Energia** (não portado) vira **bloco de redstone**, como o port do
Mekanism já faz no cubo básico. O Gerador a Calor do original (`III / WOW / CFC`) tem exatamente a receita que o port
usa para o `mekanism:heat_generator` antigo; para não brigar na bancada, o novo sai de uma troca 1:1 do antigo
(faça o antigo com a receita do original e coloque-o sozinho na bancada).

## Roteiro de teste (MCP, criativo, mundo plano ou planícies)

Preparação: `/gamerule doDaylightCycle false`, `/time set 6000` (meio-dia), `/weather clear`, `/gamemode creative`.
Medida: a energia do cubo aparece na tela dele ("x / 4,0 MJ") e no NBT do bloco (chave `energy`, Long). A tela de
cada gerador mostra "Gerando N J/t". Todas as posições abaixo com **céu aberto** (nada acima). Contagens de ticks
são a partir do momento em que os dois blocos estão colocados; os números valem com ±1 tick de geração.

1. **Solar → cubo embaixo.** Bioma planícies (`plains`). `/setblock X Y Z mekanism:basic_energy_cube` e
   `/setblock X Y+1 Z mekanismgenerators:solar_generator`. Esperado: a tela do solar mostra "Gerando 44 J/t" e
   "Vendo o sol"; **após 100 ticks o cubo tem 4.400 J** (44 x 100); o solar fica com 0 J (passa tudo).
   Num bioma sem chuva com temperatura 0,8 seria 50 J/t; no deserto, 32 J/t.
2. **Solar do lado não carrega.** Cubo ao lado (mesma altura) de um solar: o cubo fica em 0 J e o solar enche
   (96 kJ em ~2.182 ticks) — o solar só sai por baixo, como no original.
3. **Solar à noite / tapado.** `/time set 18000`: "Gerando 0 J/t", "Sem sol". De dia, um bloco de pedra em Y+2
   (em cima do solar): 0 J/t em até 20 ticks. Vidro em Y+2: continua 44 J/t.
4. **Solar Avançado.** Planícies, meio-dia, cubo em (X,Y,Z), `mekanismgenerators:advanced_solar_generator` em
   (X,Y+1,Z), 3x3 em Y+2 livre. Esperado: "Painéis ao sol: 9/9", "Gerando 264 J/t"; **após 100 ticks o cubo tem
   26.400 J**. Pondo 5 blocos de pedra no 3x3 de Y+2 (incluindo o do meio): 0 J/t ("4/9").
5. **Gerador a Calor com balde de lava → cubo na frente.** `/setblock X Y Z mekanismgenerators:heat_generator[facing=east]`
   e `/setblock X+1 Y Z mekanism:basic_energy_cube`. Abra o gerador e ponha 1 `minecraft:lava_bucket` no slot.
   Esperado: o slot fica com `minecraft:bucket`, "Lava: 1000 / 24000 mB" e logo vai baixando 10 mB/t; "Gerando
   200 J/t"; **após 100 ticks o cubo tem 20.000 J e o tanque 0 mB**; depois "Gerando 0 J/t" e o cubo para em 20.000 J.
6. **Calor com carvão.** Mesmo arranjo, 1 `minecraft:coal`: o cubo recebe **1.600 J** (8 ticks a 200 J/t).
   64 carvões de uma vez: entram um por tick conforme cabe (80 mB cada), e o cubo chega a 102.400 J.
7. **Calor com lava ao lado.** Sem combustível, `/setblock X Y+1 Z minecraft:lava` e `/setblock X Y Z+1 minecraft:lava`
   (2 lados): "Gerando 60 J/t"; **após 100 ticks o cubo tem 6.000 J**. Com o balde do passo 5 junto: 260 J/t.
8. **Eólico → cubo embaixo.** Cubo em Y 63, `mekanismgenerators:wind_generator` em Y 64, céu aberto: "Gerando
   122 J/t", **após 100 ticks o cubo tem 12.200 J**. Em Y 100 (cubo em 99): 173 J/t → 17.300 J em 100 ticks.
   Com um bloco em cima: 0 J/t em até 20 ticks.
9. **Pelos cabos até uma máquina.** Solar em (X,Y+1,Z), `mekanism:basic_universal_cable` em (X,Y,Z) e (X+1,Y,Z),
   `mekanism:crusher` em (X+2,Y,Z): a energia do triturador sobe 44 J/t; com 1 `minecraft:iron_ore` na entrada, ele
   trabalha (o solar sozinho não acompanha os 50 J/t do triturador: ele anda, para, anda). O cabo aparece ligado ao
   gerador (tag `mekanism:cable_connects` do mekanism.hmod corrigido).
10. **Receitas.** Dê `mekanism:alloy_infused`, `mekanism:ingot_osmium`, `minecraft:glass_pane`, `minecraft:redstone`
    e faça o Painel Solar; depois o Gerador Solar; 1 `mekanism:heat_generator` sozinho na bancada →
    `mekanismgenerators:heat_generator`. Quebrar um gerador com picareta solta o próprio bloco.

## Ficou de fora nesta versão (documentado)
- **Gerador de Bio-combustível**: precisa do bio-combustível (item do Mekanism, sai do Triturador com plantas) e de
  tanque de fluido; não portado nesta versão.
- **Gerador a Gás**, **Reator de Fusão**, **Turbina Industrial**, **Reator de Fissão** (e seus blocos: carcaças,
  portas, bobinas, condensador, etc.): fora, como combinado. Também os fluidos (bioetanol, deutério, trítio).
- Gerador a Calor: sem o **modelo térmico** do original (capacitor de calor, eficiência de Carnot, troca de calor com
  o bloco de baixo): o calor vira energia 1:1 (o original perde parte), sem tanque recebendo lava por canos (não há
  canos de fluido), sem "lava-logged".
- Solares e eólico: sem slot de carregar item (o original tem um slot que descarrega energia num item).
- **Gerador Eólico e Solar Avançado são um bloco só**: o original ocupa 5 blocos (eólico) e 3x3x3 (avançado) com
  blocos de borda. Aqui os modelos foram encolhidos para 2 blocos de altura visual (limite de modelo do vanilla), o
  espaço acima não é "ocupado" (pode colocar coisa ali, mas aí tapa o céu e para), e as pás do eólico não giram.
  A conta de altura do eólico continua usando Y + 4 (o topo do original).
- Biomas: usa a temperatura base do bioma (sem o ajuste pela altura; só muda algo acima de Y 80) e o bioma do bloco
  para os 9 pontos do avançado.
- Dimensões com tag `mekanism:no_wind`: o original não põe nenhuma por padrão; aqui não há essa tag.
- Os geradores antigos do port (`mekanism:heat_generator`, `mekanism:solar_generator`) continuam existindo e
  funcionando como antes (eles são do `mekanism_native`); só mudaram de nome na tela.

## Licença
Mekanism e Mekanism Generators são MIT (Aidan C. Brady e equipe; `source/LICENSE-Mekanism.txt`). Números e regras
lidos no código público do Mekanism (branch 1.21.x); texturas, modelos e nomes vêm do jar original. Port para
Horizon também MIT.

## Testes feitos aqui
- `cargo test` do runtime (`source/native`): 11 testes, todos passam (números dos 4 geradores, sol/brilho/escuridão,
  biomas, chuva, 5 de 9 pontos, vento por altura, lava/carvão, cabos e cubos do port, os 4.400 J do passo 1).
- Compilação de verdade no CI (Horizon 66f3981 + horizon-mixins 5a64af9 + todos os outros mods nativos, incluindo o
  `mekanism_native` 0.6.1): verde. Ver o relatório para o link da run.
- `check_content.py`: JSON válido, modelos/texturas/itens referidos existem (os dois conteúdos juntos): 0 erros.
- Ainda **não** testado dentro do jogo.

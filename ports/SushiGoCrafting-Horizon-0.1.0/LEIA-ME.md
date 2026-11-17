# Sushi Go Crafting para Horizon 66f3981 (port 0.1.0)

Port do [Sushi Go Crafting](https://github.com/Buuz135/SushiGoCrafting) 0.6.7 (Buuz135, branch `1.21`, commit
`c3b296b`) para o Horizon (Minecraft 26.3 em Rust). Plantações (arroz na água, pepino, soja, wasabi, gergelim),
abacateiro com geração no mundo, peixes e ingredientes, os 34 sushis do original como comida, a Tábua de Corte, a
Panela de Arroz, o Barril de Fermentação e a Esteira (Roller). Os ids são os do original (`sushigocrafting:`).

**Licença: MIT**, do código e dos assets (o repositório tem um único `LICENSE` MIT, sem licença separada para
texturas e modelos). Pode compartilhar, mantendo o aviso de licença (`LICENSE-original.txt` no `.hmod` e `NOTICE`
no `.hnmod`).

## São dois mods, os dois são necessários

- **`sushigocrafting.hmod`** (conteúdo): blocos, itens, texturas, modelos, nomes (inglês e português), receitas,
  loot, tags, geração do abacateiro, os 3 efeitos do mod.
- **`sushigocrafting_native-0.1.0.hnmod`** (nativo, `sushigocrafting_native` 0.1.0): o que precisa de código.

Sem o nativo: as sementes plantam **beterraba/trigo** (o elo semente → plantação é do nativo), o arroz não cresce,
a muda vira **carvalho**, as máquinas não funcionam e a Esteira **é gasta** na receita. Ele não depende de nenhum
outro mod (o original depende do Titanium; o que era preciso dele foi refeito dentro do `sushigocrafting_native`).

## O que tem

### Plantações (todas com 4 idades, 0 a 3, como o original)

| Semente | Planta em | Madura dá | Imatura dá |
|---|---|---|---|
| `rice_seeds` (Arroz) | **dentro da água** (fonte), sobre **terra** (`minecraft:dirt`) | 1 Arroz + 1 a 4 sementes | 1 semente |
| `cucumber_seeds` | terra arada | 1 Pepino + 1 a 4 sementes | 1 semente |
| `soy_seeds` | terra arada | 1 Grão de Soja + 1 a 4 sementes | 1 semente |
| `wasabi_seeds` | terra arada | 1 Raiz de Wasabi + 1 a 4 sementes | 1 semente |
| `sesame_seeds` | terra arada | 2 a 17 Gergelim + 1 a 4 sementes | 1 semente |

- **Crescimento**: a regra do `CropBlock.randomTick` (luz 9 ou mais, chance `1/(int(25/f)+1)` por random tick, `f`
  pela terra arada em volta, metade com plantas iguais em linha/diagonal). O arroz, sobre terra, tem `f = 1`:
  1 em 26 por random tick, como no original.
- **Farinha de osso**: nas quatro de terra arada, como a beterraba (0 ou 1 idade, ¾ das vezes 1), igual ao
  original (`getBonemealAgeIncrease / 3`). No arroz, a mesma conta, feita pelo nativo.
- **O arroz fica na água como no original** (a plantação é água + arroz; quebrar deixa a fonte de água). Só planta
  numa fonte de água com terra embaixo; em pedra, areia etc. não.
- **Sementes da grama**: quebrar grama/samambaia (curta ou alta) que soltaria semente de trigo troca por peso,
  como o `SeedsLootModifier`: trigo 100, arroz 25, wasabi 25, soja 25, pepino 25, gergelim 10 (de 210).
- **Geração selvagem**: o original **não gera plantações selvagens** no mundo (as sementes vêm da grama). O que ele
  gera é o **abacateiro**, e isso veio (abaixo).

### Abacateiro

- **Geração**: o `trees_avocado` do original nas **planícies** (`minecraft:plains`), chunks novos: 1 em 20 chunks
  ganha uma árvore (tronco de 2 ou 3, folhas em bolha raio 1, metade em estágio 0, ¼ em 1, ¼ em 2).
- **Folhas** (`avocado_leaves`, estágio 0, 1, 2): o estágio 1 amadurece para 2 (1 em 3 por random tick, ou na hora
  com farinha de osso). **Clique direito numa folha estágio 2 dá 1 Abacate** e ela volta ao estágio 1.
  Quebrar: tesoura/toque suave pega a folha; senão 20% muda (Fortuna aumenta) e 2% de 1–2 gravetos.
- **Muda**: cresce em abacateiro (luz 9+, 1 em 7 por random tick, dois passos, como a muda vanilla), ou com farinha
  de osso (45% por uso). Precisa do espaço livre (ar, grama, folhas). Tronco e "tronco com folhas" também.

### Peixes e ingredientes

| Item | Como se obtém | Porções por vez |
|---|---|---|
| Atum Cru (`raw_tuna`) | **pescando**: 15% por peixe fisgado (não há a entidade do atum) | 1 |
| Camarão (`shrimp`) | **pescando**: 15% por peixe fisgado (não há a entidade do camarão) | 1 a 2 |
| Tobiko (`tobiko`) | matar peixe (bacalhau, salmão, baiacu, tropical): 30% | 1 a 4 |
| Filé de Salmão (`salmon_fillet`) | Tábua de Corte: salmão cru | 17 a 64 |
| Filé de Atum (`tuna_fillet`) | Tábua de Corte: atum cru | 33 a 64 |
| Kani (`imitation_crab`) | Tábua de Corte: bacalhau cru | 2 a 7 |
| Fatias de Abacate | Tábua de Corte: abacate | 2 a 11 |
| Fatias de Pepino | Tábua de Corte: pepino | 3 a 13 |
| Pasta de Wasabi | Tábua de Corte: raiz de wasabi | 3 a 12 |
| Folhas de Nori (`nori_sheets`) | Tábua de Corte: **bloco de alga seca** | 5 a 8 |
| Arroz Cozido (`cooked_rice`) | Panela de Arroz: arroz + água + combustível | 2, 3, 5 ou 7 por lote de 1–4 arrozes |
| Molho de Soja (`soy_sauce`) | Barril: grão de soja + 250 mB de água | 3 a 12 |
| Queijo (`cheese`) | Barril: nada no slot + 250 mB de leite | 4 a 21 |
| Gergelim (`sesame_seed`) | colheita do gergelim | 2 a 17 |

**Porções** (simplificação do Horizon): no original esses ingredientes são itens únicos com uma quantidade em
gramas (ex.: um filé de salmão de 500 a 2000 g) e a Esteira tira gramas conforme o "peso" escolhido. Aqui cada um
é um item empilhável (64) e **1 item = 1 porção = o que a Esteira tirava no peso máximo**
(`FoodIngredient.defaultAmount`: salmão/atum/kani/arroz/camarão 30 g, abacate 45 g, pepino 15 g, tobiko e queijo
12 g, gergelim 6 g, molho e wasabi 4 g). Um corte ou um drop sorteia as gramas como o original
(`nextInt(max − min) + min`) e dá `gramas ÷ porção` (arredondado, de 1 a 64). A Panela faz 50 g por arroz.

Esses ingredientes são comestíveis como no original: 2 de fome, 1,2 de saturação, com 1% de chance de Veneno 5 s
(60% no molho de soja e na pasta de wasabi).

### Sushis (34, todos os do original)

Comida com os números do `FoodItem.Info` do original para um sushi **sem pesos** (o que ele calcula para a comida
sem o minijogo: modificador 1,25): fome = `int(soma × 1,25)`, saturação = fome × `soma_sat × 1,25 / (n + 2)` × 2
(a conta do `FoodData.eat(int, float)` do 1.21). Os efeitos são os dos ingredientes (os "adiciona" primeiro, depois
os "modifica" multiplicando o tempo e somando nível). Alguns (tabela completa em `source/content/content-report.txt`):

| Sushi | Fome | Saturação | Efeito |
|---|---|---|---|
| Maki de salmão (`salmon_maki`) | 5 | 10,0 | Mordidinhas 52,5 s |
| Maki de atum (`tuna_maki`) | 5 | 10,0 | Gosto Adquirido 52,5 s |
| Maki de abacate / pepino | 3 | 7,5 | — |
| Maki de kani (`crab_maki`) | 3 | 6,0 | Velocidade 5,25 s |
| Gunkan de salmão / atum | 5 | 10,0 | Mordidinhas / Gosto Adquirido 52,5 s |
| Gunkan de wakame | 1 | 2,0 | — |
| Nigiri de salmão / atum | 5 | 10,0 | Mordidinhas / Gosto Adquirido 52,5 s |
| Nigiri de camarão | 3 | 6,0 | Mãos Firmes 105 s |
| Onigiri | 1 | 1,875 | — |
| Temaki de salmão / atum | 7 | 17,5 | Mordidinhas / Gosto Adquirido II 63 s |
| Temaki de camarão | 6 | 15,0 | Mãos Firmes II 126 s |
| Temaki de frango | 6 | 15,0 | Queda Lenta II 21 s |
| Califórnia de atum (`tuna_california`) | 7 | 13,125 | Gosto Adquirido II 63 s |
| Califórnia de tobiko, atum e pepino | 11 | 34,375 | Gosto Adquirido II 118,1 s |
| Califórnia de kani e queijo | 7 | 17,5 | Velocidade II 6,3 s, Saturação II 31,5 s |

(A saturação acima da fome é o que a conta do original dá; o jogo limita a saturação à fome atual ao comer.)

### Máquinas

- **Tábua de Corte** (`cutting_board`): clique direito com algo cortável põe **a pilha toda** na tábua (o item
  aparece deitado nela); com o **Cutelo** (`cleaver_knife`) na mão, o **6º clique** corta 1 item e entrega o
  resultado (o original também conta 6). Agachado de mão vazia: a pilha volta. A action bar mostra
  `Tábua de Corte: 2x Salmão Cru · cortes 3/6`. Receitas: `data/sushigocrafting/recipe/cutting_board/*.json`
  (tipo `sushigocrafting:cutting_board`, campos `input`, `result` e, opcional, `count` {min, max}).
- **Panela de Arroz** (`rice_cooker`): tela de 3 slots (arroz em cima, combustível embaixo, arroz cozido à
  direita), tanque de **8000 mB de água** (balde de água com clique direito, o balde volta vazio). Cada lote:
  **100 ticks**, até **4 arrozes**, **1000 mB** de água e **1 lote de fogo** (combustível dá `tempo de queima / 200`
  lotes: carvão = 8). Funis: arroz por cima, combustível pelos lados, saída por baixo.
- **Barril de Fermentação** (`fermentation_barrel`): tela com entrada e saída, tanque de **2000 mB** (balde de
  água ou de leite; não mistura). **100 ticks** por receita: soja + 250 mB de água = molho de soja; slot vazio +
  250 mB de leite = queijo. Receitas: `recipe/fermenting_barrel/*.json` (tipo `sushigocrafting:fermenting_barrel`).
- **Esteira** (`roller`): **receitas fixas na bancada** com a Esteira na grade; **ela fica na grade** depois de
  fazer o sushi (nativo). Os padrões (N nori, C arroz cozido, F recheio, A fatias de abacate, X pepino/queijo,
  T tobiko, R Esteira):

| Tipo | Padrão | Recheios (F) |
|---|---|---|
| Maki | `NCF` / ` R ` | filé de salmão, filé de atum, fatias de abacate, fatias de pepino, kani |
| Gunkan | `F ` / `C ` / `NR` | filé de salmão, filé de atum, **capim marinho** (wakame) |
| Nigiri | `FN` / `CR` | filé de salmão, filé de atum, camarão |
| Onigiri | `N` / `C` / `R` | — |
| Temaki | `NCF` / `AR ` | filé de salmão, filé de atum, camarão, **frango assado** |
| Califórnia | `NCF` / `AXT` / ` R ` | F: atum, salmão ou kani; X: pepino, queijo ou vazio; T: tobiko ou vazio |

Receitas de bancada do original (do jar): Esteira (`BBB/SSS/BBB`, bambu e linha), Cutelo, Tábua de Corte
(lajes sobre troncos), Panela de Arroz (ferro, placa de pressão leve, fornalha, redstone), Barril (tábuas, placa de
pressão de madeira, fornalha). `c:ingots/iron` virou `minecraft:iron_ingot`.

## Instalar (Windows)

1. Feche o launcher e rode, nesta pasta: `powershell -ExecutionPolicy Bypass -File .\instalar.ps1`.
   Ele confere o SHA-256, faz backup de `horizon-mods.txt` em `mod-backups\`, copia o `.hmod` para `horizon-mods\`
   e o `.hnmod` para `horizon-native-mods\` e liga os dois. Para conferir: `-Verificar`.
2. No launcher: **Build** (Native mods ligado). Confie no código nativo se pedir.
3. Abra o jogo e confira em **Options → Mods** que `sushigocrafting` e `Sushi Go Crafting 0.1.0` estão ligados.

## Roteiro de teste (para o orquestrador)

Mundo **criativo plano** novo, de dia (`/time set day`), salvo onde diz Sobrevivência. `X Y Z` é um ponto do chão
livre num raio de 6 blocos; `Y` é a altura do chão + 1. Ids exatos.

**A. Conteúdo carregado**
1. Abra o inventário criativo: a aba **Sushi Go Crafting** existe e tem **67** itens (34 sushis).
2. `/give @s sushigocrafting:salmon_maki 1` → 1 item com textura (não o cubo roxo/preto) e nome "Maki de Salmão".
3. O log do jogo não tem "content not loaded" nem erro de `sushigocrafting`.

**B. Comida (Sobrevivência)**
1. `/gamemode survival`, `/effect give @s minecraft:hunger 30 50` até `/data get entity @s foodLevel` dar 14 ou
   menos, então `/effect clear @s`.
2. Anote `foodLevel`, coma 1 `sushigocrafting:salmon_maki`. Resultado: `foodLevel` **+5**, e
   `/data get entity @s active_effects` tem `sushigocrafting:small_bites` com `duration` perto de **1050**.
3. Coma 1 `sushigocrafting:crab_maki`: +3 e `minecraft:speed` com `duration` perto de **105**.
4. Coma 1 `sushigocrafting:tobiko_tuna_cheese_california`: +10, `sushigocrafting:acquired_taste` nível 1
   (`amplifier: 1`) ~1890 e `minecraft:saturation` `amplifier: 1` ~945. `/gamemode creative`.

**C. Plantações de terra arada**
1. `/fill X Y-1 Z X+2 Y-1 Z minecraft:farmland`, `/setblock X+1 Y-1 Z+1 minecraft:water`,
   `/give @s sushigocrafting:cucumber_seeds 3`; plante as 3. Mirando com F3: bloco `sushigocrafting:cucumber_crop`,
   `age=0` (não beterraba).
2. `/gamerule randomTickSpeed 300` e espere **60 s**: as 3 ficam `age=3` (textura madura). `/gamerule randomTickSpeed 3`.
3. `/gamemode survival`, quebre uma: solta exatamente **1 `sushigocrafting:cucumber`** e de **1 a 4**
   `sushigocrafting:cucumber_seeds`. Uma imatura (plante de novo e quebre na hora) solta só 1 semente.
4. Repita o passo 1 com `soy_seeds`, `wasabi_seeds` e `sesame_seeds` (madura: 1 grão de soja / 1 raiz de wasabi /
   **2 a 17** `sushigocrafting:sesame_seed`). `/gamemode creative`.
5. Farinha de osso numa plantação `age=0`: após **8 usos** ela está em `age=3` com probabilidade > 99%
   (¾ das vezes +1 idade). Um uso numa plantação `age=3` não gasta a farinha.

**D. Arroz na água**
1. `/fill X+4 Y-1 Z X+4 Y-1 Z minecraft:dirt`, `/setblock X+4 Y Z minecraft:water` (fonte),
   `/setblock X+5 Y-1 Z minecraft:stone`, `/setblock X+5 Y Z minecraft:water`.
2. `/give @s sushigocrafting:rice_seeds 2`. Mire a terra no fundo da água em `X+4` e clique direito: aparece o arroz
   (`sushigocrafting:rice_crop`, `age=0`), **com água em volta** (o bloco continua cheio de água). Na pedra
   (`X+5`) a semente **não planta**. Fora da água (terra seca) também não.
3. `/gamerule randomTickSpeed 1000`, espere **60 s**: `age=3`. `/gamerule randomTickSpeed 3`.
4. Quebre (Sobrevivência): **1 `sushigocrafting:rice`** + 1 a 4 sementes, e a **fonte de água fica**.
5. Farinha de osso no arroz `age=0`: ¾ das vezes +1 idade; **nunca vira capim marinho alto**.

**E. Sementes da grama (Sobrevivência)**
1. `/fill X-6 Y Z-6 X+6 Y Z+6 minecraft:short_grass replace minecraft:air` (169 tufos) e quebre todos com a mão.
2. Resultado: ~21 sementes no total (12,5%); **cerca de metade não é trigo** (arroz, soja, wasabi, pepino ~12% cada,
   gergelim ~5%). Pelo menos um tipo `sushigocrafting:*_seeds` aparece.

**F. Abacateiro**
1. `/setblock X Y-1 Z+4 minecraft:grass_block`, `/give @s sushigocrafting:avocado_sapling 1`, plante em `X Y Z+4`.
   Use `/give @s minecraft:bone_meal 32` e clique com farinha até crescer (45% por uso, 2 passos: em geral
   **≤ 10 usos**). Resultado: **2 ou 3** `sushigocrafting:avocado_log` em pé e de **9 a 17**
   `sushigocrafting:avocado_leaves` em volta do topo. **Nunca um carvalho.**
2. `/setblock X+2 Y+1 Z+4 sushigocrafting:avocado_leaves[stage=2,persistent=true]`, clique direito nela com a mão
   vazia: **+1 `sushigocrafting:avocado`** no inventário e a folha fica `stage=1`. Clique de novo: nada.
3. Farinha de osso nessa folha `stage=1` → `stage=2` na hora (1 farinha gasta fora do criativo).
4. Geração: crie um mundo normal novo (não plano), vá a uma **planície** e voe por chunks novos: aparecem
   abacateiros pequenos (cerca de 1 a cada 20 chunks de planície).

**G. Tábua de Corte**
1. `/give @s sushigocrafting:cutting_board 1`, `/give @s sushigocrafting:cleaver_knife 1`,
   `/give @s minecraft:salmon 2`, `/give @s minecraft:dried_kelp_block 1`. Coloque a tábua em `X Y Z+6`.
2. Clique direito na tábua com os 2 salmões: a mão esvazia, o salmão aparece deitado na tábua e a action bar mostra
   `Tábua de Corte: 2x … · cortes 0/6`.
3. Com o Cutelo, 5 cliques: `cortes 5/6`, nada sai. O **6º clique**: você recebe **17 a 64**
   `sushigocrafting:salmon_fillet` (action bar `Cortado: N× Filé de Salmão`) e a tábua fica com **1** salmão.
4. Agachado, mão vazia, clique: o salmão volta para a mão; a tábua fica vazia.
5. Bloco de alga seca na tábua + 6 cliques de Cutelo: **5 a 8** `sushigocrafting:nori_sheets`.
6. Uma terra (`minecraft:dirt`) na mão não entra na tábua.

**H. Panela de Arroz**
1. `/give @s sushigocrafting:rice_cooker 1`, coloque em `X+2 Y Z+6`. `/give @s minecraft:water_bucket 1`, clique
   direito na panela com o balde: action bar `Água: 1000/8000 mB` e o balde vira `minecraft:bucket`.
2. Clique direito de mão vazia: abre a tela (barra azul à esquerda, `Água: 1000/8000 mB` embaixo).
   `/give @s sushigocrafting:rice 4` e `/give @s minecraft:coal 1`; **shift-clique** no arroz: ele vai para o slot
   de cima; no carvão: vai para o slot de baixo.
3. Após **100 ticks (5 s)**: o slot da direita tem **7 `sushigocrafting:cooked_rice`**, o de cima fica vazio, a
   água cai para **0/8000 mB** e o texto do fogo (canto de cima, à direita) mostra **7.0 lotes**. Sem água, mais arroz não cozinha.

**I. Barril de Fermentação**
1. Coloque `sushigocrafting:fermentation_barrel` em `X+4 Y Z+6`; clique com `minecraft:water_bucket`:
   `Água: 1000/2000 mB`. Abra (mão vazia) e ponha 2 `sushigocrafting:soy_bean` (shift-clique vai para a entrada).
2. Após **100 ticks**: saída com **3 a 12 `sushigocrafting:soy_sauce`**, entrada com 1 grão, `Água: 750/2000 mB`.
   Após mais 100 ticks: mais 3 a 12 molhos, entrada vazia, 500 mB.
3. Outro barril em `X+6 Y Z+6`, clique com `minecraft:milk_bucket`: `Leite: 1000/2000 mB`. Entrada vazia: em 100
   ticks a saída tem **4 a 21 `sushigocrafting:cheese`** e o tanque 750 mB. Um balde de água nesse barril é recusado
   (`O tanque não cabe mais esse balde.`).

**J. Esteira e sushis (bancada)**
1. `/give @s sushigocrafting:roller 1`, `/give @s sushigocrafting:nori_sheets 4`,
   `/give @s sushigocrafting:cooked_rice 4`, `/give @s sushigocrafting:salmon_fillet 4`,
   `/give @s sushigocrafting:avocado_slices 2`, `/give @s minecraft:crafting_table 1`.
2. Na bancada: linha de cima `nori, arroz cozido, filé de salmão`, linha do meio só a Esteira no centro → sai
   **1 `sushigocrafting:salmon_maki`**; nori, arroz e salmão gastam 1 cada e **a Esteira continua na grade**.
3. `filé de salmão, nori` / `arroz cozido, Esteira` (2x2) → `salmon_nigiri`. `filé de salmão` / `arroz cozido` /
   `nori, Esteira` → `salmon_gunkan`. `nori, arroz, salmão` / `abacate, Esteira` → `salmon_temaki`.
   `nori, arroz, salmão` / `abacate` / ` , Esteira` → `salmon_california`.
4. Receita da Esteira: 3 bambus / 3 linhas / 3 bambus → 1 `sushigocrafting:roller`.

**K. Pesca e peixes (Sobrevivência)**
1. Pesque **20 peixes** com vara: em média **3 `sushigocrafting:raw_tuna`** e **3 vezes camarão** (1 ou 2
   `sushigocrafting:shrimp`) extras no inventário (15% cada por peixe fisgado).
2. `/summon minecraft:cod ~ ~ ~` na água 10 vezes e mate com espada: em média **3 drops de `sushigocrafting:tobiko`**
   (1 a 4 cada).

**L. Options → Mods**: a linha `Sushi Go Crafting 0.1.0` aparece; desligando o nativo e reiniciando, as máquinas
param (o conteúdo continua lá).

## Limitações (o que ficou fora ou mudou)

- **Minijogo da Esteira** (a tela de tipos, pesos e "perfeito"), **condimentos** (molho de soja e wasabi como
  tempero de sushi) e as **descobertas** por jogador: não vieram. Os sushis são receitas fixas de bancada com
  os valores de "sem peso". Molho e wasabi existem e são comestíveis, mas não temperam o sushi.
- **Ingredientes com gramas** viraram porções empilháveis (acima). Não há a receita de juntar quantidades.
- **Efeitos do mod** (Gosto Adquirido, Mordidinhas, Mãos Firmes) aparecem com ícone e tempo, mas **não fazem nada**
  ainda (o Horizon não dá comportamento a efeitos de mod). Os vanilla (Velocidade, Saturação, Queda Lenta,
  Resistência ao Fogo) funcionam.
- **Atum e camarão vivos**, seus baldes e o **Graveto com Alga** (que atraía o camarão) não vieram: o Horizon não
  tem entidades de peixe de mod. Atum e camarão vêm da **pesca** (15% cada), uma substituição deste port.
- **Prensa de nori com pistão** (bloco de alga seca empurrado contra um bloco de ferro): substituída pela Tábua de
  Corte (mesmos 5 a 8 por bloco).
- **Caixa Térmica** (`cooler_box`): não veio (só juntava as gramas de ingredientes, que aqui já empilham).
- **Livro Patchouli**, JEI e CraftTweaker: não.
- Panela e Barril: o tanque só enche por **balde** (não há canos de fluido); o que está no tanque se perde ao quebrar
  a máquina (os itens caem). Funil pode pôr qualquer item na entrada (a máquina só não usa).
- A Panela aceita a pilha de arroz no único slot de entrada (o original tinha 4 slots de 1); cada lote usa até 4.
- Arroz: o bloco é "substituível" como o capim marinho (colocar um bloco em cima dele o troca), e fica fora das tags
  `minecraft:crops`/`bee_growables` (abelhas e aldeões não mexem nele). Dispensador com farinha de osso não age
  no arroz nem na muda de abacate (para não virar capim alto/carvalho).
- O abacateiro crescido da muda é montado pelo nativo com a forma da feature do original; a árvore da geração é a
  feature do jogo.
- Desenho das máquinas: os modelos do original (Blockbench) num bloco de modelo de fornalha; não foi visto no jogo.

## Testes feitos aqui

- `cargo test` do runtime (`source/native`): **11 testes, todos passam** (porções, sementes da grama por peso,
  velocidade de crescimento, farinha de osso, forma da árvore, cliques da tábua, panela 100 ticks/combustível/balde,
  barril soja/queijo/saída cheia, tanques).
- `source/scripts/check_content.py`: todos os modelos, texturas, nomes (en/pt), loot, receitas (larguras e chaves)
  e tags conferidos.
- **Compilação real pelo CI** (GitHub Actions do repositório privado, `tools/ci_check.sh`: mixins aplicados +
  `cargo check -p horizon-client` junto com os outros 15 mods nativos) em **Horizon 66f3981** e **Nightly 2ebf00a**:
  verde nas duas.
- Não testado no jogo: tudo do roteiro acima.

## Arquivos

- `sushigocrafting.hmod`, `sushigocrafting_native-0.1.0.hnmod`, `SHA256SUMS.txt`, `instalar.ps1`, `APIS.md`.
- `source/content/` (o conteúdo gerado), `source/native/` (o nativo + a crate de teste), `source/scripts/`
  (`make_content.py SRC OUT` gera o conteúdo do código-fonte original, `upgrade_hmod.py --dir` o converte para 26.3,
  `check_content.py` confere, `package.py source .` empacota).

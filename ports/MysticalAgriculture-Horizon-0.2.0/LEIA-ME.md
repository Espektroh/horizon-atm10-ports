# Mystical Agriculture + Mystical Agradditions para Horizon 66f3981 (port 0.2.0)

Port do [Mystical Agriculture](https://github.com/BlakeBr0/MysticalAgriculture) 8.0.28 e do
[Mystical Agradditions](https://github.com/BlakeBr0/MysticalAgradditions) 8.0.14 (BlakeBr0, **MIT**) para o
Horizon (Minecraft 26.3 em Rust). É o núcleo da progressão do ATM10: sementes que crescem recursos.
Os ids são os do original (`mysticalagriculture:`, `mysticalagradditions:`).

## Novo na 0.2.0

- **Altar de Infusão** e **Pedestal de Infusão** (`mysticalagriculture:infusion_altar`, `infusion_pedestal`), com o
  modelo, a receita e a mecânica do original: 8 pedestais em volta, clique direito põe e tira itens, liga com
  **redstone** (ou a **Varinha**), 100 ticks (5 s) de trabalho, os itens aparecem flutuando e girando em cima dos blocos.
- As **sementes voltaram para o altar**, como no original (30 receitas `mysticalagriculture:infusion` do jar). As receitas
  de bancada `MEM/ESE/MEM` da 0.1.0 **saíram**; quem quiser continua com elas instalando o opcional
  `opcional/mysticalagriculture_seedcrafting.hmod` (é a opção `seedCraftingRecipes` do original, desligada por padrão).
- **Varinha** (`mysticalagriculture:wand`) e os **Regadores** (comum, Inferium, Prudentium, Tertium, Imperium e
  Supremium): molham a terra e aceleram o que está plantado na área, como no original.
- Mundos da 0.1.0 continuam iguais: mesmos ids, mesmos dados, o Agradditions é o mesmo arquivo (byte a byte).

## São três mods, os três são necessários

- **`mysticalagriculture.hmod`** (conteúdo): essências, lingotes, blocos, terras, minérios, sementes, receitas.
- **`mysticalagradditions.hmod`** (conteúdo): o tier 6 (Insanium), cruxes, minérios no Nether e no End.
- **`mysticalagriculture_native-0.2.0.hnmod`** (nativo): o que precisa de código (abaixo). Substitui o 0.1.0 (o
  `instalar.ps1` tira o antigo; os dois juntos têm o mesmo id e quebram o build).

Sem o nativo as sementes plantam **trigo** em vez do cultivo (o elo semente → cultivo é do nativo), a terra de
essência não dá bônus, os mobs não soltam Inferium e o altar não faz nada.

## O que tem

**Tiers 1 a 5** (Inferium, Prudentium, Tertium, Imperium, Supremium) e **6** (Insanium, Agradditions):
essência, lingote, pepita, gema, bloco de essência, bloco de lingote, bloco de gema e **terra de essência** de cada um.
Mais Prosperidade (fragmento, lingote, pepita, gema, blocos, base de semente), Cristal de Infusão (1000 usos) e
Cristal de Infusão Mestre (infinito), Essência Fertilizante, Fertilizante Místico e as aglomerações
(natureza, corante, Nether, coral, mel, prismarinho, End).

**Minérios** (como o original, só em chunks novos):

| Minério | Onde | Por veio / tentativas por chunk | Altura |
|---|---|---|---|
| Inferium (pedra/ardósia) | Overworld | 8 / 16 | -32 a 64 |
| Prosperidade (pedra/ardósia) | Overworld | 8 / 12 | -60 a 24 |
| Inferium e Prosperidade do Nether | Nether (netherrack) | 8 / 16 e 12 | 10 a 100 |
| Inferium e Prosperidade do End | End (pedra do End) | 8 / 16 e 12 | 10 a 100 |

Inferium solta 2–4 essências, Prosperidade 1–4 fragmentos (Fortuna vale; Toque Suave pega o minério).

**31 sementes:**

| Tier | Sementes (material da receita) |
|---|---|
| 1 (Inferium) | Inferium, pedra, terra, madeira (`#minecraft:logs`), gelo, ardósia |
| 2 (Prudentium) | natureza, corante, Nether, carvão, coral, mel, ametista |
| 3 (Tertium) | ferro, cobre, quartzo do Nether, glowstone, redstone, obsidiana, prismarinho, sculk |
| 4 (Imperium) | ouro, lápis, End, **ósmio** e **aço** (itens do seu Mekanism para Horizon) |
| 5 (Supremium) | diamante, esmeralda, netherita |
| 6 (Insanium) | estrela do Nether, ovo do dragão (precisam da **crux** 2 blocos abaixo do cultivo) |

A semente de Inferium é de bancada (como no original). As outras são feitas no **Altar de Infusão**: no altar a
**Base de Semente de Prosperidade**, nos 8 pedestais 4 do material e 4 da essência do tier (a ordem não importa).
Ex.: ferro = 4 lingotes de ferro + 4 Essências de Tertium; madeira = 4 troncos quaisquer + 4 Essências de Inferium;
estrela do Nether = 4 estrelas do Nether + 4 Essências de Insanium.
As 123 receitas essência → recurso do original vêm junto (ex.: 8 essências de ferro = 6 lingotes; 8 de carvão =
12 carvões; 8 de ósmio = 4 lingotes; 8 de aço = 3 lingotes).

## Altar de Infusão (como no original)

- **Receitas** (do jar): Altar = `GCG / _S_ / SSS`, Pedestal = `GCG / _S_ / _S_` (G lingote de ouro, C lã vermelha,
  S pedra); Varinha = Essência de Inferium, graveto e pedra na diagonal.
- **Montagem**: o altar e 8 pedestais **na mesma altura**, nas posições do original em volta do altar (x, z):
  `(+3, 0)`, `(0, +3)`, `(-3, 0)`, `(0, -3)`, `(+2, +2)`, `(+2, -2)`, `(-2, +2)`, `(-2, -2)`. Um pedestal fora do lugar
  não conta; um pedestal vazio também não (a receita precisa de exatamente tantos itens quantos ingredientes).
- **Clique direito** num pedestal vazio põe **1** do que está na mão; num pedestal com item, o item volta para você.
  No altar: se tem resultado, ele vem para você; senão, altar vazio recebe 1 da mão; altar com item devolve o item
  (com a **Varinha** na mão, em vez disso, liga o altar). Como no original, a mão gasta o item mesmo no criativo.
  Agachado com algo na mão, o clique é do jogo (pôr um bloco encostado).
- Ao pôr o item no altar (e com a mão vazia num altar vazio) a **action bar** mostra o estado, o que o original não
  tem: `Altar de Infusão: 8 pedestais (8 com item) · receita: Sementes de Ferro · ligue com redstone`, ou
  `· sem receita`, ou `· altar vazio`.
- **Ligar**: um sinal de **redstone** chegando ao altar (alavanca, botão, bloco de redstone encostado) ou a Varinha.
  Um pulso basta: ele fica ligado até terminar ou até a receita deixar de valer (tirou um item → zera e desliga).
- **Tempo**: **100 ticks** (5 s). Aí cada pedestal perde o seu item (fica o resto de crafting, ex. um balde), o altar
  perde a base e o resultado aparece no altar. Clique para pegar, ou um **funil embaixo** puxa o resultado.
- Os itens aparecem **flutuando** sobre o altar (1,1 bloco) e os pedestais (1,2), girando e balançando como no original.
- Quebrar o altar ou um pedestal solta o bloco e o que ele tinha.
- Receitas: todo arquivo `data/*/recipe/**.json` com `"type": "mysticalagriculture:infusion"` (o altar lê as do
  conteúdo e as de data packs), com `input`, `ingredients` (1 a 8) e `result` — ingredientes como `"minecraft:iron_ingot"`
  ou `"#minecraft:logs"`.

## Regadores

| Item | Área | Chance por regada |
|---|---|---|
| `mysticalagriculture:watering_can` (Regador) | 3x3 | 25% |
| `inferium_watering_can` | 3x3 | 25% |
| `prudentium_watering_can` | 5x5 | 30% |
| `tertium_watering_can` | 7x7 | 35% |
| `imperium_watering_can` | 9x9 | 40% |
| `supremium_watering_can` | 11x11 | 45% |

Clique direito num bloco (segurar o botão repete; no máximo 1 regada a cada 5 ticks por jogador, como o original):
toda **terra arada** (a vanilla e as de essência) na área (largura x largura x largura em volta do bloco) fica com
umidade 7; e, na chance da tabela, **cada planta da área** (até 3 blocos acima e abaixo) recebe **um random tick extra**
pelo próprio random tick do jogo: cultivos deste mod, trigo/cenoura/batata/beterraba, caules, mudas, bagas, cacau,
bambu, algas, cipós, grama, micélio, flor de chorus. As regras normais valem (luz, crux do tier 6, velocidade da terra).
Receitas do jar: o Regador (ossos/tigela/ferro) e cada regador de essência a partir do anterior
(`FGF / ISI / FGF`: fertilizante místico, lingotes do tier, regador anterior no meio).

## Os números (do código do original, configuração padrão)

- **Cultivo maduro**: 1 essência + 1 semente. Imaturo: só a semente. Farinha de osso não faz crescer (fica na mão).
- **Terra de essência**: chance de **2ª essência** e de **2ª semente** de 10% em qualquer terra de essência,
  20% na terra do próprio tier do cultivo. Tier 6 não tem esse bônus.
- **Inferium** na terra de essência: tier 1 = 1, tier 2 = 1 ou 2, tier 3 = 2, tier 4 = 2 ou 3, tier 5 = 3,
  tier 6 = 3 ou 4 essências (os "ou" são 50%).
- **Essência Fertilizante**: 10% de chance a cada cultivo de recurso maduro colhido.
- **Mobs**: 20% de chance de 1 Essência de Inferium ao morrer (quase todos os mobs; não slime, magma, ghast,
  phantom, morcego, dragão). Wither: 35% de Alma Definhada. Dragão: 8 Escamas de Dragão.
- **Essência na terra arada** (clique direito): vira terra de essência daquele tier, com a umidade de antes. Numa
  terra de essência de outro tier, troca o tier, e 25% das vezes você recebe a essência antiga de volta.
- **Fertilizante Místico** num cultivo (deste mod ou trigo/cenoura/batata/beterraba): amadurece na hora.
- **Essência Fertilizante** num cultivo deste mod: cresce como farinha de osso (2 a 5 idades).
- Tier 6 não aceita fertilizante (como no Agradditions).
- Quebrar terra de essência: terra + 25% de uma essência do tier (Toque Suave pega a terra).

## Instalar (Windows)

1. Feche o launcher e rode, nesta pasta: `powershell -ExecutionPolicy Bypass -File .\instalar.ps1`.
   Ele confere o SHA-256, faz backup de `horizon-mods.txt` em `mod-backups\`, **move o nativo 0.1.0 para o backup**,
   copia os dois `.hmod` para `horizon-mods\` e o `.hnmod` para `horizon-native-mods\` e liga os três. Para conferir:
   `-Verificar`. Para também ter as sementes na bancada: `-ReceitasDeBancada` (instala e liga o opcional).
2. No launcher: **Build** (Native mods ligado). Confie no código nativo se pedir.
3. Abra o jogo e confira em **Options → Mods** que os três aparecem ligados.

## Como testar no jogo (rápido)

O que já valia na 0.1.0 continua: semente planta o cultivo, colheita madura dá essência + semente, Essência na terra
arada vira terra de essência, Fertilizante Místico amadurece, ~1 em 5 zumbis solta Inferium.

## Roteiro de teste (0.2.0, para o orquestrador)

Mundo criativo plano, sem nada num raio de 4 blocos de `0 -60 0` (troque a altura pelo chão do seu mundo; abaixo,
`Y` é a altura do chão + 1). Todos os ids são exatos.

**A. Montagem e itens visíveis**
1. `/give @s mysticalagriculture:infusion_altar 1`, `/give @s mysticalagriculture:infusion_pedestal 8`,
   `/give @s mysticalagriculture:prosperity_seed_base 4`, `/give @s minecraft:iron_ingot 16`,
   `/give @s mysticalagriculture:tertium_essence 16`, `/give @s mysticalagriculture:wand 1`,
   `/give @s minecraft:redstone_block 1`, `/give @s minecraft:hopper 1`, `/give @s minecraft:chest 1`.
2. Coloque (com a mão, ou `/setblock`) o altar em `0 Y 0` e os 8 pedestais em `3 Y 0`, `0 Y 3`, `-3 Y 0`, `0 Y -3`,
   `2 Y 2`, `2 Y -2`, `-2 Y 2`, `-2 Y -2`. Esperado: os blocos têm o modelo do original (não cubos inteiros) e o
   contorno de seleção não é um cubo cheio.
3. Com os lingotes de ferro na mão, clique direito nos pedestais `3 Y 0`, `0 Y -3`, `2 Y 2`, `-2 Y -2`; com Essência
   de Tertium, nos outros 4. Esperado: a mão perde **1 por clique** (sobram 12 lingotes e 12 essências) e cada pedestal
   mostra o item **flutuando e girando** em cima. Clique direito de novo num deles com o ferro na mão: o item volta
   para o inventário (13 lingotes) e o pedestal fica vazio; clique outra vez para pôr de volta (12).
4. Mão vazia no altar (vazio): action bar `Altar de Infusão: 8 pedestais (8 com item) · altar vazio`.
5. Com a Base de Semente na mão, clique no altar: sobra 1 na mão, a base aparece flutuando sobre o altar e a action
   bar diz `Altar de Infusão: 8 pedestais (8 com item) · receita: Sementes de Ferro · ligue com redstone`.

**B. Craft com redstone (100 ticks)**
6. Coloque o bloco de redstone em `1 Y 0` (encostado no altar, não é lugar de pedestal). Esperado: depois de
   **100 ticks** (5 s; `/tick freeze` e `/tick step 100` para contar exato) os 8 pedestais ficam **vazios**, a base
   some do altar e aparece **1 x `mysticalagriculture:iron_seeds`** flutuando sobre ele.
7. Conservação: antes havia 4 lingotes + 4 essências nos pedestais e 1 base no altar; depois, 0 + 0 + 0 e 1 semente.
   Mão vazia no altar: `iron_seeds` x1 vai para a mão e o altar fica vazio. No inventário: 12 lingotes, 12 essências,
   3 bases.
8. Tire o bloco de redstone.

**C. Varinha, funil e receita errada**
9. Ponha de novo 4 ferro + 4 tertium nos pedestais e a base no altar (sem redstone: depois de 200 ticks nada muda).
   Clique no altar com a **Varinha**: action bar `Altar de Infusão ligado.`; 100 ticks depois sai outra
   `iron_seeds` x1. (Com o altar vazio a varinha entra no altar, como no original: clique de novo para tirá-la.)
10. Funil em `0 Y-1 0` (embaixo do altar; cave o chão) com um baú embaixo dele, e repita o passo 9: a semente desce
    para o baú e o altar fica vazio.
11. Receita errada: 3 ferro + 4 tertium (7 itens, um pedestal vazio) e a base: a action bar ao pôr a base diz
    `Altar de Infusão: 8 pedestais (7 com item) · sem receita`; com a varinha ou redstone, depois de 200 ticks **nada**
    muda (os 7 itens e a base continuam lá).
12. Quebre um pedestal com item (sobrevivência, picareta): caem o pedestal e o item. Quebrar o altar solta o altar e
    a base.

**D. Sementes fora da bancada**
13. Na bancada, `MEM/ESE/MEM` com ferro/tertium/base: **sem resultado** (a receita de bancada saiu). Com o opcional
    `mysticalagriculture_seedcrafting` ligado, o resultado é `iron_seeds` x1.

**E. Regador**
14. De dia (o trigo precisa de luz). `/gamerule randomTickSpeed 0` (só o regador faz crescer). Faça terra arada
    **seca** (umidade 0, longe de água) num
    quadrado 11x11 em volta de `10 Y-1 10` e plante trigo em tudo (`minecraft:wheat_seeds`). Plante também 1 trigo
    de controle longe (`30 Y 30`).
15. `/give @s mysticalagriculture:watering_can 1`. Clique direito na terra em `10 Y-1 10` uma vez. Esperado: a terra
    3x3 em volta (9 blocos) fica `moisture=7`; a de fora continua `moisture=0`.
16. `/give @s mysticalagriculture:supremium_watering_can 1`. Clique direito no centro **40 vezes** (espaçadas de pelo
    menos 5 ticks; segurar o botão pode repetir o clique sozinho). Esperado: as 121 terras com `moisture=7`, e o trigo da área com idade média
    **acima de 0** (vários com `age` 1 a 7), enquanto o trigo de controle continua `age=0`.
17. `/gamerule randomTickSpeed 3` no fim.

**F. Mundo da 0.1.0**
18. Abra um mundo salvo com a 0.1.0: cultivos, terras de essência e itens continuam lá, com os mesmos ids.

## Limitações desta versão (0.2.0)

- **Altar**: sem as partículas (itens indo para o altar, fumaça, "happy villager") e sem a prévia fantasma dos
  pedestais que faltam; sem saída de comparador. Funil só **tira** o resultado do altar; não põe itens no altar nem
  nos pedestais (no original o funil põe 1 no altar; aqui ele encheria o slot com uma pilha). Item que não cabe no
  inventário fica no bloco (o original joga no chão aos seus pés). O contorno do altar é uma versão de 4 caixas das
  21 do original (o Horizon aceita até 16).
- **Regadores**: não precisam ser enchidos na água (o original começa vazio e enche clicando na água) e o modo
  "ativo" dos regadores de essência (agachar + clique no ar, regar sozinho) não existe; sem partículas de chuva e sem
  som. Regar é pelo clique no bloco (segurar repete), no máximo a cada 5 ticks. Dispensador/fake player não regam.
  Sem o Regador de Supremium Desperto (precisa de Awakened Supremium).
- A semente de Inferium continua de bancada (como no original). As receitas de altar dos augments e das adagas de
  Soulium ficam fora (os itens não existem no port).
- Sem sementes de mob (precisam de Soul Jars/Soulium), elementais (ar, terra, água, fogo), Soulstone/Soulium,
  máquinas (Harvester, Reprocessor etc.), ferramentas, armaduras, augments, Awakened Supremium,
  carvões/maçãs/paxels do Agradditions, e as sementes de mods não portados (Botania, Thermal, AE2...).
  As receitas que pedem essas coisas ficam fora (lista em `source/content/content-report.txt`).
- Terra de essência **pisoteada ou seca sem cultivo** vira terra comum (regra do template farmland do
  Horizon; o original não volta a terra). Não há gancho para isso em 66f3981. Com cultivo em cima ela não seca.
- O bônus da terra só vale quando quem colhe está na mesma dimensão do cultivo (o caso normal).
- Fertilizantes por **dispensador** num cultivo deste mod não funcionam (o dispensador usa o caminho da farinha
  de osso, que o mod recusa). Os fertilizantes são cópias da farinha de osso: em plantas vanilla que não são
  cultivos (mudas, grama) eles podem agir como farinha de osso ou não fazer nada.
- A essência devolvida ao trocar o tier da terra vai para o inventário (o original solta no chão).
- Dragão: as escamas dependem do Horizon passar a morte do dragão por `native_loot_drops` (não testado).
- As texturas das terras de essência foram geradas a partir dos blocos de cada tier (o original colore a terra
  vanilla em código, e o Horizon não tem gancho de cor de bloco).
- Mod nativo fixado em 66f3981. Ver `APIS.md` para os riscos de compilação (a 0.2.0 usa funções privadas do jogo para
  o random tick do regador e um gancho sem `self` para os itens desenhados).

## Inferium Lab

O "Inferium Lab" (pehe1) já existe no registro: só a semente de Inferium, com ids `inferium_lab:`. Este port
não depende dele e não colide (namespaces diferentes); usar os dois juntos só duplica a semente de Inferium.
Este port reaproveita a técnica que ele validou em 66f3981 para ligar a semente ao cultivo (dois pontos do
carregador de conteúdo), generalizada para as 31 sementes.

## Desenvolvimento

```sh
# conteúdo (dos jars do ATM10 descompactados)
python3 source/scripts/make_content.py <MA_jar_dir> <MAA_jar_dir> source/content source/optional
python3 -I source/scripts/upgrade_hmod.py --dir source/content/mysticalagriculture   # sempre depois do make_content
python3 -I source/scripts/upgrade_hmod.py --dir source/content/mysticalagradditions
python3 source/scripts/check_assets.py source/content
# nativo: testes da lógica e checagem de tipos contra as assinaturas de 66f3981
cd source/native && cargo test --offline --lib
cd tools/stubcheck && cargo test --offline
# (lógica: 22 testes; stubcheck: 26 = os 22 + 4 do glue/helpers contra as assinaturas de 66f3981)
# empacotar (gera .hmod, opcional/*.hmod, .hnmod e SHA256SUMS.txt)
python3 source/scripts/package.py source .
```

## Licença

MIT (a mesma dos originais, © 2018 BlakeBr0; texto em `LICENSE-original.txt` dentro de cada `.hmod` e em
`NOTICE` no `.hnmod`). Texturas, modelos, nomes e receitas vêm dos jars originais; o código nativo foi escrito
para este port. Pode ser publicado, mantendo o aviso de licença.

## Correções depois do teste no jogo (11/11/2026)
- O conteúdo não carregava: `infusion_crystal.json` usava o componente `minecraft:max_stack_size` (o Horizon pede o campo `max_stack_size`).
- Cultivo maduro não dava essência: as tabelas de loot estavam no formato 1.21.1. Convertidas para o 26.3 com `source/scripts/upgrade_hmod.py`. Rode esse script depois de `make_content.py` sempre que regenerar o conteúdo.
- Minérios de Inferium e Prosperidade não eram gerados (formato antigo de worldgen). Corrigido.
- Testado: semente planta o cultivo certo, colheita madura dá essência + semente.

## Mudanças da 0.2.0 (16/11/2026)
- Altar de Infusão, Pedestal de Infusão, Varinha e 6 regadores (blocos, itens, modelos, texturas, receitas e nomes do jar).
- Receitas de semente: as 30 de altar do original (`recipe/seed/infusion/`); as 30 de bancada da 0.1.0 foram para o
  opcional `mysticalagriculture_seedcrafting`.
- Nativo 0.2.0 (mesmo id `mysticalagriculture_native`): block entities do altar e dos pedestais, tick do altar
  (redstone, 100 ticks), itens desenhados, cliques, regador. O resto é o mesmo código da 0.1.0.
- Versão do conteúdo `8.0.28-horizon.2` (o `mysticalagradditions.hmod` não mudou).


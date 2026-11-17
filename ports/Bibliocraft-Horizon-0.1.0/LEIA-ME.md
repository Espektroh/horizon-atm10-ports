# Bibliocraft Legacy para Horizon 66f3981 — port 0.1.0

Os móveis de guardar e expor mais usados do Bibliocraft Legacy 1.6.6 (o do ATM10), nas 11 madeiras do original.

| Arquivo | O que é |
|---|---|
| `bibliocraft.hmod` | conteúdo: 45 blocos, modelos, texturas, telas (tipos de menu), receitas, loot, nomes (en_us, pt_br) |
| `bibliocraft_native-0.1.0.hnmod` | nativo: block entities, telas, cliques, funis, desenho dos livros e itens |

Os dois são necessários. Sem o nativo os blocos existem, mas não guardam nada.

## O que tem
Madeiras: carvalho, abeto, bétula, selva, acácia, carvalho escuro, carmesim, distorcida, mangue, bambu e
cerejeira (`oak spruce birch jungle acacia dark_oak crimson warped mangrove bamboo cherry`). Ids iguais aos do
original, então os nomes, receitas e tags batem com o ATM10.

| Bloco | Id | Casas | Tela | Como usa |
|---|---|---|---|---|
| Estante | `bibliocraft:<madeira>_bookcase` | 16 (1 livro por casa) | sim, 2 fileiras de 8 | clique direito abre a tela; cada casa cheia mostra um livro no bloco |
| Prateleira | `bibliocraft:<madeira>_shelf` | 4 (pilha inteira) | sim, 2x2 | clique direito **na frente**, em cima de uma das 4 casas: troca a mão com a casa; nas laterais/topo (ou entre as casas): abre a tela |
| Mesa | `bibliocraft:<madeira>_table` | 1 (1 item) | não (o original também não tem) | clique direito com um item: põe 1 em cima; com a mão vazia: pega de volta |
| Assento | `bibliocraft:red_<madeira>_seat` | — | — | só bloco (sem sentar nesta versão) |
| Pedestal de espada | `bibliocraft:sword_pedestal` | 1 (espada) | não | clique direito com espada: coloca; com a mão vazia: pega |

- **Livros aceitos na estante** (`#bibliocraft:bookcase_books` → `#minecraft:bookshelf_books`): livro, livro e pena,
  livro escrito, livro encantado, livro de conhecimento. Qualquer outro item é recusado na tela e pelos funis.
- **Pedestal**: aceita `#minecraft:swords` (e, se as tags não puderem ser lidas, ids terminados em `_sword`, que
  cobre as espadas do Allthemodium). Mostra a espada inclinada a 135°, como o `SwordPedestalBER`. Cor verde (a padrão
  do original).
- **Mesa**: com o mesmo item que já está em cima, nada muda (como o original). Trocando por outro item, o que
  estava volta para a mão (se ela ficou vazia) ou para o inventário; inventário cheio: nada muda, aviso na barra.
- **Prateleira**: as 4 casas são as do `ShelfBlock.lookingAtSlot` (de quem olha: 0 em cima à esquerda, 1 em cima
  à direita, 2 embaixo à esquerda, 3 embaixo à direita), itens desenhados como no `ShelfBER`.
- **Agachado com algo na mão**: o item age (coloca o bloco), como no vanilla/original.
- **Funis**: estante (só livros, 1 por casa), prateleira (qualquer item) e pedestal (só espada) aceitam funis por
  todas as faces, e um funil embaixo tira deles. A mesa não.
- **Quebrar** solta o bloco e tudo o que ele guardava (madeira: machado; pedestal: picareta).

## Receitas (as do código do original, formato 26.3; P = tábuas, S = laje da madeira)
- Estante: `PSP / PSP / PSP` → 1. Prateleira: `SSS / " P " / SSS` → 1. Mesa: `SSS / " P " / " P "` → 1.
- Assento vermelho: `" W " / " S " / RSR` com W lã vermelha, R graveto (no original `#c:rods/wooden`) → 1.
- Pedestal de espada: `" S " / SWS` com S laje de pedra lisa e W lã verde → 1.

## Roteiro de teste (para executar no jogo)
Mundo plano, criativo, com cheats. `X Y Z` = um bloco de chão livre (o `Y` é a altura em cima do chão). O jogador
fica em `X Y Z+3`, olhando para o norte (yaw 180, pitch 0) para os blocos com `facing=south`. Antes:
`/clear @s`. Os números são exatos.

1. **Mods**: em Options → Mods aparece `Bibliocraft Legacy (Horizon) 0.1.0`. Na aba criativa `Bibliocraft` há
   **45** itens (11 estantes, 11 prateleiras, 11 mesas, 11 assentos, 1 pedestal).
2. **Estante, tela**: `/setblock X Y Z bibliocraft:oak_bookcase[facing=south]`. Esperado: estante de carvalho,
   aberta para você, **0** livros. Clique direito: abre uma tela com **16** casas (2 fileiras de 8) e o inventário embaixo.
3. **Só livros, 1 por casa**: `/give @s minecraft:book 20` e `/give @s minecraft:diamond 5`. Na tela, shift-clique
   na pilha de livros. Esperado: as 16 casas com **1** livro cada e **4** livros no inventário. Tente pôr um
   diamante numa casa: não entra (fica no cursor). Feche: a estante mostra **16** livros.
4. **Tirar**: abra de novo e tire os livros das 3 primeiras casas (fileira de cima, à esquerda). Esperado: **7**
   livros no inventário; fechada, a estante mostra **13** livros.
5. **Funil na estante**: `/setblock X+2 Y Z bibliocraft:spruce_bookcase[facing=south]`,
   `/setblock X+2 Y+1 Z minecraft:hopper[facing=down]`; ponha **5** livros e **3** diamantes no funil. Esperado: após
   **100 ticks** (5 s) a estante de abeto mostra **5** livros e o funil ficou só com os **3** diamantes.
6. **Prateleira, frente**: `/setblock X+4 Y Z bibliocraft:oak_shelf[facing=south]`; fique em `X+4 Y Z+3`.
   `/give @s minecraft:cobblestone 64`. Com o pedregulho na mão, mire na metade de cima à **esquerda** da frente e
   clique direito. Esperado: mão vazia; o pedregulho aparece em cima à esquerda. `/give @s minecraft:apple 3`, segure as maçãs e
   mire em cima à **direita**: a maçã aparece em cima à direita. Mão vazia, clique em cima à esquerda:
   **64** pedregulhos voltam para a mão.
7. **Prateleira, tela**: clique direito na face de **cima** (ou numa lateral) da prateleira. Esperado: tela com
   **4** casas em 2x2; a de cima à direita tem **3** maçãs.
8. **Pedestal**: `/setblock X+6 Y Z bibliocraft:sword_pedestal[facing=south]`, `/give @s minecraft:diamond_sword`.
   Clique direito com a espada: mão vazia, espada inclinada no pedestal. Com `/give @s minecraft:stick` na mão,
   clique direito: o graveto fica na mão e a espada continua lá. Mão vazia, clique direito: a espada volta.
9. **Mesa**: `/setblock X+8 Y Z bibliocraft:oak_table`. `/give @s minecraft:apple 10` (sem outras maçãs). Clique
   direito no tampo com as maçãs: **9** na mão, **1** maçã deitada no tampo. Clique de novo com as maçãs: nada
   muda (9). `/give @s minecraft:diamond 5`, com os diamantes na mão: **4** na mão, diamante no tampo e as maçãs
   do inventário somam **10**. Mão vazia: o diamante volta (**5**).
10. **Assento**: `/setblock X+10 Y Z bibliocraft:red_oak_seat`: banco de carvalho com almofada vermelha; clique
    direito não faz nada (sem sentar).
11. **Quebrar**: no sobrevivência (`/gamemode survival`), quebre a estante do passo 4 com um machado. Esperado: caem
    **1** `bibliocraft:oak_bookcase` e **13** livros.
12. **Receita**: numa mesa de trabalho, `PSP/PSP/PSP` com tábuas e lajes de carvalho dá **1** estante de carvalho;
    `" S "/"SWS"` com laje de pedra lisa e lã verde dá **1** pedestal.
13. **Salvar e reabrir o mundo**: a estante de abeto continua com **5** livros, a prateleira com as maçãs, o pedestal
    sem espada e a mesa vazia, tudo como antes de sair.

## Limitações (0.1.0)
- **Assentos: não dá para sentar.** O original cria uma entidade invisível e monta o jogador nela; o Horizon só deixa
  montar em barco, carrinho, almofada (cushion) e montarias, e não há API para um mod criar um assento próprio.
  Só a cor vermelha (o original tem 16 cores × 11 madeiras) e sem os encostos (`*_seat_back`).
- **Mesa sem emendas**: o original junta mesas vizinhas (tampo contínuo, `type` = one/straight/curve/three/all);
  aqui cada mesa é a de pé central (`none`). Sem toalha (o pano de carpete) e sem mapa desenhado no tampo (o item
  `filled_map` aparece como item).
- **Pedestal**: só verde (no original, cada cor de lã dá uma cor); não conserta espadas com Mending usando orbes de
  experiência (o `tick` do original) e não dá sinal de comparador.
- **Estante**: sem bônus de encantamento (+0,125 por livro), sem sinal de comparador do Livro de Redstone, sem os
  livros "pride" (arco-íris/trans) de junho.
- Telas: painel simples do Horizon (não a textura do original), com as casas nas mesmas posições.
- Fora desta versão: expositor de espadas/ferramentas (tool rack), prateleira de poções, vitrine (display case),
  armário de discos, armadura, relógios, placas, luminárias, mesa de impressão, máquina de escrever, livros
  especiais, prancheta, fita métrica e o resto do mod. Madeira pálida (pale oak) não existe no original 1.21.1.
- A casa da prateleira é calculada pelo olhar do jogador (posição e direção), não pelo ponto exato do clique; na
  divisa entre duas casas pode cair na vizinha.

## Instalar
1. Feche o launcher e rode nesta pasta: `powershell -ExecutionPolicy Bypass -File .\instalar.ps1`
   (confere o SHA-256, faz backup do `horizon-mods.txt`, copia o `.hmod` para `horizon-mods\` e o `.hnmod` para
   `horizon-native-mods\`, liga os dois). `-GameDir "C:\..."` escolhe a pasta; `-Verificar` só confere.
2. No launcher: **Build** na página Mods (Native mods ligado). Todos os mods nativos compilam juntos: se este quebrar
   o build, tire o `bibliocraft_native-0.1.0.hnmod` de `horizon-native-mods\` e me mande o `build.log`.
3. Abra o jogo e confira em **Options → Mods**.

## Detalhes técnicos
- Fixado no Horizon `66f3981` e no horizon-mixins `5a64af9`. **CI verde nas duas bases** (66f3981 e Nightly
  2ebf00a), compilando junto com todos os outros mods: https://github.com/Espektroh/horizon-atm10-privado/actions/runs/44086978237 (commit b358f9b, branch `wip/bibliocraft`).
  `APIS.md` lista cada API usada com arquivo:linha e os riscos.
- Estante e prateleira usam o modelo `minecraft:furnace` (é o que abre uma tela pelo uso, como nas máquinas do
  Powah) com tipos de menu do próprio mod (`data/bibliocraft/horizon/menu/bookcase.json` 16 casas, `shelf.json` 4);
  a mesa e o assento usam as tábuas da madeira; o pedestal, a terracota vitrificada (só pela direção). Todos com
  `occludes: false` e as formas (outline/colisão) do código do original.
- Os livros da estante são os 16 modelos `bibliocraft:block/template/bookcase/book_N` do original, assados como
  modelos avulsos (`native_standalone_models`) e desenhados pelo block entity para cada casa cheia.
- Salvo no block entity (id = id do bloco): `Items` (as casas) e `BcShown` (só um cache). Aos clientes vai
  `BcView` (lista `id`/`components` por casa).
- `source/content/bibliocraft/`: gerado por
  `python3 -I source/scripts/make_content.py <clone do Bibliocraft-Legacy> source/content/bibliocraft`, convertido
  por `python3 -I source/scripts/upgrade_hmod.py --dir source/content/bibliocraft` (45 loot tables) e conferido por
  `source/scripts/check_content.py` (0 erros). A faixa do pedestal é pintada de verde (`0x5E7C16`) pelo script.
- `source/native/`: `cargo test --offline` roda **17 testes** da lógica pura (blocos, casas e telas, livros/espadas,
  funis, giros dos desenhos contra a direção do bloco, mira nas casas da prateleira nas 4 direções, desenho = clique,
  cliques de prateleira/pedestal/mesa, troca, hash do que os clientes viram, modelos dos livros).
- `source/scripts/package.py .` monta o `.hmod`, o `.hnmod` e o `SHA256SUMS.txt` (zips determinísticos).

## Licença
Bibliocraft Legacy © MinecraftschurliMods (IchHabeHunger54, Minecraftschurli), a partir do BiblioCraft de Nuchaz.
O repositório (`MinecraftschurliMods/Bibliocraft-Legacy`, branch `version/1.21.1`, versão 1.6.6) tem uma única
licença **MIT** para tudo (código e recursos; `gradle.properties` diz `license.name=MIT`), em
`source/LICENSE-BibliocraftLegacy.txt`. O código foi consultado para números e regras e reescrito em Rust; modelos,
texturas e nomes vêm de `src/main/resources` e `src/main/generated` do repositório. As texturas de madeira e lã são
as do próprio Minecraft (referenciadas, não incluídas). Pode compartilhar, mantendo o aviso de licença. O código
deste port também é MIT.

# Functional Storage para Horizon 66f3981 — port 0.1.0

As gavetas do Functional Storage 1.5.8 (o do ATM10), com capacidade, upgrades e funis iguais ao original.

| Arquivo | O que é |
|---|---|
| `functionalstorage.hmod` | conteúdo: 30 gavetas, 6 upgrades, texturas, modelos, receitas, loot, nomes (en_us, pt_br) |
| `functionalstorage_native-0.1.0.hnmod` | nativo: block entity das gavetas, cliques, funis, desenho da frente |

Os dois são necessários. Sem o nativo, as gavetas existem mas não guardam nada.

## O que tem
- **Gavetas** 1x1, 1x2 e 2x2 das 10 madeiras do original: carvalho, abeto, bétula, selva, acácia, carvalho escuro,
  carmesim, distorcida, mangue e cerejeira (`functionalstorage:<madeira>_1`, `_2`, `_4`, os mesmos ids do original).
- **Capacidade por compartimento** (em pilhas, como no original: `DrawerType`):

  | Gaveta | Compartimentos | Pilhas por compartimento | Pedregulho (pilha de 64) | Pérola (16) | Espada (1) |
  |---|---|---|---|---|---|
  | 1x1 | 1 | 32 | 2.048 | 512 | 32 |
  | 1x2 | 2 | 16 | 1.024 cada | 256 | 16 |
  | 2x2 | 4 | 8 | 512 cada | 128 | 8 |

- **Upgrades de armazenamento** (até 4 por gaveta, os fatores se multiplicam, como no `SizeProvider` do original):
  Cobre ×8, Ouro ×16, Diamante ×24, Netherita ×32. O **Downgrade de Ferro** fixa a base em 1 pilha por compartimento
  (útil para "filtro"). Limite: 2.147.483.647 itens por compartimento (int do Java, como o original).
  Exemplos: 1x1 + Cobre = 16.384 pedregulhos; 1x1 + 2 Netherita = 2.097.152.
- **Upgrade de Vazio**: com ele, o que passar da capacidade de um item que a gaveta já guarda some (mão, duplo clique
  e funis). Itens diferentes continuam recusados.
- **Funis e automação**: todas as faces aceitam funil/conta-gotas. Um funil em cima (ou do lado) enche a gaveta;
  um funil embaixo tira dela. Um item vai para o compartimento que já tem aquele item; um compartimento vazio só
  recebe um item que nenhum outro compartimento tem.
- **Frente**: mostra o item de cada compartimento e a quantidade (1.2K, 16.4K, 2.1B...), como o original.

## Como usar (interação do original, `ItemControllableDrawerTile`)
- **Clique direito na frente** com uma pilha na mão: guarda a pilha no compartimento para onde você está olhando.
- **Clique direito duplo** (dois cliques em menos de 300 ms, 6 ticks) na frente: guarda **todos** os itens daquele
  compartimento que estão no inventário (as 36 casas).
- **Clique esquerdo na frente**: tira **1 item**; **agachado (Shift) + clique esquerdo**: tira **uma pilha**.
  Atenção: é assim no código do original 1.5.8 (`onClicked`: `isShiftKeyDown ? maxStackSize : 1`), o contrário do
  pedido ("esquerdo = 1 pilha, shift = 1 item", que é o comportamento do Storage Drawers). Para inverter, troque
  `SNEAK_TAKES_STACK` em `runtime.rs` para `false`.
  Se o inventário estiver cheio, o que não coube **fica na gaveta** (o original jogaria no chão).
- **Clique direito com um upgrade na mão** (em qualquer face): instala. Com 4 upgrades, um mais forte troca o mais
  fraco e o antigo volta para o inventário. Um downgrade de ferro não entra se a gaveta já tem mais de 1 pilha.
- **Agachado + clique direito com as duas mãos vazias**: tira um upgrade (o último de armazenamento cuja retirada
  ainda caiba no conteúdo; depois o de vazio). No original isso é feito na tela da gaveta, que não foi portada.
- **Clique direito nas laterais** (ou na frente sem nada para guardar): mostra o conteúdo na barra de ação, por
  exemplo `Gaveta 1x2: [1] 1.2K / 1K Pedregulho | [2] vazia (0 / 1K) · Cobre x8`.
- **Agachado com algo na mão**: coloca o bloco normalmente (como no vanilla).
- A frente não é quebrada pelo clique esquerdo; quebre a gaveta pelas laterais (machado é a ferramenta certa).

## Receitas (as do jar, sem as tags `c:` que o 26.3 não tem)
- Gaveta 1x1: 8 tábuas da madeira em volta de um baú (`PPP/PCP/PPP`) → 1.
- Gaveta 1x2: `PCP/PPP/PCP` → 2. Gaveta 2x2: `CPC/PPP/CPC` → 4. (Baú = baú ou baú com armadilha.)
  A de carvalho usa tábuas de carvalho (no original, também tábuas de outros mods sem gaveta própria).
- Upgrade de Cobre: `IBI/CDC/IBI` (I lingote de cobre, B bloco de cobre, C baú, D qualquer gaveta).
- Upgrade de Ouro: `IBI/CDC/BIB` (lingote e bloco de ouro, D = Upgrade de Cobre).
- Upgrade de Diamante: `IBI/CDC/IBI` (diamante e bloco de diamante, D = Upgrade de Ouro).
- Upgrade de Netherita: mesa de ferraria: modelo de melhoria de netherita + Upgrade de Diamante + lingote de netherita.
- Downgrade de Ferro: 8 lingotes de ferro em volta de uma gaveta. Upgrade de Vazio: 8 obsidianas em volta de uma gaveta.

## Roteiro de teste (para executar no jogo)
Mundo plano, criativo ou sobrevivência, com cheats. Use as coordenadas `X Y Z` de um bloco de chão livre; o jogador
fica em `X Z+3` olhando para o norte (para a gaveta). Os números são exatos.

1. **Colocar**: `/setblock X Y Z functionalstorage:oak_1[facing=south]` (frente para o sul, para você).
   Esperado: bloco de gaveta de carvalho, frente vazia.
2. **Guardar a mão**: `/give @s minecraft:cobblestone 64`, segure, mire no centro da frente e clique direito.
   Esperado: a mão fica vazia; a frente mostra pedregulho e `64`.
3. **Duplo clique**: `/give @s minecraft:cobblestone 192` (3 pilhas). Com a mão **vazia**, dois cliques direitos
   rápidos na frente. Esperado: 0 pedregulho no inventário; clique direito na lateral leste mostra
   `Gaveta 1x1: 256 / 2K Pedregulho`.
4. **Tirar**: clique esquerdo na frente → +1 pedregulho no inventário (gaveta 255).
   Agachado + clique esquerdo → +64 (gaveta 191).
5. **Capacidade**: `/clear @s minecraft:cobblestone`, `/give @s minecraft:cobblestone 2112` e duplo clique com a
   mão vazia. Esperado: a gaveta fica com **2048** (barra de ação `2K / 2K`) e sobram **255** no inventário
   (191 + 2112 − 2048).
6. **Upgrade**: `/give @s functionalstorage:copper_upgrade`, clique direito com ele. Esperado: barra
   `... 2K / 16.4K Pedregulho · Cobre x8`; outro duplo clique guarda os 255 (gaveta 2303).
7. **Funil enchendo**: `/setblock X+2 Y Z functionalstorage:oak_4[facing=south]` e
   `/setblock X+2 Y+1 Z minecraft:hopper[facing=down]`; coloque 10 terra no funil. Esperado: após 100 ticks o funil
   está vazio e a 2x2 mostra 10 terra no compartimento [1] (embaixo à direita); barra
   `Gaveta 2x2: [1] 10 / 512 Terra | [2] vazia (0 / 512) | ...`.
8. **Funil tirando**: com a 1x1 do passo 6 (2303), `/setblock X Y-1 Z minecraft:hopper[facing=east]` e
   `/setblock X+1 Y-1 Z minecraft:chest`. Esperado: após 400 ticks o funil puxou **50** (1 a cada 8 ticks;
   baú + funil = 50) e a gaveta mostra **2253**. Gaveta + baú + funil = 2303 sempre (conservação).
9. **2x2 por compartimento**: de frente para a 2x2 (em `X+2 Z+3`), segure 5 areia e clique direito no quarto
   **de cima à esquerda**: depois, na lateral, a barra mostra `[4] 5 / 512 Areia`; 5 diamantes no quarto
   **de baixo à esquerda**: `[2] 5 / 512 Diamante`.
10. **Vazio**: numa 1x1 cheia (2048 pedregulhos) com o Upgrade de Vazio, guarde mais 64: a mão esvazia e a gaveta
    continua 2048. Sem o upgrade, os 64 ficariam na mão.
11. **Quebrar**: quebrar (pela lateral) uma gaveta com itens não quebra e avisa
    `Esvazie a gaveta antes de quebrar (N itens dentro).` Esvaziada, quebra, solta a gaveta e devolve os upgrades
    ao inventário. Tirar um upgrade: agachado + clique direito com as mãos vazias (o cobre volta ao inventário).
12. **Salvar e reabrir o mundo**: as quantidades dos passos 6 e 8 continuam as mesmas, e a frente volta a mostrar.

## Formato dos dados (estável; para mundos futuros)
Cada gaveta tem um block entity com o id do próprio bloco (`functionalstorage:oak_1`...). O que fica salvo:
- **`Items`** (salvo pelo jogo): 3 casas por compartimento `i`: `3i` a pilha que os funis tiram (até uma pilha),
  `3i+1` a entrada dos funis (vazia depois de cada tick), `3i+2` **um** item guardado quando há mais de uma pilha
  (mantém o item exato, com componentes, mesmo que um cano esvazie a casa `3i`). Esta casa não tem face.
- **`Amounts`** (LongArray): por compartimento, quantos itens há **além** dessas casas.
  Total do compartimento `i` = `Amounts[i]` + `Items[3i]` + `Items[3i+1]` (se for o mesmo item) + `Items[3i+2]`.
  O item do compartimento é o da primeira casa não vazia entre `3i+2`, `3i`, `3i+1`.
- **`Upgrades`** (lista de ids, na ordem de instalação, até 4), **`Void`** (byte 0/1), **`FsFormat`** (int, 1).
- `Shown` (long) é só um cache do que os clientes viram; pode ser apagado.
Versões futuras leem esse formato (chaves que faltam valem vazio; chaves desconhecidas são mantidas).
Aos clientes vai `View` (lista: `id`, `components`, `count` por compartimento) e `Void`.

## Limitações (0.1.0)
- **Gaveta com itens não quebra** (nem por explosão): no original ela cai com o conteúdo dentro do item. Esvazie
  antes. No criativo, `/setblock ... air` remove (e perde o conteúdo).
- **Sem Storage Controller, Drawer Controller e Controller Extension** (ficam para a 0.2.0: precisam de um
  inventário que junta várias gavetas e da ferramenta de ligação). Também fora: gavetas compactadoras, de fluido,
  emolduradas (framed), do ender, Armory Cabinet, ferramentas de configuração/ligação, trava (lock), upgrades de
  coletor/puxar/empurrar/redstone/gotejamento/geradores, Max Storage e Creative Vending.
- Só as 4 direções horizontais (o original 1.21 também vira para cima/baixo).
- Sem sinal de comparador próprio (o comparador lê as 3 casas do container, não a porcentagem da gaveta) e sem
  Jade/TOP; o conteúdo aparece na barra de ação.
- O compartimento clicado é calculado pelo olhar do jogador (posição e direção), não pelo ponto exato do clique;
  na prática é o mesmo, a não ser exatamente na divisa entre dois compartimentos.
- A frente é desenhada a cada quadro (sem cache), porque a luz vem do bloco da frente. Centenas de gavetas na tela
  podem custar alguns FPS.
- A tag `drawer_storage_denylist` do original está vazia e não foi portada.

## Instalar
1. Feche o launcher e rode nesta pasta: `powershell -ExecutionPolicy Bypass -File .\instalar.ps1`
   (confere o SHA-256, faz backup do `horizon-mods.txt`, copia o `.hmod` para `horizon-mods\` e o `.hnmod` para
   `horizon-native-mods\`, liga os dois). `-GameDir "C:\..."` escolhe a pasta; `-Verificar` só confere.
2. No launcher: **Build** na página Mods (Native mods ligado). Todos os mods nativos compilam juntos: se este quebrar
   o build, tire o `functionalstorage_native-0.1.0.hnmod` de `horizon-native-mods\` e me mande o `build.log`.
3. Abra o jogo e confira em **Options → Mods**.

## Detalhes técnicos
- Fixado no Horizon `66f3981` e no horizon-mixins `5a64af9`. `APIS.md` lista cada API usada com arquivo:linha e os
  riscos de compilação (o principal: a injeção em `native_block_entity_update_tag`, função sem `self`).
- `source/content/functionalstorage/`: gerado por
  `python3 -I source/scripts/make_content.py <jar extraído> source/content/functionalstorage` e convertido para o
  26.3 por `python3 -I source/scripts/upgrade_hmod.py --dir source/content/functionalstorage` (30 loot tables).
  `source/scripts/check_content.py` confere JSON, modelos, texturas, nomes e loot (0 erros).
- `source/native/`: `cargo test --offline` roda **21 testes** da lógica pura (capacidades, upgrades, inserir/tirar,
  funis, vazio, layout salvo, conservação em 60.000 operações aleatórias, mira nos compartimentos, números).
  `source/native/tools/glue_mockcheck/` confere os tipos do `glue.rs` e dos helpers contra imitações das
  assinaturas do Horizon.
- `source/scripts/package.py` monta o `.hmod`, o `.hnmod` e o `SHA256SUMS.txt` (zips determinísticos).

## Licença
Functional Storage © Buuz135 e Rid, **MIT** (`source/LICENSE-FunctionalStorage.txt`). O código foi consultado no
GitHub (Buuz135/FunctionalStorage, branch 1.21, versão 1.5.8) para números e regras e reescrito em Rust; texturas,
modelos e nomes vêm do jar. Pode compartilhar, mantendo o aviso de licença. O código deste port também é MIT.

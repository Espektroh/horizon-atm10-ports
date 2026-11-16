# Powah para Horizon 66f3981 (port 0.1.0)

Port do [Powah! (Rearchitected)](https://github.com/Technici4n/Powah) 6.2.10 (owmii, Technici4n, shartte; **LGPLv3**)
para o Horizon (Minecraft 26.3 em Rust). Foco no começo da progressão do ATM10: o **Orbe Energizante** (aço
energizado e cristais), os primeiros geradores, a célula e o cabo. Os ids são os do original (`powah:`).

## São dois mods, os dois são necessários

- **`powah.hmod`** (conteúdo, id `powah`, versão 6.2.10-horizon.1): blocos, itens, modelos, texturas, nomes
  (en_us e pt_br do jar), receitas, loot, tags, a aba criativa "Powah" e o menu do orbe.
- **`powah_native-0.1.0.hnmod`** (nativo, id `powah_native`): energia, geradores, orbe, hastes, telas.

Sem o nativo os blocos existem mas não fazem nada (o Furnator vira um bloco parado, o orbe não energiza).

## O que tem

| Bloco | O que faz | Números (configuração padrão do Powah) |
|---|---|---|
| Orbe Energizante (`powah:energizing_orb`) | Junta até 6 ingredientes e, quando as hastes o enchem com a energia da receita, faz o resultado | 7 slots: 1 saída + 6 entradas |
| Haste Energizante inicial / básica (`powah:energizing_rod_starter` / `_basic`) | Recebe energia **pela base** e enche o orbe mais próximo até 4 blocos em cada eixo | guarda 10.000 / 40.000 FE; recebe e passa 100 / 400 FE/t; aquece 20 ticks antes de começar |
| Furnator inicial / básico (`powah:furnator_starter` / `_basic`) | Queima combustível de fornalha: 30 FE por tick de queima (carvão = 48.000 FE) | gera 20 / 80 FE/t; guarda 20.000 / 80.000 FE; manda até 80 / 320 FE/t pelos 6 lados |
| Magmator inicial / básico (`powah:magmator_starter` / `_basic`) | Lava: 10.000 FE a cada 100 mB (um balde = 100.000 FE) | tanque 4000 mB; gera 20 / 80 FE/t; guarda 20.000 / 80.000 FE; manda até 80 / 320 FE/t |
| Célula de Energia inicial / básica (`powah:energy_cell_starter` / `_basic`) | Bateria: entra e sai por todos os lados | guarda 1.000.000 / 4.000.000 FE; 1.000 / 4.000 FE/t |
| Cabo de Energia inicial / básico (`powah:energy_cable_starter` / `_basic`) | Liga geradores, células e hastes | 500 / 2.000 FE/t |

Mais: Aço Energizado, os cristais Flamejante, Niótico, Vibrante (Spirited) e de Nitro, os blocos deles (9 ⇄ 1),
Pasta Dielétrica, Haste Dielétrica (e horizontal), Invólucro Dielétrico, Capacitor Básico e Capacitor Básico (Pequeno).

### Receitas do orbe (`powah:energizing`, do jar)

| Ingredientes (um por slot) | Resultado | Energia |
|---|---|---|
| lingote de ferro + lingote de ouro | 2 Aço Energizado | 10.000 FE |
| 4 pó de blaze **ou** 1 vara de blaze | 1 Cristal Flamejante | 120.000 FE |
| diamante | 1 Cristal Niótico | 300.000 FE |
| esmeralda | 1 Cristal Vibrante | 1.000.000 FE |
| estrela do Nether + 2 blocos de redstone + bloco de cristal flamejante | 16 Cristais de Nitro | 20.000.000 FE |

Regra do original: cada slot ocupado precisa casar com um ingrediente diferente e todos os ingredientes precisam
estar lá (um item a mais ou a menos e a receita não vale). Trocar a receita no meio zera a energia acumulada.

### Bancada

As 32 receitas de bancada do jar que só usam o que foi portado (a lista do que ficou fora está em
`source/content/content-report.txt`). As tags `c:` foram trocadas pelo item vanilla: `c:ingots/iron` → lingote de
ferro, `c:storage_blocks/quartz` → bloco de quartzo, `c:glass_blocks` → vidro etc.
O caminho do começo: Pasta Dielétrica (3 carvões + 2 argilas + balde de lava = 24; ou 2 carvões + argila + pó de
blaze = 16) → Haste Dielétrica (pasta + barras de ferro) → Invólucro Dielétrico → Capacitor Básico
(ferro + pasta + bloco de redstone = 4) → Furnator/Magmator/Célula/Haste iniciais e o Orbe (vidro + invólucro + hastes
dielétricas horizontais).

## Como usar

- **Orbe**: clique direito com um item na mão põe **1** dele no primeiro slot livre; com um resultado pronto, o clique
  direito (com qualquer item na mão) entrega o resultado. **Mão vazia abre a tela** do orbe (6 entradas à esquerda,
  saída à direita, barra de progresso e "x / y FE"). Funil por cima/lados enche as entradas, por baixo tira a saída.
  Atenção: com um item na mão o clique no orbe põe o item dentro (como no original); para pôr uma haste ao lado do orbe,
  clique num bloco vizinho, não no orbe.
- **Hastes**: a haste recebe energia só pelo bloco onde está apoiada (a base): ponha-a em cima de uma célula, de um
  gerador ou de um cabo. Ela acha sozinha o orbe mais perto em até 4 blocos (procura uma vez por segundo) e, com a
  receita montada, aquece 1 segundo e passa a encher o orbe. Várias hastes somam.
- **Furnator**: mão vazia abre a tela; o combustível vai no slot de combustível (shift-clique e funil também).
  Balde de lava devolve o balde.
- **Magmator**: clique direito com balde de lava enche 1000 mB e devolve o balde vazio; ou ponha baldes de lava no slot
  da tela (o balde vazio sai no slot da direita; funil por baixo tira).
- Geradores e células mandam energia para os 6 lados, direto ou por cabos. Uma célula não passa energia para outra
  célula (evita o vai-e-vem); gerador → células e hastes; célula → hastes.
- **Mekanism** (o seu Mekanism para Horizon): uma Célula de Energia encostada na **frente** (face de saída) de um Cubo de
  Energia do Mekanism, de qualquer tier, puxa energia dele: até a saída do cubo por tick, 2 FE a cada 5 J (a conversão
  padrão do Mekanism, 1 FE = 2,5 J). Só nesse sentido; os cabos dos dois mods não se ligam.

## Instalar (Windows)

1. Feche o launcher e rode, nesta pasta: `powershell -ExecutionPolicy Bypass -File .\instalar.ps1`.
   Ele confere o SHA-256, faz backup de `horizon-mods.txt` em `mod-backups\`, copia o `.hmod` para `horizon-mods\` e o
   `.hnmod` para `horizon-native-mods\` e liga os dois. Para conferir: `-Verificar`.
2. No launcher: **Build** (Native mods ligado). Confie no código nativo se pedir.
3. Abra o jogo e confira em **Options → Mods** que `powah` e `Powah 0.1.0` aparecem ligados.

## Roteiro de teste (para executar no jogo, mundo criativo plano novo)

Coordenadas relativas a um ponto `X Y Z` no chão (Y = altura do bloco em cima do chão). Os passos usam `/setblock`;
se um bloco posto assim não abrir a tela nem funcionar (block entity não criada), coloque-o com a mão no mesmo lugar
(para pôr a haste em cima do Furnator com a mão, agache: sem agachar o clique abre o Furnator). `/time set day` e `/gamerule doDaylightCycle false` ajudam a ler a tela.

**T1 — conteúdo carregou**
1. `/give @s powah:energizing_orb`, `/give @s powah:furnator_starter`, `/give @s powah:crystal_blazing`.
2. Esperado: os três itens aparecem com textura e nome ("Orbe Energizante", "Fornador (Inicial)", "Cristal
   Flamejante"); a aba criativa "Powah" existe. Se não: o log diz "content not loaded" e o motivo.

**T2 — Furnator → cabo → célula (números exatos)**
1. `/setblock X Y Z powah:furnator_starter[facing=north]`
2. `/setblock X+1 Y Z powah:energy_cable_starter`, `/setblock X+2 Y Z powah:energy_cable_starter`
3. `/setblock X+3 Y Z powah:energy_cell_starter`
4. Mão vazia, clique direito no Furnator, ponha **1 x minecraft:coal** no slot de combustível (o de baixo, x 56).
5. Esperado logo: a tela mostra "Gerando 20 FE/t" e a barra laranja cheia; o bloco fica `lit=true`.
6. Espere **200 ticks** (10 s) e abra a célula (mão vazia): "≈ 4.000 FE / 1.000,0 kFE" (20 FE/t chegam à célula;
   a tela arredonda para múltiplos de 34 FE, então aceite 3.940 a 4.060). O carvão some do slot (está queimando: 48.000 FE = 2400 ticks).
7. Quebre um cabo: a célula para de subir.

**T3 — Orbe: Aço Energizado**
1. `/setblock X Y Z+3 powah:furnator_basic[facing=north]`
2. `/setblock X Y+1 Z+3 powah:energizing_rod_basic[facing=up]` (haste em cima do Furnator: a base é o Furnator)
3. `/setblock X+2 Y Z+3 powah:energizing_orb`
4. Mão vazia, abra o Furnator e ponha **2 x minecraft:coal** no combustível.
5. `/give @s minecraft:iron_ingot`, `/give @s minecraft:gold_ingot`. Com o ferro na mão, clique direito no orbe; depois
   com o ouro. (Cada clique põe 1 item: ferro no slot 1, ouro no slot 2.)
6. Mão vazia, clique direito no orbe: a tela mostra "x / 10.000 FE" e a barra azul anda.
7. Esperado: em até **300 ticks** (15 s; 80 FE/t do Furnator básico, 20 ticks de aquecimento da haste) o slot de saída
   do orbe tem **2 x powah:steel_energized**, e os slots de ferro e ouro ficam vazios. A barra volta a "Sem receita".
8. Clique direito no orbe com qualquer item na mão: os 2 aços vão para o inventário.
9. Variante: ponha 1 x `minecraft:blaze_rod` → a tela mostra "0 FE / 120,0 kFE" e começa a subir; não precisa esperar
   terminar (1500 ticks a 80 FE/t).

**T4 — Magmator**
1. `/setblock X Y Z+6 powah:magmator_starter[facing=north]`
2. `/give @s minecraft:lava_bucket`, clique direito no Magmator com ele: a mão fica com **minecraft:bucket**.
3. Mão vazia, abra o Magmator: "Lava: 900/4000 mB" (1000 entraram, 100 já foram para o queimador) e "Gerando 20 FE/t".
4. Sem nada ligado, depois de **1000 ticks** ele enche (20.000 FE) e para: "Gerando 0 FE/t", `lit=false`.

**T5 — Célula alimentando uma haste**
1. Com o T2 montado (a célula já tem alguns milhares de FE): `/setblock X+3 Y+1 Z powah:energizing_rod_starter[facing=up]`
   (em cima da célula) e `/setblock X+3 Y Z-2 powah:energizing_orb`.
2. Ponha 1 x `minecraft:blaze_rod` nesse orbe (clique direito com ela na mão) e abra-o com a mão vazia.
3. Esperado: depois de ~1 s de aquecimento o "x / 120,0 kFE" sobe **100 FE por tick** (2.000 por segundo, o limite
   da haste inicial) enquanto a célula tiver energia; a energia da célula desce.

**T6 (opcional) — Mekanism**: um `mekanism:basic_energy_cube[facing=east]` com energia (de um Gerador a Calor do
Mekanism) e uma `powah:energy_cell_starter` encostada a leste dele: a célula sobe até 1.000 FE/t e o cubo desce 2,5 J
por FE.

## Limitações desta versão (0.1.0)

- Só os tiers **inicial e básico**; endurecido, flamejante, niótico, vibrante e nitro ficam para depois (os cristais e
  blocos de cristal existem, mas não as máquinas desses tiers). Fora também: Painel Solar, Gerador Termoelétrico,
  Reator, Ender Cell/Gate, Funil de Energia, Descarregador, Transmissor do Jogador, baterias, chave inglesa, cartões,
  urânio/uraninita (sem minério), gelo seco, bola de neve carregada, manual.
- Hastes sem chave inglesa: ligam-se sozinhas ao orbe **mais perto** (o original liga ao primeiro achado e permite
  trocar com a chave). Não há partículas/feixe nem os itens flutuando no orbe (não há desenho de block entity aqui):
  use a tela do orbe.
- No original cada slot do orbe aceita **1** item. Aqui o clique direito põe 1 por vez (como lá), mas pela tela ou por
  funil um slot pode ter uma pilha: aí o orbe repete a receita, gastando 1 de cada slot por vez (o funil põe tudo de um
  item no primeiro slot, então receitas com o mesmo item em vários slots, como os 4 pós de blaze, só pela mão/tela).
- A saída do orbe empilha (o original não deixa pôr ingredientes com a saída ocupada); se a saída não cabe, o orbe para.
- Energia não fica no item ao quebrar o bloco (o original guarda); o conteúdo dos slots cai no chão.
- Sem configuração de lados (todos os lados de gerador/célula mandam; a célula recebe por todos), sem controle por
  redstone, sem carregar itens nos slots de carga (o original tem; não há itens com energia portados).
- Limite por cabo é por caminho (o cabo mais lento no trecho), não a rede inteira como o CableNet do original.
- O ATM10 pode mudar a configuração do Powah; aqui estão os valores padrão do mod.
- Mod nativo fixado em 66f3981. Ver `APIS.md` para os riscos de compilação.

## Desenvolvimento

```sh
cd source
python3 -I scripts/make_content.py /caminho/Powah-6.2.10.jar content
python3 -I scripts/upgrade_hmod.py --dir content/powah      # loot no formato do 26.3 (obrigatório)
python3 -I scripts/check_assets.py content
cd native && cargo test --offline                             # 14 testes da lógica (runtime.rs)
cd tools/glue_mockcheck && cargo build --offline              # tipos do glue.rs contra as assinaturas de 66f3981
cd ../../.. && python3 -I scripts/package.py . ..             # gera .hmod, .hnmod e SHA256SUMS.txt
```

## Licença

Powah é **LGPLv3** (owmii, Technici4n, shartte): texto em `LICENSE-original.txt` dentro do `powah.hmod` e aviso em
`mixins/powah_native/NOTICE` no `.hnmod`. Texturas, modelos, nomes e receitas vêm do jar original; o código nativo
foi escrito para este port, com os números e regras lidos do código-fonte público do Powah (ramo 1.21.1), e é
distribuído também sob a LGPLv3. Pode ser publicado, mantendo os avisos de licença.

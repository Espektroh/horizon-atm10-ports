# Mekanism para Horizon 0.6.0 (tiers avançado, elite e definitivo + Mekanism Tools)

Continuação da 0.5.0. Tudo o que a 0.5.0 faz continua igual e os mundos abrem do mesmo jeito: mesmos ids de
blocos e itens, mesmo NBT (a 0.6.0 só acrescenta as chaves `chemical`/`chemical_used`, e só no compressor).

São três arquivos, os três necessários:

- `mekanism.hmod` (conteúdo, id `mekanism`, versão 10.7.19-horizon.6): o da 0.5.0 + os blocos e itens novos.
- `mekanismtools.hmod` (conteúdo, id `mekanismtools`, novo): ferramentas e armaduras. Usa lingotes do `mekanism`.
- `mekanism_native-0.6.0.hnmod` (nativo, id `mekanism_native`, versão 0.6.0): substitui o `mekanism_native.hnmod`
  da 0.5.0. **Não deixe os dois nativos juntos** (mesma id): o `instalar.ps1` tira o antigo e guarda num backup.

## O que a 0.5.0 já fazia (continua igual)

Câmara de Enriquecimento, Triturador, Fundição Energizada (200 ticks, 50 J/t, 20 kJ), Infusor Metalúrgico
(carbono e redstone, buffer de 1000), Gerador a Calor (200 J/t queimando + 30 J/t por lava ao lado, 160 kJ),
Gerador Solar (50 J/t de dia com céu aberto, 96 kJ), Cubo de Energia Básico (4 MJ, 4 kJ/t, sai pela frente),
Cabo Universal Básico (8 kJ/t), melhorias de velocidade/energia (até 8 de cada, clique direito na máquina),
minério de ósmio, aço, circuito básico, liga infundida.

## O que a 0.6.0 acrescenta

### Compressor de Ósmio (`mekanism:osmium_compressor`)
- 200 ticks por operação, 100 J/t, guarda 80 kJ; aceita melhorias de velocidade e energia (como as outras).
- Tanque de ósmio líquido de 210 mB: o slot da esquerda come **lingotes de ósmio** (200 mB cada, entra um quando
  cabe inteiro). Cada operação gasta 200 mB, espalhados pelos ticks — com melhorias de velocidade continua 200 mB.
- Receitas: Pó de Obsidiana Refinada → **Lingote de Obsidiana Refinada**; Pó de Pedra Luminosa → **Lingote de
  Pedra Luminosa Refinada**.
- Funil: por cima a entrada, pelos lados o ósmio, por baixo a saída (igual ao infusor).
- Receita: `ACA / BXB / ACA` com A = liga infundida, C = circuito avançado, B = balde, X = revestimento de aço.

### Infusor: tipos novos
- **Diamante** (pó de diamante 10, diamante enriquecido 80) e **Obsidiana Refinada** (pó 10, enriquecida 80).
- Liga Reforçada = liga infundida + 20 diamante. Liga Atômica = liga reforçada + 40 obsidiana refinada.
- Pó de Obsidiana Refinada = pó de obsidiana + 10 diamante.
- Circuitos por infusão: básico + 60 redstone → avançado; avançado + 120 diamante → elite; elite + 240 obsidiana
  refinada → definitivo. Também na bancada: `ACA` (liga infundida/reforçada/atômica + circuito anterior).

### Triturador e Câmara de Enriquecimento: receitas novas
- Triturador: diamante → pó de diamante; obsidiana → 4 pó de obsidiana; lingote de obsidiana refinada → pó.
- Enriquecimento: diamante → diamante enriquecido; pó de obsidiana refinada → obsidiana enriquecida;
  pedra luminosa → 4 pó de pedra luminosa.

### Cubos de energia (saem pela frente, entram pelos lados)
| Cubo | Guarda | Entrada/saída por tick | Receita (`AEA / IPI / AEA`) |
|---|---|---|---|
| Avançado | 16 MJ | 16 kJ/t | A liga infundida, E bloco de redstone, I lingote de ósmio, P cubo básico |
| Elite | 64 MJ | 64 kJ/t | A liga reforçada, E bloco de redstone, I lingote de ouro, P cubo avançado |
| Definitivo | 256 MJ | 256 kJ/t | A liga atômica, E bloco de redstone, I diamante, P cubo elite |

O tablete de energia do original segue trocado por bloco de redstone, como no cubo básico da 0.5.0.
A energia guardada **não** passa para o cubo novo na bancada.

### Cabos universais
| Cabo | Carrega | Receita (8 cabos) |
|---|---|---|
| Avançado | 128 kJ/t | 8 cabos básicos em volta de 1 liga infundida |
| Elite | 1.024 kJ/t | 8 avançados + liga reforçada |
| Definitivo | 8.192 kJ/t | 8 elite + liga atômica |

Tiers diferentes se ligam entre si. O que chega a cada máquina/cubo é limitado pelo cabo **mais lento no caminho**
(um básico no meio de uma linha definitiva deixa passar 8 kJ/t). Máquina encostada direto no gerador/cubo não tem
limite de cabo (antes era 8 kJ/t; nenhum gerador da 0.5.0 passava disso, então nada muda para quem já tinha).

### Blocos de armazenamento
Bloco de Aço, Bloco de Obsidiana Refinada (dureza 50), Bloco de Pedra Luminosa Refinada (luz 15): 9 lingotes ⇄ 1 bloco.

### Mekanism Tools (`mekanismtools.hmod`)
Para cada material: espada, picareta, machado, pá, enxada, **paxel** (picareta + machado + pá), capacete,
peitoral, calças e botas — 50 itens, com as receitas do original (lingote + graveto).
| Material | Durabilidade | Velocidade | Dano extra | Armadura (cap/peito/calça/bota) | Dureza | Minera como |
|---|---|---|---|---|---|---|
| Ósmio | 1024 | 4 | +4 | 4/8/6/3 | 3 | ferro |
| Aço | 500 | 8 | +3 | 3/8/6/3 | 2 | ferro |
| Lápis-lazúli | 128 | 9 | +1 | 1/4/3/1 | 0 | pedra |
| Pedra Luminosa Refinada | 384 | 15 | +2 | 3/8/6/3 | 0 | diamante |
| Obsidiana Refinada | 4096 | 12 | +8 | 6/12/8/5 | 5 | netherite (não queima na lava) |

Paxel tem o dobro da durabilidade. Reparo na bigorna com o lingote/gema do material.

## Como testar no jogo (curto)

1. Rode `instalar.ps1`, faça o Build no launcher, abra um mundo da 0.5.0: as máquinas antigas continuam com
   energia, progresso e melhorias.
2. Criativo: pegue um Compressor de Ósmio, um Cubo de Energia Definitivo e cabos de tiers diferentes. Ligue um
   gerador → cabos → compressor. Ponha um lingote de ósmio à esquerda e pó de pedra luminosa (ou de obsidiana
   refinada) em cima: a barra de ósmio enche (200/210 mB), a seta anda e sai o lingote refinado.
3. Infusor: pó de diamante no slot da esquerda, liga infundida em cima → Liga Reforçada.
4. Cubo definitivo: a tela mostra "x / 256,0 MJ"; alimentando por um cabo básico ele enche a 8 kJ/t, por um
   definitivo a até 256 kJ/t.
5. Faça uma picareta de obsidiana refinada e minere obsidiana; vista a armadura de ósmio (aparece no corpo).

## Limitações desta versão

- Sem gases: Câmara de Purificação, Injeção Química, Separador, etc. continuam fora. O "ósmio líquido" é um
  tanque interno do compressor, sem tubos nem gás no mundo.
- Bloco de ósmio não entra no compressor (1800 mB não cabem no tanque de 210 mB; no original também não).
- Sem estanho/bronze (o Mekanism tem minério de estanho, ainda não portado): ferramentas de bronze ficaram de fora.
- Sem escudos do Mekanism Tools, sem guarnição (trim) nas armaduras do Tools, sem fábricas e sem Melhoria de
  Química. Sem gerador eólico/outros do Mekanism Generators.
- Machado/enxada/pá do Tools: o que o Horizon liga ao id vanilla (descascar tronco, arar terra, fazer caminho)
  pode não funcionar; minerar, dano e durabilidade funcionam.
- A energia não passa do cubo para o cubo de tier maior na bancada.
- Limite por cabo é por caminho (o mais lento no trecho), não pela soma da rede como no original.

## Licença

Mekanism, Mekanism Generators e Mekanism Tools são MIT (Aidan C. Brady e equipe; `source/LICENSE-Mekanism.txt`).
Texturas, modelos e nomes vêm dos jars originais. Port para Horizon também MIT.

## 0.6.1 (testado no jogo em 11/11/2026)
- Shift-clique no Compressor de Ósmio e no Infusor Metalúrgico manda cada item para o slot certo (ósmio/material de infusão no slot lateral, o item da receita na entrada). Antes ia para o primeiro slot livre e a máquina travava.
- Loot e worldgen convertidos para o formato do 26.3 (`source/scripts/upgrade_hmod.py`, port dos conversores do próprio horizon-port-pack): o minério de ósmio passa a ser gerado (só em chunks novos) e as tabelas de loot com condições funcionam.
- Testado: Gerador a Calor (200 J/t), cabo, Câmara de Enriquecimento (minério → 2 pós), Compressor de Ósmio (lingote de pedra luminosa refinada).

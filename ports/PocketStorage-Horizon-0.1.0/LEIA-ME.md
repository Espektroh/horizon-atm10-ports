# Pocket Storage para Horizon (port 0.1.0)

Port do **Pocket Storage** (mod id `pocketstorage`, ATM10, versão 1.2.5 para 1.21.1) para o Horizon 26.3, Nightly
`66f3981f522b24c250b21523a70080965b940424`. São dois arquivos:

| Arquivo | O que é |
|---|---|
| `pocketstorage.hmod` | mod de conteúdo: os itens `pocketstorage:psu_1` a `psu_4`, textura, modelos, receitas, aba criativa, traduções (en_us, pt_br) |
| `pocketstorage_native-0.1.0.hnmod` | mod nativo: a tela do PSU, depositar/retirar, busca, coleta automática, tooltip |

**Compilação:** compilado de verdade junto com todos os outros mods nativos pelo CI do repositório privado
(`tools/ci_check.sh`, apply dos mixins + `cargo check -p horizon-client`, bases 66f3981 e 2ebf00a): run **verde** nas duas
https://github.com/Espektroh/horizon-atm10-privado/actions/runs/44072447495 (branch `wip/pocketstorage`, commit `588fb76`).
Lógica pura: 16 testes (`cargo test --offline` em `source/native/`), todos passam.

## Instalar

```powershell
powershell -ExecutionPolicy Bypass -File .\instalar.ps1            # acha a pasta do jogo sozinho
powershell -ExecutionPolicy Bypass -File .\instalar.ps1 -Verificar # só confere
```

Depois, no launcher: Settings > Native mods ligado, **Build** na página Mods, abra o jogo e confira
"Pocket Storage (Horizon) 0.1.0" em Options > Mods.

## O que funciona (números do original, `PSUTier`)

| Item | Tipos diferentes | Máximo de cada | Raridade |
|---|---|---|---|
| `pocketstorage:psu_1` | 8 | 255 | comum |
| `pocketstorage:psu_2` | 16 | 4.095 | incomum |
| `pocketstorage:psu_3` | 32 | 65.535 | raro |
| `pocketstorage:psu_4` | 64 | 1.048.575 | épico |

- **Clique direito** com o PSU na mão (principal ou secundária) abre a tela. Cada slot mostra o item; o número no
  slot vai até 64 e a quantidade real está na descrição ("Guardado: 5.000 / 65.535").
  - Tela: tier 1 = 1 linha + controles; tier 2 = 2; tier 3 = 4; tier 4 = 5 linhas, **2 páginas** (45 + 19).
  - Linha de baixo: seta `<` (página anterior), placa/etiqueta (busca), livro (tier, capacidade, tipos usados), seta `>`.
- **Depositar**: clique (ou shift-clique) num item do seu inventário, dentro da tela: a pilha inteira entra
  (no slot que já tem o item, senão no primeiro vazio). Clique direito: entra 1. O que passar do máximo **fica no
  inventário** (não some).
- **Retirar**: clique (esquerdo ou direito) num slot do PSU: sai **1**. Shift-clique: sai **1 pack** (64, ou 16 para
  pérolas etc.). Só sai o que cabe no inventário.
- **Só itens simples**: itens com dados (encantados, renomeados, danificados, com conteúdo) e outros PSUs não entram,
  como no original. O próprio PSU aberto fica travado.
- **Busca**: `/psu buscar <texto>` (no chat, antes de abrir) e abra o PSU: só aparecem os slots cujo id contém o
  texto (`cobble` acha `minecraft:cobblestone` e `minecraft:cobbled_deepslate`; espaços viram `_`). Clique na
  etiqueta da busca (ou `/psu limpar`) para ver tudo de novo. A busca fica guardada por jogador até limpar.
- **Coleta automática** (original: "Automatically absorbs like items on pickup, voids excess"): ao pegar um item do
  chão que algum PSU nos 36 slots do inventário **já guarda**, a pilha inteira vai para o primeiro desses PSUs;
  o que passar do máximo **some** (igual ao original). Não coleta agachado, nem com um PSU aberto, nem itens com
  dados. Item novo (que nenhum PSU tem) vai para o inventário normal.
- **Os itens ficam no próprio item**, em `minecraft:custom_data` (`pocketstorage_native:slots`, texto
  `minecraft:cobblestone*5000;;minecraft:dirt*3`). Salvam com o inventário: sair e voltar ao mundo não perde nada,
  e o PSU pode ir para um baú ou outro jogador com tudo dentro.
- **Tooltip**: "Guarda 8 itens diferentes, ate 255 de cada.", "Tipos usados: 3 / 8" e o aviso da coleta.

## Receitas (as do original; tags `c:` trocadas pelo item vanilla)

```
Tier 1:  F P F    Tier 2:  O P O    Tier 3:  D P D    Tier 4:  P B P
         R B R             B 1 B             B 2 B             N 3 N
         F P F             O P O             D P D             P B P
F = barra de ferro, P = pistão, R = redstone, B = baú, O = barra de ouro, D = diamante,
N = estrela do Nether, 1/2/3 = PSU do tier anterior (VAZIO)
```

Diferença: o original passa o conteúdo para o tier novo. Aqui a melhoria **só aceita um PSU vazio** (a receita não
aparece com um PSU cheio no grid), para nunca comer itens: esvazie, melhore, guarde de novo.

## Roteiro de teste (orquestrador, por MCP)

Mundo de teste novo, criativo, com cheats. Inventário vazio no início de cada passo (`/clear`).

1. **Tela e números**: `/give @s pocketstorage:psu_1`, `/give @s pocketstorage:psu_4`. Clique direito com o tier 1
   na mão. Esperado: menu de **2 linhas** (18 slots) com título "PSU Tier 1"; slots 0..7 vazios, slot 8 vidro cinza;
   livro no slot 15 com "Guarda 8 itens diferentes, ate 255 de cada." e "Tipos usados: 0 / 8". Feche; abra o tier 4:
   menu de **6 linhas**, setas nos slots 45 e 53, livro "ate 1.048.575 de cada".
2. **Depositar e limite do tier 1**: com o tier 1 no slot 0 da hotbar, `/give @s minecraft:cobblestone 320`
   (5 pilhas de 64). Abra o PSU e clique nas 5 pilhas de pedregulho. Esperado: slot 0 do PSU com descrição
   "Guardado: 255 / 255"; no inventário sobram **65** de `minecraft:cobblestone` (64+1 ou em um só slot).
   `custom_data` do PSU = `{"pocketstorage_native:slots":"minecraft:cobblestone*255"}`.
3. **5000 num tier 3 e persistência**: `/clear`, `/give @s pocketstorage:psu_3[minecraft:custom_data={"pocketstorage_native:slots":"minecraft:cobblestone*4990"}]`
   e `/give @s minecraft:cobblestone 10`. Abra o PSU, clique nos 10. Esperado: "Guardado: 5.000 / 65.535".
   Saia para o menu principal, entre de novo no mundo, abra o PSU. Esperado: continua "Guardado: 5.000 / 65.535"
   (`custom_data` `minecraft:cobblestone*5000`).
4. **Retirar**: no mesmo PSU, clique no pedregulho: inventário ganha **1** (4.999 no PSU). Shift-clique: ganha
   **64** (4.935 no PSU). Esperado: 65 `minecraft:cobblestone` no inventário.
5. **Itens com dados recusados**: `/give @s minecraft:diamond_sword[minecraft:damage=10]` e
   `/give @s minecraft:stone[minecraft:custom_name="x"]`. Abra o PSU, clique neles. Esperado: nada muda (continuam
   no inventário; "Tipos usados" igual). (Uma espada nova, sem dano, é "item simples" e entra, como no original.)
6. **Coleta automática**: `/gamemode survival`, `/clear`, `/give @s pocketstorage:psu_1[minecraft:custom_data={"pocketstorage_native:slots":"minecraft:dirt*250"}]`.
   `/summon item ~1 ~ ~ {Item:{id:"minecraft:dirt",count:10}}` e espere 20 ticks parado em cima.
   Esperado: o item some do chão; o PSU fica com `minecraft:dirt*255` (5 entraram, **5 sumiram**); **0** de terra no
   inventário. Depois `/summon item ~1 ~ ~ {Item:{id:"minecraft:sand",count:3}}`: a areia vai para o inventário (3),
   o PSU não muda.
7. **Sem coleta agachado**: segure shift (agachado) e `/summon item ~ ~ ~ {Item:{id:"minecraft:dirt",count:4}}`.
   Esperado: 4 de terra no inventário; PSU continua `dirt*255`.
8. **Busca**: `/give @s pocketstorage:psu_2[minecraft:custom_data={"pocketstorage_native:slots":"minecraft:cobblestone*5;minecraft:dirt*5;minecraft:cobbled_deepslate*5"}]`.
   `/psu buscar cobble`. Esperado no chat: `PSU: busca "cobble". ...`. Abra o PSU: só 2 slots preenchidos
   (pedregulho e ardósia), etiqueta "Busca: cobble" com "2 encontrados". Clique na etiqueta: voltam os 16 slots
   (3 preenchidos). `/psu buscar nada` e abra: 0 slots, "0 encontrados".
9. **Páginas**: `/give @s pocketstorage:psu_4`, abra, clique na seta do slot 53. Esperado: página 2 com 19 slots
   (45..63) e o resto vidro cinza; a seta mostra "(2/2)".
10. **Melhoria**: na bancada, com 4 barras de ouro, 2 pistões, 2 baús e um `psu_1` **vazio**: sai `psu_2`. Com o
    `psu_1` do passo 2 (cheio): não sai nada.

## Fora desta versão (documentado)

- **Agachar + clique direito / esquerdo num baú** (despejar o PSU no baú / puxar do baú o que ele já guarda):
  não portado (precisa do inventário de blocos de outros mods).
- **Melhoria com conteúdo**: o hook de resultado da bancada não vê o grid; a melhoria só aceita PSU vazio.
- **Tecla Q no slot** (o original joga 1, ou 64 com Ctrl): não portado. Clique-meio, arrastar e trocar com a hotbar
  não fazem nada na tela do PSU.
- **Excesso no depósito**: o original apaga o que passa do máximo no shift-clique; aqui o excesso fica no
  inventário (só a coleta automática apaga, como diz a descrição do item).
- **Comandos** `/pocketstorage list|open|recover` (o original guarda os itens num arquivo do mundo por UUID; aqui
  ficam no item, então não há o que recuperar).
- **Tela própria** com barra de busca digitável e o "Count" desenhado: aqui é um menu de baú; a busca é pelo comando.
- Busca por nome traduzido (aqui é pelo id do item), coleta pela mão secundária, som de coleta.

## Licença

- Pocket Storage: **MIT** (`LICENSE` do repositório `Flanks255/PocketStorage`, copyright 2020 Flanks255).
  Código consultado: branch `master`, commit `7257f80`, versão 1.2.5 para 1.21.1 (`gradle.properties`).
  Textura, modelos, en_us e receitas vêm desse repositório; aviso de licença em
  `source/content/pocketstorage/NOTICE-original.txt` (também dentro do `.hmod`) e em
  `mixins/pocketstorage_native/NOTICE`. Pode publicar, mantendo os avisos.
- A tradução pt_br é própria deste port.
- O port (código nativo e scripts) segue MIT.
- Existe um port parcial de terceiros (`AdLf0208/pocketstorage`, 11/11/2026, só o tier 1, sem tela, para outro
  commit do Horizon, fora do registro `horizon-mods`). Não foi usado.

## Fontes

`source/content/pocketstorage/` (conteúdo), `source/native/` (mod nativo e crate de teste), `source/scripts/`
(`package.py SOURCE PACOTE` gera os dois arquivos e o SHA256SUMS; `upgrade_hmod.py` foi rodado no conteúdo:
0 mudanças, não há loot nem worldgen). APIs do Horizon usadas: `APIS.md`.

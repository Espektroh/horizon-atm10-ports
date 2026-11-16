# Mystical Agriculture + Mystical Agradditions para Horizon 66f3981 (port 0.1.0)

Port do [Mystical Agriculture](https://github.com/BlakeBr0/MysticalAgriculture) 8.0.28 e do
[Mystical Agradditions](https://github.com/BlakeBr0/MysticalAgradditions) 8.0.14 (BlakeBr0, **MIT**) para o
Horizon (Minecraft 26.3 em Rust). É o núcleo da progressão do ATM10: sementes que crescem recursos.
Os ids são os do original (`mysticalagriculture:`, `mysticalagradditions:`).

## São três mods, os três são necessários

- **`mysticalagriculture.hmod`** (conteúdo): essências, lingotes, blocos, terras, minérios, sementes, receitas.
- **`mysticalagradditions.hmod`** (conteúdo): o tier 6 (Insanium), cruxes, minérios no Nether e no End.
- **`mysticalagriculture_native-0.1.0.hnmod`** (nativo): o que precisa de código (abaixo).

Sem o nativo as sementes plantam **trigo** em vez do cultivo (o elo semente → cultivo é do nativo), a terra de
essência não dá bônus e os mobs não soltam Inferium.

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

Fora o Inferium, cada semente é feita **na bancada**: `MEM / ESE / MEM`, M = material, E = essência do tier,
S = Base de Semente de Prosperidade. Isso é a opção `seedCraftingRecipes` do próprio original (veja Limitações).
As 123 receitas essência → recurso do original vêm junto (ex.: 8 essências de ferro = 6 lingotes; 8 de carvão =
12 carvões; 8 de ósmio = 4 lingotes; 8 de aço = 3 lingotes).

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
   Ele confere o SHA-256, faz backup de `horizon-mods.txt` em `mod-backups\`, copia os dois `.hmod` para
   `horizon-mods\` e o `.hnmod` para `horizon-native-mods\` e liga os três. Para conferir: `-Verificar`.
2. No launcher: **Build** (Native mods ligado). Confie no código nativo se pedir.
3. Abra o jogo e confira em **Options → Mods** que os três aparecem ligados.

## Como testar no jogo (5 minutos, mundo criativo novo)

1. `/give @s mysticalagriculture:inferium_seeds 4`, `/give @s mysticalagriculture:inferium_essence 16`,
   `/give @s mysticalagriculture:tertium_essence 4`, `/give @s mysticalagriculture:iron_seeds 4`.
2. Are 4 blocos de terra perto de água. Clique com Essência de Inferium em 2 deles: viram **Terra de Inferium**
   (verde). Clique com Essência de Tertium em outro: **Terra de Tertium**. A essência não é gasta no criativo.
3. Plante Inferium na terra comum e na de Inferium; ferro na de Tertium. A semente planta o **cultivo**, não trigo
   (se plantar trigo, o nativo não entrou no build).
4. `/gamerule randomTickSpeed 1000` por alguns segundos, depois `/gamerule randomTickSpeed 3`. Farinha de osso
   não deve fazer crescer.
5. Em **sobrevivência** (`/gamemode survival`), colha: ferro na Terra de Tertium dá 2 essências 20% das vezes;
   Inferium na Terra de Inferium dá 1. Teste o Fertilizante Místico num trigo (amadurece na hora).
6. Bancada: 8 Essências de Ferro = 6 lingotes de ferro; `MEM/ESE/MEM` para uma semente.
7. Mate alguns zumbis: ~1 em 5 solta Essência de Inferium.
8. Estrela do Nether: plante as sementes sem a crux 2 blocos abaixo e veja que não crescem; com a
   **Crux de Estrela do Nether** embaixo da terra, crescem.
9. Minério: em chunks **novos**, Inferium aparece entre -32 e 64.

## Limitações desta versão (0.1.0)

- **Sem Infusion Altar / pedestais.** Dava para fazer em 66f3981 (block entities, uso de item, item_display),
  mas é grande e arriscado para o build único; nesta versão as receitas de semente do altar são as receitas de
  **bancada** que o próprio original tem (`seedCraftingRecipes`), com os mesmos 8 ingredientes em volta da base.
  Fica para a 0.2.0.
- Sem sementes de mob (precisam de Soul Jars/Soulium), elementais (ar, terra, água, fogo), Soulstone/Soulium,
  máquinas (Harvester, Reprocessor etc.), ferramentas, armaduras, augments, watering can, Awakened Supremium,
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
- Mod nativo fixado em 66f3981. Ver `APIS.md` para os riscos de compilação.

## Inferium Lab

O "Inferium Lab" (pehe1) já existe no registro: só a semente de Inferium, com ids `inferium_lab:`. Este port
não depende dele e não colide (namespaces diferentes); usar os dois juntos só duplica a semente de Inferium.
Este port reaproveita a técnica que ele validou em 66f3981 para ligar a semente ao cultivo (dois pontos do
carregador de conteúdo), generalizada para as 31 sementes.

## Desenvolvimento

```sh
# conteúdo (dos jars do ATM10 descompactados)
python3 source/scripts/make_content.py <MA_jar_dir> <MAA_jar_dir> source/content
python3 source/scripts/check_assets.py source/content
# nativo: testes da lógica e checagem de tipos contra as assinaturas de 66f3981
cd source/native && cargo test --offline --lib
cd tools/stubcheck && cargo test --offline
# empacotar (gera .hmod, .hnmod e SHA256SUMS.txt)
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

# Time In A Bottle para Horizon (port 0.1.0)

Port do **Time In A Bottle** (mod id `tiab`, ATM10) para o Horizon 26.3, Nightly
`66f3981f522b24c250b21523a70080965b940424`. São dois arquivos:

| Arquivo | O que é |
|---|---|
| `tiab.hmod` | mod de conteúdo: o item `tiab:time_in_a_bottle`, textura animada, modelo, receita, traduções (en_us, pt_br) |
| `tiab_native-0.1.0.hnmod` | mod nativo: acumular tempo, tooltip e acelerar blocos |

**Compilação:** compilado de verdade junto com todos os outros mods nativos pelo CI do repositório privado
(`tools/ci_check.sh`, apply dos mixins + `cargo check -p horizon-client`): run verde
https://github.com/Espektroh/horizon-atm10-privado/actions/runs/43978484029 (branch `wip/tiab`).
Lógica pura: 9 testes (`cargo test --offline` em `source/native/`), todos passam.

## Instalar

```powershell
powershell -ExecutionPolicy Bypass -File .\instalar.ps1            # acha a pasta do jogo sozinho
powershell -ExecutionPolicy Bypass -File .\instalar.ps1 -Verificar # só confere
```

Depois, no launcher: Settings > Native mods ligado, **Build** na página Mods, abra o jogo e confira
"Time In A Bottle 0.1.0" em Options > Mods.

## O que funciona (números do original, config padrão)

- **Acumula tempo**: em qualquer slot do inventário (36 slots, armadura, mão secundária), a cada 20 ticks
  ganha +20 ticks de tempo guardado e de tempo total (máximo 622.080.000 ticks = 8.640 h). Guardado em
  `minecraft:custom_data` como o original guardava em NBT: `storedTime` e `totalAccumulatedTime`, em
  **ticks** (20 = 1 s).
- **Mais de uma garrafa**: a cada 10 s, uma garrafa que tem menos tempo que outra do mesmo inventário é
  zerada (igual ao original, para não multiplicar tempo).
- **Tooltip**: "Tempo guardado HH:MM:SS" (verde) e "Tempo total acumulado HH:MM:SS" (cinza).
- **Clique direito num bloco**:
  - 1º clique: x2 por 30 s, custa 30 s de tempo guardado.
  - cada clique seguinte no mesmo bloco dobra: x4, x8, x16, x32, x64, x128, x256 (máximo). Custo de
    x2N = N/2 × 30 s (x4 = 30 s, x8 = 60 s, x16 = 2 min, ... x256 = 32 min). Subir de nível devolve metade
    do tempo já gasto do efeito (`remaining += usado/2`), como o original.
  - Criativo não gasta tempo.
  - "x2N" quer dizer N ticks extras por tick do jogo (o original mostra "x" + 2 × taxa).
  - Mensagem na barra de ação com a velocidade e os segundos restantes, e a nota de harpa do original
    (mais aguda a cada nível).
- **O que acelera**:
  - block entities de mods nativos que têm tick (Iron Furnaces, Mekanism, Powah, Mystical Agriculture...):
    N ticks extras do próprio mod por tick;
  - fornalha, alto-forno e defumador vanilla: N ticks extras do cozimento vanilla do Horizon;
  - blocos com random tick (trigo, cenoura, muda, cana, cacto, grama...): cada tick extra tem 1 chance em
    1365 de dar um random tick (`averageUpdateRandomTick` = 1365, que é a taxa vanilla), então x2N cresce
    ~N vezes mais rápido que o normal.
  - Em blocos sem nada disso o clique não faz nada e não gasta tempo.
- Bloco quebrado ou trocado: o acelerador some (como a entidade do original).
- Chunk que não está ticando: o acelerador pausa.

## Receita (a do original, bancada)

```
O O O     O = barra de ouro
D R D     D = diamante, R = relógio
L G L     L = lápis-lazúli, G = frasco de vidro
```

O original usa as tags `forge:ingots/gold`, `forge:gems/diamond` e `forge:gems/lapis`. Aqui são os itens
vanilla. **O ATM10 troca essa receita** (script KubeJS: picaretas de Neptunium, Vengeance, Fluix,
Iesnium, Blazegold, upgrades do ProductiveLib e frasco de experiência). Esses itens não estão portados,
então fica a receita do original. Quando estiverem, dá para trocar só o JSON da receita.

## Roteiro de teste (orquestrador, por MCP)

Mundo de teste novo, sobrevivência, `/gamerule doDaylightCycle false`. Em todos os passos, conte os ticks
pelo `/time query gametime` (ou `game_wait` de 20 ticks por segundo).

1. **Acúmulo**: `/give @s tiab:time_in_a_bottle`. Espere 200 ticks.
   Esperado: tooltip "Tempo guardado 00:00:10" (aceita 00:00:09 a 00:00:11); `custom_data`
   `storedTime` = 200 (±20) e `totalAccumulatedTime` igual.
2. **Duas garrafas**: dê uma 2ª com `/give @s tiab:time_in_a_bottle[minecraft:custom_data={storedTime:12000,totalAccumulatedTime:12000}]`.
   Espere até 200 ticks. Esperado: a 1ª garrafa (a de menos tempo) fica com `storedTime` 0; a 2ª continua
   subindo. Jogue a 1ª fora (`/clear` ou drop) antes do passo 3.
3. **Fornalha vanilla, x4**: coloque uma `minecraft:furnace`, ponha 64 `minecraft:raw_iron` em cima e
   8 `minecraft:coal` no combustível. Espere ela acender (1 tick). Com a garrafa de 12000 na mão, clique
   direito na fornalha **duas vezes seguidas** (x2 e depois x4).
   Esperado logo após: barra de ação "Velocidade do tempo: x4 (30 s restantes)"; `storedTime` da
   garrafa = 12000 − 600 − 600 = 10800 (± o acúmulo de 1 s), tooltip ~00:09:00.
   Espere **600 ticks**. Esperado: a saída tem **9 `minecraft:iron_ingot`** (aceita 8 a 9) em vez dos
   **3** de uma fornalha sem garrafa ao lado (faça uma de controle igual, sem clicar). A taxa é 3 ticks de
   cozimento por tick (1 normal + 2 extras), 200 ticks por item.
4. **Máximo**: numa 2ª fornalha, `/gamemode creative`, clique 9 vezes. Esperado: a 8ª mostra "x256", a 9ª
   "Ja esta na velocidade maxima: x256"; em criativo o `storedTime` não muda.
5. **Sem tempo**: `/gamemode survival`, garrafa nova (`storedTime` 0) num bloco de pedra e depois numa
   fornalha. Esperado: na pedra nada acontece (o clique não é tomado); na fornalha "Tempo guardado
   insuficiente: precisa de 30 s".
6. **Mod nativo (Iron Furnaces)**: `ironfurnaces:iron_furnace` com 64 raw_iron e 8 carvões, x4 como no
   passo 3. Esperado após 600 ticks: 3× a saída de uma Iron Furnace de controle no mesmo tempo (a de
   controle cozinha 600 / tempo-de-item; a acelerada 1800 / tempo-de-item; ±1 item).
7. **Random tick (cana)**: cana-de-açúcar de 1 bloco em areia ao lado de água. Com a garrafa de muito
   tempo (`storedTime:1000000`), clique 8 vezes (x256). Espere 600 ticks.
   Esperado: a cana chega a **3 blocos** de altura (aceita 2; ~56 random ticks em 30 s, 16 por bloco). Uma cana de
   controle sem garrafa fica, quase sempre, com 1 bloco no mesmo tempo.
8. **Quebrar**: acelere uma fornalha, quebre-a. Esperado: nada fica acelerado no lugar (coloque outra
   fornalha no mesmo ponto: cozinha na velocidade normal).

## Fora desta versão (documentado)

- **Efeito visual do acelerador** (o cubo com "x4" desenhado nas faces do bloco) e partículas: não há
  entidade; no lugar, a mensagem na barra de ação e a nota de harpa.
- **Acelerador não é salvo**: o original usa uma entidade salva no mundo; aqui o efeito vive na memória e
  some ao sair do mundo (perde no máximo o que restava do efeito; o tempo gasto não volta).
- **Lista de blocos bloqueados**: o código público (branch master, 1.20.1) não tem lista; a versão 1.21.1
  que o ATM10 usa parece ter uma (o ATM10 traduz "Blocos na Lista-negra do TIAB" para o EMI), mas o
  código dela não é público e o ATM10 não tem config do tiab. Não há lista aqui.
- **Block entities vanilla além das fornalhas** (funil, suporte de poções, fogueira, spawner, colmeia...):
  o Horizon não expõe o tick de uma só delas para mods; a garrafa não toma o clique nelas.
- **Time Charm**: não existe no código público do mod (só a garrafa).
- **Comandos** `/tiab addTime` e `/tiab removeTime`: não portados (use o `/give` com `custom_data`).
- Os eventos `native_furnace_*` que outros mods nativos ouvem não disparam nos ticks extras da fornalha
  vanilla.

## Licença

- Time In A Bottle: **MIT** (`LICENSE.txt` do repositório `haoict/time-in-a-bottle`, copyright lumien
  2015, extraído do Random Things). Código consultado: branch `master` (4.0.0, 1.20.1, commit
  `19cedfc`), o único público: o repositório está arquivado e não tem branch 1.21.1. Textura, modelo,
  receita e en_us vêm desse repositório; aviso de licença em `source/content/tiab/LICENSE-original.txt`
  (também dentro do `.hmod`) e em `mixins/tiab_native/NOTICE`.
- A tradução pt_br é própria deste port (a do ATM10 é All Rights Reserved e não foi copiada).
- O port (código nativo e scripts) segue MIT.

## Fontes

`source/content/tiab/` (conteúdo), `source/native/` (mod nativo e crate de teste), `source/scripts/`
(`package.py` gera os dois arquivos e o SHA256SUMS; `upgrade_hmod.py` foi rodado no conteúdo: 0 mudanças,
não há loot nem worldgen). APIs do Horizon usadas: `APIS.md`.

# Torchmaster para Horizon 66f3981 — port 0.1.0

Torchmaster do ATM10: um mod de conteúdo e um mod nativo, que dá para ligar ou desligar em **Options → Mods**.

| Mod | Conteúdo | Nativo | Licença |
|---|---|---|---|
| Torchmaster 21.1.12 | `torchmaster.hmod` | `torchmaster_native-0.1.0.hnmod` | MIT (Xalcon). Pode compartilhar, mantendo o aviso de licença. |

O mod precisa do conteúdo **e** do nativo. Sem o nativo, os blocos e itens existem, mas não fazem nada.

O **Elevator** ficou fora deste pacote. Já existe um port jogável dele
([Joeypc6/Elevator-Horizon](https://github.com/Joeypc6/Elevator-Horizon)), que usa a id `elevatorid`.

## Torchmaster (`torchmaster` + `torchmaster_native`)

- **Mega Tocha**: bloqueia o spawn **natural** de monstros (categoria `monster`) num **cubo de 64 blocos** em cada
  direção, igual à config padrão do original (`megaTorchRadius = 64`).
  - **Regra de altura**: o cubo também vale para Y. Uma tocha em y=70 protege de y=6 até y=135
    (`IDistanceLogic.Cubic`: de tocha−64 até tocha+64+1, com o mob no centro do bloco). Uma caverna 70 blocos
    abaixo da tocha **não** fica protegida.
  - Spawners, ovos, `/summon` e reforços de zumbi continuam funcionando (`blockOnlyNaturalSpawns = true`, o padrão).
  - Também bloqueia o **cerco de zumbis** à vila quando o ponto do cerco cai no cubo (`blockVillageSieges = true`).
- **Lâmpada do Medo**: a mesma coisa para os **animais** (todas as categorias amigáveis: animais, morcegos,
  lulas, peixes, axolotes; `misc` fica de fora), cubo de 64.
- **Lanterna Sinalizadora Feroz**: a cada 5 ticks sorteia um ponto num cubo de 16 em volta e, se ele for ar com luz
  de bloco abaixo de 10, coloca ali uma **luz invisível** (nível 15). Limite de 255 luzes por lanterna. Um ponto
  sorteado mais de 4 blocos acima da superfície desce para superfície+4. Quebrar a lanterna **com a mão ou
  ferramenta** remove as luzes dela.
- Receitas (sem as tags `c:`, que o 26.3 não tem):
  - Mega Tocha: 3 tochas / diamante, tronco, diamante / bloco de ouro, tronco, bloco de ouro.
  - Lâmpada do Medo: 3 obsidianas / painel de vidro, pedra luminosa, painel de vidro / obsidiana, corante preto, obsidiana.
  - Lanterna Feroz: lingote de ouro em cima e embaixo, no meio vidro, pó de pedra luminosa, vidro.

### Como testar (Torchmaster)
1. Crie um mundo de teste em Survival, dificuldade Normal, e use `/time set midnight`.
2. Coloque uma Mega Tocha numa planície. Ande uns 30 blocos e espere: nenhum monstro novo deve nascer à sua volta.
   Ande para mais de 65 blocos dela: lá eles voltam a nascer.
3. Regra de altura: desça mais de 64 blocos abaixo da tocha, numa caverna escura. Lá os monstros nascem.
4. Coloque uma Lanterna Feroz numa caverna escura e espere 1–2 minutos. A caverna fica clara.
   Confira com F3 (luz de bloco do ponto onde você está). Quebre a lanterna e a caverna volta a escurecer.
5. Lâmpada do Medo: num bioma de planície à noite (ou numa praia, por causa das lulas), os animais param de nascer
   no cubo. Os que já existem ficam.

### Limitações (Torchmaster)
- O original guarda as tochas num arquivo do mundo. Aqui cada tocha avisa pelo seu block entity a cada tick, então
  vale onde o chunk da tocha está carregado (o mesmo lugar onde o spawn natural acontece). Ao abrir o mundo, a
  proteção começa no primeiro tick da tocha. Ao quebrar, ela some na hora se quebrada por jogador, ou em até
  2 s (40 ticks do nível) se quebrada por explosão ou pistão.
- Lanterna: sem a tela de "linha de visão" (fica sempre desligada, que é o padrão), sem a Pérola Congelada,
  só de chão ou de teto (template de lanterna, sem as paredes). A lanterna quebrada por explosão deixa as luzes.
  Nesse caso, apague a área com
  `/fill ~-16 ~-16 ~-16 ~16 ~16 ~16 air replace torchmaster:invisible_light`.
- Sem os overrides de lista de entidades da config, sem a regra especial do Lycanites Mobs e sem os tooltips.
- O número aleatório da lanterna é do próprio mod (guardado no block entity), não o `level.random` do jogo.

## Instalar
1. Feche o launcher e rode, nesta pasta: `powershell -ExecutionPolicy Bypass -File .\instalar.ps1`.
   O script confere o SHA-256, faz backup do `horizon-mods.txt`, copia os `.hmod` para `horizon-mods\` e os `.hnmod`
   para `horizon-native-mods\`, e liga os dois em `horizon-mods.txt`.
   Opções: `-GameDir "C:\..."` escolhe a pasta;
   `-Verificar` só confere.
2. No launcher: **Build** na página Mods (Native mods ligado). Como todos os mods nativos compilam juntos, um erro
   de compilação aqui quebra o build inteiro. Se isso acontecer, tire o `.hnmod` de `horizon-native-mods\`
   (o resto volta a compilar) e me mande o `build.log`.
3. Abra o jogo e confira em **Options → Mods**.

## Detalhes técnicos
- Fixado no Horizon `66f3981` (o mesmo dos seus outros mods nativos) e no horizon-mixins `5a64af9`.
- `APIS.md` lista cada função do Horizon usada, com arquivo:linha, e os riscos de compilação.
- **Ponto de atenção**: no 66f3981 o spawn natural só consulta os mods (`native_potential_spawns`) quando há um
  *trait mod* carregado (`NaturalLevel::spawn_hooks`). O `torchmaster_native` injeta um HEAD cancelável nessa
  função, que responde "sim" enquanto existir uma Mega Tocha ou Lâmpada registrada. É a primeira vez que um mod seu
  usa um alvo `<Tipo as Trait>::método`. Se o horizon-mixins recusar esse alvo, o build falha com
  `E_TARGET_NOT_FOUND` citando `spawn_hooks`.
- Fonte: `source/content/` (gerado por `source/scripts/make_content.py <jar torchmaster> source/content`),
  `source/native/` (crate de teste, `cargo test --offline`) e `source/scripts/package.py` (monta os
  `.hnmod` e o `SHA256SUMS.txt`).

## Licenças
- Torchmaster © Xalcon, MIT (`source/content/torchmaster/LICENSE-torchmaster.txt`). O código foi consultado no
  GitHub (xalcon/TorchMaster, branch mc/1.21.1) para os números e as regras, e reescrito em Rust. O código nativo
  deste port também é MIT.

## Testado no jogo (11/11/2026)
- Mega Tocha: nenhum monstro nasceu no cubo de 64 blocos em 100 s de noite; fora dele, ~25.
- Loot dos blocos convertido para o formato do 26.3 (`source/scripts/upgrade_hmod.py`).

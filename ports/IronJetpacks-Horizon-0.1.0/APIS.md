# APIs do Horizon usadas pelo `ironjetpacks_native` 0.1.0

Conferidas no código do Horizon `66f3981f522b24c250b21523a70080965b940424` (caminhos relativos a
`crates/horizon-client/src/`, salvo indicação). Mixins: horizon-mixins `5a64af93`.

Todas as chamadas foram compiladas contra stubs que copiam estas assinaturas
(`source/native/glue_check`, 18 testes), mas o Horizon real não foi compilado aqui.

## Pontos de injeção (todos HEAD, sem `cancellable`)

| Injeção | Alvo | Arquivo:linha | Assinatura usada |
|---|---|---|---|
| tick | `Game::native_tick` | game/native_hooks.rs:2279 | `(&mut self)` — chamado em todo `Game`; o glue só age com `native_window()` |
| movement-input | `Game::native_movement_input` | game/native_hooks.rs:2728 | `(&self, forward: &mut f32, strafe: &mut f32, jump: &mut bool, sneak: &mut bool)`; binding por valor `*forward` (desreferência explícita, permitida por docs/mixin-api.md) |
| key | `Game::native_key` | game/native_hooks.rs:2617 | `(&mut self, key: &str, pressed: bool)` (igual ao Backpacks 0.2.0) |
| hud | `Game::native_hud` | game/native_hooks.rs:2508 | `(&mut self, batch: &mut HudBatch, corners: &mut Corners)`; o helper recebe `&*self` |
| tooltip | `Game::native_tooltip` | game/native_hooks.rs:6515 | `(&self, stack: &ItemStack, lines: &mut Vec<(Text, u32)>)` |
| item-bar | `Game::native_item_bar` | game/native_hooks.rs:6560 | `(&self, stack: &ItemStack, bar: &mut Option<(i32, u32)>)` |
| net-from-client | `Game::native_net_from_client` | game/native_hooks.rs:6970 | `(&mut self, player: u128, channel: &str, data: &[u8])` (igual ao Backpacks) |
| player-tick | `Game::native_player_tick` | game/native_hooks.rs:2595 | `(&mut self)` (igual ao Backpacks); chamado em game/lan_actions.rs:3998 dentro de `as_guest` |
| server-tick | `Game::native_server_tick` | game/native_hooks.rs:2489 | `(&mut self, context, phase)`; só `self` é ligado (igual ao Backpacks) |
| right-click | `Game::native_right_click_block` | game/native_hooks.rs:6892 | `(&mut self, pos: [i32; 3], face: [i32; 3], canceled: &mut bool)` (igual ao Mekanism) |
| list | `Game::native_mods` | game/native_hooks.rs:3039 | `(&mut self, mods: &mut Vec<NativeMod>)` |

Módulos de apoio (`[[module]]`): `ironjetpacks_rt` (runtime.rs, só std) e `ironjetpacks_glue` (glue.rs), em
`crates/horizon-client/src/game.rs`, emitidos como `pub(crate) mod … { … }`. Sem atributos de arquivo
(`#![…]`), que o transformador recusa (horizon-mixins docs/boundary-head-and-modules.md).

## Helpers chamados

| Função | Arquivo:linha | Uso |
|---|---|---|
| `native_push_player(&mut self, kick: [f32; 3])` | game/native_motion.rs:31 | o empurrão do jetpack na velocidade do jogador da janela (API 105) |
| `native_window(&self) -> bool` | game/native_hooks.rs:8872 | só a janela empurra |
| `native_on(&self, &str) -> bool` | game/native_hooks.rs:8558 | mod ligado |
| `native_playing(&self) -> bool` | game/native_hooks.rs:8865 | teclas só jogando |
| `native_mod_key(&self, &str, &str, &str) -> String` | game/native_hooks.rs:9280 | teclas configuráveis |
| `native_player(&self) -> Option<NativePlayer>` | game/native_hooks.rs:9236 | `yaw` (graus, 0 = sul) |
| `native_net_send_to_server(&mut self, &str, &[u8]) -> Result<(), String>` | game/native_net.rs:139 | canais `ironjetpacks_native:input` e `:toggle` (API 40) |
| `native_acting_player(&self) -> Option<NativeServerPlayer>` | game/native_server.rs:446 | uuid e `game_mode` |
| `native_action_bar(&mut self, u128, Text) -> bool` | game/native_server.rs:234 | "Motor: LIGADO", carga |
| `native_block(&self, [i32; 3]) -> String` | game/native_hooks.rs:9161 | o bloco clicado é cubo/célula? |
| `native_with_mod_block_entity<R>(&mut self, [i32; 3], impl FnOnce(&mut ModBlockEntity) -> R) -> Option<R>` | game/native_hooks.rs:8826 | lê/grava `energy` (Long) do cubo/célula |
| `native_corner_shown`, `native_text_width`, `native_fill`, `native_text` | game/native_hooks.rs:8916, 8921, 8925, 9222 | o HUD |
| `Corners::place(&mut self, Corner, f32, f32) -> (f32, f32)` | extras_hud.rs:30 | posição da caixa do HUD |
| `Text::literal`, `Text::legacy` | horizon-client-core/src/text.rs:191, 409 | textos |
| `StackData::other` / `set_other` / `wire`, `ItemStack::data` / `data_mut` / `tidy` | horizon-items/src/item_data.rs:136, 165, 69, 267, 280, 284 | `minecraft:custom_data` do jetpack (mesmo padrão do Backpacks e de game/native_item_energy.rs) |

## Campos lidos/escritos (módulo filho de `game`, acesso a campos privados)

| Campo | Arquivo:linha | Uso |
|---|---|---|
| `Game.player: Option<PlayerMotion>` | game.rs:2454 | velocidade, chão, água, planar, corrida (janela); `fall_distance = 0` (servidor) |
| `Game.armor: [Option<ItemStack>; 4]` (cabeça, peito, pernas, pés) | game.rs:2740 | o jetpack é `armor[1]`; no servidor é o do jogador da vez (trocado em game/lan_actions.rs `enter_guest`) |
| `Game.inventory`, `Game.selected`, `Game.hand` | game.rs:2464, 2465, 2748 | o jetpack na mão principal ao clicar no cubo |
| `Game.creative: bool` | game.rs:2497 | criativo voa sem energia (janela) |
| `Game.lan: Option<lan::LanHost>` → `hub.guests[..].above_ground_ticks`, `uuid` | game.rs:3074, game/lan.rs:226, lan/hub.rs:296, lan/guest.rs:193/107 | zera o contador de "flutuando" (kick após 80 ticks em servidor sem allow-flight) |
| `PlayerMotion.velocity / ground.on_ground / fall_distance / spectator / fall_flying / sprinting / fluids.was_touching_water` | motion.rs:287, 288, 291, 311, 331, 356, 400; motion/fluids.rs:87; horizon-world/src/ground_support.rs:47 | |
| `ModBlockEntity` → `nbt` (Deref/DerefMut para `ModBlockEntityFields`) | horizon-client-render/src/chunk/block_entity.rs:183-194 | |

## Riscos de compilação que vejo

- **Baixo**: tudo acima foi conferido na assinatura exata e compilado contra stubs. Os hooks novos para os ports do
  usuário são `native_tick`, `native_movement_input`, `native_hud`, `native_tooltip`, `native_item_bar` (nenhum
  mod instalado os usa ainda). O mais "diferente" é o binding `*forward` (desreferência de `&mut f32`), que a
  documentação do horizon-mixins permite ("explicit dereference").
- Acesso a campos privados de `Game` (`player`, `armor`, `creative`, `lan`, `hand`) a partir do módulo filho: é o
  mesmo mecanismo que o Backpacks 0.2.0 usa (`game.inventory`, `game.offhand`, `game.dropped`, `game.lan`).
- `lan::LanHost` é `pub(super)` em `game::lan`: visível em `game` e filhos (o Backpacks já usa `game.lan`/`host.hub`).

## Riscos de comportamento

- O empurrão vai na velocidade do cliente (o movimento é do cliente, como no vanilla). Num servidor LAN/dedicado, o
  anticheat de movimento do servidor vê um movimento que não sabe explicar: o Horizon só registra no log, a menos que o
  servidor esteja configurado para puxar o jogador de volta (docs/native-mods.md, "A player's motion").
- Num servidor sem `allow-flight`, o glue zera `above_ground_ticks` todo tick em que o jetpack voa (como o original
  zera `aboveGroundTickCount`). No mundo single player (`allow_flight: true`, game/integrated.rs:757) isso nem é preciso.

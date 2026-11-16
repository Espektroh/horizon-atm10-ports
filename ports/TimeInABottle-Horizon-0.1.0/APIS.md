# APIs do Horizon usadas por `tiab_native` 0.1.0

Conferidas no código de `horizon-rust/horizon` em **66f3981f522b24c250b21523a70080965b940424** (caminhos
relativos a `crates/`) e horizon-mixins **5a64af9**. Compilado de verdade junto com todos os outros mods
pelo CI (`tools/ci_check.sh` do repositório privado, branch `wip/tiab`): veja o LEIA-ME.

## Ganchos (injeções HEAD, alvo `horizon_client::game::native_hooks::Game<'_>::...`)

| Injeção | Assinatura (66f3981) | Onde | Já usado por |
|---|---|---|---|
| `use-item-on` | `native_use_item_on(&mut self, stack: &ItemStack, pos: [i32; 3], face: [i32; 3], claimed: &mut bool)` | `horizon-client/src/game/native_hooks.rs:6584` | MysticalAgriculture 0.1.0/0.2.0 (testado) |
| `inventory-tick` | `native_inventory_tick(&mut self, slot: i32, selected: bool)` | `native_hooks.rs:7716` | — (primeiro uso) |
| `tooltip` | `native_tooltip(&self, stack: &ItemStack, lines: &mut Vec<(Text, u32)>)` | `native_hooks.rs:6515` | IronJetpacks 0.1.0 |
| `level-tick-post` | `native_level_tick_post(&mut self, dimension: &str)` (chamado com o nível já carregado, `game/levels.rs:505`; só no servidor que roda o mundo, inclusive o integrado do single player) | `native_hooks.rs:2501` | — (primeiro uso) |
| `server-start` | `native_server_about_to_start(&mut self, dedicated: bool)` | `native_hooks.rs:2437` | — (primeiro uso) |
| `list` | `native_mods(&mut self, mods: &mut Vec<NativeMod>)` | `native_hooks.rs:3039` | todos |

## Funções e campos usados no glue (`crate::game::tiab_glue`)

| Uso | Onde (66f3981) | Já usado por |
|---|---|---|
| `Game::native_on(&self, &str) -> bool` | native_hooks.rs:8558 | todos |
| `Game::native_carried_stack(&mut self, slot: i32) -> Option<&mut ItemStack>` | native_hooks.rs:8552 | — |
| `Game::native_ticks(&self, name: &str) -> bool` (o bloco do mod tem block entity com tick) | native_hooks.rs:9776 | — (o próprio `ticking_states`) |
| `Game::tick_mod_block_entity(&mut self, pos: Pos)` (um tick de block entity de mod nativo: Iron Furnaces, Mekanism...) | native_hooks.rs:10034 | — (o próprio `tick_block_entities`) |
| `Game::native_set_block_property_with(pos, key, value, flags) -> bool` (estado `lit` da fornalha) | native_hooks.rs:8749 | MysticalAgriculture |
| `GameLevel::native_play_sound(&mut self, &str, [f64; 3], usize, f32, f32) -> bool` (nota de harpa) | native_hooks.rs:8295 | — |
| `Game::campfire_drop(&mut self, Pos, ItemStack)` (sobra de combustível que a fornalha derruba) | game/campfire.rs:184 | — (o próprio `tick_furnace`) |
| `Game::native_acting_player()` (`game_mode`, `uuid`), `Game::native_action_bar(uuid, Text)` | game/native_server.rs:446, :234 | MysticalAgriculture, simplemagnets |
| `Game::native_dimension() -> &'static str` | game/machines/native_items.rs:33 | MysticalAgriculture |
| `Game::held_mut(&mut self) -> &mut Option<ItemStack>` (privada; glue é filho de `game`) | game.rs:3715 | MysticalAgriculture, simplemagnets |
| `Game::with_level(...)`, `Game::process_level_events()` (privadas) | game.rs:4589, :5321 | MysticalAgriculture 0.2.0 (regadores) |
| `Behaviours::random_tick(&self, &mut L, &mut Ticks, BlockPos)` | horizon-world/src/level/tick.rs:1912 | MysticalAgriculture 0.2.0 |
| Campos privados de `Game`: `inventory: Vec<Option<ItemStack>>` :2464, `armor: [Option<ItemStack>; 4]` :2740, `offhand` :2742, `world_tables` :2822, `tickets: ChunkTickets` :2853, `world` :2430, `res` :2385 | game.rs | (o próprio game.rs) |
| `ChunkTickets::levels() -> Levels`, `Levels::block_ticking_at(x, z)` | horizon-client/src/ticket_levels.rs:267, :228 | — (o próprio `block_ticking_area`) |
| `world.block(x, y, z) -> BlockStateId`, `world.game_time`, `world.block_entities.{get, get_mut, is_asleep}` | horizon-client-render/src/world.rs:394, :98, :84; block_entities.rs:298, :301, :318 | MysticalAgriculture |
| `BlockEntity::{Furnace, Mod}` | horizon-client-render/src/chunk/block_entity.rs:44 | MysticalAgriculture |
| `<Furnace as horizon_client::furnace::FurnaceCooking>::tick(&mut self, &Recipes, &Items, &Providers, &mut Vec<ItemStack>) -> Option<bool>` | horizon-client/src/furnace.rs:26 | — (o próprio `tick_furnace`, block_entity_ticks.rs:218) |
| macros `world_recipes!`, `world_items!` | game.rs:17, :9 | MysticalAgriculture (`world_items!`) |
| `Resources::facts()` :2036 (`StateFacts.name`, `random_ticks`: horizon-items/src/blocks.rs:42/:52), `Resources::providers()` :2060 | game.rs | MysticalAgriculture (`facts`) |
| `ItemStack::{data, data_mut, tidy}`, `StackData::{other, set_other, wire}` | horizon-items/src/item_data.rs:267, :280, :284, :136, :165, :69 | SophisticatedBackpacks 0.2.0 |
| `Tag::{Compound, Int, ...}`, `Compound = BTreeMap<JavaString, Tag>` | horizon-world/src/nbt.rs:42, :35 | SophisticatedBackpacks |
| `Text::{translate_with_fallback, literal, with_color}` | horizon-client-core/src/text.rs:251, :191, :374 | — |
| `horizon_world::level::flags::UPDATE_ALL` | horizon-world/src/level/mod.rs:163 | MysticalAgriculture |

## Riscos que vejo

- O CI compila; o comportamento em jogo ainda precisa do roteiro de teste (LEIA-ME).
- `native_inventory_tick`, `native_level_tick_post` e `native_server_about_to_start` são usados aqui pela
  primeira vez nos ports.
- O tick extra da fornalha vanilla reproduz `tick_furnace` (privada em `block_entity_ticks.rs`), sem os
  eventos `native_furnace_*` que outros mods ouvem. Se a próxima Nightly mudar `tick_furnace`, revisar.
- `GameLevel::native_play_sound` só toca no servidor autoritativo (é o caso: o uso é server-side).

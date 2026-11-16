# APIs do Horizon usadas por `mysticalagriculture_native` 0.1.0

Conferidas no código de `horizon-rust/horizon` em **66f3981f522b24c250b21523a70080965b940424** (caminhos relativos
a `crates/`) e horizon-mixins **5a64af9**. O glue e os helpers também foram compilados contra cópias dessas
assinaturas (`source/native/tools/stubcheck`, `cargo test`: 15 testes passam), o que pega erro de tipo e de
empréstimo no código do mod, mas não substitui o build real no launcher.

## Ganchos (injeções HEAD)

| Injeção | Alvo | Onde (66f3981) | Já usado por |
|---|---|---|---|
| `loot-drops` | `Game::native_loot_drops(&mut self, context: &NativeLootDrops<'_>, drops: &mut Vec<ItemStack>)` | `horizon-client/src/game/native_hooks.rs:7560`; struct `horizon-client/src/mod_api.rs:143` | — (novo para você) |
| `use-item-on` | `Game::native_use_item_on(&mut self, stack: &ItemStack, pos: [i32; 3], face: [i32; 3], claimed: &mut bool)` | `native_hooks.rs:6584` | (seu `native_use_item` é o irmão) |
| `random-tick` | `GameLevel::native_random_tick(&mut self, ticks, dimension, pos: BlockPos, state: BlockStateId, handled: &mut bool)` | `native_hooks.rs:1583` | — |
| `bone-meal` | `GameLevel::native_is_valid_bonemeal_target(&self, dimension, pos, state, valid: &mut Option<bool>)` | `native_hooks.rs:1768` | Inferium Lab (validado em 66f3981) |
| `remainders` (cancelável) | `horizon_client::crafting::Recipes::remainders(&self, recipe: &str, grid: &[Option<ItemStack>], size: usize) -> Vec<Option<ItemStack>>` | `horizon-client/src/crafting.rs:1899` | atm_tools_native (mesmo alvo e mesmo padrão) |
| `list` | `Game::native_mods(&mut self, mods: &mut Vec<NativeMod>)` | `native_hooks.rs:3039`; `NativeMod` em `mod_api.rs:307` | todos os seus mods |
| `crop-seed` | `horizon_client_lib::mod_content::Plan<'_>::override_gameplay(&self, b: &NewBlock, combo, row: &mut J)` | `horizon-client/src/mod_content.rs:1319` | Inferium Lab |
| `seed-crop` | `horizon_client_lib::mod_content::J::rename(&mut self, r: &Rename, ids: bool)` | `mod_content.rs:237` (`Rename` :263, `NewBlock` :634, `J` :101) | Inferium Lab |

## Helpers e campos usados no glue (`crate::game::mysticalagriculture_glue`)

| Uso | Onde |
|---|---|
| `game.native_on(&str) -> bool` | `native_hooks.rs:8558` (`pub(super)`, impl `Game`) |
| `game.native_dimension() -> &'static str` | `game/machines/native_items.rs:33` (`pub(in crate::game)`) |
| `game.native_block([i32;3]) -> String` | `native_hooks.rs:9161` |
| `game.native_block_property([i32;3], &str) -> Option<String>` | `native_hooks.rs:9184` |
| `game.native_set_block_property_with([i32;3], &str, &str, i32) -> bool` | `native_hooks.rs:8749` |
| `game.native_set_block(&str, [i32;3], BlockStateId, i32) -> bool` | `native_hooks.rs:8566` |
| `game.native_acting_player() -> Option<NativeServerPlayer>` (`game_mode`) | `game/native_server.rs:446`; struct `mod_api.rs:373` |
| `game.native_take_items(&str, u32)`, `game.native_give_items(&str, u32)` | `game/native_server.rs:657`, `:682` |
| `game.res.facts().get(state).name` | `game.rs:2385` (`res`), `game.rs:2036` (`facts`), `horizon-items/src/blocks.rs:516` / `:42` (`name: Arc<str>`) — mesmo uso do atm_tools |
| `game.pack.assets.registry.default_state(&str)` | `game.rs:2420` (`pack`), `game.rs:2178` (`assets`), `horizon-client-render/src/assets.rs:76` (`pub registry`), `horizon-world/src/registry.rs:170` |
| `level.facts`, `level.world.block(x, y, z)` (`GameLevel`) | `game.rs:949-952`; `horizon-client-render/src/world.rs:394` |
| `horizon_world::level::flags::{UPDATE_CLIENTS, UPDATE_ALL}` | `horizon-world/src/level/mod.rs:152`, `:163` |
| `ItemStack::new(&str, u32)`, campos `item`, `count`, `damage` | `horizon-items/src/item_data.rs:259`, `horizon-items/src/items.rs:331` |
| `horizon_core::position::BlockPos { x, y, z }` (Copy) | `horizon-core/src/position.rs:6` |
| Tag de bloco `minecraft:supports_crops` (onde a semente pode ser plantada) | `horizon-world/src/block_survival.rs:503`, `level/survive.rs:386` |
| Tag `minecraft:maintains_farmland` (terra com cultivo não seca) | `horizon-world/src/level/tick.rs:1019` |
| Tag `minecraft:grows_crops` (terra de essência conta como terra arada na velocidade de crescimento, `CropBlock.getGrowthSpeed`) | `horizon-world/src/level/tick.rs:821`, `level/data.rs:461` |

## Riscos de compilação que eu vejo

1. **`crop-seed` / `seed-crop`** miram funções privadas do carregador de conteúdo (`mod_content.rs`), não pontos de
   gancho estáveis. Funcionam em 66f3981 (o Inferium Lab compilou e validou exatamente esses dois alvos, com
   horizon-mixins e8cd0aa; aqui o pin é 5a64af9, mais novo, que só acrescentou a resolução por nome da crate).
   Os helpers são autocontidos (rodam na crate `horizon_client_lib`, sem `crate::game`). Se o Horizon mudar
   `NewBlock`, `J` ou `Rename`, o build quebra: ao atualizar a Nightly, confira esses dois primeiro.
2. **`remainders`** tem dois mods no mesmo HEAD cancelável (atm_tools_native, prioridade 90, roda antes; este,
   80). Cada um só age quando o seu item está na grade e chama `recipes.remainders` de novo com uma trava
   própria (`thread_local`), então os dois convivem. O launcher vai avisar "both inject at ... HEAD" — é esperado.
3. Campos privados de `Game`/`GameLevel` (`res`, `pack`, `world`, `facts`) são lidos do módulo do glue, que o
   horizon-mixins coloca dentro de `game.rs` (filho de `game`, como o glue do atm_tools, que lê `game.res`,
   `game.world`). `pack.assets.registry` é o único caminho novo: `pack` e `assets` são campos privados
   definidos em `game.rs`, `registry` é `pub`.
4. `native_dimension` é `pub(in crate::game)`: visível no glue, que está dentro de `crate::game`.
5. O módulo `runtime.rs` traz `#[cfg(test)] mod tests` (como o glue do atm_tools); só entra em `cargo test`.
6. Nada de `use` no topo de arquivos de helper; os módulos `mysticalagriculture_rt`/`_glue` e as funções
   `mysticalagriculture_*` têm prefixo único.

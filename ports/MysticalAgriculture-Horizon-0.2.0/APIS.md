# APIs do Horizon usadas por `mysticalagriculture_native` 0.2.0

Conferidas no código de `horizon-rust/horizon` em **66f3981f522b24c250b21523a70080965b940424** (caminhos relativos
a `crates/`) e horizon-mixins **5a64af9**. O glue e os helpers também foram compilados contra cópias dessas
assinaturas (`source/native/tools/stubcheck`, `cargo test --offline`: 26 testes passam, os 22 da lógica
mais 4 do glue), o que pega erro de tipo e de
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

## Novas na 0.2.0 (Altar de Infusão, pedestais, varinha, regadores)

### Ganchos (injeções HEAD, alvo `horizon_client::game::native_hooks::Game<'_>::...`)

| Injeção | Assinatura em 66f3981 (`horizon-client/src/game/native_hooks.rs`) | Helper | Já usado por |
|---|---|---|---|
| `block-entity` | `native_block_entity(&mut self, block: &str, entity: &mut Option<(String, Compound)>)` :3065 | block_entity.rs | Mekanism 0.6.1, IronFurnaces (testados) |
| `ticks` | `native_block_entity_ticks(&self, block: &str, ticks: &mut bool)` :3076 | ticks.rs | Mekanism 0.6.1, IronFurnaces |
| `container` | `native_block_entity_container(&self, block: &str, container: &mut Option<ModContainer>)` :3115 | container.rs | Mekanism 0.6.1, IronFurnaces |
| `tick` | `native_block_entity_tick(&mut self, pos: [i32; 3], entity: &mut ModBlockEntity)` :3202 | tick.rs | Mekanism 0.6.1, IronFurnaces |
| `update-tag` | `native_block_entity_update_tag(pos: [i32; 3], entity: &ModBlockEntity, tag: &mut Option<Compound>)` :3221, **sem self** | update_tag.rs | FunctionalStorage 0.1.0 (não testado no jogo) |
| `submit` | `native_block_entity_submit(&self, pos, block: &str, state: BlockStateId, entity: &ModBlockEntity, draw: &mut BlockEntityDraw)` :3270 | submit.rs | FunctionalStorage 0.1.0 (não testado) |
| `right-click` | `native_right_click_block(&mut self, pos: [i32; 3], face: [i32; 3], canceled: &mut bool)` :6892 | right_click.rs | Mekanism 0.6.1 |

O regador entra pelo `use-item-on` que a 0.1.0 já usa (mesmo helper, ramo novo no glue).

### Funções e campos usados no glue

| Uso | Onde (66f3981) | Já usado por |
|---|---|---|
| `Game::native_neighbor_signal(&mut self, dimension: &str, pos) -> i32` (redstone do altar, como `hasNeighborSignal`) | native_hooks.rs:8603 | — (primeiro uso) |
| `Game::native_with_mod_block_entity(pos, FnOnce(&mut ModBlockEntity) -> R) -> Option<R>` | native_hooks.rs:8826 | Mekanism 0.6.1 |
| `Game::native_block_entity_sync(&mut self, pos)` | native_hooks.rs:8856 | FunctionalStorage |
| `Game::native_recipes_of_type(&self, &str) -> &[CustomRecipe]` (`id`, `json: Arc<Value>`, crafting.rs:1290) | native_hooks.rs:9375 | Mekanism 0.6.1 |
| `Game::native_recipe_ingredient(&self, &Value) -> Option<Ingredient>`, `Ingredient::test(&self, &ItemStack) -> bool` | native_hooks.rs:9395; crafting.rs:210 | Mekanism 0.6.1 |
| `Game::native_recipe_result(&self, &Value) -> Option<ItemStack>` | native_hooks.rs:9425 | Mekanism 0.6.1 |
| `Game::native_acting_id(&mut self) -> u128` | native_hooks.rs:9484 | FunctionalStorage |
| `Game::native_action_bar(&mut self, uuid, Text) -> bool` | game/native_server.rs:234 | simplemagnets_native (testado) |
| `Game::native_give_stack(&mut self, uuid, ItemStack) -> Option<ItemStack>` (o que não coube volta) | game/native_server.rs:707 | FunctionalStorage |
| `Game::native_sneaking(&self) -> bool` | game/native_host_bridge.rs:119 | FunctionalStorage |
| `Game::held_mut(&mut self) -> &mut Option<ItemStack>` (privada; glue é filho de `game`) | game.rs:3715 | simplemagnets_native (testado) |
| `Game::with_level(&mut self, FnOnce(&Behaviours, &mut GameLevel, &mut Ticks) -> R) -> R` (privada) | game.rs:4589 | — (primeiro uso; o próprio game.rs usa assim, ex. :5114, :5212) |
| `Behaviours::random_tick<L: LevelAccess + ?Sized>(&self, &mut L, &mut Ticks, BlockPos)` (o random tick do regador) | horizon-world/src/level/tick.rs:1912 (`impl Behaviours` :1724) | — |
| `Game::process_level_events(&mut self) -> Result<()>` (privada; depois do random tick, como `native_set_block`) | game.rs:5321 | — |
| `Game::native_draw_item(&self, draw, &ItemStack, DisplayContext, Option<f32>, i32) -> bool` | game/native_block_entity_draw.rs:722 | FunctionalStorage |
| `BlockEntityDraw` (`partial`, `game_time`, `light` pub; `push`, `pop`, `translate`, `rotate_about`, `scale`) | native_block_entity_draw.rs:47-55, 169-196; reexport game.rs:437 | FunctionalStorage |
| `horizon_assets::item_model::DisplayContext::Ground` | horizon-assets/src/item_model.rs:159 (dependência em horizon-client/Cargo.toml:16) | FunctionalStorage (`Fixed`) |
| `World.block_entities.get(&(x, y, z))`, `World.game_time`, `World::light`, `World::block` | horizon-client-render/src/world.rs:84, 98, 446, 394; block_entities.rs:298 | FunctionalStorage, Mekanism |
| `containers::{BlockEntity::Mod, ModBlockEntity (Deref para id, nbt, items), ModContainer {slots, faces, inputs, outputs, menu}}` | horizon-client-render/src/chunk/block_entity.rs:44, 108, 118, 202 | Mekanism 0.6.1 |
| Conteúdo de um bloco com `ModContainer` cai no chão quando ele quebra | game/spill.rs:512-530 | (comportamento do Horizon) |
| `Items::get`, `ItemInfo.block`, `ItemInfo.remainder` (resto de crafting: balde) | horizon-items/src/items.rs:994, 146, 174 | Mekanism (`max_stack`) |
| macro `world_items!` | game.rs:9 | Mekanism 0.6.1 |
| `item_data::{encode_components, decode_components}` | horizon-items/src/item_data.rs:715, 938; reexport client_lib.rs:176 | FunctionalStorage |
| `text::Text::{literal, translate, append}` | horizon-client-core/src/text.rs:191, 239, 276; reexport client_lib.rs:159 | simplemagnets (`literal`) |
| `nbt::{Compound, JavaString, Tag}` | horizon-world/src/nbt.rs:6, 35, 42 | Mekanism 0.6.1 |
| `horizon_world::level::flags::UPDATE_ALL` | horizon-world/src/level/mod.rs:163 | 0.1.0 |
| `BlockPos::new` | horizon-core/src/position.rs:13 | — |

### Conteúdo (API 0.2)
- Altar e pedestal: template `minecraft:stone` com `properties.occludes: false` (horizon-mods/src/content.rs:95) e
  `shapes.default` (content.rs:150-195, no máximo 16 caixas; o altar do original tem 21, juntei em 4). O block entity vem
  do nativo (`native_block_entity` funciona em qualquer template, docs/native-mods.md).
- Receitas `mysticalagriculture:infusion` em `data/mysticalagriculture/recipe/seed/infusion/*.json`: tipo fora de
  `minecraft`, ficam carregadas como estão e o nativo lê com `native_recipes_of_type` (docs/native-mods.md:312, API 36).
  Os ingredientes são strings (`"minecraft:iron_ingot"`, `"#minecraft:logs"`), que é o que `native_recipe_ingredient` lê
  (native_hooks.rs:9395-9424 aceita string ou lista).

### Riscos de compilação novos
1. **`with_level` e `process_level_events`** são privadas de `Game` (game.rs) e é o primeiro mod seu a usá-las. O glue
   fica dentro de `game.rs` (módulo filho de `crate::game`), então a privacidade deixa. Se o build reclamar delas, o
   erro cita `water` no glue: o regador é o único que as usa (dá para apagar o bloco final de `water` e o regador
   continua molhando a terra).
2. **`update-tag`** é uma injeção HEAD numa função sem `self` (como a do FunctionalStorage). Se o horizon-mixins recusar,
   o erro cita `update-tag`; sem ela o altar funciona, só não desenha os itens (o estado aparece na action bar).
3. **`submit`/`native_draw_item`/`BlockEntityDraw`** e `native_neighbor_signal`, `native_give_stack`, `native_sneaking`
   não estão em nenhum mod seu testado; as assinaturas foram conferidas linha a linha (tabela acima) e o stubcheck
   compila contra cópias delas.
4. Mais quatro mods no mesmo HEAD de `native_block_entity*`/`native_right_click_block` (Mekanism, IronFurnaces, e
   FunctionalStorage se instalado): cada um só age nos seus blocos; o launcher vai avisar "both inject at" — esperado.

## Riscos de compilação da 0.1.0 (continuam valendo)

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

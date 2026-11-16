# APIs do Horizon usadas pelo `functionalstorage_native` 0.1.0

Todas conferidas no código do Horizon `66f3981f522b24c250b21523a70080965b940424` (só leitura). Caminhos relativos a
`crates/`. "Usado por" indica um mod que já compila e roda no jogo com a mesma API (referência de risco).

## Pontos de injeção (HEAD, horizon-mixins `5a64af9`)

| Alvo (`horizon_client::game::native_hooks::Game<'_>::...`) | Assinatura (arquivo:linha) | Helper | Usado por |
|---|---|---|---|
| `native_block_entity` | `(&mut self, block: &str, entity: &mut Option<(String, Compound)>)` — horizon-client/src/game/native_hooks.rs:3065 | block_entity.rs | Mekanism 0.6.1 |
| `native_block_entity_ticks` | `(&self, block: &str, ticks: &mut bool)` — native_hooks.rs:3076 | ticks.rs | Mekanism 0.6.1 |
| `native_block_entity_container` | `(&self, block: &str, container: &mut Option<ModContainer>)` — native_hooks.rs:3115 | container.rs | Mekanism 0.6.1 |
| `native_block_entity_tick` | `(&mut self, pos: [i32; 3], entity: &mut ModBlockEntity)` — native_hooks.rs:3202 | tick.rs | Mekanism 0.6.1 |
| `native_container_accepts` | `(&self, pos, entity: &ModBlockEntity, slot: usize, stack: &ItemStack, face: Option<Direction>, cancel: &mut bool)` — native_hooks.rs:3238 | accepts.rs | IronFurnaces-Horizon e HostileNeuralNetworks-Horizon (terceiros) |
| `native_block_entity_update_tag` | `(pos: [i32; 3], entity: &ModBlockEntity, tag: &mut Option<Compound>)`, **sem self** — native_hooks.rs:3221 | update_tag.rs | Elevator-Horizon (terceiro, via hook nomeado) |
| `native_block_entity_submit` | `(&self, pos, block: &str, state: BlockStateId, entity: &ModBlockEntity, draw: &mut BlockEntityDraw)` — native_hooks.rs:3270 | submit.rs | Elevator-Horizon (terceiro) |
| `native_right_click_block` | `(&mut self, pos: [i32; 3], face: [i32; 3], canceled: &mut bool)` — native_hooks.rs:6892 | right_click.rs | Mekanism 0.6.1 |
| `native_left_click_block` | `(&mut self, pos, face: [i32; 3], state: BlockStateId, canceled: &mut bool)` — native_hooks.rs:2581 | left_click.rs | (primeiro uso nos seus mods) |
| `native_destroy_block` | `(&mut self, pos, state: BlockStateId, canceled: &mut bool)` — native_hooks.rs:6865 | destroy.rs | Mekanism 0.6.1 |
| `native_explosion_detonate` | `(&mut self, dimension: &str, center: [f64; 3], blocks: &mut Vec<[i32; 3]>)` — native_hooks.rs:6905 | explosion.rs | (primeiro uso) |
| `native_mods` | `(&mut self, mods: &mut Vec<NativeMod>)` — native_hooks.rs:3039 | list.rs | todos |

## Funções e campos usados no `glue.rs`

| Item | Onde (66f3981) |
|---|---|
| `Game::native_with_mod_block_entity(pos, FnOnce(&mut ModBlockEntity) -> R) -> Option<R>` | horizon-client/src/game/native_hooks.rs:8826 |
| `Game::native_block_entity_sync(&mut self, pos)` | native_hooks.rs:8856 |
| `Game::native_block_property(&self, pos, key) -> Option<String>` | native_hooks.rs:9184 |
| `Game::native_acting_id(&mut self) -> u128` | native_hooks.rs:9484 |
| `Game::native_action_bar(&mut self, uuid: u128, line: Text) -> bool` | horizon-client/src/game/native_server.rs:234 |
| `Game::native_acting_player(&self) -> Option<NativeServerPlayer>` (`uuid`, `position`, `yaw`, `pitch`) | native_server.rs:446; struct em horizon-client/src/mod_api.rs:373 |
| `Game::native_give_stack(&mut self, uuid, stack) -> Option<ItemStack>` (o que não coube) | native_server.rs:707 |
| `Game::native_sneaking(&self) -> bool` | horizon-client/src/game/native_host_bridge.rs:119 |
| `Game::held_mut(&mut self) -> &mut Option<ItemStack>` (privada; o glue é módulo filho de `game`) | horizon-client/src/game.rs:3715 (usada pelo seu simplemagnets_native) |
| campos privados `world` 2430, `res` 2385, `inventory` 2464, `offhand` 2742, `dim` 2818; `Dim::id(self)` 3297; `Resources::facts()` 2036 | horizon-client/src/game.rs |
| macro `world_items!` | horizon-client/src/game.rs:9 (usada pelo Mekanism) |
| `pub use native_block_entity_draw::BlockEntityDraw` | game.rs:437 |
| `BlockEntityDraw.light` (pub), `push` 169, `pop` 174, `translate` 180, `rotate_about` 186, `scale` 196 | horizon-client/src/game/native_block_entity_draw.rs:55 e linhas citadas |
| `Game::native_draw_text(&self, draw, &Text, [f32; 2], u32, bool, u32) -> f32` | native_block_entity_draw.rs:582 |
| `Game::native_draw_item(&self, draw, &ItemStack, DisplayContext, Option<f32>, i32) -> bool` | native_block_entity_draw.rs:722 |
| `Game::native_block_state_property(&self, BlockStateId, &str) -> Option<&str>` | native_block_entity_draw.rs:745 |
| `horizon_assets::item_model::DisplayContext::Fixed` | horizon-assets/src/item_model.rs:159 |
| `World.block_entities` (pub) `.get(&(i32, i32, i32))`, `World.game_time`, `World::light(x, y, z) -> u8` | horizon-client-render/src/world.rs:84, 98, 446; block_entities.rs:31, 298 |
| `containers::{BlockEntity::Mod(Box<ModBlockEntity>), ModBlockEntity (Deref/DerefMut para id, nbt, items, data), ModContainer {slots, faces, inputs, outputs, menu}}` | horizon-client-render/src/chunk/block_entity.rs:91, 108, 118, 202 |
| `items::ItemStack {item: Arc<str>, count, damage, extra}`, `ItemStack::new(&str, u32)`, `custom_name()`, `same_item_same_components()` | horizon-items/src/items.rs:331; item_data.rs:259, 348, 408 |
| `items::Items::get(&str) -> Option<&ItemInfo>`, `ItemInfo.max_stack`, `ItemInfo.block: Option<BlockStateId>` | horizon-items/src/items.rs:994, 145, 147 |
| `BlockFacts::get(BlockStateId) -> &StateFacts`, `StateFacts.name: Arc<str>` | horizon-items/src/blocks.rs:516, 42 |
| `horizon_client::item_data::{encode_components, decode_components}` | horizon-items/src/item_data.rs:715, 938; reexportado em horizon-client/src/client_lib.rs:176 |
| `text::Text::{literal, translate, append}` | horizon-client-core/src/text.rs:191, 239, 276 |
| `nbt::{Compound = BTreeMap<JavaString, Tag>, JavaString::from(&str), to_string_lossy, Tag::{Byte, Int, Long, String, List, Compound, LongArray}}` | horizon-world/src/nbt.rs:6, 8, 30, 35, 42 |
| `registry::BlockStateId` | horizon-world/src/registry.rs:16 |

## Conteúdo (API 0.2)
- Bloco com template `minecraft:white_glazed_terracotta` e `properties` `hardness`, `explosion_resistance`, `sounds`,
  `requires_tool`, `map_color`: horizon-mods/src/content.rs:77 (`BlockProperties`). Block entity em template sem
  block entity: docs/native-mods.md (API 129, "It works on any template"); o desenho desses blocos passa por
  `untemplated_mod_block_entities` (native_block_entity_draw.rs:408).
- Ingrediente em lista (`["minecraft:chest", "minecraft:trapped_chest"]`) e tag (`#functionalstorage:drawer`):
  horizon-client/src/crafting.rs:98-118.

## Riscos de compilação que vejo
1. **`native_block_entity_update_tag` é função associada sem `self`.** É a primeira injeção HEAD sua num alvo sem
   receptor. O horizon-mixins aceita funções livres e métodos; não achei motivo para recusar esta, mas se recusar,
   o erro cita `update-tag` / `native_block_entity_update_tag`. Sem ela, as gavetas funcionam mas a frente não
   mostra o item (o cliente não recebe o conteúdo).
2. **Campos e métodos privados de `Game`** (`world`, `res`, `inventory`, `offhand`, `dim`, `held_mut`,
   `Resources::facts`): por isso o mod fixa o commit 66f3981 (como o mekanism_native e o simplemagnets_native).
3. **`horizon_client::item_data`** (reexport de `horizon_items::item_data`) e **`horizon_assets::item_model`**: o
   crate `horizon-client` depende dos dois (Cargo.toml linhas 16 e 21) e o próprio `game/` os usa.
4. Conferência de tipos: `source/native/tools/glue_mockcheck` (`python3 gen.py && cargo build --offline`) compila
   `runtime.rs`, `glue.rs` e cada helper, embutido no início de uma imitação do seu alvo com os bindings do
   manifesto, contra imitações com as assinaturas acima (e a mesma privacidade). Compila sem erro e sem aviso nos
   arquivos do mod. Não prova que as assinaturas reais são essas: isso é esta tabela.

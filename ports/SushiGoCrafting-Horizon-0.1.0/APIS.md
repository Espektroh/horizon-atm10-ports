# APIs do Horizon usadas por `sushigocrafting_native` 0.1.0

Conferidas no código de **66f3981** (`/home/claude/ext/horizon-66f3981`, arquivo:linha abaixo) e comparadas com o
**Nightly 2ebf00a** (mesma assinatura em todas). A prova final: o CI do repositório privado compilou o mod junto com
os outros 15 nativos nas duas bases (verde). Caminhos relativos a `crates/horizon-client/src/`, salvo indicação.

## Pontos de injeção (HEAD)

| Injeção | Alvo | Onde (66f3981) | Já usado por |
|---|---|---|---|
| block-entity | `Game::native_block_entity` | game/native_hooks.rs:3065 | Mekanism, MA |
| ticks | `Game::native_block_entity_ticks` | game/native_hooks.rs:3076 | Mekanism, MA |
| container | `Game::native_block_entity_container` | game/native_hooks.rs:3115 | Mekanism, MA |
| tick | `Game::native_block_entity_tick` | game/native_hooks.rs:3202 | Mekanism, MA |
| screen | `Game::native_screen` | game/native_hooks.rs:3409 | Mekanism |
| screen-draw | `Game::native_screen_draw` | game/native_hooks.rs:3419 | Mekanism |
| quick-move | `Game::native_menu_quick_move` | game/native_hooks.rs:3962 | Mekanism 0.6.1 |
| update-tag | `Game::native_block_entity_update_tag` (associada, sem `self`) | game/native_hooks.rs:3221 | MA |
| submit | `Game::native_block_entity_submit` | game/native_hooks.rs:3270 | MA |
| right-click | `Game::native_right_click_block` | game/native_hooks.rs:6892 | Mekanism, MA |
| use-item-on | `Game::native_use_item_on` | game/native_hooks.rs:6584 | MA |
| random-tick | `GameLevel::native_random_tick` (`&mut self`, `ticks: &mut Ticks`) | game/native_hooks.rs:1583 | MA (só leitura) |
| bone-meal | `GameLevel::native_is_valid_bonemeal_target` | game/native_hooks.rs:1768 | MA |
| may-place-on | `GameLevel::native_may_place_on` | game/native_hooks.rs:1668 | **novo** (API 70) |
| loot-drops | `Game::native_loot_drops` | game/native_hooks.rs:7560 | MA |
| fish | `Game::native_player_fish` (v1) | game/native_hooks.rs:4845 | **novo** (API 160; em 2ebf00a o v1 continua chamado antes do v2) |
| remainders | `horizon_client::crafting::Recipes::remainders` (cancellable) | crafting.rs:1899 | MA |
| list | `Game::native_mods` | game/native_hooks.rs:3039 | todos |
| crop-seed | `horizon_client_lib::mod_content::Plan::override_gameplay` | mod_content.rs:1319 | MA |
| seed-crop | `horizon_client_lib::mod_content::J::rename` | mod_content.rs:237 | MA |

## Funções e campos usados no glue

| API | Onde (66f3981) |
|---|---|
| `Game::native_on` | game/native_hooks.rs:8558 |
| `Game::native_block`, `native_block_property` | game/native_hooks.rs:9161, 9184 |
| `Game::native_set_block_property_with` | game/native_hooks.rs:8749 |
| `Game::native_with_mod_block_entity`, `native_block_entity_sync` | game/native_hooks.rs:8826, 8856 |
| `Game::native_recipes_of_type`, `native_recipe_ingredient`, `native_recipe_result` | game/native_hooks.rs:9375, 9395, 9425 |
| `Game::native_fill`, `native_text` | game/native_hooks.rs:8925, 9222 |
| `Game::native_acting_id` | game/native_hooks.rs:9484 |
| `Game::native_acting_player`, `native_action_bar`, `native_take_items`, `native_give_items`, `native_give_stack` | game/native_server.rs:446, 234, 657, 682, 707 |
| `Game::native_sneaking` | game/native_host_bridge.rs:119 |
| `Game::native_draw_item`, `BlockEntityDraw` (`push`, `pop`, `translate`, `rotate_about` em graus, `scale`, `light`) | game/native_block_entity_draw.rs:722, 169–196 |
| `Game::held_mut`, `spawn_item`, `with_level`, `process_level_events` (privadas de `game.rs`, visíveis ao módulo filho) | game.rs:3715, 4087, 4589, 5321 |
| `world_items!`, `Game.res.providers().burn_time(items, &ItemStack, false)`, `Game.res.facts()`, `Game.world.block_entities`, `Game.world.light` | game.rs:9; horizon-client-render/src/providers.rs:108 |
| `GameLevel` campos `world`, `facts`, `darken`, `behaviours`, `registry`, `native_authoritative` | game.rs:949–1021 |
| `GameLevel::native_set_block_property(ticks, pos, key, value, flags)`, `native_state_with_property` | game/native_hooks.rs:8405, 8474 |
| `Behaviours::set_block(level, ticks, pos, state, flags)` | horizon-world/src/level/mod.rs:904 |
| `BlockRegistry::state`, `default_state`; `BlockState.properties.get` | horizon-world/src/registry.rs:161, 170; block_state_properties.rs:94 |
| `horizon_world::level::{Ticks, flags::{UPDATE_ALL, UPDATE_CLIENTS}}` | horizon-world/src/level/mod.rs:147 |
| `ModBlockEntity` (`id`, `nbt`, `items`, `data`), `ModContainer` (`slots`, `faces`, `inputs`, `outputs`, `menu`), `BlockEntity::Mod` | horizon-client-render/src/chunk/block_entity.rs:108–208 |
| `NativeScreen` (`width`, `height`, `slots`, `inventory_y`, `inventory_x`, `own_labels`, `own_background`) | mod_api.rs:289 |
| `NativeLootDrops` (`table`, `block`, `entity`, `used`) | mod_api.rs:143 |
| `ItemStack` (`item`, `count`, `damage`, `extra`), `ItemStack::new`, `same_item_same_components` | horizon-items/src/items.rs:331; item_data.rs:259, 408 |
| `item_data::encode_components`, `decode_components` | horizon-items/src/item_data.rs |
| `Text::literal`, `Text::translate`, `append` | horizon-client (text) |
| `CustomRecipe.json`, `Ingredient::test` | crafting.rs:1290 |
| mod_content: `J::at`, `J::as_str`, `J::set`, `Rename.to`, `NewBlock.decl.id` | mod_content.rs:216, 230, 208, 263, 634 |

## Conteúdo (formato de dados que o mod usa)

- Blocos por template: `minecraft:seagrass` (arroz: `state_properties` age 0–3, `random_ticks` por estado),
  `minecraft:beetroots` (as outras quatro), `minecraft:oak_leaves` (+ `stage` 0–2, `random_ticks: true`),
  `minecraft:oak_log`, `minecraft:oak_sapling`, `minecraft:furnace` (as 4 máquinas; `shapes` por `facing` na tábua,
  `requires_tool: false`, `occludes: false`). horizon-mods/src/content.rs:77 (`BlockProperties`), 231 (`BlockFile`).
- Itens: `minecraft:food` e `minecraft:consumable` com `apply_effects` (horizon-items/src/items.rs:78, 795, 840);
  efeitos do mod pelo id (`potions.rs:114`, `mob_effect/*.json`).
- Receitas de tipos próprios (`sushigocrafting:cutting_board`, `sushigocrafting:fermenting_barrel`) caem em
  `Recipes.custom` (crafting.rs:1536) e o nativo lê com `native_recipes_of_type`.
- Geração: `worldgen/configured_feature`, `placed_feature` e `neoforge/biome_modifier` (docs/mods.md:429), passados
  pelo `upgrade_hmod.py` (feature sem `config`, `{id, properties}`, `weighted`).

## Riscos de compilação que vejo

- Nenhum conhecido: as duas bases compilaram no CI com todos os mods juntos. Avisos do compilador: nenhum deste mod
  depois da 1ª rodada (dois itens mortos removidos).
- A injeção em `native_random_tick` empresta `&mut *self` e `&mut *ticks` (MA empresta só `&*self`); compilou junto
  com a do MA. Se uma Nightly futura mudar o tipo de `ticks` ou tornar `GameLevel` diferente, quebra aqui.
- `native_may_place_on` e `native_player_fish` são usados só por este mod; uma Nightly que os renomeie (o
  `native_player_fish_v2` já existe em 2ebf00a) quebraria o build. Hoje existem iguais nas duas.
- `Game::held_mut`, `spawn_item`, `with_level` e `process_level_events` são funções privadas de `game.rs` (como o MA
  já usa as três primeiras): mudanças internas do Horizon podem quebrá-las.

## Riscos de comportamento (não testado no jogo)

- O arroz depende de o template `minecraft:seagrass` + `state_properties` funcionar como planta d'água com idade e
  de a semente (cópia de `wheat_seeds` com `fields/block` trocado) colocá-lo pela regra do capim marinho.
- A muda usa o template `oak_sapling`: se o crescimento nativo falhar, a farinha de osso é recusada (nunca um
  carvalho), mas a muda não cresce.
- As telas usam o menu de fornalha com o painel simples do Horizon (sem a arte do original).

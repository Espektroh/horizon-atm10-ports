# APIs do Horizon usadas por `bibliocraft_native` 0.1.0

Conferidas no código do Horizon `66f3981f522b24c250b21523a70080965b940424` (arquivo:linha) e, as novas para os
ports do usuário, também no Nightly `2ebf00a0281c592f79191e76fce0e7d522c6f425`. horizon-mixins `5a64af9`.
O CI (`Espektroh/horizon-atm10-privado`, `tools/ci_check.sh`) compilou este mod junto com todos os outros nas
duas bases (link da run no LEIA-ME).

## Pontos de injeção (HEAD)

| Hook | Arquivo:linha (66f3981) | Para quê | Já usado por |
|---|---|---|---|
| `Game::native_block_entity(block, entity)` | `game/native_hooks.rs:3065` | block entity de cada móvel | functionalstorage, powah |
| `Game::native_block_entity_ticks(block, ticks)` | `game/native_hooks.rs:3076` | os móveis tickam | idem |
| `Game::native_block_entity_container(block, container)` | `game/native_hooks.rs:3115` | casas, faces dos funis, tipo de menu | idem |
| `Game::native_block_entity_tick(pos, entity)` | `game/native_hooks.rs:3202` | reenviar aos clientes quando o conteúdo muda | idem |
| `Game::native_container_accepts(pos, entity, slot, stack, face, cancel)` | `game/native_hooks.rs:3238` | funis: só livros na estante, só espada no pedestal | functionalstorage |
| `Game::native_block_entity_update_tag(pos, entity, tag)` (sem `self`) | `game/native_hooks.rs:3221` | o item de cada casa para os clientes (`BcView`) | functionalstorage, mysticalagriculture |
| `Game::native_block_entity_submit(pos, block, state, entity, draw)` | `game/native_hooks.rs:3270` | desenhar livros e itens | idem |
| `Game::native_screen(block, screen)` | `game/native_hooks.rs:3409` | posição das casas nas telas | mekanism, powah |
| `Game::native_right_click_block(pos, face, canceled)` | `game/native_hooks.rs:6892` | prateleira, pedestal e mesa | functionalstorage, mysticalagriculture |
| `Game::native_mods(mods)` | `game/native_hooks.rs:3039` | linha em Options > Mods | todos |
| **novo** `Game::native_standalone_models(models)` (sem `self`) | `game/native_hooks.rs:3035`; 2ebf00a `:3231` | assar os 16 modelos de livro | nenhum ainda |
| **novo** `horizon_client::menu::native_menu_slot_rules(menu, slot, stack, may_place, max_stack)` | `menu.rs:487`; 2ebf00a `:491` | estante: só livros, 1 por casa | nenhum ainda |

## Helpers e campos

| API | Arquivo:linha (66f3981) |
|---|---|
| `Game::native_with_mod_block_entity(pos, f)` | `game/native_hooks.rs:8826` |
| `Game::native_block_entity_sync(pos)` | `game/native_hooks.rs:8856` |
| `Game::native_block_property(pos, key) -> Option<String>` | `game/native_hooks.rs:9184` |
| `Game::native_acting_id() -> u128` | `game/native_hooks.rs:9484` |
| `Game::native_acting_player() -> Option<NativeServerPlayer>` (`position`, `yaw`, `pitch`, `uuid`) | `game/native_server.rs:446`, `mod_api.rs:373` |
| `Game::native_give_stack(uuid, stack) -> Option<ItemStack>` | `game/native_server.rs:707` |
| `Game::native_action_bar(uuid, Text) -> bool` | `game/native_server.rs:234` |
| `Game::native_sneaking() -> bool` | `game/native_host_bridge.rs:119` |
| `Game::native_server_item_in_tag(stack, "minecraft:swords") -> Result<bool, _>` | `game/native_world_read.rs:79` (2ebf00a `:76`) |
| `Game::native_block_state_property(state, key) -> Option<&str>` | `game/native_block_entity_draw.rs:745` |
| **novo** `Game::native_draw_model(draw, model, tint) -> bool` | `game/native_block_entity_draw.rs:692` (2ebf00a `:692`) |
| `Game::native_draw_item(draw, stack, DisplayContext::Fixed, None, seed) -> bool` | `game/native_block_entity_draw.rs:722` |
| `BlockEntityDraw::{push, pop, translate, rotate_about, scale, keep}` | `game/native_block_entity_draw.rs:47-212` |
| `Game::held_mut()` (privado do módulo `game`; os filhos o veem) e o campo `Game::offhand` | `game.rs:3715`, `game.rs:2742` |
| `Game::world.block_entities.get(..)`, `BlockEntity::Mod` | como functionalstorage |
| `ModContainer { slots, faces, inputs, outputs, menu }`, `ModBlockEntity` (`id`, `nbt`, `items`) | `horizon-client-render/src/chunk/block_entity.rs:202`, `:118` |
| `NativeScreen { width, height, slots, inventory_y, inventory_x, own_labels, own_background }` | `mod_api.rs:289` (2ebf00a `:395`, mesmos campos) |
| `item_data::encode_components` / `decode_components` | `horizon-items/src/item_data.rs:715`, `:938` |
| `world_items!` (macro do `game.rs`) não é usado | — |

## Comportamentos do Horizon dos quais o port depende
- Um `menu` em `ModContainer` só vale se o tipo de menu tiver o mesmo número de casas (`native_hooks.rs:9699`,
  `mod_menu` em `:10351`). Os tipos `bibliocraft:bookcase` (16) e `bibliocraft:shelf` (4) estão em
  `data/bibliocraft/horizon/menu/`. O menu abre pelo uso do bloco, por isso a estante e a prateleira usam o modelo
  `minecraft:furnace` (`game.rs:5979`, `open_mod_menu` em `native_hooks.rs:9783`), como as máquinas do Powah.
- Itens de um container de mod caem quando o bloco quebra (`game/spill.rs:512-517`).
- `native_menu_slot_rules` vale só para tipos de menu de mods de conteúdo (`menu.rs:475-487`), que é o caso.

## Riscos de compilação
1. **Baixo**: `native_menu_slot_rules` é injetado na biblioteca `horizon-client` (`menu.rs`), não no `game.rs`. O
   helper não usa nada do jogo (a lista de livros está escrita nele), como os ganchos de ingrediente do
   sophisticatedbackpacks em `crafting.rs`.
2. **Baixo**: `native_standalone_models` e `native_draw_model` ainda não eram usados por nenhum mod do usuário.
   Assinaturas iguais nas duas bases.
3. **Baixo**: `held_mut` e `offhand` são privados do `Game`; o glue é filho do módulo `game`, como no
   functionalstorage (que passa no CI).
4. Nomes com prefixo `bibliocraft_` (módulos `bibliocraft_rt`, `bibliocraft_glue`, funções dos helpers, a const
   `BIBLIOCRAFT_BOOKS`).

# APIs do Horizon usadas por `mekmm_native` 0.1.0

Alvo: Horizon `66f3981f522b24c250b21523a70080965b940424`, horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.
Linhas conferidas em `/home/claude/ext/horizon-66f3981/`. **Compilado de verdade no CI** (todos os mods nativos do
repositório juntos, `cargo check -p horizon-client`), nas duas bases: 66f3981 e Nightly 2ebf00a. Ver o relatório
para o link da run.

Os módulos entram em `crates/horizon-client/src/game.rs` como `crate::game::mekmm_native` (runtime, só std) e
`crate::game::mekmm_glue` (glue). Por serem filhos de `game`, o glue alcança campos privados de `Game` (`world`,
`world_tables`, `res`), como o glue do `mekanism_native` faz.

## Pontos de injeção (HEAD): os mesmos do `mekanism_native` 0.6.1 (testado no jogo), menos o do cabo
| Alvo | native_hooks.rs | Helper |
|---|---|---|
| `Game::native_block_entity(block, entity)` | 3065 | `block_entity.rs` |
| `Game::native_block_entity_ticks(block, ticks)` | 3076 | `ticks.rs` |
| `Game::native_block_entity_container(block, container)` | 3115 | `container.rs` |
| `Game::native_block_entity_tick(pos, entity)` | 3202 | `tick.rs` |
| `Game::native_screen(block, screen)` | 3409 | `screen.rs` |
| `Game::native_screen_draw(block, data, batch, origin)` | 3419 | `screen_draw.rs` |
| `Game::native_mods(mods)` | 3039 | `list.rs` |
| `Game::native_right_click_block(pos, face, canceled)` (só `pos` e `canceled`) | 6892 | `right_click.rs` |
| `Game::native_destroy_block(pos, state, canceled)` (só `pos` e `canceled`) | 6865 | `destroy.rs` |
| `Game::native_menu_quick_move(pos, entity, container_slots, slot, stack, handled)` | 3962 | `quick_move.rs` |

Todos retornam na hora para bloco/entidade que não é uma das 5 máquinas (`MmKind::of`). O `right_click` também não
faz nada se outro mod já cancelou o clique; o `quick_move` só age no Estampador e na Prensa e só com `handled` falso.

## Funções, tipos e campos usados no glue
| Uso | Onde (66f3981) |
|---|---|
| `Game::native_block(pos) -> String` | game/native_hooks.rs:9161 |
| `Game::native_block_property(pos, key) -> Option<String>` | game/native_hooks.rs:9184 |
| `Game::native_set_block_property(pos, key, value) -> bool` | game/native_hooks.rs:8736 |
| `Game::native_with_mod_block_entity(pos, FnOnce(&mut ModBlockEntity) -> R) -> Option<R>` | game/native_hooks.rs:8826 |
| `Game::native_recipes_of_type(&str) -> &[CustomRecipe]`; `CustomRecipe { id, json: Arc<Value> }` | game/native_hooks.rs:9375; crafting.rs:1290 |
| `Game::native_recipe_ingredient(&Value) -> Option<Ingredient>`; `Ingredient::test(&ItemStack)` | game/native_hooks.rs:9395; crafting.rs:210 |
| `Game::native_recipe_result(&Value) -> Option<ItemStack>` | game/native_hooks.rs:9425 |
| `Game::native_fill(batch, [f32;4], u32)`, `Game::native_text(batch, &str, [f32;2], u32, bool)` | game/native_hooks.rs:8925, 9222 |
| `Game::native_held() -> Option<(String, u32, bool)>`, `native_take_items(&str, u32)`, `native_give_items(&str, u32)` | game/native_server.rs:632, 657, 682 |
| `world_items!(game)` (macro do game.rs) | game.rs:9 |
| `game.world.game_time` | horizon-client-render/src/world.rs:98 |
| `ModBlockEntity` (`id`, `nbt`, `items: Vec<Option<ItemStack>>`, `data: Vec<i32>`) | horizon-client-render/src/chunk/block_entity.rs:118 |
| `ModContainer { slots, faces, inputs, outputs, menu }` | horizon-client-render/src/chunk/block_entity.rs:202 |
| `NativeScreen { width, height, slots, inventory_y, inventory_x, own_labels, own_background }` | mod_api.rs:289 |
| `NativeMod { id, title, description, keys }` | mod_api.rs:307 |
| `ItemStack::same_item_same_components`, `.count`, `.item`; `Items::get(..).max_stack` | horizon-items/src/item_data.rs:408; horizon-items/src/items.rs:145 |
| `horizon_world::nbt::{Compound, JavaString, Tag}` | iguais ao `mekanism_native` 0.6.1 |
| Menu `minecraft:furnace` (3 slots) | horizon-client-render/src/chunk/block_entity.rs (`MOD_MENUS`) |
| Menu próprio `mekmm:presser` (4 slots, 4 data slots, resultado no 3) declarado em `data/mekmm/horizon/menu/presser.json` | docs/mods.md "Menu types" (API 85); o Powah port usa o mesmo recurso |

Nenhuma API **nova** em relação ao `mekanism_native` 0.6.1 e ao `mekanismgenerators_native` 0.1.0, a não ser o menu
próprio (conteúdo, não código) e `native_recipe_ingredient` sobre os campos `mold`, `primary_input`, etc. (mesma
função, outra chave do JSON).

## Interação com o `mekanism_native` e o `mekanismgenerators_native`
- Não chama nada deles (o build não depende deles). Tem uma cópia da tabela do port: nomes e taxas dos cabos (com as
  variantes `basic_universal_cable_m1..m63`), alcance de 512 cabos, e a saída por tick e as faces de saída de cada
  fonte (cubos 4/16/64/256 kJ/t pela frente; gerador a calor antigo 400 J/t e solar antigo 100 J/t por todas as faces;
  geradores do Mekanism Generators: solar 100 J/t por baixo, solar avançado 600 e eólico 960 pela frente e por baixo,
  a calor 1.020 pela frente).
- **Puxa** a energia (os cubos e geradores do port só entregam para os blocos que eles conhecem): lê e escreve a chave
  `energy` (Long) do NBT da fonte, e acrescenta nela `mekmm_pull_tick` e `mekmm_pull_amount` (Long), para que todas
  as máquinas do MoreMachine juntas não tirem mais que a saída por tick da fonte. Os outros mods ignoram chaves que não
  conhecem (o `write` deles só insere as próprias).
- Risco funcional (não de compilação): se uma versão futura do port mudar taxas ou faces, a tabela `mm_source` em
  runtime.rs precisa ser atualizada.

## Riscos de compilação que eu vejo
- Nenhum método novo: tudo já é usado pelo `mekanism_native` 0.6.1 (compila e foi testado no jogo).
- `serde_json::Value` aparece no glue (assinatura de `native_recipe_ingredient`); o horizon-client depende de
  `serde_json` (crates/horizon-client/Cargo.toml:56).
- Nomes: tudo com prefixo `mm_`/`Mm`/`MM_`/`mekmm_`, sem `use` no topo do game.rs (os `use` ficam dentro do módulo
  `mekmm_glue`).

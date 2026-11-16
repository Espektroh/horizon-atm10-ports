# APIs do Horizon usadas pelo mekanism_native 0.6.0

Alvo: Horizon `66f3981f522b24c250b21523a70080965b940424`, horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.
Linhas conferidas em `crates/horizon-client/src/` desse commit.

**A 0.6.0 não usa nenhum hook nem API nova.** As injeções (`horizon-mixin.toml`) são as mesmas 10 da 0.5.0, com os
mesmos bindings, e o `glue.rs` só chama funções que o `glue.rs` da 0.5.0 já chamava (e que compilaram no
`build.log` do usuário, "Finished `fast` profile ... Build completed"). O que mudou é lógica do próprio mod.

## Hooks (HEAD), iguais à 0.5.0

| Hook | Onde | Uso |
|---|---|---|
| `Game::native_block_entity` | game/native_hooks.rs:3065 | block entity para cada `Kind` (agora 11) |
| `Game::native_block_entity_ticks` | game/native_hooks.rs:3076 | todos tickam |
| `Game::native_block_entity_container` | game/native_hooks.rs:3115 | slots/faces (compressor = infusor; cubos novos = cubo básico) |
| `Game::native_block_entity_tick` | game/native_hooks.rs:3202 | tick de máquinas, geradores, cubos |
| `Game::native_screen` | game/native_hooks.rs:3409 | layout das telas |
| `Game::native_screen_draw` | game/native_hooks.rs:3419 | barras e textos |
| `Game::native_mods` | game/native_hooks.rs:3039 (`NativeMod` em mod_api.rs:307) | linha em Options > Mods |
| `Game::native_right_click_block` | game/native_hooks.rs:6892 | melhorias com clique direito |
| `Game::native_destroy_block` | game/native_hooks.rs:6865 | devolve as melhorias |
| `GameLevel::native_update_shape` | game/native_hooks.rs:1616 | formas dos cabos legados da 0.4.0 |

## Funções chamadas pelo glue.rs (todas já usadas na 0.5.0)

| API | Onde | Novo uso na 0.6.0 |
|---|---|---|
| `Game::native_block(&self, [i32;3]) -> String` | game/native_hooks.rs:9161 | busca da rede (cabos de tiers novos) |
| `Game::native_block_property(&self, [i32;3], &str) -> Option<String>` | game/native_hooks.rs:9184 | face de saída dos cubos novos |
| `Game::native_set_block_property(&mut self, [i32;3], &str, &str) -> bool` | game/native_hooks.rs:8736 | `lit` do compressor |
| `Game::native_with_mod_block_entity<R>(&mut self, [i32;3], impl FnOnce(&mut ModBlockEntity) -> R) -> Option<R>` | game/native_hooks.rs:8826 | entrega de energia |
| `Game::native_recipes_of_type(&self, &str) -> &[CustomRecipe]` | game/native_hooks.rs:9375 (`CustomRecipe { id, json: Arc<Value> }` em crafting.rs:1290) | tipos novos `mekanism:compressing` e `mekanism:chemical_conversion` (qualquer tipo fora de `minecraft` serve, ver doc da função) |
| `Game::native_recipe_ingredient(&self, &Value) -> Option<Ingredient>` | game/native_hooks.rs:9395 | idem |
| `Game::native_recipe_result(&self, &Value) -> Option<ItemStack>` | game/native_hooks.rs:9425 | idem |
| `Game::native_fill(&self, &mut HudBatch, [f32;4], u32)` | game/native_hooks.rs:8925 | tela do compressor |
| `Game::native_text(&self, &mut HudBatch, &str, [f32;2], u32, bool)` | game/native_hooks.rs:9222 | tela do compressor |
| `Game::native_held / native_take_items / native_give_items` | game/native_server.rs:632 / 657 / 682 | sem mudança |
| `world_items!` / `world_recipes!` | game.rs:9 / game.rs:17 | `world_items!` no compressor |
| `ModContainer { slots, faces, inputs, outputs, menu }` | horizon-client-render/src/chunk/block_entity.rs:202 | sem mudança de forma |
| `ModBlockEntity` (Deref para `ModBlockEntityFields { id, nbt, items, data }`) | horizon-client-render/src/chunk/block_entity.rs:108/118 | idem |
| `NativeScreen { width, height, slots, inventory_y, inventory_x, own_labels, own_background }` | mod_api.rs:289 | idem |
| `Compound = BTreeMap<JavaString, Tag>`, `Tag::{Byte, Int, Long, Short}` | horizon-world/src/nbt.rs:35/42 | chaves novas `chemical`, `chemical_used` (Int) |
| `ItemStack { item: Arc<str>, count, .. }`, `same_item_same_components` | horizon-items/src/items.rs:331, item_data.rs:408 | sem mudança |
| `Items::get(&str) -> Option<&ItemInfo>`, `ItemInfo.max_stack` | horizon-items/src/items.rs:994 / 145 | sem mudança |

## Conteúdo (`.hmod`)

- Itens com `components` (ferramentas/armaduras do Mekanism Tools): `docs/mods.md` "An item copies its template's
  data components"; `check_components` em horizon-mods/src/content.rs:958 só exige ids com namespace. O formato
  de `minecraft:tool` (`rules[].blocks/speed/correct_for_drops`, `default_mining_speed`, `damage_per_block`),
  `minecraft:equippable` (`slot`, `asset_id`, `equip_sound`) e `minecraft:attribute_modifiers` é o que
  horizon-items/src/items.rs:765-940 lê.
- Armadura vestida: `assets/<ns>/equipment/<nome>.json` + `textures/entity/equipment/humanoid{,_leggings}/`
  (game/armor_stands.rs:130-170, teste `content_mods_equipment_loads_under_its_namespace`).
- Blocos: só chaves já usadas pela 0.5.0 (`template`, `properties.hardness`, `connects_to`, `thickness`) e
  `properties.light` (teste em horizon-mods/src/content.rs:1233).

## Riscos de compilação que eu vejo

1. **Baixo.** `runtime.rs` compila e passa 28 testes com `cargo test` (Rust 1.97, edição 2024). O usuário compila
   com 1.99.1; nada de API instável.
2. **Baixo.** `glue.rs` foi checado contra imitações das APIs acima (`source/native/tools/glue_mockcheck`):
   compila sem erro, inclusive os `match` exaustivos sobre `Kind` com as 4 variantes novas. O mesmo checador
   compila o `glue.rs` da 0.5.0, que compilou de verdade no Horizon.
3. **Baixo.** `let`-chains (`if let .. && ..`) em `runtime.rs`: o `glue.rs` da 0.5.0 já usava, então o crate
   `horizon-client` é edição 2024.
4. Não compilei dentro do Horizon (proibido nesta sessão): o primeiro Build do launcher é o teste final.

# APIs do Horizon usadas pelo powah_native 0.1.0

Alvo: Horizon `66f3981f522b24c250b21523a70080965b940424`, horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.
Linhas conferidas em `crates/` desse commit (`/home/claude/ext/horizon-66f3981`).

**Nenhum hook nem helper novo em relação ao `mekanism_native` 0.6.x do usuário**, que compilou no build dele: os 8
hooks abaixo são um subconjunto dos 11 do Mekanism, com os mesmos bindings (mesmo texto no `horizon-mixin.toml`), e
o `glue.rs` só chama funções que o glue do Mekanism já chama. Os módulos se chamam `powah_native` e `powah_glue`
(não colidem com `mekanism_native`/`mekanism_glue` nem com os outros mods), as funções dos helpers têm prefixo
`powah_` e são locais à função-alvo (horizon-mixins README: "compiled as a local function inside the target function").

## Hooks (HEAD)

| Hook | Onde | Uso |
|---|---|---|
| `Game::native_block_entity` | horizon-client/src/game/native_hooks.rs:3065 | block entity para orbe, hastes, Furnator, Magmator, célula |
| `Game::native_block_entity_ticks` | native_hooks.rs:3076 | todos tickam |
| `Game::native_block_entity_container` | native_hooks.rs:3115 | slots/faces/menu (o orbe com o menu de conteúdo `powah:energizing_orb`) |
| `Game::native_block_entity_tick` | native_hooks.rs:3202 | lógica de tudo |
| `Game::native_screen` | native_hooks.rs:3409 | layout das telas |
| `Game::native_screen_draw` | native_hooks.rs:3419 | barras e textos |
| `Game::native_mods` | native_hooks.rs:3039 (`NativeMod` em mod_api.rs:307) | linha em Options > Mods |
| `Game::native_right_click_block` | native_hooks.rs:6892 (chamado em game.rs:6610; `canceled` encerra o clique, a outra mão não é tentada) | orbe (pôr/tirar), Magmator (balde de lava) |

`native_right_click_block` em 66f3981 tem também `face`; os bindings só pegam `pos` e `canceled`, como no Mekanism.

## Funções e tipos chamados pelo glue.rs

| API | Onde |
|---|---|
| `Game::native_block(&self, [i32;3]) -> String` | native_hooks.rs:9161 |
| `Game::native_block_property(&self, [i32;3], &str) -> Option<String>` | native_hooks.rs:9184 (`facing` da end rod e do cubo do Mekanism) |
| `Game::native_set_block_property(&mut self, [i32;3], &str, &str) -> bool` | native_hooks.rs:8736 (`lit` dos geradores) |
| `Game::native_with_mod_block_entity<R>(&mut self, [i32;3], impl FnOnce(&mut ModBlockEntity) -> R) -> Option<R>` | native_hooks.rs:8826 (energia para células/hastes, haste → orbe, cubo do Mekanism) |
| `Game::native_recipes_of_type(&self, &str) -> &[CustomRecipe]` | native_hooks.rs:9375 (`CustomRecipe { id: String, json: Arc<Value> }` em crafting.rs:1290) |
| `Game::native_recipe_ingredient(&self, &Value) -> Option<Ingredient>` | native_hooks.rs:9395 (lê só string ou lista de strings: as receitas são escritas assim) |
| `Ingredient::test(&self, &ItemStack) -> bool` | crafting.rs:210 |
| `Game::native_recipe_result(&self, &Value) -> Option<ItemStack>` | native_hooks.rs:9425 |
| `Game::native_fill(&self, &mut HudBatch, [f32;4], u32)` | native_hooks.rs:8925 |
| `Game::native_text(&self, &mut HudBatch, &str, [f32;2], u32, bool)` | native_hooks.rs:9222 |
| `Game::native_held / native_take_items / native_give_items` | game/native_server.rs:632 / 657 / 682 |
| `world_items!` | game.rs:9 |
| `game.res.providers().burn_time(&Items, &ItemStack, bool) -> i32` | game.rs:2060, horizon-client-render/src/providers.rs:108 |
| `game.world.game_time: i64` | game.rs:2430 (`world: World`), horizon-client-render/src/world.rs:98 |
| `ModContainer { slots, faces, inputs, outputs, menu }` | horizon-client-render/src/chunk/block_entity.rs:202 |
| `ModBlockEntity` (Deref/DerefMut para `ModBlockEntityFields { id, nbt, items, data }`) | block_entity.rs:108 / 118 |
| `NativeScreen { width, height, slots, inventory_y, inventory_x, own_labels, own_background }` | mod_api.rs:289 |
| `Compound`, `JavaString: From<&str>`, `Tag::{Byte, Short, Int, Long}` | horizon-world/src/nbt.rs:6 / 8 / 42 |
| `ItemStack { item: Arc<str>, count, .. }`, `ItemStack::new(&str, u32)`, `same_item_same_components` | horizon-items/src/items.rs:331, item_data.rs:259 / 408 |
| `Items::get(&str) -> Option<&ItemInfo>`, `ItemInfo.max_stack: u32` | horizon-items/src/items.rs:994 / 145 |

## Conteúdo (`.hmod`) além do que os ports do usuário já usam

- **Menu de conteúdo** `data/powah/horizon/menu/energizing_orb.json` = `{"slots": 7, "data_slots": 5, "results": [0]}`:
  `MenuType` em horizon-mods/src/content.rs:438 (docs/mods.md "Menu types"); o container o abre pelo nome
  (`mod_menu` em native_hooks.rs:10351, `mod_menu_type` em horizon-client-render/src/containers.rs:54); os 5 números
  são `entity.data`, enviados como i16 (lan/containers.rs:887-892), por isso o orbe manda valores de 15 bits.
  **Nenhum mod do usuário usou menu de conteúdo ainda**: é só dado (não afeta a compilação), mas é o ponto menos
  testado. Se a tela do orbe não abrir, o clique direito com item continua pondo/tirando.
- `shapes` com só `default` no orbe (content.rs:150, docs/mods.md), `properties.occludes: false` (content.rs:95) no
  orbe e na célula, `properties.light: 0` (content.rs:82) nas hastes (template `minecraft:end_rod`).
- Cabos: `minecraft:chorus_plant` + `connects_to` + `thickness`, como os cabos do Mekanism 0.6.
- Loot convertido com `scripts/upgrade_hmod.py` (lição de 11/11); itens sem o componente `minecraft:max_stack_size`.

## Riscos de compilação que eu vejo

1. **Baixo.** `runtime.rs` só usa `std` e passa 14 testes (`cargo test --offline`, Rust 1.97, edição 2024; o crate
   `horizon-client` também é 2024, o glue do Mekanism já usa let-chains). Um identificador `gen` (palavra reservada
   na 2024) foi pego e trocado.
2. **Baixo.** `glue.rs` e os 8 helpers compilam contra imitações das APIs acima
   (`source/native/tools/glue_mockcheck`, o mesmo checador do Mekanism, com `ItemStack::new(&str, u32)` igual ao real
   e os helpers incluídos). Prova tipos, sintaxe e `match` exaustivo; não prova que a assinatura real é a imitada
   (isso é a tabela acima, conferida linha a linha).
3. **Baixo.** `rustfmt --edition 2024 --config max_width=120 --check` passa em todos os `.rs` (só sintaxe e forma).
4. Não compilei dentro do Horizon (proibido nesta sessão): o primeiro Build do launcher é o teste final.

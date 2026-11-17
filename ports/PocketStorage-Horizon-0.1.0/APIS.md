# APIs do Horizon usadas por `pocketstorage_native` 0.1.0

Alvo: Horizon `66f3981f522b24c250b21523a70080965b940424`, horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.
Caminhos relativos a `crates/` do Horizon, linhas em 66f3981. Cada assinatura foi conferida também no Nightly
`2ebf00a` (a 2ª base que o CI compila): todas iguais (só as linhas mudam).
"(SB)" = a mesma API, com o mesmo uso, já compila no Sophisticated Backpacks 0.2.0; "(TIAB)" = idem no Time In A Bottle 0.1.0.

## Pontos de injeção (HEAD)

| Hook | Onde | Uso |
|---|---|---|
| `Game::native_use_item(&mut self, stack: &ItemStack, claimed: &mut bool)` | `horizon-client/src/game/native_hooks.rs:6625` | clique direito com PSU pede para abrir (SB) |
| `Game::native_server_tick(&mut self, context, phase)` | `native_hooks.rs:2489` | abre os menus pedidos (SB) |
| `Game::native_menu_clicked(&mut self, uuid, window, slot, button, mode)` | `native_hooks.rs:6493` | depositar/retirar/páginas/limpar busca (SB) |
| `Game::native_menu_closed(&mut self, uuid, window)` | `native_hooks.rs:6505` | fechar (SB) |
| `Game::native_item_picked_up(&mut self, player: u128, item: &str, count: u32, canceled: &mut bool)` | `native_hooks.rs:7597` | coleta automática (SB) |
| `Game::native_player_toggle_sneak(&mut self, player: u128, name: &str, sneaking: bool)` | `native_hooks.rs:4195` (chamado em `game/lan.rs:2148`) | sem coleta agachado (**novo**) |
| `Game::native_register_commands(&mut self, commands: &mut Vec<NativeCommand>)` | `native_hooks.rs:3987` | `/psu buscar <texto>`, `/psu limpar` (**novo**) |
| `Game::native_perform_command(&mut self, player: &NativeServerPlayer, command: &str, ran: &mut bool)` | `native_hooks.rs:3997` | executa `/psu` (**novo**) |
| `Game::native_tooltip(&self, stack, lines: &mut Vec<(Text, u32)>)` | `native_hooks.rs:6515` | capacidade e tipos usados (TIAB) |
| `Game::native_mods(&mut self, mods: &mut Vec<NativeMod>)` | `native_hooks.rs:3039` | linha em Options > Mods (TIAB) |
| `crafting::native_ingredient` / `native_ingredient_test` | `horizon-client/src/crafting.rs:31` / `:46` | `pocketstorage:empty_psu`: melhoria de tier só com PSU vazio (SB) |

## Helpers e campos usados pelo glue

| API | Onde | Obs. |
|---|---|---|
| `native_open_menu(uuid, rows: u8, title: Text, items) -> Option<i32>` | `game/native_menus.rs:124` | `pub(super)` (SB) |
| `native_close_menu(uuid)` | `game/native_menus.rs:151` | (SB) |
| `Hub::set_native_menu(conn, items)` | `horizon-client/src/lan/containers.rs:423` | `pub` (SB) |
| `native_acting_player(&self) -> Option<NativeServerPlayer>` (`uuid`, `dimension`, `position`, `game_mode`) | `game/native_server.rs:446`, struct `mod_api.rs:373` | (SB) |
| `native_feedback(&mut self, line: Text)` | `game/native_server.rs:436` | `pub(super)`, **novo** |
| `NativeCommand::new(usage: &str)` | `horizon-client/src/mod_api.rs:416` (struct `#[non_exhaustive]` `:408`, por isso o `new`) | **novo** |
| `native_on(&self, mod_id) -> bool` | `native_hooks.rs:8558` | (SB) |
| `native_item_entities(dim, min, max) -> Vec<NativeItemEntity>` (`id`, `position: [f64;3]`, `stack`, `pickup_delay: u32`) | `native_hooks.rs:8681`, struct `:10424` | (SB) |
| `native_set_item_entity(dim, id, stack) -> bool` (count 0 remove) | `native_hooks.rs:8709` | (SB) |
| `horizon_client::lan::playerdata::uuid_ints(u128) -> [i32; 4]` | `lan/playerdata.rs:37` | (SB) |
| `Guest.{conn, uuid, inventory, offhand, selected, placed, dead}` | `lan/guest.rs:86, 107, 271, 273, 274, 340, 336` | `pub` (SB) |
| Campos privados de `Game` (vistos do módulo filho `game::pocketstorage_glue`): `res` 2385, `inventory` 2464, `dropped` 2658, `hand` 2748, `lan` 3074 | `horizon-client/src/game.rs` | (SB) |
| `ItemEntity.id/target` (`Option<[i32;4]>`) | `horizon-client/src/entities.rs:205, 209` | (SB) |
| `Resources::items().get(&str) -> Option<&ItemInfo>`, `ItemInfo.max_stack` | `horizon-items/src/items.rs:994, 145` | (SB) |
| `ItemStack { item, count, damage, extra }`, `new`, `data`, `data_mut`, `tidy`, `same_item_same_components` | `horizon-items/src/items.rs:331-336`, `item_data.rs:259-408` | `extra`/`damage` `pub` |
| `StackData.custom_name`, `.wire`, `.other()`, `.set_other()` (`minecraft:custom_data`, `minecraft:lore`) | `horizon-items/src/item_data.rs:21, 69, 136, 165` | lore lido em `horizon-client/src/item_tooltip.rs:1069` |
| `Tag`, `Tag::get`, `JavaString: From<&str>`, `to_string_lossy` | `horizon-world/src/nbt.rs:42, 76, 8, 30` | (SB) |
| `Text::literal`, `Text::with_color` | `horizon-client-core/src/text.rs:191, 374` | (TIAB) |
| `Hand::{Main, Off}` | `horizon-client/src/first_person.rs:344` | (SB) |

## Como foi verificado

1. `source/native`: `cargo test --offline` — 16 testes da lógica pura (`runtime.rs`), todos passam.
2. `rustfmt --edition 2024 --check` em todos os `.rs`: nenhum erro de sintaxe.
3. **CI do repositório privado** (`tools/ci_check.sh`: apply de todos os mixins + `cargo check -p horizon-client`,
   nas bases 66f3981 e 2ebf00a): verde nas duas, run 44072447495
   (https://github.com/Espektroh/horizon-atm10-privado/actions/runs/44072447495). Único aviso do mod: o esperado
   "type `Game` is more private than the item ..." das funções do glue (igual ao SB e ao TIAB).

## Riscos

- Compilação: coberta pelo CI (verde). Os hooks novos (`toggle_sneak`, comandos) não têm outro usuário no seu build;
  se um mod futuro também registrar `/psu`, o Horizon deixa um de fora e avisa no log (`command_tree`).
- Comportamento (lido no código, não executado):
  - A coleta acha a entidade pelo item e quantidade na caixa de coleta do jogador (como o SB); com duas entidades
    idênticas pode pegar "a outra": nada duplica nem some.
  - A chamada de `native_item_picked_up` em `game.rs:4519` (jogador do próprio `Game`) roda com `self.dropped`
    vazio: ali o PSU não coleta. Todos, inclusive no single player, passam por `guest_pickups` (servidor integrado).
  - O estado "agachado" vem de `native_player_toggle_sneak`; um jogador que entra já agachado conta como em pé até
    soltar e apertar de novo.
  - `custom_data` do PSU vai para o cliente; o tooltip "Tipos usados" depende disso (se não chegar, mostra 0).

# APIs do Horizon usadas — hostilenetworks_native 0.1.0

Conferidas em Horizon `66f3981` (`crates/horizon-client/src/...`) + horizon-mixins `5a64af9`. Todas compilam
juntas com os outros mods do repositório no CI (`tools/ci_check.sh`, jobs 66f3981 e Nightly 2ebf00a).

## Ganchos (injeções HEAD)
| Gancho | Onde (66f3981) | Já usado por | Para quê |
|---|---|---|---|
| `Game::native_block_entity` | game/native_hooks.rs:3065 | Powah, Mekanism | block entity das 2 máquinas |
| `Game::native_block_entity_ticks` | native_hooks.rs:3076 | Powah | máquinas tickam |
| `Game::native_block_entity_container` | native_hooks.rs:3115 | Powah | slots, faces de funil, menu próprio |
| `Game::native_block_entity_tick` | native_hooks.rs:3202 | Powah | simulação / fabricação / energia |
| `Game::native_screen` / `native_screen_draw` | native_hooks.rs:3409 / 3419 | Powah | layout e desenho das telas |
| `Game::native_screen_click` | native_hooks.rs:3890 | **novo** | clique na grade de drops do Fabricador |
| `Game::native_screen_tooltip` | native_hooks.rs:3903 | **novo** | dica da barra de energia e dos drops |
| `Game::native_menu_button` | native_hooks.rs:3916 | **novo** | o clique chega ao servidor e grava `sel_<mob>` |
| `Game::native_container_accepts` | native_hooks.rs:3238 | Bibliocraft | funil só põe Matrix / Prediction certas |
| `horizon_client::menu::native_menu_slot_rules` | menu.rs:487 | Bibliocraft | slot 0 só Data Model (máx. 1), slot 1 só Matrix; Prediction no Fabricador. Vale também para o shift-clique do menu de mod, por isso `native_menu_quick_move` não é usado |
| `Game::native_loot_drops` | native_hooks.rs:7560 (`NativeLootDrops` em mod_api.rs:143) | Mystical Agriculture | morte de mob com crédito a jogador (qualquer arma) treina os Data Models |
| `Game::native_interact_entity` | native_hooks.rs:2564 | **novo** | Estrutura de Modelo clicada num mob vira o Data Model dele |
| `Game::native_use_item` | native_hooks.rs:6625 | Sophisticated Backpacks | Deep Learner põe/tira modelos |
| `Game::native_tooltip` | native_hooks.rs:6515 | Time in a Bottle | tier/dados/precisão |
| `Game::native_mods` | native_hooks.rs:3039 | todos | linha em Opções > Mods |

## Ajudantes e campos
`native_with_mod_block_entity` (8826), `native_block` (9161), `native_block_property` (9184),
`native_set_block_property` (8736), `native_on` (8558), `native_acting_id` (9484), `native_fill` (8925),
`native_text` (9222), `native_item` (8985); em game/native_server.rs: `native_take_items` (657),
`native_give_items` (682), `native_acting_player` (446), `native_action_bar` (234); macro `world_items!`
(game.rs:9). Campos privados de `Game` (acessíveis do módulo filho): `inventory`, `selected`, `offhand`, `player`.
Tipos: `ItemStack`/`StackData` (horizon-items: `data()`, `data_mut()`, `container`, `set_other`, `wire`, `tidy`),
`ModContainer`, `ModBlockEntity`, `NativeScreen` (mod_api.rs:289), `Text::literal`/`with_color`.

## Gancho de morte
Não há "LivingDeathEvent com killer" direto: `native_mob_killed` só ouve golpe corpo a corpo e `native_living_death`
não diz o jogador. O port usa `native_loot_drops` com `context.entity` + `context.player` (o mob lembra o golpe do
jogador por 100 ticks, como o `getKillCredit` do vanilla): conta flechas e golpes, não conta morte por lava muito depois.

## Riscos
- Os 3 ganchos novos (screen_click, screen_tooltip, menu_button, interact_entity) só foram conferidos por leitura e
  pelo `cargo check` do CI; nenhum outro port os testou no jogo.
- Treino por abate só alcança o jogador "atuante" (host / single player; convidado LAN só quando o estado dele está
  trocado para dentro).

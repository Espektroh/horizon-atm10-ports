# APIs do Horizon usadas por `mekanismgenerators_native` 0.1.0

Alvo: Horizon `66f3981f522b24c250b21523a70080965b940424`, horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.
Linhas conferidas em `/home/claude/ext/horizon-66f3981/`. **Compilado de verdade no CI** (todos os mods nativos
juntos, `cargo check -p horizon-client`): verde.

Os módulos entram em `crates/horizon-client/src/game.rs` como `crate::game::mekanismgenerators_native` (runtime, só
std) e `crate::game::mekanismgenerators_glue` (glue). Por serem filhos de `game`, o glue alcança campos e métodos
privados de `Game` (como o glue do `mekanism_native` faz).

## Pontos de injeção (HEAD), os mesmos do `mekanism_native` 0.6.1 (que compila)
| Alvo | native_hooks.rs | Helper |
|---|---|---|
| `Game::native_block_entity(block, entity)` | 3065 | `block_entity.rs` |
| `Game::native_block_entity_ticks(block, ticks)` | 3076 | `ticks.rs` |
| `Game::native_block_entity_container(block, container)` | 3115 | `container.rs` |
| `Game::native_block_entity_tick(pos, entity)` | 3202 | `tick.rs` |
| `Game::native_screen(block, screen)` | 3409 | `screen.rs` |
| `Game::native_screen_draw(block, data, batch, origin)` | 3419 | `screen_draw.rs` |
| `Game::native_mods(mods)` | 3039 | `list.rs` |
| `Game::native_menu_quick_move(entity, container_slots, slot, stack, handled)` | 3962 | `quick_move.rs` |

Todos retornam na hora para bloco/entidade que não é um dos 4 geradores (`MgKind::of`). O `quick_move` só age no
Gerador a Calor e só quando `handled` ainda é falso.

## Funções e campos usados no glue
| Uso | Onde |
|---|---|
| `Game::native_block(pos) -> String` | native_hooks.rs:9161 |
| `Game::native_block_property(pos, key) -> Option<String>` | native_hooks.rs:9184 |
| `Game::native_set_block_property(pos, key, value) -> bool` | native_hooks.rs:8736 |
| `Game::native_with_mod_block_entity(pos, FnOnce(&mut ModBlockEntity) -> R) -> Option<R>` | native_hooks.rs:8826 |
| `Game::native_fill(batch, [f32;4], u32)`, `Game::native_text(batch, &str, [f32;2], u32, bool)` | native_hooks.rs:8925, 9222 |
| `NativeScreen { width, height, slots, inventory_y, inventory_x, own_labels, own_background }`, `NativeMod { id, title, description, keys }` | iguais ao `mekanism_native` 0.6.1 |
| `world_items!(game)` (macro do game.rs) | game.rs:9 |
| `Game::weather_levels(&self, f32) -> (f32, f32)` (privado, inerente) — **novo** em relação ao Mekanism | game.rs:3465 |
| `Game::sky_darken(&self) -> i32` (privado, inerente, no `impl Game<'_>` da linha 3452) — **novo** | game.rs:11891 |
| `game.res` (`Arc<Resources>`), `Resources::facts()`, `::biomes()` (**novo**), `::providers()` | game.rs:2385, 2036, 2048, 2060 |
| `Providers::burn_time(&Items, &ItemStack, bool) -> i32` | horizon-client-render/src/providers.rs:108 |
| `game.world` (`World`): `.block(x,y,z)`, `.day_time`, `.game_time`, `.shape` (**novo**), `.chunks` (**novo**) | game.rs:2430; horizon-client-render/src/world.rs:394, 99, 98, 62, 64 |
| `LevelShape { sky_light, min_y, height, logical_height }` (**novo**) | horizon-client-render/src/chunk.rs:22-28 |
| `chunk.biome(x, y, z) -> u16` (**novo**; mesmo uso de `Game::precipitation`, game.rs:14807) | horizon-client-render/src/chunk.rs:769 |
| `Biomes::get(u16) -> &BiomeInfo`; `BiomeInfo.temperature/.downfall/.precipitation` (**novo**) | horizon-client-render/src/biome.rs:374, 95, 98, 99 |
| `StateFacts.skylight_down` (**novo**; o Mekanism usa `full_collision` do mesmo struct) | horizon-items/src/blocks.rs:45 |
| `game.dim` / `Dim::Nether` (com `matches!`) | game.rs:2818, 3291 |
| `ModBlockEntity { id, nbt, items, data }`, `ModContainer { slots, faces, inputs, outputs, menu }`, `ItemStack::new`, `.count`, `.item`, `.same_item_same_components`, `Items::get(..).max_stack` | iguais ao `mekanism_native` 0.6.1 |
| `horizon_world::nbt::{Compound, JavaString, Tag}` | iguais ao `mekanism_native` 0.6.1 |

## Interação com o `mekanism_native`
- Não chama nada do `mekanism_native` (o build não depende dele). Tem uma cópia das regras de rede do port:
  nomes e taxas dos cabos (inclusive as variantes `basic_universal_cable_m1..m63` da 0.4.0), o alcance de 512 cabos,
  e as capacidades das máquinas (20 kJ, compressor 80 kJ, x 10^(n/8) com `energy_upgrades`) e dos cubos (4/16/64/256 MJ,
  entrada 4/16/64/256 kJ/t, nunca pela frente).
- Escreve só a chave `energy` (Long) do NBT do alvo; lê `energy` e `energy_upgrades`.
- Risco funcional (não de compilação): se uma versão futura do Mekanism port mudar capacidades ou criar máquinas
  novas, esta tabela precisa ser atualizada (`mg_target` em runtime.rs); máquinas desconhecidas simplesmente não
  recebem dos geradores novos.

## Riscos de compilação que eu vejo
- Os métodos privados `weather_levels` e `sky_darken` de `Game`: acessíveis porque o módulo é filho de `game`; se o
  horizon-mixins mudar onde injeta módulos, quebraria. Hoje compila (CI).
- Nomes: tudo com prefixo `mg_`/`Mg`/`MG_`/`mekanismgenerators_`, sem `use` no topo do game.rs.

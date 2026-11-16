# APIs do Horizon usadas (conferidas em 66f3981f522b24c250b21523a70080965b940424)

Caminhos relativos a `crates/`. "Visível" = o glue é um módulo filho de `crate::game` (injetado em
`horizon-client/src/game.rs` como `[[module]]`), então enxerga `pub(super)` dos submódulos de `game` e
os campos/métodos privados de `Game` (como os glues do ironfurnaces e do sophisticatedbackpacks).

## torchmaster_native

### Alvos de injeção
| Alvo | Onde | Assinatura usada | Risco |
|---|---|---|---|
| `horizon_client::game::natural_spawning::<GameLevel<'_,'_> as NaturalLevel>::spawn_hooks` (HEAD cancelável, sem bindings, `ControlFlow<bool>`) | `horizon-client/src/game/natural_spawning.rs:449-452` (`impl NaturalLevel for GameLevel<'_, '_>`, `fn spawn_hooks(&self) -> bool`). Consultado em `horizon-client/src/mobs/spawn_selection.rs:692` e `:1287` | — | **Médio, o maior do pacote.** Nenhum mod seu mira `<Tipo as Trait>` ainda. O ID foi montado como o horizon-mixins 5a64af9 monta (`horizon-mixins-core/src/index.rs:450-466`: tira os espaços e troca lifetimes por `'_`), dando `GameLevel<'_,'_>`. Se não bater: `E_TARGET_NOT_FOUND` no apply. Sem essa injeção, `native_potential_spawns` só roda quando há trait mods. |
| `Game<'_>::native_potential_spawns` | `game/native_hooks.rs:5909` (`&self, dimension: &str, category: &str, pos: [i32; 3], entries, canceled: &mut bool`); chamado em `game/natural_spawning.rs:453-464` | dimension, category, pos, canceled | Baixo. `category` vem de `Category::name()` (`mobs/spawn_selection.rs:61-72`). |
| `Game<'_>::native_village_siege` | `game/native_hooks.rs:5966` (`&mut self, dimension, player: u128, attempted_pos: [f64; 3], canceled`) | dimension, attempted_pos, canceled | Baixo |
| `Game<'_>::native_level_tick_pre` | `game/native_hooks.rs:2497` (`&mut self, dimension: &str`) | dimension | Baixo |
| `Game<'_>::native_world_lifecycle` | `game/native_hooks.rs:2321` | nenhum binding | Baixo |
| `Game<'_>::native_block_entity` / `_ticks` / `_tick` | `game/native_hooks.rs:3065`, `:3076`, `:3202` | iguais aos do ironfurnaces_native (que compila) | Baixo |
| `Game<'_>::native_destroy_block` (prioridade 10) | `game/native_hooks.rs:6865` (`pos: [i32; 3], state, canceled: &mut bool`) | game, pos, canceled | Baixo (atm_tools/mekanism usam o mesmo alvo) |
| `Game<'_>::native_mods` | `game/native_hooks.rs:3039`, `NativeMod` em `mod_api.rs:307` | igual aos seus mods | Baixo |

### Chamadas no glue
| API | Onde | Notas |
|---|---|---|
| `Game::native_block(pos) -> String` | `game/native_hooks.rs:9161` | `pub(super)` |
| `Game.dim: Dim`, `Dim::id(self) -> &'static str` | `game.rs:2818`, `game.rs:3297` | campo/método privados, visíveis |
| `Game::native_surface_heights(&str, &[(i32,i32)], NativeHeightmap) -> Result<Vec<Option<i32>>, String>` | `game/native_world_columns.rs:22` | `pub(super)` |
| `crate::game::native_world_edit::NativeHeightmap::WorldSurface` | `game/native_world_edit.rs:7`; `mod native_world_edit;` privado em `game.rs:253` | visível por ser descendente de `game` |
| `Game.world.bottom_y()` / `top_y()` | campo `game.rs:2430`; métodos `horizon-client-render/src/world.rs:295,299` | usado igual em `game/beacon.rs:210` |
| `Game::native_light(pos) -> (u8, u8)` (bloco, céu) | `game/native_hooks.rs:8618` | |
| `Game.world.loaded(x, z) -> bool` | `horizon-client-render/src/world.rs:454` (`pub fn`) | o `level.isLoaded(targetPos)` da Lanterna Feroz (revisão: coluna não carregada é pulada) |
| `Game.pack.assets.registry.default_state(&str) -> Option<BlockStateId>` | `game.rs:2420`, `:2178`; `horizon-client-render/src/assets.rs:76`; `horizon-world/src/registry.rs:170` | o mesmo caminho é usado em `game/brush.rs:232` e `game/dragon.rs:3889` |
| `Game::native_set_block(&str, [i32;3], BlockStateId, i32) -> bool` | `game/native_hooks.rs:8566` | flags 3 = `UPDATE_ALL` (`horizon-world/src/level/mod.rs:163`) |
| `horizon_client::containers::ModBlockEntity { id, nbt }` | como no ironfurnaces_native | |
| `horizon_world::nbt::{Compound, JavaString, Tag}` (`Tag::Int/Long/IntArray`) | `horizon-world/src/nbt.rs:35,42-56` | |

## Riscos gerais
- O mod compila junto com todos os outros. Nomes com prefixo (`torchmaster_rt`, `torchmaster_glue`,
  helpers `torchmaster_*`, statics privados dos módulos),
  sem `use` no topo dos helpers.
- `runtime.rs` usa só `std`. Ele foi testado com `cargo test --offline` e todos os `.rs` passaram
  pelo `rustfmt --edition 2024`. O glue não pôde ser compilado aqui: o build do Horizon não roda nesta sessão.
- O conteúdo foi conferido contra `horizon-mods/src/content.rs` (`BlockFile`/`ItemFile`/`BlockProperties`, todos com
  `deny_unknown_fields`; `occludes` fica dentro de `properties`; o `.hmod` só tem `horizon-mod.toml`, `assets/` e `data/`).
  O `cargo horizon-mod check` não rodou aqui.

// Mekanism Generators for Horizon: the runtime (`mekanismgenerators_native`)
// over Horizon's block entities, sky, weather, biomes and fuels, and the
// generators' screens. Added to the game as `crate::game::mekanismgenerators_glue`;
// the hook helpers call it. Every function returns at once for a block that
// isn't one of this mod's four generators.

use super::Game;
use super::mekanismgenerators_native as mg;
use super::native_hooks::NativeScreen;
use horizon_client::containers::{ModBlockEntity, ModContainer};
use horizon_client::hud::HudBatch;
use horizon_client::items::ItemStack;
use horizon_world::nbt::{Compound, JavaString, Tag};
use mg::MgKind;

/// The furnace menu's slots: 0 unused, 1 the heat generator's fuel, 2 unused.
const MG_FUEL: usize = 1;

fn mg_get(nbt: &Compound, key: &str) -> i64 {
    match nbt.get(&JavaString::from(key)) {
        Some(Tag::Long(v)) => *v,
        Some(Tag::Int(v)) => i64::from(*v),
        Some(Tag::Short(v)) => i64::from(*v),
        Some(Tag::Byte(v)) => i64::from(*v),
        _ => 0,
    }
}

fn mg_read(nbt: &Compound) -> mg::MgState {
    mg::MgState {
        energy: mg_get(nbt, "energy").max(0),
        lava: mg_get(nbt, "lava").clamp(0, i64::from(mg::MG_HEAT_TANK)) as i32,
    }
}

fn mg_write(nbt: &mut Compound, kind: MgKind, s: &mg::MgState) {
    nbt.insert("energy".into(), Tag::Long(s.energy));
    if kind == MgKind::Heat {
        nbt.insert("lava".into(), Tag::Int(s.lava));
    }
}

/// Each generator gets a block entity of its own type.
pub(super) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none()
        && let Some(kind) = MgKind::of(block)
    {
        *entity = Some((kind.entity().to_owned(), Compound::new()));
    }
}

pub(super) fn ticks(block: &str, ticks: &mut bool) {
    if MgKind::of(block).is_some() {
        *ticks = true;
    }
}

/// The heat generator: its fuel slot from every side (hoppers). The others:
/// a menu for their screen, no slots used.
pub(super) fn container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() {
        return;
    }
    let Some(kind) = MgKind::of(block) else {
        return;
    };
    let menu = Some("minecraft:furnace".to_owned());
    *container = Some(match kind {
        MgKind::Heat => ModContainer {
            slots: 3,
            faces: std::array::from_fn(|_| vec![MG_FUEL]),
            inputs: vec![MG_FUEL],
            outputs: Vec::new(),
            menu,
        },
        MgKind::Solar | MgKind::AdvancedSolar | MgKind::Wind => ModContainer {
            slots: 3,
            faces: Default::default(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            menu,
        },
    });
}

/// `Level.canSeeSky` from `(x, y, z)` up: every block from there to the top
/// of the world lets sky light straight down (air, glass...).
fn mg_sees_sky(game: &Game<'_>, x: i32, y: i32, z: i32) -> bool {
    let top = game.world.shape.min_y + game.world.shape.height;
    let facts = game.res.facts();
    (y..top).all(|yy| facts.get(game.world.block(x, yy, z)).skylight_down)
}

/// The biome at `pos` as the solar check reads it (plains when the chunk
/// isn't there).
fn mg_climate(game: &Game<'_>, pos: [i32; 3]) -> mg::MgClimate {
    let [x, y, z] = pos;
    let Some(chunk) = game.world.chunks.get(&(x >> 4, z >> 4)) else {
        return mg::MgClimate::PLAINS;
    };
    let biomes = game.res.biomes();
    let b = biomes.get(chunk.biome(x, y, z));
    mg::MgClimate {
        temperature: b.temperature,
        downfall: b.downfall,
        precipitation: b.precipitation,
    }
}

/// The sun on a solar panel: sky light in the dimension and the sky not too
/// dark (`getSkyDarken() < 4`), as `WorldUtils.canSeeSun`.
fn mg_daylight(game: &Game<'_>) -> bool {
    game.world.shape.sky_light && game.sky_darken() < 4
}

/// The solar generators: (J/t made, extra data). The sky over the panel is
/// looked at once a second and kept in `sky` (a bit per point).
fn mg_solar(
    game: &Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    kind: MgKind,
) -> (i64, [i32; 2]) {
    let points: &[[i32; 3]] = if kind == MgKind::AdvancedSolar {
        &mg::MG_ADVANCED_POINTS
    } else {
        &mg::MG_ADVANCED_POINTS[..1]
    };
    let key = JavaString::from("sky");
    let sky = if game.world.game_time % 20 == 0 || entity.nbt.get(&key).is_none() {
        let mut bits = 0i32;
        for (i, d) in points.iter().enumerate() {
            if mg_sees_sky(game, pos[0] + d[0], pos[1] + 1, pos[2] + d[2]) {
                bits |= 1 << i;
            }
        }
        entity.nbt.insert("sky".into(), Tag::Int(bits));
        bits
    } else {
        mg_get(&entity.nbt, "sky") as i32
    };
    let (rain, thunder) = game.weather_levels(1.0);
    let raining = rain > 0.2 || thunder > 0.9;
    let day = mg_daylight(game);
    let climate = mg_climate(game, pos);
    let brightness = mg::mg_sun_brightness(game.world.day_time, rain, thunder);
    let permille = (brightness * 1000.0).round() as i32;
    if kind == MgKind::AdvancedSolar {
        let mut seen = [false; 9];
        for (i, s) in seen.iter_mut().enumerate() {
            *s = day && sky & (1 << i) != 0;
        }
        let (sees, multiplier) = mg::mg_advanced_solar(&seen, climate, raining);
        let count = seen.iter().filter(|s| **s).count() as i32;
        let made = if sees {
            mg::mg_solar_production(mg::MG_ADVANCED_SOLAR_GENERATION, brightness, multiplier)
        } else {
            0
        };
        (made, [count, permille])
    } else {
        let sees = day && sky & 1 != 0;
        let multiplier = climate.generation_multiplier(sees, raining);
        (
            mg::mg_solar_production(mg::MG_SOLAR_GENERATION, brightness, multiplier),
            [i32::from(sees), permille],
        )
    }
}

/// The wind generator: its J/t by height, looked at once a second and kept
/// in `wind` (0 without open sky above it).
fn mg_wind(game: &Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) -> i64 {
    let key = JavaString::from("wind");
    if game.world.game_time % 20 == 0 || entity.nbt.get(&key).is_none() {
        let shape = &game.world.shape;
        let wind = if shape.sky_light && mg_sees_sky(game, pos[0], pos[1] + 1, pos[2]) {
            mg::mg_wind_generation(pos[1] + 4, shape.min_y, shape.height, shape.logical_height)
        } else {
            0
        };
        entity.nbt.insert("wind".into(), Tag::Long(wind));
        wind
    } else {
        mg_get(&entity.nbt, "wind")
    }
}

/// What a used fuel item leaves in the slot.
fn mg_remainder(fuel: &ItemStack) -> Option<ItemStack> {
    match &*fuel.item {
        "minecraft:lava_bucket" => Some(ItemStack::new("minecraft:bucket", 1)),
        _ => None,
    }
}

/// The heat generator's fuel slot into its lava tank: one item at a time,
/// burn time / 20 mB, while it fits (`FluidFuelInventorySlot.fillOrBurn`).
fn mg_heat_fuel(game: &Game<'_>, entity: &mut ModBlockEntity, state: &mut mg::MgState) {
    let Some(fuel) = entity.items[MG_FUEL].take() else {
        return;
    };
    let items = world_items!(game);
    let providers = game.res.providers();
    let amount = mg::mg_fuel_lava(providers.burn_time(items, &fuel, false));
    if !mg::mg_heat_fits(state.lava, amount) {
        entity.items[MG_FUEL] = Some(fuel);
        return;
    }
    state.lava += amount;
    let rest = mg_remainder(&fuel);
    let mut left = fuel;
    left.count = left.count.saturating_sub(1);
    entity.items[MG_FUEL] = if left.count > 0 { Some(left) } else { rest };
}

/// Whether the block at `p` takes energy coming in along `came`: a machine
/// or an energy cube of the Mekanism port (a cube never through its front).
fn mg_accepts(game: &Game<'_>, p: [i32; 3], came: [i32; 3]) -> bool {
    let Some(target) = mg::mg_target(&game.native_block(p)) else {
        return false;
    };
    if target.cube_rate.is_some() {
        let front =
            mg::mg_facing_offset(&game.native_block_property(p, "facing").unwrap_or_default());
        return [-came[0], -came[1], -came[2]] != front;
    }
    true
}

/// Energy from the generator at `pos` out of its sides to the port's
/// machines and cubes, along its cables, as the port's own generators give.
fn mg_give(game: &mut Game<'_>, pos: [i32; 3], kind: MgKind, state: &mut mg::MgState) {
    let amount = state.energy.min(kind.output());
    if amount <= 0 {
        return;
    }
    let facing = game
        .native_block_property(pos, "facing")
        .unwrap_or_default();
    let sides = mg::mg_side_offsets(kind, &facing);
    let targets = {
        let g: &Game<'_> = game;
        mg::mg_acceptors(pos, &sides, &mut |p| g.native_block(p), &mut |p, came| {
            mg_accepts(g, p, came)
        })
    };
    if targets.is_empty() {
        return;
    }
    let limits: Vec<i64> = targets.iter().map(|(_, l)| *l).collect();
    let mut rooms = Vec::with_capacity(targets.len());
    for (t, _) in &targets {
        let target = mg::mg_target(&game.native_block(*t));
        let room = game
            .native_with_mod_block_entity(*t, |m| {
                let energy = mg_get(&m.nbt, "energy");
                let upgrades = mg_get(&m.nbt, "energy_upgrades").clamp(0, 8) as u8;
                (energy, upgrades)
            })
            .zip(target)
            .map_or(0, |((energy, upgrades), target)| {
                target.room(energy, upgrades)
            });
        rooms.push(room);
    }
    let shares = mg::mg_share_out(amount, &rooms, &limits);
    for ((t, _), &g) in targets.iter().zip(&shares) {
        if g <= 0 {
            continue;
        }
        let given = game.native_with_mod_block_entity(*t, |m| {
            let energy = mg_get(&m.nbt, "energy");
            m.nbt.insert("energy".into(), Tag::Long(energy + g));
            g
        });
        state.energy -= given.unwrap_or(0);
    }
}

/// One server tick of a generator at `pos`.
pub(super) fn tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let Some(kind) = MgKind::of(&entity.id) else {
        return;
    };
    if entity.items.len() < 3 {
        entity.items.resize(3, None);
    }
    let mut state = mg_read(&entity.nbt);
    let (made, active, extra) = match kind {
        MgKind::Solar | MgKind::AdvancedSolar => {
            let (made, extra) = mg_solar(game, pos, entity, kind);
            let made = state.add(kind, made);
            (made, made > 0, extra)
        }
        MgKind::Wind => {
            let wind = mg_wind(game, pos, entity);
            let made = state.add(kind, wind);
            let permille = (wind * 1000 / mg::MG_WIND_MIN).clamp(0, 32_767) as i32;
            (made, wind > 0, [permille, i32::from(wind > 0)])
        }
        MgKind::Heat => {
            mg_heat_fuel(game, entity, &mut state);
            let lava_sides = mg::MG_DIRS
                .iter()
                .filter(|d| game.native_block(mg::mg_offset(pos, **d)) == "minecraft:lava")
                .count() as u32;
            let nether = matches!(game.dim, super::Dim::Nether);
            let (burning, made) = mg::mg_heat_tick(&mut state, lava_sides, nether);
            (made, burning, [0, i32::from(burning)])
        }
    };
    mg_give(game, pos, kind, &mut state);
    mg_write(&mut entity.nbt, kind, &state);
    entity.data = mg::mg_data(kind, &state, made, extra).to_vec();
    let lit = if active { "true" } else { "false" };
    if game.native_block_property(pos, "lit").as_deref() != Some(lit) {
        game.native_set_block_property(pos, "lit", lit);
    }
}

/// The screens: the furnace menu, with only the heat generator's fuel slot.
pub(super) fn screen(block: &str, screen: &mut Option<NativeScreen>) {
    let Some(kind) = MgKind::of(block) else {
        return;
    };
    let slots = match kind {
        MgKind::Heat => vec![None, Some([56.0, 35.0]), None],
        MgKind::Solar | MgKind::AdvancedSolar | MgKind::Wind => vec![None, None, None],
    };
    *screen = Some(NativeScreen {
        width: 176.0,
        height: 166.0,
        slots,
        inventory_y: 84.0,
        inventory_x: 8.0,
        own_labels: false,
        own_background: false,
    });
}

const MG_DARK: u32 = 0x40_4040;
const MG_BAR_BG: u32 = 0xFF_2B_2B_2B;
const MG_ENERGY: u32 = 0xFF_3C_D2_3C;
const MG_LAVA: u32 = 0xFF_E6_6A_14;
const MG_SUN: u32 = 0xFF_F0_D2_3C;

/// Draws a generator's screen: the energy bar on the right, what it makes,
/// and its own numbers (lava, sun, height).
pub(super) fn screen_draw(
    game: &Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut HudBatch,
    origin: [f32; 2],
) {
    let Some(kind) = MgKind::of(block) else {
        return;
    };
    let d = |i: usize| data.get(i).copied().unwrap_or(0).max(0);
    let [ox, oy] = origin;
    let energy = i64::from(d(0)) * mg::mg_energy_unit(kind);
    let cap = kind.capacity();
    let (bx, by, bw, bh) = (ox + 158.0, oy + 14.0, 10.0, 60.0);
    game.native_fill(batch, [bx, by, bw, bh], MG_BAR_BG);
    let fill = (bh * (energy as f32 / cap as f32)).clamp(0.0, bh);
    game.native_fill(batch, [bx + 1.0, by + bh - fill, bw - 2.0, fill], MG_ENERGY);
    let text = format!("{} / {}", mg::mg_joules(energy), mg::mg_joules(cap));
    game.native_text(batch, &text, [ox + 8.0, oy + 62.0], MG_DARK, false);
    let line = format!("Gerando {} J/t", d(1));
    game.native_text(batch, &line, [ox + 8.0, oy + 18.0], MG_DARK, false);
    match kind {
        MgKind::Heat => {
            // The lava tank: a bar right of the fuel slot, and its amount.
            let amount = d(2).min(mg::MG_HEAT_TANK);
            let (lx, ly, lw, lh) = (ox + 80.0, oy + 37.0, 60.0, 12.0);
            game.native_fill(batch, [lx, ly, lw, lh], MG_BAR_BG);
            game.native_fill(
                batch,
                [lx, ly, lw * amount as f32 / mg::MG_HEAT_TANK as f32, lh],
                MG_LAVA,
            );
            let lava = format!("Lava: {} / {} mB", amount, mg::MG_HEAT_TANK);
            game.native_text(batch, &lava, [ox + 8.0, oy + 28.0], MG_DARK, false);
        }
        MgKind::Solar | MgKind::AdvancedSolar => {
            let sun = if kind == MgKind::Solar {
                if d(2) != 0 {
                    "Vendo o sol".to_owned()
                } else {
                    "Sem sol".to_owned()
                }
            } else {
                format!("Painéis ao sol: {}/9", d(2))
            };
            game.native_text(batch, &sun, [ox + 8.0, oy + 28.0], MG_DARK, false);
            let (sx, sy, sw, sh) = (ox + 8.0, oy + 40.0, 100.0, 8.0);
            game.native_fill(batch, [sx, sy, sw, sh], MG_BAR_BG);
            game.native_fill(
                batch,
                [sx, sy, sw * d(3).min(1000) as f32 / 1000.0, sh],
                MG_SUN,
            );
            let max = format!(
                "Máx. {} J/t · saída {} J/t",
                kind.max_production(),
                kind.output()
            );
            game.native_text(batch, &max, [ox + 8.0, oy + 50.0], MG_DARK, false);
        }
        MgKind::Wind => {
            let m = d(2);
            let state = if d(3) != 0 {
                format!("Vento: x{},{:03}", m / 1000, m % 1000)
            } else {
                "Sem vento (céu fechado)".to_owned()
            };
            game.native_text(batch, &state, [ox + 8.0, oy + 28.0], MG_DARK, false);
            let max = format!(
                "{}-{} J/t pela altura (Y)",
                mg::MG_WIND_MIN,
                mg::MG_WIND_MAX
            );
            game.native_text(batch, &max, [ox + 8.0, oy + 40.0], MG_DARK, false);
        }
    }
}

/// A shift-click from the player's inventory into the heat generator: a fuel
/// goes to the fuel slot (the furnace menu would put a smeltable fuel, a log,
/// in the hidden input slot). Left to the game for everything else.
pub(super) fn quick_move(
    game: &mut Game<'_>,
    entity: &mut ModBlockEntity,
    container_slots: usize,
    slot: usize,
    stack: &mut ItemStack,
    handled: &mut bool,
) {
    if *handled || slot < container_slots || stack.count == 0 || MG_FUEL >= container_slots {
        return;
    }
    if MgKind::of(&entity.id) != Some(MgKind::Heat) {
        return;
    }
    let items = world_items!(game);
    if game.res.providers().burn_time(items, &*stack, false) <= 0 {
        return;
    }
    if entity.items.len() < 3 {
        entity.items.resize(3, None);
    }
    let max = items.get(&stack.item).map_or(64, |i| i.max_stack);
    let moved = match entity.items[MG_FUEL].as_mut() {
        None => stack.count.min(max),
        Some(have) if have.same_item_same_components(stack) => {
            max.saturating_sub(have.count).min(stack.count)
        }
        Some(_) => 0,
    };
    if moved == 0 {
        return;
    }
    match entity.items[MG_FUEL].as_mut() {
        Some(have) => have.count += moved,
        None => {
            let mut put = stack.clone();
            put.count = moved;
            entity.items[MG_FUEL] = Some(put);
        }
    }
    stack.count -= moved;
    *handled = true;
}

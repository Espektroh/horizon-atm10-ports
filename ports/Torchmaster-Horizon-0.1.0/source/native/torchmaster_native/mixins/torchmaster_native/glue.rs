// Torchmaster for Horizon: the rules (`torchmaster_rt`) over Horizon's block
// entities, natural spawning and blocks. Added to the game as
// `crate::game::torchmaster_glue`; the hook helpers call it.
//
// Each mega torch and dread lamp has a ticking block entity that tells the
// registry it is still there; natural spawning asks the registry
// (`native_potential_spawns`, which empties the spawn list it blocks). The
// feral flare lantern's block entity places `torchmaster:invisible_light`
// blocks around it and keeps their positions; breaking it takes them out.

use super::Game;
use super::torchmaster_rt as rt;
use horizon_client::containers::ModBlockEntity;
use horizon_world::nbt::{Compound, JavaString, Tag};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static REGISTRY: Mutex<rt::Registry> = Mutex::new(rt::Registry::new());
/// Whether the registry holds any light: read where natural spawning starts,
/// without the lock.
static ANY: AtomicBool = AtomicBool::new(false);
/// Each lantern's lights, by level and position, for when it is broken.
static FLARES: Mutex<Vec<(String, [i32; 3], Vec<[i32; 3]>)>> = Mutex::new(Vec::new());

const LIGHTS: &str = "lights";
const TICKS: &str = "ticks";
const SEED: &str = "torchmaster_seed";
/// `Level.setBlock`'s `UPDATE_ALL` (horizon_world::level::flags::UPDATE_ALL).
const UPDATE_ALL: i32 = 3;

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn sync_any(r: &rt::Registry) {
    ANY.store(!r.is_empty(), Ordering::Relaxed);
}

/// Natural spawning only asks mods for the spawn list when it thinks one
/// listens (`NaturalLevel::spawn_hooks`): while a light is registered, say yes.
pub(crate) fn any_lights() -> bool {
    ANY.load(Ordering::Relaxed)
}

/// `native_potential_spawns`: a blocked natural spawn gets an empty list.
pub(crate) fn potential_spawns(
    dimension: &str,
    category: &str,
    pos: [i32; 3],
    canceled: &mut bool,
) {
    if *canceled || !any_lights() {
        return;
    }
    if lock(&REGISTRY).blocks_spawn(dimension, category, pos) {
        *canceled = true;
    }
}

/// `native_village_siege`: `blockVillageSieges`.
pub(crate) fn village_siege(dimension: &str, at: [f64; 3], canceled: &mut bool) {
    if *canceled || !any_lights() {
        return;
    }
    if lock(&REGISTRY).blocks_siege(dimension, at) {
        *canceled = true;
    }
}

/// `native_level_tick_pre`: the level's clock, which ages its lights.
pub(crate) fn level_tick(dimension: &str) {
    let mut r = lock(&REGISTRY);
    r.tick_level(dimension);
    sync_any(&r);
}

/// `native_world_lifecycle`: a world opened or closed; nothing carries over.
pub(crate) fn world_changed() {
    let mut r = lock(&REGISTRY);
    r.clear();
    sync_any(&r);
    lock(&FLARES).clear();
}

/// `EntityBlock.newBlockEntity`: our three blocks get one each.
pub(crate) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none()
        && let Some(id) = rt::entity_for(block)
    {
        *entity = Some((id.to_owned(), Compound::new()));
    }
}

pub(crate) fn ticks(block: &str, ticks: &mut bool) {
    if rt::entity_for(block).is_some() {
        *ticks = true;
    }
}

/// One server tick of one of our block entities.
pub(crate) fn tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    match entity.id.as_str() {
        rt::ENTITY_MEGA_TORCH | rt::ENTITY_DREAD_LAMP => {
            let block = game.native_block(pos);
            let Some(kind) = rt::Kind::of_block(&block) else {
                return;
            };
            let dimension = game.dim.id().to_owned();
            let mut r = lock(&REGISTRY);
            r.seen(&dimension, pos, kind);
            sync_any(&r);
        }
        rt::ENTITY_FERAL_FLARE => flare_tick(game, pos, &mut entity.nbt),
        _ => {}
    }
}

fn read_flare(nbt: &Compound, pos: [i32; 3]) -> rt::Flare {
    let lights = match nbt.get(&JavaString::from(LIGHTS)) {
        Some(Tag::IntArray(v)) => v.iter().map(|e| rt::decode_offset(pos, *e)).collect(),
        _ => Vec::new(),
    };
    let ticks = match nbt.get(&JavaString::from(TICKS)) {
        Some(Tag::Int(v)) => *v,
        _ => 0,
    };
    let seed = match nbt.get(&JavaString::from(SEED)) {
        Some(Tag::Long(v)) if *v != 0 => *v as u64,
        _ => rt::seed_for(pos),
    };
    rt::Flare {
        lights,
        ticks,
        check: 0,
        seed,
    }
}

fn write_flare(nbt: &mut Compound, pos: [i32; 3], f: &rt::Flare) {
    let encoded: Vec<i32> = f
        .lights
        .iter()
        .map(|l| rt::encode_offset(pos, *l))
        .collect();
    nbt.insert(JavaString::from(LIGHTS), Tag::IntArray(encoded));
    nbt.insert(JavaString::from(TICKS), Tag::Int(f.ticks));
    nbt.insert(JavaString::from(SEED), Tag::Long(f.seed as i64));
}

fn flare_tick(game: &mut Game<'_>, pos: [i32; 3], nbt: &mut Compound) {
    if game.native_block(pos) != rt::FERAL_FLARE {
        return;
    }
    let dimension = game.dim.id().to_owned();
    let mut f = read_flare(nbt, pos);
    // The light checked last time, taken from the saved order.
    let check = match nbt.get(&JavaString::from("torchmaster_check")) {
        Some(Tag::Int(v)) => usize::try_from(*v).unwrap_or(0),
        _ => 0,
    };
    f.check = check;
    if let Some(i) = f.next_check() {
        if game.native_block(f.lights[i]) == rt::INVISIBLE_LIGHT {
            f.checked_ok();
        } else {
            f.checked_gone(i);
        }
    }
    // `level.isLoaded(targetPos)`: an unloaded column is left alone.
    if let rt::FlareStep::Try(target) = f.step(pos)
        && game.world.loaded(target[0], target[2])
    {
        let surface = game
            .native_surface_heights(
                &dimension,
                &[(target[0], target[2])],
                crate::game::native_world_edit::NativeHeightmap::WorldSurface,
            )
            .ok()
            .and_then(|v| v.first().copied().flatten());
        let (min_y, height) = (game.world.bottom_y(), game.world.top_y() - 1);
        let target = rt::clamp_target(target, surface, min_y, height);
        let (block_light, _sky) = game.native_light(target);
        let empty = rt::is_air(&game.native_block(target));
        let light_state: Option<horizon_world::registry::BlockStateId> =
            game.pack.assets.registry.default_state(rt::INVISIBLE_LIGHT);
        if empty && block_light < rt::FLARE_MIN_LIGHT {
            if let Some(state) = light_state {
                if game.native_set_block(&dimension, target, state, UPDATE_ALL) {
                    f.lights.push(target);
                }
            }
        }
    }
    write_flare(nbt, pos, &f);
    nbt.insert(
        JavaString::from("torchmaster_check"),
        Tag::Int(i32::try_from(f.check).unwrap_or(0)),
    );
    let mut flares = lock(&FLARES);
    flares.retain(|(d, p, _)| !(*d == dimension && *p == pos));
    flares.push((dimension, pos, f.lights));
}

/// `native_destroy_block`: a lantern broken by a player takes its lights
/// with it (`removeChildLights`); a mega torch or dread lamp leaves the
/// registry at once.
pub(crate) fn destroy_block(game: &mut Game<'_>, pos: [i32; 3], canceled: bool) {
    if canceled {
        return;
    }
    let block = game.native_block(pos);
    let dimension = game.dim.id().to_owned();
    if rt::Kind::of_block(&block).is_some() {
        let mut r = lock(&REGISTRY);
        r.remove(&dimension, pos);
        sync_any(&r);
        return;
    }
    if block != rt::FERAL_FLARE {
        return;
    }
    let lights = {
        let mut flares = lock(&FLARES);
        let at = flares
            .iter()
            .position(|(d, p, _)| *d == dimension && *p == pos);
        match at {
            Some(i) => flares.remove(i).2,
            None => return,
        }
    };
    let air: Option<horizon_world::registry::BlockStateId> =
        game.pack.assets.registry.default_state("minecraft:air");
    let Some(air) = air else {
        return;
    };
    for light in lights {
        if game.native_block(light) == rt::INVISIBLE_LIGHT {
            game.native_set_block(&dimension, light, air, UPDATE_ALL);
        }
    }
}

// Time In A Bottle for Horizon: the rules (`tiab_rt`) over the game. Added as
// `crate::game::tiab_glue`; the hook helpers call it.
//
// The bottle keeps its time (in ticks, like the original) in `minecraft:custom_data`
// (`storedTime`, `totalAccumulatedTime`). A right click on a block starts or doubles an
// accelerator there; each server level tick (`native_level_tick_post`) gives the block
// `rate` extra ticks: a native mod's block entity through `tick_mod_block_entity`, a
// vanilla furnace / blast furnace / smoker through its own `FurnaceCooking::tick`, and a
// randomly ticking block (crops, saplings, cane...) random ticks through `Behaviours`.
// Accelerators live in memory (the original's entity is saved with the world).

use super::Game;
use super::tiab_rt as rt;
use horizon_client::containers::BlockEntity;
use horizon_client::items::ItemStack;
use horizon_client::text::Text;
use horizon_world::nbt::{Compound, JavaString, Tag};

static TIAB_ACCELERATORS: std::sync::Mutex<rt::Accelerators> =
    std::sync::Mutex::new(rt::Accelerators::new());

fn tiab_lock() -> std::sync::MutexGuard<'static, rt::Accelerators> {
    match TIAB_ACCELERATORS.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    }
}

/// What the bottle can speed up at a block.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TiabTarget {
    Furnace,
    ModEntity,
    Random,
    Nothing,
}

fn tiab_target(game: &Game<'_>, pos: [i32; 3]) -> TiabTarget {
    let key = (pos[0], pos[1], pos[2]);
    let state = game.world.block(pos[0], pos[1], pos[2]);
    match game.world.block_entities.get(&key) {
        Some(BlockEntity::Furnace(_)) => TiabTarget::Furnace,
        Some(BlockEntity::Mod(_)) => {
            let name: std::sync::Arc<str> = game.res.facts().get(state).name.clone();
            if game.native_ticks(&name) {
                TiabTarget::ModEntity
            } else {
                TiabTarget::Nothing
            }
        }
        // Other vanilla block entities (hoppers, brewing stands...) aren't sped up here.
        Some(_) => TiabTarget::Nothing,
        None => {
            if game.res.facts().get(state).random_ticks {
                TiabTarget::Random
            } else {
                TiabTarget::Nothing
            }
        }
    }
}

fn tiab_int(stack: &ItemStack, key: &str) -> i32 {
    let Some(Tag::Compound(map)) = stack.data().and_then(|d| d.other("minecraft:custom_data"))
    else {
        return 0;
    };
    match map.get(&JavaString::from(key)) {
        Some(Tag::Int(v)) => *v,
        Some(Tag::Long(v)) => (*v).clamp(0, i32::MAX as i64) as i32,
        Some(Tag::Short(v)) => *v as i32,
        Some(Tag::Byte(v)) => *v as i32,
        _ => 0,
    }
}

/// Writes both counters into the stack's `minecraft:custom_data`, the rest of it kept.
fn tiab_write(stack: &mut ItemStack, stored: i32, total: i32) {
    let mut map: Compound = match stack.data().and_then(|d| d.other("minecraft:custom_data")) {
        Some(Tag::Compound(m)) => m.clone(),
        _ => Default::default(),
    };
    map.insert(JavaString::from(rt::KEY_STORED), Tag::Int(stored));
    map.insert(JavaString::from(rt::KEY_TOTAL), Tag::Int(total));
    let data = stack.data_mut();
    data.set_other("minecraft:custom_data", Some(Tag::Compound(map)));
    data.wire = Default::default();
    stack.tidy();
}

/// `native_inventory_tick`: the bottle gains time while carried (every 20 ticks), and
/// is emptied every 10 s if another bottle in the inventory holds more.
pub(super) fn inventory_tick(game: &mut Game<'_>, slot: i32) {
    if !game.native_on(rt::ID) {
        return;
    }
    let now = game.world.game_time;
    if now.rem_euclid(rt::TICK_CONST as i64) != 0 {
        return;
    }
    let check = rt::duplicate_check_tick(now);
    // The other bottles' time, before this one is borrowed.
    let mut others: Vec<i32> = Vec::new();
    if check {
        for (i, s) in game.inventory.iter().enumerate() {
            if let Some(s) = s.as_ref()
                && &*s.item == rt::ITEM
                && i as i32 != slot
            {
                others.push(tiab_int(s, rt::KEY_STORED));
            }
        }
        for (i, s) in game.armor.iter().enumerate() {
            if let Some(s) = s.as_ref()
                && &*s.item == rt::ITEM
                && 103 - i as i32 != slot
            {
                others.push(tiab_int(s, rt::KEY_STORED));
            }
        }
        if let Some(s) = game.offhand.as_ref()
            && &*s.item == rt::ITEM
            && slot != -106
        {
            others.push(tiab_int(s, rt::KEY_STORED));
        }
    }
    let Some(stack) = game.native_carried_stack(slot) else {
        return;
    };
    if &*stack.item != rt::ITEM {
        return;
    }
    let stored = tiab_int(stack, rt::KEY_STORED);
    let total = tiab_int(stack, rt::KEY_TOTAL);
    let Some((mut s2, t2)) = rt::accumulate(now, stored, total) else {
        return;
    };
    if check {
        s2 = rt::after_duplicate_check(s2, &others);
    }
    if (s2, t2) != (stored, total) {
        tiab_write(stack, s2, t2);
    }
}

/// `native_tooltip`: the stored and the total time, as the original shows them.
pub(super) fn tooltip(stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    if &*stack.item != rt::ITEM {
        return;
    }
    let (h, m, s) = rt::hms(tiab_int(stack, rt::KEY_STORED));
    let line = Text::translate_with_fallback(
        "item.tiab.time_in_a_bottle.tooltip.stored_time",
        "Stored time %s:%s:%s",
        &[Text::literal(h), Text::literal(m), Text::literal(s)],
    )
    .with_color(0x55FF55);
    lines.push((line, u32::MAX));
    let (h, m, s) = rt::hms(tiab_int(stack, rt::KEY_TOTAL));
    let line = Text::translate_with_fallback(
        "item.tiab.time_in_a_bottle.tooltip.total_accumulated_time",
        "Total accumulated time %s:%s:%s",
        &[Text::literal(h), Text::literal(m), Text::literal(s)],
    )
    .with_color(0xAAAAAA);
    lines.push((line, u32::MAX));
}

fn tiab_tell(game: &mut Game<'_>, line: Text) {
    if let Some(p) = game.native_acting_player() {
        game.native_action_bar(p.uuid, line);
    }
}

/// `native_use_item_on`: the bottle on a block (`AbstractTiabItem.useOn`).
pub(super) fn use_item_on(
    game: &mut Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    if *claimed || &*stack.item != rt::ITEM || !game.native_on(rt::ID) {
        return;
    }
    if tiab_target(game, pos) == TiabTarget::Nothing {
        return;
    }
    *claimed = true;
    let creative = game
        .native_acting_player()
        .is_some_and(|p| p.game_mode == 1);
    let dim: &'static str = game.native_dimension();
    let stored = tiab_int(stack, rt::KEY_STORED);
    let existing = tiab_lock().get(dim, pos);
    match rt::plan_use(existing, stored, creative) {
        rt::UseOutcome::AtMax => {
            let shown = Text::literal(rt::shown_rate(rt::max_rate()).to_string());
            let line = Text::translate_with_fallback(
                "message.tiab.max",
                "Already at the maximum rate: x%s",
                &[shown],
            );
            tiab_tell(game, line);
        }
        rt::UseOutcome::NotEnough { cost } => {
            let secs = Text::literal((cost / rt::TICK_CONST).to_string());
            let line = Text::translate_with_fallback(
                "message.tiab.not_enough",
                "Not enough stored time: %s s needed",
                &[secs],
            );
            tiab_tell(game, line);
        }
        rt::UseOutcome::Apply { accel, cost } => {
            tiab_lock().set(dim, pos, accel);
            if cost > 0 {
                let slot = game.held_mut();
                if let Some(held) = slot.as_mut()
                    && &*held.item == rt::ITEM
                {
                    let s = rt::spend(tiab_int(held, rt::KEY_STORED), cost);
                    let t = tiab_int(held, rt::KEY_TOTAL);
                    tiab_write(held, s, t);
                }
            }
            let pitch = rt::note_pitch(accel.rate);
            let at = [
                f64::from(pos[0]) + 0.5,
                f64::from(pos[1]) + 0.5,
                f64::from(pos[2]) + 0.5,
            ];
            // SoundSource.BLOCKS is index 4 of SOUND_SOURCES.
            game.with_level(|_, level, _| {
                let _ =
                    level.native_play_sound("minecraft:block.note_block.harp", at, 4, 0.5, pitch);
            });
            let line = Text::translate_with_fallback(
                "message.tiab.rate",
                "Time rate: x%s (%s s left)",
                &[
                    Text::literal(rt::shown_rate(accel.rate).to_string()),
                    Text::literal((accel.remaining / rt::TICK_CONST).to_string()),
                ],
            );
            tiab_tell(game, line);
        }
    }
}

/// One extra tick of a vanilla furnace, as `tick_furnace` does it (without the
/// furnace events other mods hear).
fn tiab_tick_furnace(game: &mut Game<'_>, pos: [i32; 3]) {
    let key = (pos[0], pos[1], pos[2]);
    let mut drops: Vec<ItemStack> = Vec::new();
    let lit = {
        let Some(BlockEntity::Furnace(f)) = game.world.block_entities.get_mut(&key) else {
            return;
        };
        <horizon_client::containers::Furnace as horizon_client::furnace::FurnaceCooking>::tick(
            f,
            world_recipes!(game),
            world_items!(game),
            game.res.providers(),
            &mut drops,
        )
    };
    for stack in drops {
        game.campfire_drop(key, stack);
    }
    if let Some(lit) = lit {
        let _ = game.native_set_block_property_with(
            pos,
            "lit",
            if lit { "true" } else { "false" },
            horizon_world::level::flags::UPDATE_ALL,
        );
    }
}

/// `native_level_tick_post`: every accelerator of this level gives its block `rate`
/// extra ticks and counts down (`TimeAcceleratorEntity.tick`).
pub(super) fn level_tick_post(game: &mut Game<'_>, dimension: &str) {
    if !game.native_on(rt::ID) {
        return;
    }
    let here = tiab_lock().of(dimension);
    if here.is_empty() {
        return;
    }
    let ticking = game.tickets.levels();
    let now = game.world.game_time;
    for (pos, rate) in here {
        // A chunk that doesn't tick doesn't tick its accelerator either.
        if !ticking.block_ticking_at(pos[0], pos[2]) {
            continue;
        }
        let key = (pos[0], pos[1], pos[2]);
        match tiab_target(game, pos) {
            TiabTarget::Furnace => {
                if !game.world.block_entities.is_asleep(&key) {
                    for _ in 0..rate {
                        tiab_tick_furnace(game, pos);
                    }
                }
            }
            TiabTarget::ModEntity => {
                if !game.world.block_entities.is_asleep(&key) {
                    for _ in 0..rate {
                        game.tick_mod_block_entity(key);
                    }
                }
            }
            TiabTarget::Random => {
                let mut rng = rt::Rng::new(rt::seed(now, pos));
                let n = rt::random_ticks(rate, &mut rng);
                if n > 0 {
                    let at = horizon_core::position::BlockPos::new(pos[0], pos[1], pos[2]);
                    game.with_level(|b, level, ticks| {
                        for _ in 0..n {
                            b.random_tick(level, ticks, at);
                        }
                    });
                    let _ = game.process_level_events();
                }
            }
            TiabTarget::Nothing => {
                // The block is gone (the original's entity removes itself).
                tiab_lock().remove(dimension, pos);
                continue;
            }
        }
        tiab_lock().count_down(dimension, pos);
    }
}

/// `native_server_about_to_start`: a new world starts with no accelerators.
pub(super) fn server_start() {
    tiab_lock().clear();
}

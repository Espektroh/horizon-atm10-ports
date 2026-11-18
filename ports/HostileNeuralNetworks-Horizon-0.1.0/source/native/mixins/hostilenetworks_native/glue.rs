// Hostile Neural Networks for Horizon: the rules (`hostilenetworks_native`)
// over the game. Added as `crate::game::hostilenetworks_glue`; the hook
// helpers call it. Only uses what the Powah, Mekanism, Mystical Agriculture,
// Time in a Bottle and Bibliocraft ports' glue already use (they build on
// 66f3981), plus the screen-click / menu-button pair and the entity
// interaction hook (APIS.md).
//
// - A Data Model is one item per mob (`hostilenetworks:data_model_zombie`)
//   with its data in `minecraft:custom_data` (`data`, `iterations`).
// - A Deep Learner keeps up to 4 models in its `minecraft:container`.
// - A kill credited to the acting player (`native_loot_drops`, any weapon)
//   adds data to the matching models in the Deep Learners they carry.
// - The Simulation Chamber and the Loot Fabricator keep `energy` (FE, a long)
//   in their block entity and draw it from a Mekanism energy cube whose front
//   faces them or from a Powah energy cell beside them.

use super::Game;
use super::hostilenetworks_native as rt;
use super::native_hooks::NativeScreen;
use horizon_client::containers::{ModBlockEntity, ModContainer};
use horizon_client::hud::HudBatch;
use horizon_client::items::ItemStack;
use horizon_client::mod_api::NativeLootDrops;
use horizon_client::text::Text;
use horizon_world::nbt::{Compound, JavaString, Tag};

fn hnn_get(nbt: &Compound, key: &str) -> i64 {
    match nbt.get(&JavaString::from(key)) {
        Some(Tag::Long(v)) => *v,
        Some(Tag::Int(v)) => i64::from(*v),
        Some(Tag::Short(v)) => i64::from(*v),
        Some(Tag::Byte(v)) => i64::from(*v),
        _ => 0,
    }
}

fn hnn_int(nbt: &Compound, key: &str) -> i32 {
    hnn_get(nbt, key).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

/// A number kept in a stack's `minecraft:custom_data`.
fn stack_int(stack: &ItemStack, key: &str) -> i32 {
    match stack.data().and_then(|d| d.other("minecraft:custom_data")) {
        Some(Tag::Compound(map)) => hnn_int(map, key),
        _ => 0,
    }
}

/// Writes a number into a stack's `minecraft:custom_data`, the rest kept.
fn set_stack_int(stack: &mut ItemStack, key: &str, value: i32) {
    let mut map: Compound = match stack.data().and_then(|d| d.other("minecraft:custom_data")) {
        Some(Tag::Compound(m)) => m.clone(),
        _ => Default::default(),
    };
    map.insert(JavaString::from(key), Tag::Int(value));
    let data = stack.data_mut();
    data.set_other("minecraft:custom_data", Some(Tag::Compound(map)));
    data.wire = Default::default();
    stack.tidy();
}

/// The models a Deep Learner holds, by slot.
fn learner_models(stack: &ItemStack) -> Vec<(u8, ItemStack)> {
    stack.data().map_or_else(Vec::new, |d| d.container.clone())
}

fn set_learner_models(stack: &mut ItemStack, models: Vec<(u8, ItemStack)>) {
    let data = stack.data_mut();
    data.container = models;
    data.wire = Default::default();
    stack.tidy();
}

/// Adds a kill of `model`'s mob to every matching model in the learner;
/// whether anything changed.
fn learner_kill(stack: &mut ItemStack, model: usize) -> bool {
    let mut models = learner_models(stack);
    let mut changed = false;
    for (_, m) in models.iter_mut() {
        if rt::model_of_data_item(&m.item) == Some(model) {
            let data = stack_int(m, rt::KEY_DATA);
            let next = rt::data_after_kill(model, data);
            if next != data {
                set_stack_int(m, rt::KEY_DATA, next);
                changed = true;
            }
        }
    }
    if changed {
        set_learner_models(stack, models);
    }
    changed
}

fn hnn_tell(game: &mut Game<'_>, line: String) {
    if let Some(p) = game.native_acting_player() {
        game.native_action_bar(p.uuid, Text::literal(line));
    }
}

/// `native_loot_drops`: a mob died and a player gets the kill
/// (`LivingDeathEvent` + `getKillCredit`). Only the acting player's
/// inventory is reached (the host, or a guest whose state is swapped in).
pub(super) fn loot_drops(
    game: &mut Game<'_>,
    context: &NativeLootDrops<'_>,
    drops: &mut Vec<ItemStack>,
) {
    let _ = drops;
    if context.used || !game.native_on(rt::ID) {
        return;
    }
    let (Some((_, kind)), Some(player)) = (context.entity, context.player) else {
        return;
    };
    let Some(model) = rt::model_for_kind(kind) else {
        return;
    };
    if player != game.native_acting_id() || game.player.is_none() {
        return;
    }
    let mut changed = false;
    for slot in game.inventory.iter_mut().take(36) {
        if let Some(s) = slot.as_mut()
            && &*s.item == rt::DEEP_LEARNER
        {
            changed |= learner_kill(s, model);
        }
    }
    if let Some(s) = game.offhand.as_mut()
        && &*s.item == rt::DEEP_LEARNER
    {
        changed |= learner_kill(s, model);
    }
    let _ = changed;
}

/// `native_interact_entity`: a Blank Data Model (Model Framework) used on a
/// mob with a model becomes that mob's Data Model (`HostileEvents.modelAttunement`).
pub(super) fn interact_entity(
    game: &mut Game<'_>,
    kind: &str,
    stack: Option<&ItemStack>,
    claimed: &mut bool,
) {
    if *claimed || !game.native_on(rt::ID) {
        return;
    }
    let Some(stack) = stack else {
        return;
    };
    if &*stack.item != rt::BLANK_DATA_MODEL {
        return;
    }
    *claimed = true;
    let Some(model) = rt::model_for_kind(kind) else {
        hnn_tell(game, "Nenhum Data Model para este mob".to_owned());
        return;
    };
    if game.native_take_items(rt::BLANK_DATA_MODEL, 1) != 1 {
        return;
    }
    let item = rt::data_model_item(model);
    if game.native_give_items(&item, 1) == 1 {
        let name = rt::mob_pt(rt::MODELS[model].name);
        hnn_tell(game, format!("Data Model de {name} construído"));
    } else {
        game.native_give_items(rt::BLANK_DATA_MODEL, 1);
    }
}

/// `native_use_item`: a Deep Learner in one hand takes the Data Model in the
/// other; with the other hand empty it gives back its last model; else it
/// tells what it holds. (The original opens a 4-slot screen.)
pub(super) fn use_item(game: &mut Game<'_>, stack: &ItemStack, claimed: &mut bool) {
    if *claimed || &*stack.item != rt::DEEP_LEARNER || !game.native_on(rt::ID) {
        return;
    }
    *claimed = true;
    let sel = game.selected;
    let main_is = game
        .inventory
        .get(sel)
        .and_then(|s| s.as_ref())
        .is_some_and(|s| &*s.item == rt::DEEP_LEARNER);
    // The learner and the other hand: (learner, other) as taken out.
    let (learner, other) = if main_is {
        (
            game.inventory.get_mut(sel).and_then(|s| s.take()),
            game.offhand.take(),
        )
    } else {
        (
            game.offhand.take(),
            game.inventory.get_mut(sel).and_then(|s| s.take()),
        )
    };
    let Some(mut learner) = learner else {
        // Nothing to do: put the other hand back.
        if main_is {
            game.offhand = other;
        } else if let Some(s) = game.inventory.get_mut(sel) {
            *s = other;
        }
        return;
    };
    let mut other = other;
    let mut models = learner_models(&learner);
    let mut message = None;
    match other.as_ref() {
        Some(o) if rt::model_of_data_item(&o.item).is_some() => {
            let free = (0..rt::LEARNER_SLOTS as u8).find(|i| models.iter().all(|(s, _)| s != i));
            match free {
                Some(slot) => {
                    let mut one = o.clone();
                    one.count = 1;
                    let line = rt::model_line(
                        rt::model_of_data_item(&one.item).unwrap_or(0),
                        stack_int(&one, rt::KEY_DATA),
                    );
                    models.push((slot, one));
                    models.sort_by_key(|(s, _)| *s);
                    set_learner_models(&mut learner, models);
                    let left = o.count.saturating_sub(1);
                    if left == 0 {
                        other = None;
                    } else if let Some(o) = other.as_mut() {
                        o.count = left;
                    }
                    message = Some(format!("Inserido: {line}"));
                }
                None => message = Some("Deep Learner cheio (4 modelos)".to_owned()),
            }
        }
        None => {
            if let Some(at) = models.iter().map(|(s, _)| *s).max() {
                let pos = models.iter().position(|(s, _)| *s == at).unwrap_or(0);
                let (_, m) = models.remove(pos);
                set_learner_models(&mut learner, models);
                other = Some(m);
                message = Some("Data Model retirado do Deep Learner".to_owned());
            } else {
                message = Some("Deep Learner vazio: segure um Data Model na outra mão".to_owned());
            }
        }
        Some(_) => {
            let lines: Vec<String> = models
                .iter()
                .filter_map(|(_, m)| {
                    rt::model_of_data_item(&m.item)
                        .map(|i| rt::model_line(i, stack_int(m, rt::KEY_DATA)))
                })
                .collect();
            message = Some(if lines.is_empty() {
                "Deep Learner vazio".to_owned()
            } else {
                lines.join(" | ")
            });
        }
    }
    if main_is {
        if let Some(s) = game.inventory.get_mut(sel) {
            *s = Some(learner);
        }
        game.offhand = other;
    } else {
        game.offhand = Some(learner);
        if let Some(s) = game.inventory.get_mut(sel) {
            *s = other;
        }
    }
    if let Some(m) = message {
        hnn_tell(game, m);
    }
}

/// `native_tooltip`: a Data Model's tier, data and accuracy; a Deep
/// Learner's models.
pub(super) fn tooltip(stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    if let Some(model) = rt::model_of_data_item(&stack.item) {
        let data = stack_int(stack, rt::KEY_DATA);
        let tier = rt::TIERS[rt::tier_of(data)];
        lines.push((
            Text::literal(rt::model_line(model, data)).with_color(tier.rgb),
            u32::MAX,
        ));
        let iters = stack_int(stack, rt::KEY_ITERS);
        if iters > 0 {
            lines.push((
                Text::literal(format!("Simulações: {iters}")).with_color(0xAAAAAA),
                u32::MAX,
            ));
        }
        return;
    }
    if &*stack.item == rt::DEEP_LEARNER {
        let models = learner_models(stack);
        if models.is_empty() {
            lines.push((
                Text::literal("Vazio: use com um Data Model na outra mão").with_color(0xAAAAAA),
                u32::MAX,
            ));
        }
        for (_, m) in models {
            if let Some(i) = rt::model_of_data_item(&m.item) {
                let data = stack_int(&m, rt::KEY_DATA);
                let tier = rt::TIERS[rt::tier_of(data)];
                lines.push((
                    Text::literal(rt::model_line(i, data)).with_color(tier.rgb),
                    u32::MAX,
                ));
            }
        }
    }
}

/// Both machines get a block entity and tick.
pub(super) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && rt::is_ours(block) {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

pub(super) fn ticks(block: &str, ticks: &mut bool) {
    if rt::is_ours(block) {
        *ticks = true;
    }
}

/// The chamber: its own menu (model, matrix, two outputs); hoppers put
/// matrices in from the top and sides and take the outputs from below.
/// The fabricator: the prediction from the top and sides, 16 outputs below.
pub(super) fn container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() {
        return;
    }
    if block == rt::SIM_CHAMBER {
        let outs = vec![rt::SIM_BASE_OUT, rt::SIM_PRED_OUT];
        let ins = vec![rt::SIM_INPUT];
        *container = Some(ModContainer {
            slots: rt::SIM_SLOTS,
            faces: [
                outs.clone(),
                ins.clone(),
                ins.clone(),
                ins.clone(),
                ins.clone(),
                ins.clone(),
            ],
            inputs: ins,
            outputs: outs,
            menu: Some(rt::SIM_MENU.to_owned()),
        });
    } else if block == rt::LOOT_FABRICATOR {
        let outs: Vec<usize> = (1..rt::FAB_SLOTS).collect();
        let ins = vec![rt::FAB_INPUT];
        *container = Some(ModContainer {
            slots: rt::FAB_SLOTS,
            faces: [
                outs.clone(),
                ins.clone(),
                ins.clone(),
                ins.clone(),
                ins.clone(),
                ins.clone(),
            ],
            inputs: ins,
            outputs: outs,
            menu: Some(rt::FAB_MENU.to_owned()),
        });
    }
}

/// What a hopper may put in: matrices in the chamber's slot 1, mob
/// predictions in the fabricator's slot 0.
pub(super) fn accepts(entity: &ModBlockEntity, slot: usize, stack: &ItemStack, cancel: &mut bool) {
    if entity.id == rt::SIM_CHAMBER {
        let ok = match slot {
            rt::SIM_MODEL => rt::model_of_data_item(&stack.item).is_some(),
            rt::SIM_INPUT => &*stack.item == rt::PREDICTION_MATRIX,
            _ => false,
        };
        if !ok {
            *cancel = true;
        }
    } else if entity.id == rt::LOOT_FABRICATOR
        && (slot != rt::FAB_INPUT || rt::model_of_prediction(&stack.item).is_none())
    {
        *cancel = true;
    }
}

fn max_of(game: &Game<'_>, item: &str) -> u32 {
    world_items!(game)
        .get(item)
        .map_or(64, |i| i.max_stack)
        .max(1)
}

/// Whether `slot` takes `n` more of `item` (plain).
fn room_for(game: &Game<'_>, slot: &Option<ItemStack>, item: &str, n: u32) -> bool {
    match slot {
        None => true,
        Some(s) => &*s.item == item && s.extra.is_none() && s.count + n <= max_of(game, item),
    }
}

fn grow(slot: &mut Option<ItemStack>, item: &str, n: u32) {
    if n == 0 {
        return;
    }
    match slot.as_mut() {
        Some(s) => s.count += n,
        None => *slot = Some(ItemStack::new(item, n)),
    }
}

fn shrink(slot: &mut Option<ItemStack>) {
    let empty = match slot.as_mut() {
        Some(s) => {
            s.count = s.count.saturating_sub(1);
            s.count == 0
        }
        None => false,
    };
    if empty {
        *slot = None;
    }
}

/// Energy from the neighbours: a Mekanism energy cube whose front faces the
/// machine (2 FE for 5 J, at most the cube's output a tick) and a Powah
/// energy cell on any side (at most its own transfer a tick).
fn draw_energy(game: &mut Game<'_>, pos: [i32; 3], energy: &mut i64, cap: i64) {
    for d in rt::DIRS {
        let room = cap - *energy;
        if room <= 0 {
            return;
        }
        let q = rt::offset(pos, d);
        let name = game.native_block(q);
        if let Some(output) = rt::mekanism_cube_output(&name) {
            let front =
                rt::facing_offset(&game.native_block_property(q, "facing").unwrap_or_default());
            if rt::offset(q, front) != pos {
                continue;
            }
            let gained = game
                .native_with_mod_block_entity(q, |m| {
                    if m.id != name {
                        return 0;
                    }
                    let joules = hnn_get(&m.nbt, "energy");
                    let (fe, taken) = rt::mekanism_draw(joules, output, room);
                    if fe > 0 {
                        m.nbt.insert("energy".into(), Tag::Long(joules - taken));
                    }
                    fe
                })
                .unwrap_or(0);
            *energy += gained.max(0);
        } else if let Some(output) = rt::powah_cell_output(&name) {
            let gained = game
                .native_with_mod_block_entity(q, |m| {
                    if m.id != name {
                        return 0;
                    }
                    let have = hnn_get(&m.nbt, "energy");
                    let fe = have.min(output).min(room).max(0);
                    if fe > 0 {
                        m.nbt.insert("energy".into(), Tag::Long(have - fe));
                    }
                    fe
                })
                .unwrap_or(0);
            *energy += gained.max(0);
        }
    }
}

fn set_lit(game: &mut Game<'_>, pos: [i32; 3], lit: bool) {
    let want = if lit { "true" } else { "false" };
    if game.native_block_property(pos, "lit").as_deref() != Some(want) {
        game.native_set_block_property(pos, "lit", want);
    }
}

fn sim_tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    if entity.items.len() < rt::SIM_SLOTS {
        entity.items.resize(rt::SIM_SLOTS, None);
    }
    let mut s = rt::SimState {
        energy: hnn_get(&entity.nbt, "energy").clamp(0, rt::SIM_POWER_CAP),
        runtime: hnn_int(&entity.nbt, "runtime").max(0),
        success: hnn_int(&entity.nbt, "success").max(0),
        fail: hnn_int(&entity.nbt, "fail"),
        model: hnn_int(&entity.nbt, "model").max(0),
    };
    draw_energy(game, pos, &mut s.energy, rt::SIM_POWER_CAP);
    let model = entity.items[rt::SIM_MODEL]
        .as_ref()
        .and_then(|m| rt::model_of_data_item(&m.item).map(|i| (i, stack_int(m, rt::KEY_DATA))));
    let slots = match model {
        Some((i, data)) => rt::SimSlots {
            model: Some((i, data)),
            matrix: entity.items[rt::SIM_INPUT]
                .as_ref()
                .is_some_and(|m| &*m.item == rt::PREDICTION_MATRIX && m.count > 0),
            base_room: room_for(
                game,
                &entity.items[rt::SIM_BASE_OUT],
                rt::MODELS[i].base_drop,
                1,
            ),
            pred_room: room_for(
                game,
                &entity.items[rt::SIM_PRED_OUT],
                &rt::prediction_item(i),
                1,
            ),
        },
        None => rt::SimSlots {
            model: None,
            matrix: false,
            base_room: false,
            pred_room: false,
        },
    };
    let done = rt::sim_tick(&mut s, &slots, rt::roll);
    if done.consumed {
        shrink(&mut entity.items[rt::SIM_INPUT]);
    }
    if done.finished
        && let Some((i, _)) = model
    {
        grow(
            &mut entity.items[rt::SIM_BASE_OUT],
            rt::MODELS[i].base_drop,
            1,
        );
        let pred = rt::prediction_item(i);
        grow(
            &mut entity.items[rt::SIM_PRED_OUT],
            &pred,
            done.predictions.max(0) as u32,
        );
        if let Some(m) = entity.items[rt::SIM_MODEL].as_mut() {
            let iters = stack_int(m, rt::KEY_ITERS);
            set_stack_int(m, rt::KEY_ITERS, iters.saturating_add(1));
        }
    }
    let nbt = &mut entity.nbt;
    nbt.insert("energy".into(), Tag::Long(s.energy));
    nbt.insert("runtime".into(), Tag::Int(s.runtime));
    nbt.insert("success".into(), Tag::Int(s.success));
    nbt.insert("fail".into(), Tag::Int(s.fail));
    nbt.insert("model".into(), Tag::Int(s.model));
    let (lo, hi) = rt::split15(s.energy);
    let data = model.map_or(0, |(_, d)| d.clamp(0, 0x7FFF));
    entity.data = vec![lo, hi, s.runtime, s.fail, s.model, data, s.success];
    set_lit(game, pos, s.runtime > 0 && s.fail == rt::Fail::None as i32);
}

/// The drop chosen for a model in the fabricator (`sel_<model>`), if any.
fn fab_selection(nbt: &Compound, model: usize) -> Option<usize> {
    let key = format!("sel_{}", rt::MODELS[model].name);
    match nbt.get(&JavaString::from(key.as_str())) {
        Some(Tag::Int(v)) if *v >= 0 => Some(*v as usize),
        _ => None,
    }
}

fn fab_tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    if entity.items.len() < rt::FAB_SLOTS {
        entity.items.resize(rt::FAB_SLOTS, None);
    }
    let mut s = rt::FabState {
        energy: hnn_get(&entity.nbt, "energy").clamp(0, rt::FAB_POWER_CAP),
        runtime: hnn_int(&entity.nbt, "runtime").max(0),
        current: hnn_int(&entity.nbt, "current"),
        model: hnn_int(&entity.nbt, "model"),
    };
    draw_energy(game, pos, &mut s.energy, rt::FAB_POWER_CAP);
    let input = entity.items[rt::FAB_INPUT]
        .as_ref()
        .and_then(|p| rt::model_of_prediction(&p.item));
    let selection = input.and_then(|m| fab_selection(&entity.nbt, m));
    if let rt::FabStep::Output(model, sel) = rt::fab_tick(&mut s, input, selection) {
        let (item, count) = rt::MODELS[model].fab[sel];
        let max = max_of(game, item);
        let outs: Vec<rt::Out> = entity.items[1..rt::FAB_SLOTS]
            .iter()
            .map(|o| match o {
                None => rt::Out::Empty,
                Some(o) if &*o.item == item && o.extra.is_none() => rt::Out::Same {
                    count: o.count,
                    max,
                },
                Some(_) => rt::Out::Other,
            })
            .collect();
        if let Some(plan) = rt::insert_plan(&outs, count, max) {
            for (i, n) in plan {
                grow(&mut entity.items[1 + i], item, n);
            }
            shrink(&mut entity.items[rt::FAB_INPUT]);
            s.runtime = 0;
        }
    }
    let nbt = &mut entity.nbt;
    nbt.insert("energy".into(), Tag::Long(s.energy));
    nbt.insert("runtime".into(), Tag::Int(s.runtime));
    nbt.insert("current".into(), Tag::Int(s.current));
    nbt.insert("model".into(), Tag::Int(s.model));
    let (lo, hi) = rt::split15(s.energy);
    let model = input.map_or(0, |m| m as i32 + 1);
    let sel = selection.map_or(0, |v| v as i32 + 1);
    entity.data = vec![lo, hi, s.runtime, model, sel];
    set_lit(game, pos, s.runtime > 0);
}

/// One server tick of a machine.
pub(super) fn tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    if entity.id == rt::SIM_CHAMBER {
        sim_tick(game, pos, entity);
    } else if entity.id == rt::LOOT_FABRICATOR {
        fab_tick(game, pos, entity);
    }
}

/// A click on one of the fabricator's drops (button id = its index) picks
/// it for the model of the prediction inside; button 99 clears the pick.
pub(super) fn menu_button(entity: &mut ModBlockEntity, id: i32) {
    if entity.id != rt::LOOT_FABRICATOR {
        return;
    }
    let Some(model) = entity
        .items
        .get(rt::FAB_INPUT)
        .and_then(|s| s.as_ref())
        .and_then(|p| rt::model_of_prediction(&p.item))
    else {
        return;
    };
    let key = format!("sel_{}", rt::MODELS[model].name);
    if id == FAB_CLEAR {
        entity.nbt.insert(key.as_str().into(), Tag::Int(-1));
    } else if id >= 0 && (id as usize) < rt::MODELS[model].fab.len() {
        entity.nbt.insert(key.as_str().into(), Tag::Int(id));
    }
}

const FAB_CLEAR: i32 = 99;
/// The fabricator screen's drop grid: 3 columns from (30, 8), 18 apart.
const FAB_GRID: [f32; 2] = [30.0, 8.0];

fn fab_choice_at(at: [f32; 2], count: usize) -> Option<usize> {
    let x = at[0] - FAB_GRID[0];
    let y = at[1] - FAB_GRID[1];
    if x < 0.0 || y < 0.0 {
        return None;
    }
    let (c, r) = ((x / 18.0) as usize, (y / 18.0) as usize);
    if c >= 3 || x - c as f32 * 18.0 >= 16.0 || y - r as f32 * 18.0 >= 16.0 {
        return None;
    }
    let i = r * 3 + c;
    (i < count).then_some(i)
}

fn in_rect(at: [f32; 2], r: [f32; 4]) -> bool {
    at[0] >= r[0] && at[1] >= r[1] && at[0] < r[0] + r[2] && at[1] < r[1] + r[3]
}

const ENERGY_BAR: [f32; 4] = [8.0, 28.0, 12.0, 48.0];

/// `native_screen_click` (client): a drop of the fabricator's grid, or the
/// clear button under it.
pub(super) fn screen_click(block: &str, data: &[i32], at: [f32; 2], button: &mut Option<i32>) {
    if block != rt::LOOT_FABRICATOR || button.is_some() {
        return;
    }
    let model = data.get(3).copied().unwrap_or(0);
    if model <= 0 || model as usize > rt::MODELS.len() {
        return;
    }
    let m = &rt::MODELS[model as usize - 1];
    if let Some(i) = fab_choice_at(at, m.fab.len()) {
        *button = Some(i as i32);
    } else if in_rect(at, FAB_CLEAR_RECT) {
        *button = Some(FAB_CLEAR);
    }
}

const FAB_CLEAR_RECT: [f32; 4] = [30.0, 62.0, 52.0, 10.0];

/// `native_screen_tooltip` (client): the energy bar, a drop of the grid.
pub(super) fn screen_tooltip(block: &str, data: &[i32], at: [f32; 2], lines: &mut Vec<String>) {
    if !rt::is_ours(block) {
        return;
    }
    let d = |i: usize| data.get(i).copied().unwrap_or(0);
    if in_rect(at, ENERGY_BAR) {
        let cap = if block == rt::SIM_CHAMBER {
            rt::SIM_POWER_CAP
        } else {
            rt::FAB_POWER_CAP
        };
        lines.push(format!(
            "{} / {} FE",
            rt::fe(rt::join15(d(0), d(1))),
            rt::fe(cap)
        ));
        return;
    }
    if block == rt::LOOT_FABRICATOR {
        let model = d(3);
        if model > 0 && model as usize <= rt::MODELS.len() {
            let m = &rt::MODELS[model as usize - 1];
            if let Some(i) = fab_choice_at(at, m.fab.len()) {
                let (item, count) = m.fab[i];
                lines.push(format!("{count}x {item}"));
                lines.push("Clique para escolher".to_owned());
            } else if in_rect(at, FAB_CLEAR_RECT) {
                lines.push("Limpar escolha".to_owned());
            }
        }
    }
}

/// The screens' layout.
pub(super) fn screen(block: &str, screen: &mut Option<NativeScreen>) {
    let slots = if block == rt::SIM_CHAMBER {
        vec![
            Some([28.0, 8.0]),
            Some([28.0, 58.0]),
            Some([150.0, 8.0]),
            Some([150.0, 58.0]),
        ]
    } else if block == rt::LOOT_FABRICATOR {
        let mut v = vec![Some([8.0, 8.0])];
        for r in 0..4 {
            for c in 0..4 {
                v.push(Some([98.0 + 18.0 * c as f32, 8.0 + 18.0 * r as f32]));
            }
        }
        v
    } else {
        return;
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

const DARK: u32 = 0x40_4040;
const BAR_BG: u32 = 0xFF_2B_2B_2B;
const ENERGY: u32 = 0xFF_E6_3C_3C;
const WORK: u32 = 0xFF_3C_96_E6;
const PICK: u32 = 0xFF_5A_C8_5A;

fn energy_bar(game: &Game<'_>, batch: &mut HudBatch, origin: [f32; 2], energy: i64, cap: i64) {
    let [x, y, w, h] = ENERGY_BAR;
    let (x, y) = (origin[0] + x, origin[1] + y);
    game.native_fill(batch, [x, y, w, h], BAR_BG);
    let fill = (h * (energy as f32 / cap.max(1) as f32)).clamp(0.0, h);
    game.native_fill(batch, [x + 1.0, y + h - fill, w - 2.0, fill], ENERGY);
}

/// Draws a machine's screen: energy, progress, the model and its state; the
/// fabricator's drop choices.
pub(super) fn screen_draw(
    game: &Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut HudBatch,
    origin: [f32; 2],
) {
    if !rt::is_ours(block) {
        return;
    }
    let d = |i: usize| data.get(i).copied().unwrap_or(0);
    let [ox, oy] = origin;
    let energy = rt::join15(d(0), d(1));
    if block == rt::SIM_CHAMBER {
        energy_bar(game, batch, origin, energy, rt::SIM_POWER_CAP);
        let (px, py, pw, ph) = (ox + 50.0, oy + 72.0, 96.0, 4.0);
        game.native_fill(batch, [px, py, pw, ph], BAR_BG);
        let runtime = d(2);
        if runtime > 0 {
            let done = (rt::SIM_RUNTIME - runtime) as f32 / rt::SIM_RUNTIME as f32;
            game.native_fill(batch, [px, py, pw * done.clamp(0.0, 1.0), ph], WORK);
        }
        let model = d(4);
        if model > 0 && model as usize <= rt::MODELS.len() {
            let i = model as usize - 1;
            let data = d(5);
            let tier = rt::TIERS[rt::tier_of(data)];
            game.native_text(
                batch,
                rt::mob_pt(rt::MODELS[i].name),
                [ox + 50.0, oy + 10.0],
                DARK,
                false,
            );
            game.native_text(
                batch,
                tier.label_pt,
                [ox + 50.0, oy + 21.0],
                tier.rgb,
                false,
            );
            let acc = format!("Precisão {:.1}%", rt::accuracy(data) * 100.0);
            game.native_text(batch, &acc, [ox + 50.0, oy + 32.0], DARK, false);
            let cost = format!("{} FE/t", rt::MODELS[i].sim_cost);
            game.native_text(batch, &cost, [ox + 50.0, oy + 43.0], DARK, false);
        }
        let fail = rt::Fail::from_i32(d(3));
        let line = if fail == rt::Fail::None && runtime > 0 {
            if d(6) > 0 {
                "Simulando: sucesso"
            } else {
                "Simulando: falha"
            }
        } else {
            fail.short_pt()
        };
        game.native_text(batch, line, [ox + 50.0, oy + 58.0], DARK, false);
        return;
    }
    energy_bar(game, batch, origin, energy, rt::FAB_POWER_CAP);
    let (px, py, pw, ph) = (ox + 8.0, oy + 78.0, 82.0, 3.0);
    game.native_fill(batch, [px, py, pw, ph], BAR_BG);
    game.native_fill(
        batch,
        [
            px,
            py,
            pw * (d(2).clamp(0, rt::FAB_RUNTIME) as f32 / rt::FAB_RUNTIME as f32),
            ph,
        ],
        WORK,
    );
    let model = d(3);
    if model <= 0 || model as usize > rt::MODELS.len() {
        game.native_text(batch, "Insira", [ox + 30.0, oy + 10.0], DARK, false);
        game.native_text(batch, "uma", [ox + 30.0, oy + 21.0], DARK, false);
        game.native_text(batch, "Prediction", [ox + 30.0, oy + 32.0], DARK, false);
        return;
    }
    let m = &rt::MODELS[model as usize - 1];
    let sel = d(4) - 1;
    for (i, (item, _)) in m.fab.iter().enumerate() {
        let x = ox + FAB_GRID[0] + 18.0 * (i % 3) as f32;
        let y = oy + FAB_GRID[1] + 18.0 * (i / 3) as f32;
        if i as i32 == sel {
            game.native_fill(batch, [x - 1.0, y - 1.0, 18.0, 18.0], PICK);
        }
        game.native_item(batch, item, [x, y], 16.0);
    }
    let [cx, cy, cw, ch] = FAB_CLEAR_RECT;
    game.native_fill(batch, [ox + cx, oy + cy, cw, ch], BAR_BG);
    game.native_text(
        batch,
        "Limpar",
        [ox + cx + 8.0, oy + cy + 1.0],
        0xE0E0E0,
        false,
    );
}

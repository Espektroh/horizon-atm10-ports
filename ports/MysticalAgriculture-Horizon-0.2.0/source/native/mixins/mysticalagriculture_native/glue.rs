// Mystical Agriculture for Horizon: the rules (`mysticalagriculture_rt`) over the game. Added
// as `crate::game::mysticalagriculture_glue`; the hook helpers call it.

use super::mysticalagriculture_rt as rt;
use super::{Game, GameLevel};
use horizon_client::containers::{BlockEntity, ModBlockEntity, ModContainer};
use horizon_client::items::{ItemStack, Items};
use horizon_client::mod_api::NativeLootDrops;
use horizon_client::text::Text;
use horizon_core::position::BlockPos;
use horizon_world::nbt::{Compound, JavaString, Tag};
use horizon_world::registry::BlockStateId;

/// Vanilla crops the Mystical Fertilizer ripens at once, with their last age.
fn vanilla_crop_max_age(block: &str) -> Option<u32> {
    match block {
        "minecraft:wheat" | "minecraft:carrots" | "minecraft:potatoes" => Some(7),
        "minecraft:beetroots" => Some(3),
        _ => None,
    }
}

/// `native_loot_drops`: a ripe crop's drops as the original rolls them (the farmland below
/// decides the second essence and seed; Inferium grows with the farmland's tier; resource crops
/// sometimes give Fertilized Essence), and Inferium Essence from mobs, and Agradditions' boss drops.
pub(crate) fn loot_drops(
    game: &mut Game<'_>,
    context: &NativeLootDrops<'_>,
    drops: &mut Vec<ItemStack>,
) {
    if context.used || !game.native_on(rt::ID) {
        return;
    }
    if let Some((pos, state)) = context.block {
        let name = game.res.facts().get(state).name.to_string();
        let Some(crop) = rt::crop_of_block(&name) else {
            return;
        };
        // Ripe: its table rolled its essence (the table gives it at age 7 only).
        let Some(at) = drops.iter().position(|s| &*s.item == crop.essence) else {
            return;
        };
        // The level in place is the one `game.world` holds; another one's farmland isn't read.
        let below = if context.dimension == game.native_dimension() {
            rt::farmland_tier(&game.native_block([pos[0], pos[1] - 1, pos[2]]))
        } else {
            None
        };
        let h = rt::harvest(crop, below, rt::roll);
        drops[at].count = h.essence;
        if h.seeds > 1 {
            let seeds = format!("{}_seeds", crop.block.trim_end_matches("_crop"));
            if let Some(s) = drops.iter_mut().find(|s| *s.item == *seeds) {
                s.count = h.seeds;
            }
        }
        if h.fertilized > 0 {
            drops.push(ItemStack::new(rt::FERTILIZED_ESSENCE, h.fertilized));
        }
        return;
    }
    if let Some((_, kind)) = context.entity {
        // An empty table: the mob_drops rule is off (or the mob has none, as the dragon).
        if !context.table.is_empty() && rt::drops_inferium(kind, rt::roll()) {
            drops.push(ItemStack::new(rt::INFERIUM_ESSENCE, 1));
        }
        if let Some((item, count)) = rt::boss_drop(kind, rt::roll()) {
            drops.push(ItemStack::new(item, count));
        }
    }
}

/// `native_use_item_on`: an essence on farmland makes essence farmland (keeping its moisture),
/// the Mystical Fertilizer ripens a crop at once, Fertilized Essence grows one of this mod's
/// crops as bone meal grows wheat. Anything else is left to the item's own use.
pub(crate) fn use_item_on(
    game: &mut Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    if *claimed || !game.native_on(rt::ID) {
        return;
    }
    let item: String = stack.item.to_string();
    if !item.starts_with("mysticalagr") {
        return;
    }
    // 0.2.0: a watering can wets the farmland around and sometimes grows what is planted.
    if let Some(can) = rt::watering_can(&item) {
        *claimed = true;
        water(game, pos, can);
        return;
    }
    let block = game.native_block(pos);
    let creative = game
        .native_acting_player()
        .is_some_and(|p| p.game_mode == 1);
    if item == rt::MYSTICAL_FERTILIZER || item == rt::FERTILIZED_ESSENCE {
        let ours = rt::crop_of_block(&block);
        let max = match ours {
            Some(crop) if !rt::fertilizable(crop) => {
                // Tier 6 crops take no fertilizer (fertilizableTier6Crops = false).
                *claimed = true;
                return;
            }
            Some(_) => 7,
            // Fertilized Essence on anything else is bone meal (its template).
            None if item == rt::FERTILIZED_ESSENCE => return,
            None => match vanilla_crop_max_age(&block) {
                Some(max) => max,
                None => return,
            },
        };
        let Some(age) = game
            .native_block_property(pos, "age")
            .and_then(|a| a.parse::<u32>().ok())
        else {
            return;
        };
        if age >= max {
            return;
        }
        let next = if item == rt::MYSTICAL_FERTILIZER {
            max
        } else {
            rt::fertilized_age(age, max, rt::roll())
        };
        *claimed = true;
        let flags = horizon_world::level::flags::UPDATE_CLIENTS;
        if game.native_set_block_property_with(pos, "age", &next.to_string(), flags) && !creative {
            game.native_take_items(&item, 1);
        }
        return;
    }
    let Some((to, refund)) = rt::convert_farmland(&block, &item) else {
        return;
    };
    let moisture = game.native_block_property(pos, "moisture");
    let Some(state) = game.pack.assets.registry.default_state(to) else {
        return;
    };
    let dimension = game.native_dimension();
    if !game.native_set_block(
        dimension,
        pos,
        state,
        horizon_world::level::flags::UPDATE_ALL,
    ) {
        return;
    }
    *claimed = true;
    if let Some(m) = moisture {
        let _ = game.native_set_block_property_with(
            pos,
            "moisture",
            &m,
            horizon_world::level::flags::UPDATE_CLIENTS,
        );
    }
    if !creative {
        game.native_take_items(&item, 1);
    }
    if let Some(essence) = refund {
        if rt::roll() < rt::FARMLAND_REFUND_CHANCE {
            game.native_give_items(essence, 1);
        }
    }
}

/// `native_random_tick`: an Agradditions crop doesn't grow without its crux two blocks below.
pub(super) fn random_tick(
    level: &GameLevel<'_>,
    pos: BlockPos,
    state: BlockStateId,
    handled: &mut bool,
) {
    if *handled {
        return;
    }
    let name: &str = &level.facts.get(state).name;
    let Some(crop) = rt::crop_of_block(name) else {
        return;
    };
    if crop.crux.is_none() {
        return;
    }
    let two_below = level.world.block(pos.x, pos.y - 2, pos.z);
    let two_below: &str = &level.facts.get(two_below).name;
    if !rt::can_grow(crop, two_below) {
        *handled = true;
    }
}

/// `native_is_valid_bonemeal_target`: bone meal doesn't grow this mod's crops (the original's
/// `isBonemealSuccess` is false); it is kept, as Inferium Lab does. Fertilizers go through
/// `use_item_on` instead.
pub(super) fn bonemeal_target(
    level: &GameLevel<'_>,
    state: BlockStateId,
    valid: &mut Option<bool>,
) {
    if valid.is_none() && rt::crop_of_block(&level.facts.get(state).name).is_some() {
        *valid = Some(false);
    }
}

// ---- 0.2.0: the Infusion Altar and its pedestals, the wand, the watering cans ----------------
//
// The altar and each pedestal get a block entity of their own type with a container (the
// altar: input and output; the pedestal: one slot), so what they hold is saved with the world
// and spills when the block breaks (`spill.rs`, `Containers.dropContents`). Hoppers may take
// the altar's output, as from the original's output slot; nothing is put in by hoppers (the
// original's input takes one item at a time, which a hopper would overfill here). Clicks
// insert and take; the altar's tick reads its eight pedestals and the
// `mysticalagriculture:infusion` recipes and crafts after 100 ticks once a redstone signal (or
// the wand) switched it on. Clients get each slot's item in the update tag and draw it.

fn key(k: &str) -> JavaString {
    JavaString::from(k)
}

fn saved_int(nbt: &Compound, k: &str) -> i64 {
    match nbt.get(&key(k)) {
        Some(Tag::Long(v)) => *v,
        Some(Tag::Int(v)) => i64::from(*v),
        Some(Tag::Short(v)) => i64::from(*v),
        Some(Tag::Byte(v)) => i64::from(*v),
        _ => 0,
    }
}

/// An item stack as the altar's rules count it.
#[derive(Clone)]
struct Stack(ItemStack);

impl rt::Counted for Stack {
    fn count(&self) -> u32 {
        self.0.count
    }
    fn with_count(&self, n: u32) -> Self {
        let mut s = self.0.clone();
        s.count = n;
        Stack(s)
    }
}

/// What crafting leaves of one `s` (`getCraftingRemainingItem`: a bucket of a water bucket).
fn crafting_remainder(items: &Items, s: &Stack) -> Option<Stack> {
    let rest = items.get(&s.0.item)?.remainder.as_deref()?;
    Some(Stack(ItemStack::new(rest, 1)))
}

/// The altar's or a pedestal's block id and slots, read without changing them.
fn infusion_snapshot(game: &Game<'_>, pos: [i32; 3]) -> Option<(String, Vec<Option<ItemStack>>)> {
    match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
        Some(BlockEntity::Mod(m)) if rt::is_infusion_block(&m.id) => {
            let mut items = m.items.clone();
            items.resize(rt::infusion_slots(&m.id), None);
            Some((m.id.clone(), items))
        }
        _ => None,
    }
}

/// The pedestals around the altar at `pos` that stand where they should: their place and item.
fn pedestals_of(game: &Game<'_>, pos: [i32; 3]) -> Vec<([i32; 3], Option<ItemStack>)> {
    let mut out = Vec::new();
    for p in rt::pedestal_positions(pos) {
        if let Some((id, items)) = infusion_snapshot(game, p) {
            if id == rt::INFUSION_PEDESTAL {
                let item = items.into_iter().next().flatten().filter(|s| s.count > 0);
                out.push((p, item));
            }
        }
    }
    out
}

/// A recipe found for an altar: what it makes, and the pedestals it read.
struct Infusion {
    result: ItemStack,
    pedestals: Vec<([i32; 3], Option<ItemStack>)>,
}

/// The `mysticalagriculture:infusion` recipe the altar at `pos` makes with `input` and its
/// pedestals' items now (`InfusionRecipe.matches`), the first in the recipe table's order.
fn find_infusion(game: &Game<'_>, pos: [i32; 3], input: &ItemStack) -> Option<Infusion> {
    let pedestals = pedestals_of(game, pos);
    let held: Vec<Option<ItemStack>> = pedestals.iter().map(|(_, s)| s.clone()).collect();
    let filled = held.iter().flatten().count();
    for r in game.native_recipes_of_type(rt::INFUSION_TYPE) {
        let Some(list) = r.json["ingredients"].as_array() else {
            continue;
        };
        if list.len() != filled {
            continue;
        }
        let Some(center) = game.native_recipe_ingredient(&r.json["input"]) else {
            continue;
        };
        let mut ingredients = Vec::with_capacity(list.len());
        for j in list {
            match game.native_recipe_ingredient(j) {
                Some(i) => ingredients.push(i),
                None => break,
            }
        }
        if ingredients.len() != list.len() {
            continue;
        }
        if !rt::infusion_matches(&center, &ingredients, input, &held, |i, s| i.test(s)) {
            continue;
        }
        let Some(result) = game.native_recipe_result(&r.json["result"]) else {
            continue;
        };
        return Some(Infusion { result, pedestals });
    }
    None
}

pub(super) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && rt::is_infusion_block(block) {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

/// Only the altar ticks (the pedestals have no ticker in the original either).
pub(super) fn ticks(block: &str, ticks: &mut bool) {
    if block == rt::INFUSION_ALTAR {
        *ticks = true;
    }
}

pub(super) fn container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() {
        return;
    }
    if block == rt::INFUSION_ALTAR {
        *container = Some(ModContainer {
            slots: rt::ALTAR_SLOTS,
            faces: std::array::from_fn(|_| vec![rt::ALTAR_OUTPUT]),
            inputs: Vec::new(),
            outputs: vec![rt::ALTAR_OUTPUT],
            menu: None,
        });
    } else if block == rt::INFUSION_PEDESTAL {
        *container = Some(ModContainer {
            slots: rt::PEDESTAL_SLOTS,
            faces: Default::default(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            menu: None,
        });
    }
}

/// One server tick of an altar (`InfusionAltarTileEntity.tick`).
pub(super) fn altar_tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    if entity.id != rt::INFUSION_ALTAR || !game.native_on(rt::ID) {
        return;
    }
    if entity.items.len() < rt::ALTAR_SLOTS {
        entity.items.resize(rt::ALTAR_SLOTS, None);
    }
    let saved = rt::Altar {
        progress: saved_int(&entity.nbt, rt::KEY_PROGRESS).clamp(0, i64::from(rt::INFUSION_TICKS))
            as u32,
        active: saved_int(&entity.nbt, rt::KEY_ACTIVE) != 0,
    };
    let mut a = saved;
    let input: Option<ItemStack> = entity.items[rt::ALTAR_INPUT]
        .clone()
        .filter(|s| s.count > 0);
    let output_free = entity.items[rt::ALTAR_OUTPUT].is_none();
    let powered = if input.is_some() && !a.active {
        let dimension = game.native_dimension();
        game.native_neighbor_signal(dimension, pos) > 0
    } else {
        false
    };
    let step = {
        let g: &Game<'_> = game;
        rt::altar_step(&mut a, input.is_some(), powered, || {
            if !output_free {
                return None;
            }
            find_infusion(g, pos, input.as_ref()?)
        })
    };
    if let rt::AltarStep::Craft(inf) = step {
        let mut slot_in = entity.items[rt::ALTAR_INPUT].take().map(Stack);
        let mut slot_out = entity.items[rt::ALTAR_OUTPUT].take().map(Stack);
        let mut peds: Vec<Option<Stack>> = inf
            .pedestals
            .iter()
            .map(|(_, s)| s.clone().map(Stack))
            .collect();
        let done = {
            let items = world_items!(game);
            rt::infuse(
                &mut slot_in,
                &mut slot_out,
                &mut peds,
                Stack(inf.result),
                |s| crafting_remainder(items, s),
            )
        };
        entity.items[rt::ALTAR_INPUT] = slot_in.map(|s| s.0);
        entity.items[rt::ALTAR_OUTPUT] = slot_out.map(|s| s.0);
        if done {
            for ((p, old), new) in inf.pedestals.iter().zip(peds) {
                let new = new.map(|s| s.0);
                if *old == new {
                    continue;
                }
                game.native_with_mod_block_entity(*p, move |m| {
                    if m.id == rt::INFUSION_PEDESTAL {
                        if m.items.len() < rt::PEDESTAL_SLOTS {
                            m.items.resize(rt::PEDESTAL_SLOTS, None);
                        }
                        m.items[0] = new;
                    }
                });
                game.native_block_entity_sync(*p);
            }
            game.native_block_entity_sync(pos);
        }
    }
    // Clients get the altar again when what it shows changed by other means (a hopper took
    // the output): its update tag is sent only when asked.
    let shown = rt::shown_signature(
        entity
            .items
            .iter()
            .map(|s| s.as_ref().filter(|s| s.count > 0).map_or("", |s| &*s.item)),
    );
    let same = matches!(entity.nbt.get(&key(rt::KEY_SHOWN)), Some(Tag::String(v)) if v.to_string_lossy() == shown);
    if !same {
        entity.nbt.insert(
            key(rt::KEY_SHOWN),
            Tag::String(JavaString::from(shown.as_str())),
        );
        game.native_block_entity_sync(pos);
    }
    if a != saved {
        entity
            .nbt
            .insert(key(rt::KEY_PROGRESS), Tag::Int(a.progress as i32));
        entity
            .nbt
            .insert(key(rt::KEY_ACTIVE), Tag::Byte(i8::from(a.active)));
    }
}

/// What clients get of an altar or a pedestal: each slot's item (with its components).
pub(super) fn update_tag(entity: &ModBlockEntity, tag: &mut Option<Compound>) {
    if !rt::is_infusion_block(&entity.id) {
        return;
    }
    let mut view = Vec::with_capacity(rt::ALTAR_SLOTS);
    for i in 0..rt::infusion_slots(&entity.id) {
        let mut c = Compound::new();
        match entity
            .items
            .get(i)
            .and_then(Option::as_ref)
            .filter(|s| s.count > 0)
        {
            Some(s) => {
                c.insert(key("id"), Tag::String(JavaString::from(&*s.item)));
                if let Some(components) = horizon_client::item_data::encode_components(s) {
                    c.insert(key("components"), components);
                }
            }
            None => {
                c.insert(key("id"), Tag::String(JavaString::from("")));
            }
        }
        view.push(Tag::Compound(c));
    }
    let mut out = tag.take().unwrap_or_default();
    out.insert(key(rt::KEY_VIEW), Tag::List(view));
    *tag = Some(out);
}

/// The slots a client shows: from the update tag, or (the server's own copy, with no tag
/// merged) from the block entity's slots.
fn shown_items(entity: &ModBlockEntity) -> Vec<Option<ItemStack>> {
    if let Some(Tag::List(list)) = entity.nbt.get(&key(rt::KEY_VIEW)) {
        let mut out = Vec::with_capacity(list.len());
        for t in list {
            let Tag::Compound(c) = t else {
                out.push(None);
                continue;
            };
            let id = match c.get(&key("id")) {
                Some(Tag::String(s)) => s.to_string_lossy(),
                _ => String::new(),
            };
            if id.is_empty() {
                out.push(None);
                continue;
            }
            let (damage, extra) =
                horizon_client::item_data::decode_components(c.get(&key("components")));
            out.push(Some(ItemStack {
                item: std::sync::Arc::from(id.as_str()),
                count: 1,
                damage,
                extra,
            }));
        }
        return out;
    }
    entity
        .items
        .iter()
        .map(|s| s.clone().filter(|s| s.count > 0))
        .collect()
}

/// Draws the item floating over the altar or the pedestal, turning and bobbing
/// (`InfusionAltarRenderer`, `InfusionPedestalRenderer`), lit by the block above it.
pub(super) fn submit(
    game: &Game<'_>,
    pos: [i32; 3],
    block: &str,
    entity: &ModBlockEntity,
    draw: &mut super::BlockEntityDraw,
) {
    if !rt::is_infusion_block(block) {
        return;
    }
    let items = shown_items(entity);
    let filled: Vec<bool> = items.iter().map(Option::is_some).collect();
    let Some((slot, height)) = rt::shown_slot(block, &filled) else {
        return;
    };
    let Some(stack) = items.into_iter().nth(slot).flatten() else {
        return;
    };
    let packed = game.world.light(pos[0], pos[1] + 1, pos[2]);
    draw.light = (u32::from(packed >> 4) * 16) | ((u32::from(packed & 15) * 16) << 8);
    let is_block = world_items!(game)
        .get(&stack.item)
        .and_then(|info| info.block)
        .is_some();
    let ticks = draw.game_time as f64 + f64::from(draw.partial);
    let (scale, bob, yaw) = rt::float_pose(ticks, is_block);
    draw.push();
    draw.translate([0.5, height, 0.5]);
    draw.scale([scale, scale, scale]);
    draw.translate([0.0, bob, 0.0]);
    draw.rotate_about([0.0, 1.0, 0.0], yaw, [0.0, 0.0, 0.0]);
    game.native_draw_item(
        draw,
        &stack,
        horizon_assets::item_model::DisplayContext::Ground,
        None,
        slot as i32,
    );
    draw.pop();
}

/// Takes `n` from the acting hand's stack.
fn take_held(game: &mut Game<'_>, n: u32) {
    let slot = game.held_mut();
    let left = slot.as_ref().map_or(0, |s| s.count.saturating_sub(n));
    if left == 0 {
        *slot = None;
    } else if let Some(s) = slot.as_mut() {
        s.count = left;
    }
}

/// Gives `stack` to the acting player: into the empty hand that clicked, else the inventory.
/// What doesn't fit comes back.
fn give_player(game: &mut Game<'_>, stack: ItemStack) -> Option<ItemStack> {
    {
        let hand = game.held_mut();
        if hand.is_none() {
            *hand = Some(stack);
            return None;
        }
    }
    let uuid = game.native_acting_id();
    game.native_give_stack(uuid, stack)
}

fn tell(game: &mut Game<'_>, line: Text) {
    if let Some(p) = game.native_acting_player() {
        game.native_action_bar(p.uuid, line);
    }
}

/// The item's name as the game translates it.
fn item_name(game: &Game<'_>, stack: &ItemStack) -> Text {
    let id: &str = &stack.item;
    let (ns, path) = id.split_once(':').unwrap_or(("minecraft", id));
    let is_block = world_items!(game)
        .get(id)
        .and_then(|i| i.block)
        .is_some_and(|b| &*game.res.facts().get(b).name == id);
    let kind = if is_block { "block" } else { "item" };
    Text::translate(&format!("{kind}.{ns}.{path}"), &[])
}

/// "Altar de Infusão: 8/8 pedestais com item · receita: Sementes de Ferro · 40%".
fn altar_status(game: &Game<'_>, pos: [i32; 3], items: &[Option<ItemStack>]) -> Text {
    let pedestals = pedestals_of(game, pos);
    let filled = pedestals.iter().filter(|(_, s)| s.is_some()).count();
    let mut line = Text::literal(format!(
        "Altar de Infusão: {} pedestais ({} com item)",
        pedestals.len(),
        filled
    ));
    match items.get(rt::ALTAR_INPUT).and_then(Option::as_ref) {
        None => line = line.append(Text::literal(" · altar vazio")),
        Some(input) => match find_infusion(game, pos, input) {
            Some(inf) => {
                line = line
                    .append(Text::literal(" · receita: "))
                    .append(item_name(game, &inf.result))
                    .append(Text::literal(" · ligue com redstone"));
            }
            None => line = line.append(Text::literal(" · sem receita")),
        },
    }
    line
}

/// A right click on an altar or a pedestal, before the block and the item act
/// (`InfusionAltarBlock.useItemOn`, `InfusionPedestalBlock.useItemOn`). Sneaking with something
/// in hand is left to the game (a block placed against it).
pub(super) fn right_click(game: &mut Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if *canceled {
        return;
    }
    let Some((id, items)) = infusion_snapshot(game, pos) else {
        return;
    };
    if !game.native_on(rt::ID) {
        return;
    }
    let held: Option<ItemStack> = game.held_mut().clone().filter(|s| s.count > 0);
    let hands_full = held.is_some() || game.offhand.as_ref().is_some_and(|s| s.count > 0);
    if game.native_sneaking() && hands_full {
        return;
    }
    *canceled = true;
    let altar = id == rt::INFUSION_ALTAR;
    let held_id: Option<String> = held.as_ref().map(|s| s.item.to_string());
    let click = if altar {
        rt::altar_click(
            items[rt::ALTAR_INPUT].is_some(),
            items[rt::ALTAR_OUTPUT].is_some(),
            held_id.as_deref(),
        )
    } else {
        rt::pedestal_click(items[0].is_some(), held.is_some())
    };
    let slots = rt::infusion_slots(&id);
    match click {
        rt::Click::Insert => {
            let Some(h) = held else {
                return;
            };
            let mut one = h.clone();
            one.count = 1;
            let mut now = items.clone();
            now[0] = Some(one.clone());
            let put = game
                .native_with_mod_block_entity(pos, move |m| {
                    if m.items.len() < slots {
                        m.items.resize(slots, None);
                    }
                    if m.items[0].is_none() {
                        m.items[0] = Some(one);
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false);
            if put {
                take_held(game, 1);
                game.native_block_entity_sync(pos);
                if altar {
                    // What the altar would make now (the original shows nothing; this helps).
                    let line = altar_status(game, pos, &now);
                    tell(game, line);
                }
            }
        }
        rt::Click::TakeOutput | rt::Click::TakeInput => {
            let slot = if click == rt::Click::TakeOutput {
                rt::ALTAR_OUTPUT
            } else {
                0
            };
            let taken = game
                .native_with_mod_block_entity(pos, move |m| {
                    m.items.get_mut(slot).and_then(Option::take)
                })
                .flatten();
            let Some(stack) = taken else {
                return;
            };
            if let Some(back) = give_player(game, stack) {
                // No room: it stays where it was.
                game.native_with_mod_block_entity(pos, move |m| {
                    if let Some(s) = m.items.get_mut(slot) {
                        *s = Some(back);
                    }
                });
                tell(game, Text::literal("Inventário cheio."));
            }
            game.native_block_entity_sync(pos);
        }
        rt::Click::Activate => {
            game.native_with_mod_block_entity(pos, |m| {
                m.nbt.insert(key(rt::KEY_ACTIVE), Tag::Byte(1));
            });
            tell(game, Text::literal("Altar de Infusão ligado."));
        }
        rt::Click::Nothing => {
            if altar {
                let line = altar_status(game, pos, &items);
                tell(game, line);
            }
        }
    }
}

/// When each player last watered (game time), for the cans' 5-tick cooldown.
static MYSTICALAGRICULTURE_WATERED: std::sync::Mutex<Vec<(u128, i64)>> =
    std::sync::Mutex::new(Vec::new());

fn watering_allowed(uuid: u128, now: i64) -> bool {
    let Ok(mut v) = MYSTICALAGRICULTURE_WATERED.lock() else {
        return true;
    };
    let last = v.iter().find(|(u, _)| *u == uuid).map(|(_, t)| *t);
    if !rt::may_water(last, now) {
        return false;
    }
    match v.iter_mut().find(|(u, _)| *u == uuid) {
        Some(e) => e.1 = now,
        None => {
            if v.len() >= 256 {
                v.remove(0);
            }
            v.push((uuid, now));
        }
    }
    true
}

fn block_name_at(game: &Game<'_>, x: i32, y: i32, z: i32) -> std::sync::Arc<str> {
    let state = game.world.block(x, y, z);
    game.res.facts().get(state).name.clone()
}

/// A watering can used on the block at `pos` (`BaseWateringCanItem.doWater`): farmland in its
/// area (`range` wide, as high) gets moisture 7, and `chance` of the time every plant in it (3
/// blocks up and down) gets a random tick, through the game's own random tick (crop growth,
/// sapling growth, this mod's cruxes all as usual). The original's rain particles and sound
/// aren't drawn; the can needs no filling here (see LEIA-ME).
fn water(game: &mut Game<'_>, pos: [i32; 3], can: &rt::WateringCan) {
    let uuid = game.native_acting_id();
    let now = game.world.game_time;
    if !watering_allowed(uuid, now) {
        return;
    }
    let r = rt::watering_radius(can.range);
    let mut wet: Vec<[i32; 3]> = Vec::new();
    for x in -r..=r {
        for y in -r..=r {
            for z in -r..=r {
                let p = [pos[0] + x, pos[1] + y, pos[2] + z];
                if rt::is_farmland(&block_name_at(game, p[0], p[1], p[2])) {
                    wet.push(p);
                }
            }
        }
    }
    for p in wet {
        let dry = game
            .native_block_property(p, "moisture")
            .and_then(|m| m.parse::<u8>().ok())
            .is_some_and(|m| m < 7);
        if dry {
            let _ = game.native_set_block_property_with(
                p,
                "moisture",
                "7",
                horizon_world::level::flags::UPDATE_ALL,
            );
        }
    }
    if rt::roll() > can.chance {
        return;
    }
    let mut cells: Vec<BlockPos> = Vec::new();
    for x in -r..=r {
        for y in -rt::WATERING_HEIGHT..=rt::WATERING_HEIGHT {
            for z in -r..=r {
                let (px, py, pz) = (pos[0] + x, pos[1] + y, pos[2] + z);
                if rt::is_growable(&block_name_at(game, px, py, pz)) {
                    cells.push(BlockPos::new(px, py, pz));
                }
            }
        }
    }
    if cells.is_empty() {
        return;
    }
    game.with_level(|b, level, ticks| {
        for p in &cells {
            b.random_tick(level, ticks, *p);
        }
    });
    let _ = game.process_level_events();
}

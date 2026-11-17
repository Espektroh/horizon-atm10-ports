// Sushi Go Crafting for Horizon: the rules (`sushigocrafting_rt`) over the game. Added as
// `crate::game::sushigocrafting_glue`; the hook helpers call it.

use super::native_hooks::NativeScreen;
use super::sushigocrafting_rt as rt;
use super::{Game, GameLevel};
use horizon_client::containers::{BlockEntity, ModBlockEntity, ModContainer};
use horizon_client::hud::HudBatch;
use horizon_client::items::ItemStack;
use horizon_client::mod_api::NativeLootDrops;
use horizon_client::text::Text;
use horizon_core::position::BlockPos;
use horizon_world::level::Ticks;
use horizon_world::nbt::{Compound, JavaString, Tag};
use horizon_world::registry::BlockStateId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Board,
    Cooker,
    Barrel,
}

fn kind_of(block: &str) -> Option<Kind> {
    match block {
        rt::CUTTING_BOARD => Some(Kind::Board),
        rt::RICE_COOKER => Some(Kind::Cooker),
        rt::FERMENTATION_BARREL => Some(Kind::Barrel),
        _ => None,
    }
}

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

fn slot_of(s: &Option<ItemStack>) -> Option<rt::Slot> {
    s.as_ref().filter(|s| s.count > 0).map(|s| rt::Slot {
        item: s.item.to_string(),
        count: s.count,
    })
}

/// A slot the rules changed: the same item keeps its stack (components and all) with the new
/// count; another item is a new plain stack.
fn apply(old: Option<ItemStack>, new: Option<rt::Slot>) -> Option<ItemStack> {
    let new = new?;
    match old {
        Some(mut s) if *s.item == *new.item => {
            s.count = new.count;
            Some(s)
        }
        _ => Some(ItemStack::new(&new.item, new.count)),
    }
}

fn write_back(
    entity: &mut ModBlockEntity,
    before: &[Option<rt::Slot>],
    after: Vec<Option<rt::Slot>>,
) {
    for (i, new) in after.into_iter().enumerate() {
        if before.get(i) == Some(&new) {
            continue;
        }
        let old = entity.items.get_mut(i).and_then(Option::take);
        if let Some(slot) = entity.items.get_mut(i) {
            *slot = apply(old, new);
        }
    }
}

// ---------------------------------------------------------------- block entities

pub(super) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && kind_of(block).is_some() {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

pub(super) fn ticks(block: &str, ticks: &mut bool) {
    if matches!(kind_of(block), Some(Kind::Cooker | Kind::Barrel)) {
        *ticks = true;
    }
}

/// The board: one slot, no menu, nothing through its faces (clicks put and take). The cooker: the
/// furnace's faces (rice from above, fuel from the sides, cooked rice from below). The barrel: its
/// input from above and the sides, its output from below.
pub(super) fn container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() {
        return;
    }
    *container = match kind_of(block) {
        Some(Kind::Board) => Some(ModContainer {
            slots: 1,
            faces: Default::default(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            menu: None,
        }),
        Some(Kind::Cooker) => Some(ModContainer {
            slots: rt::COOKER_SLOTS,
            faces: [
                vec![rt::OUTPUT],
                vec![rt::INPUT],
                vec![rt::FUEL],
                vec![rt::FUEL],
                vec![rt::FUEL],
                vec![rt::FUEL],
            ],
            inputs: vec![rt::INPUT, rt::FUEL],
            outputs: vec![rt::OUTPUT],
            menu: Some("minecraft:furnace".to_owned()),
        }),
        Some(Kind::Barrel) => Some(ModContainer {
            slots: rt::COOKER_SLOTS,
            faces: [
                vec![rt::OUTPUT],
                vec![rt::INPUT],
                vec![rt::INPUT],
                vec![rt::INPUT],
                vec![rt::INPUT],
                vec![rt::INPUT],
            ],
            inputs: vec![rt::INPUT],
            outputs: vec![rt::OUTPUT],
            menu: Some("minecraft:furnace".to_owned()),
        }),
        None => return,
    };
}

/// The fermenting recipes (`sushigocrafting:fermenting_barrel`), each with whether `input` fits it.
fn barrel_recipes(game: &Game<'_>, input: Option<&ItemStack>) -> Vec<(rt::Ferment, bool)> {
    let mut out = Vec::new();
    for r in game.native_recipes_of_type(rt::FERMENTING_TYPE) {
        let inp = &r.json["input"];
        let takes_item = !(inp.is_null() || inp.as_array().is_some_and(|a| a.is_empty()));
        let fits = if takes_item {
            let Some(ingredient) = game.native_recipe_ingredient(inp) else {
                continue;
            };
            input.is_some_and(|s| ingredient.test(s))
        } else {
            false
        };
        let Some(result) = game.native_recipe_result(&r.json["output"]) else {
            continue;
        };
        let fluid = r.json["fluid"]["id"].as_str().unwrap_or("").to_owned();
        let amount = r.json["fluid"]["amount"]
            .as_i64()
            .unwrap_or(0)
            .clamp(0, i64::from(rt::BARREL_CAPACITY)) as i32;
        out.push((
            rt::Ferment {
                takes_item,
                fluid,
                amount,
                output: result.item.to_string(),
                count: result.count,
            },
            fits,
        ));
    }
    out
}

/// One server tick of the rice cooker or the fermentation barrel.
pub(super) fn tick(game: &mut Game<'_>, _pos: [i32; 3], entity: &mut ModBlockEntity) {
    let Some(kind) = kind_of(&entity.id) else {
        return;
    };
    if kind == Kind::Board || !game.native_on(rt::ID) {
        return;
    }
    if entity.items.len() < rt::COOKER_SLOTS {
        entity.items.resize(rt::COOKER_SLOTS, None);
    }
    let before: Vec<Option<rt::Slot>> = entity.items.iter().map(slot_of).collect();
    let mut slots = before.clone();
    if kind == Kind::Cooker {
        let mut s = rt::Cooker {
            water: saved_int(&entity.nbt, "water").clamp(0, i64::from(rt::COOKER_WATER_CAPACITY))
                as i32,
            burn: saved_int(&entity.nbt, "burn").clamp(-100_000, 100_000) as i32,
            progress: saved_int(&entity.nbt, "progress").clamp(0, i64::from(rt::WORK_TICKS)) as i32,
        };
        {
            let items = world_items!(game);
            let providers = game.res.providers();
            rt::cooker_tick(
                &mut s,
                &mut slots,
                |id| providers.burn_time(items, &ItemStack::new(id, 1), false),
                |id| items.get(id).and_then(|i| i.remainder.clone()),
            );
        }
        write_back(entity, &before, slots);
        entity.nbt.insert(key("water"), Tag::Int(s.water));
        entity.nbt.insert(key("burn"), Tag::Int(s.burn));
        entity.nbt.insert(key("progress"), Tag::Int(s.progress));
        entity.data = vec![
            rt::permille(s.progress, rt::WORK_TICKS),
            s.water,
            s.burn.clamp(0, 32_767),
            0,
        ];
    } else {
        let mut s = rt::Barrel {
            fluid: saved_int(&entity.nbt, "fluid").clamp(0, 2) as u8,
            amount: saved_int(&entity.nbt, "amount").clamp(0, i64::from(rt::BARREL_CAPACITY))
                as i32,
            progress: saved_int(&entity.nbt, "progress").clamp(0, i64::from(rt::WORK_TICKS)) as i32,
        };
        let recipes = {
            let g: &Game<'_> = game;
            barrel_recipes(g, entity.items[rt::INPUT].as_ref().filter(|s| s.count > 0))
        };
        rt::barrel_tick(&mut s, &mut slots, &recipes, rt::roll());
        write_back(entity, &before, slots);
        entity
            .nbt
            .insert(key("fluid"), Tag::Int(i32::from(s.fluid)));
        entity.nbt.insert(key("amount"), Tag::Int(s.amount));
        entity.nbt.insert(key("progress"), Tag::Int(s.progress));
        entity.data = vec![
            rt::permille(s.progress, rt::WORK_TICKS),
            s.amount,
            i32::from(s.fluid),
            0,
        ];
    }
}

// ---------------------------------------------------------------- screens

/// The furnace menu's three slots: the cooker's as a furnace's; the barrel's input and output
/// (its middle slot is hidden).
pub(super) fn screen(block: &str, screen: &mut Option<NativeScreen>) {
    let slots = match kind_of(block) {
        Some(Kind::Cooker) => vec![Some([56.0, 17.0]), Some([56.0, 53.0]), Some([116.0, 35.0])],
        Some(Kind::Barrel) => vec![Some([56.0, 35.0]), None, Some([116.0, 35.0])],
        _ => return,
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
const WATER_COLOR: u32 = 0xFF_3F_76_E4;
const MILK_COLOR: u32 = 0xFF_F0_F0_F0;
const WORK: u32 = 0xFF_3C_D2_3C;
const FIRE: u32 = 0xFF_F0_8C_28;

/// A tank (left), the work arrow, the cooker's fire, and the amounts as text.
pub(super) fn screen_draw(
    game: &Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut HudBatch,
    origin: [f32; 2],
) {
    let Some(kind) = kind_of(block) else {
        return;
    };
    if kind == Kind::Board {
        return;
    }
    let d = |i: usize| data.get(i).copied().unwrap_or(0).max(0);
    let [ox, oy] = origin;
    let (capacity, color, label) = if kind == Kind::Cooker {
        (rt::COOKER_WATER_CAPACITY, WATER_COLOR, "Água")
    } else {
        match d(2) {
            1 => (rt::BARREL_CAPACITY, WATER_COLOR, "Água"),
            2 => (rt::BARREL_CAPACITY, MILK_COLOR, "Leite"),
            _ => (rt::BARREL_CAPACITY, WATER_COLOR, "Vazio"),
        }
    };
    let amount = d(1).min(capacity);
    let (tx, ty, tw, th) = (ox + 8.0, oy + 17.0, 10.0, 52.0);
    game.native_fill(batch, [tx, ty, tw, th], BAR_BG);
    let h = th * amount as f32 / capacity as f32;
    game.native_fill(batch, [tx + 1.0, ty + th - h, tw - 2.0, h], color);
    let (ax, ay, aw, ah) = (ox + 79.0, oy + 39.0, 24.0, 8.0);
    game.native_fill(batch, [ax, ay, aw, ah], BAR_BG);
    game.native_fill(batch, [ax, ay, aw * d(0) as f32 / 1000.0, ah], WORK);
    let line = if label == "Vazio" {
        format!("Vazio (0/{capacity} mB)")
    } else {
        format!("{label}: {amount}/{capacity} mB")
    };
    game.native_text(batch, &line, [ox + 22.0, oy + 72.0 - 10.0], DARK, false);
    if kind == Kind::Cooker {
        // The fire: one batch is 200 fuel ticks.
        let burn = d(2);
        let (fx, fy, fw, fh) = (ox + 57.0, oy + 37.0, 14.0, 13.0);
        game.native_fill(batch, [fx, fy, fw, fh], BAR_BG);
        let f = fh * (burn.min(rt::BURN_PER_BATCH) as f32 / rt::BURN_PER_BATCH as f32);
        game.native_fill(batch, [fx + 1.0, fy + fh - f, fw - 2.0, f], FIRE);
        let batches = format!(
            "{:.1} lotes",
            f64::from(burn) / f64::from(rt::BURN_PER_BATCH)
        );
        game.native_text(batch, &batches, [ox + 108.0, oy + 18.0], DARK, false);
    }
}

/// Shift-click from the inventory: rice to the cooker's input and fuel to its fuel slot; what a
/// fermenting recipe takes to the barrel's input. Anything else is left to the game.
pub(super) fn quick_move(
    game: &mut Game<'_>,
    entity: &mut ModBlockEntity,
    container_slots: usize,
    slot: usize,
    stack: &mut ItemStack,
    handled: &mut bool,
) {
    if *handled || slot < container_slots || stack.count == 0 {
        return;
    }
    let Some(kind) = kind_of(&entity.id) else {
        return;
    };
    let target = match kind {
        Kind::Cooker => {
            if *stack.item == *rt::RICE {
                rt::INPUT
            } else {
                let fuel = {
                    let items = world_items!(game);
                    game.res.providers().burn_time(items, stack, false)
                };
                if fuel > 0 {
                    rt::FUEL
                } else {
                    return;
                }
            }
        }
        Kind::Barrel => {
            let g: &Game<'_> = game;
            if barrel_recipes(g, Some(&*stack))
                .iter()
                .any(|(_, fits)| *fits)
            {
                rt::INPUT
            } else {
                return;
            }
        }
        Kind::Board => return,
    };
    if target >= container_slots {
        return;
    }
    if entity.items.len() < rt::COOKER_SLOTS {
        entity.items.resize(rt::COOKER_SLOTS, None);
    }
    let max = world_items!(game)
        .get(&stack.item)
        .map_or(64, |i| i.max_stack);
    let moved = match entity.items[target].as_mut() {
        None => stack.count.min(max),
        Some(have) if have.same_item_same_components(stack) => {
            max.saturating_sub(have.count).min(stack.count)
        }
        Some(_) => 0,
    };
    if moved == 0 {
        return;
    }
    match entity.items[target].as_mut() {
        Some(have) => have.count += moved,
        None => {
            let mut put = stack.clone();
            put.count = moved;
            entity.items[target] = Some(put);
        }
    }
    stack.count -= moved;
    *handled = true;
}

// ---------------------------------------------------------------- clicks

fn tell(game: &mut Game<'_>, line: Text) {
    if let Some(p) = game.native_acting_player() {
        game.native_action_bar(p.uuid, line);
    }
}

fn creative(game: &Game<'_>) -> bool {
    game.native_acting_player()
        .is_some_and(|p| p.game_mode == 1)
}

/// The item's name as the game translates it.
fn item_name(game: &Game<'_>, id: &str) -> Text {
    let (ns, path) = id.split_once(':').unwrap_or(("minecraft", id));
    let is_block = world_items!(game)
        .get(id)
        .and_then(|i| i.block)
        .is_some_and(|b| &*game.res.facts().get(b).name == id);
    let kind = if is_block { "block" } else { "item" };
    Text::translate(&format!("{kind}.{ns}.{path}"), &[])
}

/// Gives `stack` to the acting player: into the empty hand that clicked, else the inventory; what
/// doesn't fit drops on top of the block at `pos`.
fn give_player(game: &mut Game<'_>, stack: ItemStack, pos: [i32; 3]) {
    {
        let hand = game.held_mut();
        if hand.is_none() {
            *hand = Some(stack);
            return;
        }
    }
    let uuid = game.native_acting_id();
    if let Some(rest) = game.native_give_stack(uuid, stack) {
        let at = [
            f64::from(pos[0]) + 0.5,
            f64::from(pos[1]) + 1.0,
            f64::from(pos[2]) + 0.5,
        ];
        game.spawn_item(rest, at, [0.0, 0.0, 0.0], 10);
    }
}

/// The cutting recipe for `stack` (`sushigocrafting:cutting_board`): its result and, for a plain
/// item, the count range.
fn cut_recipe(game: &Game<'_>, stack: &ItemStack) -> Option<(String, u32, u32)> {
    for r in game.native_recipes_of_type(rt::CUTTING_TYPE) {
        let Some(ingredient) = game.native_recipe_ingredient(&r.json["input"]) else {
            continue;
        };
        if !ingredient.test(stack) {
            continue;
        }
        let Some(id) = r.json["result"].as_str() else {
            continue;
        };
        if world_items!(game).get(id).is_none() {
            continue;
        }
        let min = r.json["count"]["min"].as_u64().unwrap_or(1).clamp(1, 64) as u32;
        let max = r.json["count"]["max"]
            .as_u64()
            .unwrap_or(u64::from(min))
            .clamp(u64::from(min), 64) as u32;
        return Some((id.to_owned(), min, max));
    }
    None
}

/// How many a cut gives: an amount item's rolled portions, else the recipe's range.
fn cut_count(result: &str, min: u32, max: u32) -> u32 {
    match rt::amount_of(result) {
        Some(a) => rt::roll_portions(a, rt::roll()),
        None => min + rt::next_int(rt::roll(), max - min + 1),
    }
}

fn board_snapshot(game: &Game<'_>, pos: [i32; 3]) -> Option<(Option<ItemStack>, u32)> {
    match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
        Some(BlockEntity::Mod(m)) if m.id == rt::CUTTING_BOARD => {
            let item = m.items.first().cloned().flatten().filter(|s| s.count > 0);
            let clicks = saved_int(&m.nbt, "clicks").clamp(0, i64::from(rt::CUTS)) as u32;
            Some((item, clicks))
        }
        _ => None,
    }
}

fn board_status(game: &Game<'_>, item: Option<&ItemStack>, clicks: u32) -> Text {
    let mut line = Text::literal("Tábua de Corte: ");
    match item {
        None => line = line.append(Text::literal("vazia")),
        Some(s) => {
            line = line
                .append(Text::literal(format!("{}x ", s.count)))
                .append(item_name(game, &s.item))
                .append(Text::literal(format!(" · cortes {clicks}/{}", rt::CUTS)));
        }
    }
    line
}

/// A right click on the cutting board (`CuttingBoardTile.onActivated`), on a ripe avocado leaf
/// (`AvocadoLeavesBlock.useItemOn`), or with a water or milk bucket on the cooker or the barrel.
/// Sneaking with something in hand is left to the game (a block placed against it).
pub(super) fn right_click(game: &mut Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if *canceled || !game.native_on(rt::ID) {
        return;
    }
    let block = game.native_block(pos);
    if !block.starts_with("sushigocrafting:") {
        return;
    }
    let held: Option<ItemStack> = game.held_mut().clone().filter(|s| s.count > 0);
    let sneaking = game.native_sneaking();
    if sneaking && held.is_some() {
        return;
    }
    if block == rt::AVOCADO_LEAVES {
        if game.native_block_property(pos, "stage").as_deref() == Some("2") {
            *canceled = true;
            game.native_set_block_property_with(
                pos,
                "stage",
                "1",
                horizon_world::level::flags::UPDATE_ALL,
            );
            give_player(game, ItemStack::new(rt::AVOCADO, 1), pos);
        }
        return;
    }
    match kind_of(&block) {
        Some(Kind::Board) => board_click(game, pos, held, sneaking, canceled),
        Some(kind @ (Kind::Cooker | Kind::Barrel)) => {
            let Some(h) = held else {
                return;
            };
            let fluid = rt::bucket_fluid(&h.item);
            if fluid == 0 || (kind == Kind::Cooker && fluid != 1) {
                return;
            }
            *canceled = true;
            let filled = game
                .native_with_mod_block_entity(pos, |m| {
                    if kind == Kind::Cooker {
                        let have = saved_int(&m.nbt, "water") as i32;
                        if !rt::tank_takes(1, have, 1, rt::COOKER_WATER_CAPACITY) {
                            return None;
                        }
                        m.nbt.insert(key("water"), Tag::Int(have + rt::BUCKET));
                        Some(have + rt::BUCKET)
                    } else {
                        let held_fluid = saved_int(&m.nbt, "fluid").clamp(0, 2) as u8;
                        let have = saved_int(&m.nbt, "amount") as i32;
                        if !rt::tank_takes(held_fluid, have, fluid, rt::BARREL_CAPACITY) {
                            return None;
                        }
                        m.nbt.insert(key("fluid"), Tag::Int(i32::from(fluid)));
                        m.nbt.insert(key("amount"), Tag::Int(have + rt::BUCKET));
                        Some(have + rt::BUCKET)
                    }
                })
                .flatten();
            match filled {
                Some(now) => {
                    if !creative(game) {
                        *game.held_mut() = Some(ItemStack::new("minecraft:bucket", 1));
                    }
                    let name = if fluid == 2 { "Leite" } else { "Água" };
                    let cap = if kind == Kind::Cooker {
                        rt::COOKER_WATER_CAPACITY
                    } else {
                        rt::BARREL_CAPACITY
                    };
                    tell(game, Text::literal(format!("{name}: {now}/{cap} mB")));
                }
                None => tell(game, Text::literal("O tanque não cabe mais esse balde.")),
            }
        }
        _ => {}
    }
}

fn board_click(
    game: &mut Game<'_>,
    pos: [i32; 3],
    held: Option<ItemStack>,
    sneaking: bool,
    canceled: &mut bool,
) {
    let Some((on_board, clicks)) = board_snapshot(game, pos) else {
        return;
    };
    *canceled = true;
    let cuttable = held.as_ref().is_some_and(|h| cut_recipe(game, h).is_some());
    let click = rt::board_click(
        held.as_ref().map(|s| &*s.item),
        cuttable,
        on_board.is_some(),
        sneaking,
        clicks,
    );
    match click {
        rt::BoardClick::Insert => {
            let Some(h) = held else {
                return;
            };
            let put = h.clone();
            let ok = game
                .native_with_mod_block_entity(pos, move |m| {
                    if m.items.is_empty() {
                        m.items.resize(1, None);
                    }
                    if m.items[0].as_ref().is_some_and(|s| s.count > 0) {
                        return false;
                    }
                    m.items[0] = Some(put);
                    m.nbt.insert(key("clicks"), Tag::Int(0));
                    true
                })
                .unwrap_or(false);
            if ok {
                *game.held_mut() = None;
                game.native_block_entity_sync(pos);
                let line = board_status(game, Some(&h), 0);
                tell(game, line);
            }
        }
        rt::BoardClick::Knife(cut) => {
            let Some(item) = on_board else {
                return;
            };
            if !cut {
                let next = clicks + 1;
                game.native_with_mod_block_entity(pos, |m| {
                    m.nbt.insert(key("clicks"), Tag::Int(next as i32));
                });
                let line = board_status(game, Some(&item), next);
                tell(game, line);
                return;
            }
            let recipe = cut_recipe(game, &item);
            game.native_with_mod_block_entity(pos, |m| {
                m.nbt.insert(key("clicks"), Tag::Int(0));
                if recipe.is_some()
                    && let Some(s) = m.items.get_mut(0)
                {
                    let left = s.as_ref().map_or(0, |s| s.count.saturating_sub(1));
                    if left == 0 {
                        *s = None;
                    } else if let Some(s) = s.as_mut() {
                        s.count = left;
                    }
                }
            });
            game.native_block_entity_sync(pos);
            if let Some((result, min, max)) = recipe {
                let n = cut_count(&result, min, max);
                give_player(game, ItemStack::new(&result, n), pos);
                let line =
                    Text::literal(format!("Cortado: {n}x ")).append(item_name(game, &result));
                tell(game, line);
            }
        }
        rt::BoardClick::Take => {
            let taken = game
                .native_with_mod_block_entity(pos, |m| {
                    m.nbt.insert(key("clicks"), Tag::Int(0));
                    m.items.get_mut(0).and_then(Option::take)
                })
                .flatten();
            game.native_block_entity_sync(pos);
            if let Some(stack) = taken {
                give_player(game, stack, pos);
            }
        }
        rt::BoardClick::Nothing => {
            let line = board_status(game, on_board.as_ref(), clicks);
            tell(game, line);
        }
    }
}

/// Bone meal on rice (`CropBlock.performBonemeal`, a quarter of the time nothing), on the avocado
/// sapling (`SaplingBlock`: 45% to advance, an avocado tree, not an oak) and on avocado leaves
/// (stage 1 ripens). The meal is used up as vanilla uses it up (not in creative).
pub(super) fn use_item_on(
    game: &mut Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    if *claimed || *stack.item != *"minecraft:bone_meal" || !game.native_on(rt::ID) {
        return;
    }
    let block = game.native_block(pos);
    let flags = horizon_world::level::flags::UPDATE_ALL;
    match block.as_str() {
        rt::RICE_CROP => {
            let age = game
                .native_block_property(pos, "age")
                .and_then(|a| a.parse::<u32>().ok())
                .unwrap_or(rt::MAX_AGE);
            if age >= rt::MAX_AGE {
                return;
            }
            *claimed = true;
            let next = (age + rt::bonemeal_ages(rt::roll())).min(rt::MAX_AGE);
            if next != age {
                game.native_set_block_property_with(pos, "age", &next.to_string(), flags);
            }
        }
        rt::AVOCADO_SAPLING => {
            *claimed = true;
            if rt::roll() < rt::SAPLING_BONEMEAL_CHANCE {
                if game.native_block_property(pos, "stage").as_deref() == Some("1") {
                    game.with_level(|_, level, ticks| grow_tree(level, ticks, pos));
                    let _ = game.process_level_events();
                } else {
                    game.native_set_block_property_with(pos, "stage", "1", flags);
                }
            }
        }
        rt::AVOCADO_LEAVES => {
            let stage = game.native_block_property(pos, "stage");
            if stage.as_deref() == Some("0") || stage.is_none() {
                return;
            }
            *claimed = true;
            if stage.as_deref() == Some("1") {
                game.native_set_block_property_with(pos, "stage", "2", flags);
            }
        }
        _ => return,
    }
    if !creative(game) {
        game.native_take_items("minecraft:bone_meal", 1);
    }
}

// ---------------------------------------------------------------- the level: growth

fn state_prop(level: &GameLevel<'_>, state: BlockStateId, k: &str) -> Option<String> {
    level.registry.state(state)?.properties.get(k).cloned()
}

fn name_at(level: &GameLevel<'_>, p: [i32; 3]) -> String {
    let s = level.world.block(p[0], p[1], p[2]);
    level.facts.get(s).name.to_string()
}

/// `getRawBrightness(pos, darken)`: the block light, or the sky's less `darken`.
fn brightness(level: &GameLevel<'_>, p: [i32; 3], darken: i32) -> i32 {
    let l = level.world.light(p[0], p[1], p[2]);
    let block = i32::from(l & 15);
    let sky = i32::from(l >> 4) - darken;
    block.max(sky)
}

/// `CropBlock.getGrowthSpeed` read from the world around `p`.
fn growth_speed(level: &GameLevel<'_>, p: [i32; 3], crop: &str) -> f32 {
    let mut soil = [rt::Soil::Other; 9];
    let mut i = 0;
    for dx in -1..=1 {
        for dz in -1..=1 {
            let s = level.world.block(p[0] + dx, p[1] - 1, p[2] + dz);
            soil[i] = match state_prop(level, s, "moisture") {
                Some(m) if m != "0" => rt::Soil::Moist,
                Some(_) => rt::Soil::Farmland,
                None => rt::Soil::Other,
            };
            i += 1;
        }
    }
    let same = |dx: i32, dz: i32| name_at(level, [p[0] + dx, p[1], p[2] + dz]) == crop;
    let rows = (same(-1, 0) || same(1, 0)) && (same(0, -1) || same(0, 1));
    let diagonal = same(-1, -1) || same(1, -1) || same(1, 1) || same(-1, 1);
    rt::growth_speed(&soil, rows, diagonal)
}

fn is_tree_room(name: &str) -> bool {
    matches!(
        name,
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:short_grass"
            | "minecraft:tall_grass"
            | "minecraft:fern"
            | "minecraft:large_fern"
            | "minecraft:vine"
            | rt::AVOCADO_SAPLING
    ) || name.ends_with("_leaves")
}

/// Grows the avocado tree from the sapling at `pos` (`TreeGrower("avocado")`, the configured
/// feature's shapes): false when something is in the way (the sapling stays).
fn grow_tree(level: &mut GameLevel<'_>, ticks: &mut Ticks, pos: [i32; 3]) -> bool {
    if !level.native_authoritative {
        return false;
    }
    let parts = rt::avocado_tree(rt::roll);
    let at = |o: [i32; 3]| [pos[0] + o[0], pos[1] + o[1], pos[2] + o[2]];
    if parts
        .iter()
        .any(|(o, _)| !is_tree_room(&name_at(level, at(*o))))
    {
        return false;
    }
    let Some(log) = level.registry.default_state(rt::AVOCADO_LOG) else {
        return false;
    };
    let Some(leaves) = level.registry.default_state(rt::AVOCADO_LEAVES) else {
        return false;
    };
    let b = level.behaviours;
    for (o, part) in parts {
        let state = match part {
            rt::TreePart::Log => Some(log),
            rt::TreePart::Leaves { distance, stage } => level
                .native_state_with_property(leaves, "distance", &distance.to_string())
                .and_then(|s| level.native_state_with_property(s, "persistent", "false"))
                .and_then(|s| level.native_state_with_property(s, "stage", &stage.to_string())),
        };
        if let Some(state) = state {
            let p = at(o);
            b.set_block(
                level,
                ticks,
                BlockPos::new(p[0], p[1], p[2]),
                state,
                horizon_world::level::flags::UPDATE_ALL,
            );
        }
    }
    true
}

/// `native_random_tick`: the crops grow as `CropBlock.randomTick` (rice on dirt, the others with
/// their farmland), the sapling grows an avocado tree, and stage 1 avocado leaves ripen.
pub(super) fn random_tick(
    level: &mut GameLevel<'_>,
    ticks: &mut Ticks,
    pos: BlockPos,
    state: BlockStateId,
    handled: &mut bool,
) {
    if *handled {
        return;
    }
    let name = level.facts.get(state).name.to_string();
    if !name.starts_with("sushigocrafting:") {
        return;
    }
    let p = [pos.x, pos.y, pos.z];
    let clients = horizon_world::level::flags::UPDATE_CLIENTS;
    if rt::is_crop(&name) {
        *handled = true;
        let Some(age) = state_prop(level, state, "age").and_then(|a| a.parse::<u32>().ok()) else {
            return;
        };
        let light = brightness(level, p, 0);
        let speed = growth_speed(level, p, &name);
        if rt::crop_grows(age, light, speed, rt::roll()) {
            level.native_set_block_property(ticks, p, "age", &(age + 1).to_string(), clients);
        }
    } else if name == rt::AVOCADO_SAPLING {
        *handled = true;
        let darken = level.darken;
        if !rt::sapling_advances(
            brightness(level, [p[0], p[1] + 1, p[2]], darken),
            rt::roll(),
        ) {
            return;
        }
        if state_prop(level, state, "stage").as_deref() == Some("1") {
            grow_tree(level, ticks, p);
        } else {
            level.native_set_block_property(ticks, p, "stage", "1", clients);
        }
    } else if name == rt::AVOCADO_LEAVES {
        let stage = state_prop(level, state, "stage")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        if rt::leaves_ripen(stage, rt::roll()) {
            level.native_set_block_property(
                ticks,
                p,
                "stage",
                "2",
                horizon_world::level::flags::UPDATE_ALL,
            );
        }
    }
}

/// Bone meal from a dispenser (or anything but a player's click, which `use_item_on` takes)
/// would grow tall seagrass from rice and an oak from the sapling: refused.
pub(super) fn bonemeal_target(
    level: &GameLevel<'_>,
    state: BlockStateId,
    valid: &mut Option<bool>,
) {
    if valid.is_some() {
        return;
    }
    let name: &str = &level.facts.get(state).name;
    if name == rt::RICE_CROP || name == rt::AVOCADO_SAPLING {
        *valid = Some(false);
    }
}

/// `WaterCropBlock`'s ground: rice stands on dirt only (seagrass, its template, on anything sturdy).
pub(super) fn may_place_on(
    level: &GameLevel<'_>,
    state: BlockStateId,
    ground: BlockStateId,
    may: &mut Option<bool>,
) {
    if may.is_some() {
        return;
    }
    if *level.facts.get(state).name == *rt::RICE_CROP {
        *may = Some(*level.facts.get(ground).name == *"minecraft:dirt");
    }
}

// ---------------------------------------------------------------- drops

/// Grass and ferns: a wheat seed becomes one of the mod's seeds by weight (`SeedsLootModifier`);
/// an amount item drops as rolled portions (`ItemAmountLootModifier`); fish drop tobiko 3 in 10.
pub(super) fn loot_drops(
    game: &mut Game<'_>,
    context: &NativeLootDrops<'_>,
    drops: &mut Vec<ItemStack>,
) {
    if context.used || !game.native_on(rt::ID) {
        return;
    }
    if let Some((_, state)) = context.block {
        let name = game.res.facts().get(state).name.to_string();
        if rt::is_grass(&name) {
            for s in drops.iter_mut() {
                if *s.item == *"minecraft:wheat_seeds" {
                    s.item = std::sync::Arc::from(rt::grass_seed(rt::roll()));
                }
            }
        }
        for s in drops.iter_mut() {
            if let Some(a) = rt::amount_of(&s.item) {
                s.count = rt::roll_portions(a, rt::roll());
            }
        }
        return;
    }
    if let Some((_, kind)) = context.entity
        && !context.table.is_empty()
        && rt::is_fish(kind)
        && rt::roll() < rt::TOBIKO_CHANCE
        && let Some(a) = rt::amount_of(rt::TOBIKO)
    {
        drops.push(ItemStack::new(rt::TOBIKO, rt::roll_portions(a, rt::roll())));
    }
}

/// A fishing rod's catch (state 2) sometimes brings raw tuna or shrimp, which have no entities here.
pub(super) fn fish(game: &mut Game<'_>, state: i32, cancel: &mut bool) {
    if *cancel || state != 2 || !game.native_on(rt::ID) {
        return;
    }
    if rt::roll() < rt::TUNA_FISHING_CHANCE {
        game.native_give_items(rt::RAW_TUNA, 1);
    }
    if rt::roll() < rt::SHRIMP_FISHING_CHANCE
        && let Some(a) = rt::amount_of(rt::SHRIMP)
    {
        game.native_give_items(rt::SHRIMP, rt::roll_portions(a, rt::roll()));
    }
}

// ---------------------------------------------------------------- the board's item, drawn

const KEY_VIEW: &str = "sushigocrafting_view";

/// What clients get of a board: its item (with components).
pub(super) fn update_tag(entity: &ModBlockEntity, tag: &mut Option<Compound>) {
    if entity.id != rt::CUTTING_BOARD {
        return;
    }
    let mut c = Compound::new();
    match entity
        .items
        .first()
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
    let mut out = tag.take().unwrap_or_default();
    out.insert(key(KEY_VIEW), Tag::Compound(c));
    *tag = Some(out);
}

fn shown_item(entity: &ModBlockEntity) -> Option<ItemStack> {
    if let Some(Tag::Compound(c)) = entity.nbt.get(&key(KEY_VIEW)) {
        let id = match c.get(&key("id")) {
            Some(Tag::String(s)) => s.to_string_lossy(),
            _ => String::new(),
        };
        if id.is_empty() {
            return None;
        }
        let (damage, extra) =
            horizon_client::item_data::decode_components(c.get(&key("components")));
        return Some(ItemStack {
            item: std::sync::Arc::from(id.as_str()),
            count: 1,
            damage,
            extra,
        });
    }
    entity
        .items
        .first()
        .cloned()
        .flatten()
        .filter(|s| s.count > 0)
}

/// Draws the board's item lying on it (`CuttingBoardRenderer`), lit by the block above.
pub(super) fn submit(
    game: &Game<'_>,
    pos: [i32; 3],
    block: &str,
    entity: &ModBlockEntity,
    draw: &mut super::native_block_entity_draw::BlockEntityDraw,
) {
    if block != rt::CUTTING_BOARD {
        return;
    }
    let Some(stack) = shown_item(entity) else {
        return;
    };
    let packed = game.world.light(pos[0], pos[1] + 1, pos[2]);
    draw.light = (u32::from(packed >> 4) * 16) | ((u32::from(packed & 15) * 16) << 8);
    let yaw = match game.native_block_property(pos, "facing").as_deref() {
        Some("east") => 270.0,
        Some("south") => 180.0,
        Some("west") => 90.0,
        _ => 0.0,
    };
    draw.push();
    draw.translate([0.5, 0.09, 0.5]);
    draw.rotate_about([0.0, 1.0, 0.0], yaw, [0.0, 0.0, 0.0]);
    draw.rotate_about([1.0, 0.0, 0.0], 90.0, [0.0, 0.0, 0.0]);
    draw.scale([0.5, 0.5, 0.5]);
    game.native_draw_item(
        draw,
        &stack,
        horizon_assets::item_model::DisplayContext::Fixed,
        None,
        0,
    );
    draw.pop();
}

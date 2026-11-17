// Bibliocraft Legacy for Horizon: the furniture's rules (`bibliocraft_rt`)
// over Horizon's block entities, item stacks and players. Added to the game
// as `crate::game::bibliocraft_glue`; the hook helpers call it.
//
// Each bookcase, shelf, table and the sword pedestal gets a block entity
// named after its block, with a container (16, 4, 1 and 1 slots). The
// bookcase and the shelf open their content mod's menu type (a screen laid
// out by `screen`); the shelf, the table and the pedestal also take and give
// items with a right click. Clients get each slot's item in the update tag
// (`BcView`) and draw it: the books as the original's book models, the other
// items as `BlockEntityRenderer`s do.

use super::Game;
use super::bibliocraft_rt as rt;
use horizon_client::containers::{BlockEntity, ModBlockEntity, ModContainer};
use horizon_client::items::ItemStack;
use horizon_client::text::Text;
use horizon_world::nbt::{Compound, JavaString, Tag};
use horizon_world::registry::BlockStateId;

fn key(k: &str) -> JavaString {
    JavaString::from(k)
}

/// Every furniture block gets a block entity of its own type, named after it.
pub(super) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && rt::Kind::of(block).is_some() {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

/// They tick to tell clients when what they hold changed (a menu or a
/// hopper moved an item).
pub(super) fn ticks(block: &str, ticks: &mut bool) {
    if rt::Kind::of(block).is_some() {
        *ticks = true;
    }
}

/// The slots, the faces hoppers reach them through and the menu type.
pub(super) fn container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() {
        return;
    }
    let Some(kind) = rt::Kind::of(block) else {
        return;
    };
    let n = kind.slots();
    let reach: Vec<usize> = if kind.hoppers() { (0..n).collect() } else { Vec::new() };
    *container = Some(ModContainer {
        slots: n,
        faces: std::array::from_fn(|_| reach.clone()),
        inputs: reach.clone(),
        outputs: reach,
        menu: kind.menu().map(str::to_owned),
    });
}

fn slot_empty(items: &[Option<ItemStack>], slot: usize) -> bool {
    items.get(slot).is_none_or(|s| s.as_ref().is_none_or(|s| s.count == 0))
}

/// `#minecraft:swords` (the server's tags), or a `_sword` id when they
/// can't be read.
fn is_sword(game: &Game<'_>, stack: &ItemStack) -> bool {
    game.native_server_item_in_tag(stack, "minecraft:swords")
        .unwrap_or(false)
        || rt::is_sword_id(&stack.item)
}

/// A hopper putting `stack` in slot `slot`: books into an empty bookcase
/// slot, a sword onto an empty pedestal, anything onto a shelf.
pub(super) fn accepts(game: &Game<'_>, entity: &ModBlockEntity, slot: usize, stack: &ItemStack, cancel: &mut bool) {
    if *cancel {
        return;
    }
    let Some(kind) = rt::Kind::of(&entity.id) else {
        return;
    };
    let ok = match kind {
        rt::Kind::Bookcase => rt::is_book(&stack.item),
        rt::Kind::Pedestal => is_sword(game, stack),
        rt::Kind::Shelf => true,
        rt::Kind::Table => false,
    };
    if !rt::hopper_accepts(kind, slot, slot_empty(&entity.items, slot), ok) {
        *cancel = true;
    }
}

/// One server tick: when what the block holds changed since clients were
/// last sent it, they get it again.
pub(super) fn tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let Some(kind) = rt::Kind::of(&entity.id) else {
        return;
    };
    let h = rt::view_hash((0..kind.slots()).map(|i| {
        entity
            .items
            .get(i)
            .and_then(Option::as_ref)
            .filter(|s| s.count > 0)
            .map(|s| (&*s.item, s.count, s.damage))
    }));
    if matches!(entity.nbt.get(&key(rt::KEY_SHOWN)), Some(Tag::Long(v)) if *v == h) {
        return;
    }
    entity.nbt.insert(key(rt::KEY_SHOWN), Tag::Long(h));
    game.native_block_entity_sync(pos);
}

/// What clients get: each slot's item (with its components), as `BcView`.
pub(super) fn update_tag(entity: &ModBlockEntity, tag: &mut Option<Compound>) {
    let Some(kind) = rt::Kind::of(&entity.id) else {
        return;
    };
    let mut view = Vec::with_capacity(kind.slots());
    for i in 0..kind.slots() {
        let mut c = Compound::new();
        match entity.items.get(i).and_then(Option::as_ref).filter(|s| s.count > 0) {
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

/// The slots a client shows: from the update tag or, on the server's own
/// copy (no tag merged), from the block entity's slots.
fn shown_items(entity: &ModBlockEntity, n: usize) -> Vec<Option<ItemStack>> {
    if let Some(Tag::List(list)) = entity.nbt.get(&key(rt::KEY_VIEW)) {
        let mut out = Vec::with_capacity(n);
        for t in list.iter().take(n) {
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
            let (damage, extra) = horizon_client::item_data::decode_components(c.get(&key("components")));
            out.push(Some(ItemStack {
                item: std::sync::Arc::from(id.as_str()),
                count: 1,
                damage,
                extra,
            }));
        }
        return out;
    }
    (0..n)
        .map(|i| entity.items.get(i).cloned().flatten().filter(|s| s.count > 0))
        .collect()
}

fn draw_fixed(game: &Game<'_>, draw: &mut super::BlockEntityDraw, stack: &ItemStack, seed: i32) {
    let _ = game.native_draw_item(
        draw,
        stack,
        horizon_assets::item_model::DisplayContext::Fixed,
        None,
        seed,
    );
}

/// Draws what the block holds: the bookcase's books (`BookcaseModel`), the
/// shelf's four items (`ShelfBER`), the pedestal's sword (`SwordPedestalBER`)
/// and the item on the table (`TableBER`).
pub(super) fn submit(
    game: &Game<'_>,
    _pos: [i32; 3],
    block: &str,
    state: BlockStateId,
    entity: &ModBlockEntity,
    draw: &mut super::BlockEntityDraw,
) {
    let Some(kind) = rt::Kind::of(block) else {
        return;
    };
    let facing = game
        .native_block_state_property(state, "facing")
        .unwrap_or("south")
        .to_owned();
    let items = shown_items(entity, kind.slots());
    let center = [0.5f32, 0.5, 0.5];
    let up = [0.0f32, 1.0, 0.0];
    match kind {
        rt::Kind::Bookcase => {
            draw.push();
            draw.rotate_about(up, rt::model_yaw(&facing), center);
            for (i, s) in items.iter().enumerate() {
                if s.is_some() {
                    let _ = game.native_draw_model(draw, &rt::book_model(i), 0xFF_FFFF);
                }
            }
            draw.pop();
            // Only the block's state, its block entity and its own light: kept until they change.
            draw.keep();
        }
        rt::Kind::Shelf => {
            draw.push();
            draw.rotate_about(up, rt::ber_yaw(&facing), center);
            draw.translate(center);
            for (i, s) in items.iter().enumerate() {
                let Some(stack) = s else {
                    continue;
                };
                draw.push();
                draw.translate(rt::shelf_offset(i));
                draw.scale([rt::SHELF_SCALE; 3]);
                draw.rotate_about(up, 180.0, [0.0; 3]);
                draw_fixed(game, draw, stack, i as i32);
                draw.pop();
            }
            draw.pop();
        }
        rt::Kind::Pedestal => {
            let Some(Some(stack)) = items.first() else {
                return;
            };
            draw.push();
            draw.rotate_about(up, rt::ber_yaw(&facing), center);
            draw.translate(center);
            draw.rotate_about(up, 180.0, [0.0; 3]);
            draw.translate([0.0, 1.0 / 16.0, 0.0]);
            draw.rotate_about([0.0, 0.0, 1.0], 135.0, [0.0; 3]);
            draw.scale([0.6; 3]);
            draw_fixed(game, draw, stack, 0);
            draw.pop();
        }
        rt::Kind::Table => {
            let Some(Some(stack)) = items.first() else {
                return;
            };
            draw.push();
            draw.translate([0.5, 1.03125, 0.5]);
            draw.rotate_about(up, 180.0, [0.0; 3]);
            draw.rotate_about([1.0, 0.0, 0.0], 90.0, [0.0; 3]);
            draw.scale([0.5; 3]);
            draw_fixed(game, draw, stack, 0);
            draw.pop();
        }
    }
}

/// The book models, baked beside the block models.
pub(super) fn standalone_models(models: &mut Vec<String>) {
    for m in rt::standalone_models() {
        if !models.contains(&m) {
            models.push(m);
        }
    }
}

/// The bookcase's and the shelf's screens: their slots where the original's
/// menus put them, on a plain panel, the inventory below.
pub(super) fn screen(block: &str, screen: &mut Option<super::native_hooks::NativeScreen>) {
    if screen.is_some() {
        return;
    }
    let Some(slots) = rt::Kind::of(block).and_then(rt::screen_slots) else {
        return;
    };
    *screen = Some(super::native_hooks::NativeScreen {
        width: 176.0,
        height: 166.0,
        slots: slots.into_iter().map(Some).collect(),
        inventory_y: 84.0,
        inventory_x: 8.0,
        own_labels: false,
        own_background: false,
    });
}

fn tell(game: &mut Game<'_>, line: Text) {
    if let Some(p) = game.native_acting_player() {
        game.native_action_bar(p.uuid, line);
    }
}

/// The furniture block entity at `pos`: its kind and slots (as many as it has).
fn snapshot(game: &Game<'_>, pos: [i32; 3]) -> Option<(rt::Kind, Vec<Option<ItemStack>>)> {
    match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
        Some(BlockEntity::Mod(m)) if !m.id.is_empty() => {
            let kind = rt::Kind::of(&m.id)?;
            let mut items = m.items.clone();
            items.resize(kind.slots().max(items.len()), None);
            Some((kind, items))
        }
        _ => None,
    }
}

/// Writes the slots back and tells clients.
fn store(game: &mut Game<'_>, pos: [i32; 3], items: Vec<Option<ItemStack>>) {
    let _ = game.native_with_mod_block_entity(pos, move |m| {
        if m.items != items {
            m.items = items;
        }
    });
    game.native_block_entity_sync(pos);
}

/// The shelf slot the acting player aims at, from their eyes and look (the
/// hook gives the face, not the point hit).
fn aimed_shelf_slot(game: &Game<'_>, pos: [i32; 3], facing: &str) -> Option<usize> {
    let p = game.native_acting_player()?;
    let eye_height = if game.native_sneaking() {
        rt::EYE_SNEAKING
    } else {
        rt::EYE_STANDING
    };
    let eye = [p.position[0], p.position[1] + eye_height, p.position[2]];
    let (u, v) = rt::front_hit(facing, pos, eye, rt::look_dir(p.yaw, p.pitch), rt::SHELF_INSET)?;
    rt::shelf_slot_at(u, v)
}

/// A right click on a shelf, the pedestal or a table, before the block and
/// the item act. The bookcase is left to its template (its screen opens).
pub(super) fn right_click(game: &mut Game<'_>, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
    if *canceled {
        return;
    }
    let Some((kind, mut items)) = snapshot(game, pos) else {
        return;
    };
    if kind == rt::Kind::Bookcase {
        return;
    }
    let sneaking = game.native_sneaking();
    let held: Option<ItemStack> = game.held_mut().clone().filter(|s| s.count > 0);
    let hands_full = held.is_some() || game.offhand.as_ref().is_some_and(|s| s.count > 0);
    let bottom = face == [0, -1, 0];
    match kind {
        rt::Kind::Bookcase => {}
        rt::Kind::Shelf | rt::Kind::Pedestal => {
            let click = if kind == rt::Kind::Shelf {
                let facing = game.native_block_property(pos, "facing").unwrap_or_default();
                let front = rt::facing_normal(&facing) == Some(face);
                let slot = if front {
                    aimed_shelf_slot(game, pos, &facing)
                } else {
                    None
                };
                rt::shelf_click(sneaking, hands_full, front, slot)
            } else {
                let sword = held.as_ref().map(|h| is_sword(game, h));
                rt::pedestal_click(sneaking, hands_full, bottom, sword)
            };
            let rt::Click::Swap(slot) = click else {
                return;
            };
            *canceled = true;
            let mut hand = held;
            if let Some(s) = items.get_mut(slot) {
                rt::swap(s, &mut hand);
            }
            *game.held_mut() = hand;
            store(game, pos, items);
        }
        rt::Kind::Table => {
            let on_top = items.first().cloned().flatten().filter(|s| s.count > 0);
            let act = rt::table_click(
                sneaking,
                hands_full,
                bottom,
                held.as_ref().map(|s| &*s.item),
                on_top.as_ref().map(|s| &*s.item),
            );
            match act {
                rt::TableClick::Pass => {}
                rt::TableClick::Nothing => *canceled = true,
                rt::TableClick::Take => {
                    *canceled = true;
                    *game.held_mut() = on_top;
                    items[0] = None;
                    store(game, pos, items);
                }
                rt::TableClick::Put => {
                    *canceled = true;
                    let Some(h) = held else {
                        return;
                    };
                    let mut one = h.clone();
                    one.count = 1;
                    let mut rest = h.clone();
                    rest.count = h.count - 1;
                    let rest = (rest.count > 0).then_some(rest);
                    let rest_empty = rest.is_none();
                    match (on_top, rest_empty) {
                        (Some(old), true) => *game.held_mut() = Some(old),
                        (Some(old), false) => {
                            let uuid = game.native_acting_id();
                            if game.native_give_stack(uuid, old).is_some() {
                                // No room for what lay there: nothing changes.
                                tell(
                                    game,
                                    Text::literal("Inventário cheio: libere espaço para trocar o item da mesa."),
                                );
                                return;
                            }
                            *game.held_mut() = rest;
                        }
                        (None, _) => *game.held_mut() = rest,
                    }
                    items[0] = Some(one);
                    store(game, pos, items);
                }
            }
        }
    }
}

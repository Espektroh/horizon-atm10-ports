// Functional Storage for Horizon: the drawers' rules (`functionalstorage_rt`)
// over Horizon's block entities, item stacks and players. Added to the game
// as `crate::game::functionalstorage_glue`; the hook helpers call it.
//
// A drawer's block entity keeps, per drawer slot, three of its container's
// item slots (the stack hoppers take, the slot hoppers fill, one kept item)
// and, in its saved compound, the count beyond them (`Amounts`), its storage
// upgrades and its void upgrade. Its tick folds what hoppers put in into the
// count and lays the slots out again. Clients get what each slot holds
// through the update tag (`View`) and draw it on the front.

use super::Game;
use super::functionalstorage_rt as rt;
use horizon_client::containers::{BlockEntity, ModBlockEntity, ModContainer};
use horizon_client::items::{ItemStack, Items};
use horizon_client::text::Text;
use horizon_world::nbt::{Compound, JavaString, Tag};
use horizon_world::registry::BlockStateId;

/// Item stacks as the rules see them, with the item table for stack sizes.
struct St<'a> {
    items: &'a Items,
}

impl rt::Stacks<ItemStack> for St<'_> {
    fn count(&self, s: &ItemStack) -> u32 {
        s.count
    }
    fn same(&self, a: &ItemStack, b: &ItemStack) -> bool {
        a.same_item_same_components(b)
    }
    fn max_stack(&self, s: &ItemStack) -> u32 {
        self.items.get(&s.item).map_or(64, |i| i.max_stack).max(1)
    }
    fn with_count(&self, s: &ItemStack, count: u32) -> ItemStack {
        let mut c = s.clone();
        c.count = count;
        c
    }
}

/// The same without an item table, where none is at hand (the update tag,
/// asked off the server thread): only reading, which needs no stack size.
struct Plain;

impl rt::Stacks<ItemStack> for Plain {
    fn count(&self, s: &ItemStack) -> u32 {
        s.count
    }
    fn same(&self, a: &ItemStack, b: &ItemStack) -> bool {
        a.same_item_same_components(b)
    }
    fn max_stack(&self, _s: &ItemStack) -> u32 {
        64
    }
    fn with_count(&self, s: &ItemStack, count: u32) -> ItemStack {
        let mut c = s.clone();
        c.count = count;
        c
    }
}

fn key(k: &str) -> JavaString {
    JavaString::from(k)
}

fn saved_amounts(nbt: &Compound) -> Vec<i64> {
    match nbt.get(&key(rt::KEY_AMOUNTS)) {
        Some(Tag::LongArray(v)) => v.clone(),
        Some(Tag::List(l)) => l
            .iter()
            .map(|t| match t {
                Tag::Long(x) => *x,
                Tag::Int(x) => i64::from(*x),
                _ => 0,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn saved_upgrades(nbt: &Compound) -> Vec<rt::Upgrade> {
    match nbt.get(&key(rt::KEY_UPGRADES)) {
        Some(Tag::List(l)) => l
            .iter()
            .filter_map(|t| match t {
                Tag::String(s) => rt::upgrade_of(&s.to_string_lossy()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn saved_void(nbt: &Compound) -> bool {
    matches!(nbt.get(&key(rt::KEY_VOID)), Some(Tag::Byte(v)) if *v != 0)
}

fn read_drawer(
    size: rt::Size,
    items: &[Option<ItemStack>],
    nbt: &Compound,
    st: &impl rt::Stacks<ItemStack>,
) -> rt::Drawer<ItemStack> {
    rt::Drawer::read(
        size,
        items,
        &saved_amounts(nbt),
        &saved_upgrades(nbt),
        saved_void(nbt),
        st,
    )
}

/// What to write back into a drawer's block entity: only what differs.
struct Write {
    items: Option<Vec<Option<ItemStack>>>,
    tags: Vec<(&'static str, Tag)>,
    shown: bool,
}

fn shown_hash(d: &rt::Drawer<ItemStack>) -> i64 {
    d.shown_hash(|s| s.item.to_string())
}

/// The block entity's slots and saved keys for `d`, compared with what it
/// holds now. With `with_shown`, also whether clients need it again.
fn prepare(
    items: &[Option<ItemStack>],
    nbt: &Compound,
    d: &rt::Drawer<ItemStack>,
    st: &St<'_>,
    with_shown: bool,
) -> Write {
    let (new_items, amounts) = d.layout(st);
    let mut tags: Vec<(&'static str, Tag)> = vec![
        (rt::KEY_FORMAT, Tag::Int(rt::FORMAT)),
        (rt::KEY_AMOUNTS, Tag::LongArray(amounts)),
        (
            rt::KEY_UPGRADES,
            Tag::List(
                d.upgrades
                    .iter()
                    .map(|u| Tag::String(JavaString::from(u.id())))
                    .collect(),
            ),
        ),
        (rt::KEY_VOID, Tag::Byte(i8::from(d.void))),
    ];
    let mut shown = false;
    if with_shown {
        let h = shown_hash(d);
        shown = !matches!(nbt.get(&key(rt::KEY_SHOWN)), Some(Tag::Long(v)) if *v == h);
        tags.push((rt::KEY_SHOWN, Tag::Long(h)));
    }
    tags.retain(|(k, t)| nbt.get(&key(k)) != Some(t));
    Write {
        items: (items != new_items.as_slice()).then_some(new_items),
        tags,
        shown,
    }
}

fn apply(entity: &mut ModBlockEntity, w: Write) {
    if let Some(items) = w.items {
        entity.items = items;
    }
    for (k, t) in w.tags {
        entity.nbt.insert(key(k), t);
    }
}

/// A drawer's block entity as it is now, read without changing it.
fn snapshot(game: &Game<'_>, pos: [i32; 3]) -> Option<(rt::Size, Vec<Option<ItemStack>>, Compound)> {
    match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
        Some(BlockEntity::Mod(m)) if !m.id.is_empty() => {
            let size = rt::drawer_of(&m.id)?;
            Some((size, m.items.clone(), m.nbt.clone()))
        }
        _ => None,
    }
}

/// Writes `d` into the drawer at `pos` (read from `items` and `nbt` before).
fn store(game: &mut Game<'_>, pos: [i32; 3], items: &[Option<ItemStack>], nbt: &Compound, d: &rt::Drawer<ItemStack>) {
    let w = {
        let st = St {
            items: world_items!(game),
        };
        prepare(items, nbt, d, &st, false)
    };
    if w.items.is_none() && w.tags.is_empty() {
        return;
    }
    game.native_with_mod_block_entity(pos, |m| apply(m, w));
}

/// Every drawer gets a block entity of its own type, named after its block.
pub(super) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && rt::drawer_of(block).is_some() {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

pub(super) fn ticks(block: &str, ticks: &mut bool) {
    if rt::drawer_of(block).is_some() {
        *ticks = true;
    }
}

/// Three slots per drawer slot. Every face reaches the out and in slots,
/// as Functional Storage gives its item handler on every side; the kept
/// slot is on no face. No menu: the drawer is used with clicks.
pub(super) fn container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() {
        return;
    }
    let Some(size) = rt::drawer_of(block) else {
        return;
    };
    let face = rt::container_face(size);
    *container = Some(ModContainer {
        slots: size.container_slots(),
        faces: std::array::from_fn(|_| face.clone()),
        inputs: rt::container_inputs(size),
        outputs: rt::container_outputs(size),
        menu: None,
    });
}

/// One server tick: what hoppers put in joins the count, a void drawer
/// drops its overflow, the slots are laid out again, and clients are sent
/// the drawer when what it shows changed.
pub(super) fn tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let Some(size) = rt::drawer_of(&entity.id) else {
        return;
    };
    let w = {
        let st = St {
            items: world_items!(game),
        };
        let mut d = read_drawer(size, &entity.items, &entity.nbt, &st);
        d.settle(&st);
        prepare(&entity.items, &entity.nbt, &d, &st, true)
    };
    let shown = w.shown;
    if w.items.is_some() || !w.tags.is_empty() {
        apply(entity, w);
    }
    if shown {
        game.native_block_entity_sync(pos);
    }
}

/// A hopper (or `native_insert_item`) putting `stack` in slot `slot`: only
/// a drawer slot's input, for the item it holds while it has room (or a
/// void upgrade), or an empty slot when no other slot holds the item.
pub(super) fn accepts(game: &Game<'_>, entity: &ModBlockEntity, slot: usize, stack: &ItemStack, cancel: &mut bool) {
    if *cancel {
        return;
    }
    let Some(size) = rt::drawer_of(&entity.id) else {
        return;
    };
    let (i, role) = rt::container_slot(slot);
    if role != rt::IN {
        *cancel = true;
        return;
    }
    let st = St {
        items: world_items!(game),
    };
    let d = read_drawer(size, &entity.items, &entity.nbt, &st);
    if !d.accepts(i, stack, &st) {
        *cancel = true;
    }
}

/// What clients get: each slot's item (with its components) and count.
pub(super) fn update_tag(entity: &ModBlockEntity, tag: &mut Option<Compound>) {
    let Some(size) = rt::drawer_of(&entity.id) else {
        return;
    };
    let d = read_drawer(size, &entity.items, &entity.nbt, &Plain);
    let mut view = Vec::with_capacity(d.slots.len());
    for slot in &d.slots {
        let mut c = Compound::new();
        match slot.proto.as_ref().filter(|_| slot.total > 0) {
            Some(p) => {
                c.insert(key("id"), Tag::String(JavaString::from(&*p.item)));
                c.insert(key("count"), Tag::Long(slot.total.min(i64::MAX as u64) as i64));
                if let Some(components) = horizon_client::item_data::encode_components(p) {
                    c.insert(key("components"), components);
                }
            }
            None => {
                c.insert(key("id"), Tag::String(JavaString::from("")));
                c.insert(key("count"), Tag::Long(0));
            }
        }
        view.push(Tag::Compound(c));
    }
    let mut out = tag.take().unwrap_or_default();
    out.insert(key(rt::KEY_VIEW), Tag::List(view));
    out.insert(key(rt::KEY_VOID), Tag::Byte(i8::from(d.void)));
    *tag = Some(out);
}

/// What a client shows of slot by slot: from the update tag, or (the
/// server's own copy, with no tag merged) from the drawer itself.
fn client_view(entity: &ModBlockEntity, size: rt::Size) -> Vec<(usize, ItemStack, u64)> {
    let mut out = Vec::new();
    if let Some(Tag::List(list)) = entity.nbt.get(&key(rt::KEY_VIEW)) {
        for (i, t) in list.iter().enumerate().take(size.slots()) {
            let Tag::Compound(c) = t else {
                continue;
            };
            let id = match c.get(&key("id")) {
                Some(Tag::String(s)) => s.to_string_lossy(),
                _ => continue,
            };
            let count = match c.get(&key("count")) {
                Some(Tag::Long(v)) => (*v).max(0) as u64,
                Some(Tag::Int(v)) => i64::from(*v).max(0) as u64,
                _ => 0,
            };
            if id.is_empty() || count == 0 {
                continue;
            }
            let (damage, extra) = horizon_client::item_data::decode_components(c.get(&key("components")));
            let stack = ItemStack {
                item: std::sync::Arc::from(id.as_str()),
                count: 1,
                damage,
                extra,
            };
            out.push((i, stack, count));
        }
        return out;
    }
    let d = read_drawer(size, &entity.items, &entity.nbt, &Plain);
    for (i, slot) in d.slots.iter().enumerate() {
        if let Some(p) = slot.proto.as_ref().filter(|_| slot.total > 0) {
            out.push((i, p.clone(), slot.total));
        }
    }
    out
}

/// Draws each slot's item and count on the drawer's front, lit by the block
/// in front of it (`BaseDrawerRenderer`, `DrawerRenderer.renderStack`).
pub(super) fn submit(
    game: &Game<'_>,
    pos: [i32; 3],
    block: &str,
    state: BlockStateId,
    entity: &ModBlockEntity,
    draw: &mut super::BlockEntityDraw,
) {
    let Some(size) = rt::drawer_of(block) else {
        return;
    };
    let facing = game
        .native_block_state_property(state, "facing")
        .unwrap_or("south")
        .to_owned();
    if let Some(n) = rt::facing_normal(&facing) {
        let packed = game.world.light(pos[0] + n[0], pos[1] + n[1], pos[2] + n[2]);
        draw.light = (u32::from(packed >> 4) * 16) | ((u32::from(packed & 15) * 16) << 8);
    }
    let view = client_view(entity, size);
    draw.push();
    draw.rotate_about([0.0, 1.0, 0.0], rt::facing_yaw(&facing), [0.5, 0.5, 0.5]);
    for (i, stack, count) in view {
        let ([cx, cy], s) = rt::slot_center(size, i);
        let is_block = world_items!(game)
            .get(&stack.item)
            .and_then(|info| info.block)
            .is_some();
        draw.push();
        draw.translate([cx, cy, rt::FRONT_Z + 0.001]);
        if is_block {
            draw.scale([0.75 * s, 0.75 * s, 0.125 * s]);
        } else {
            draw.scale([0.4 * s, 0.4 * s, 0.4 * s]);
        }
        draw.rotate_about([0.0, 1.0, 0.0], 180.0, [0.0, 0.0, 0.0]);
        game.native_draw_item(
            draw,
            &stack,
            horizon_assets::item_model::DisplayContext::Fixed,
            None,
            i as i32,
        );
        draw.pop();
        let text = rt::format_big(count);
        let width = rt::text_width(&text).max(1.0);
        let max_scale = if size == rt::Size::X1 { 0.015 } else { 0.02 };
        let px = (0.4 / width).min(max_scale) * 0.5 * s;
        draw.push();
        draw.translate([cx, cy - 0.3725 * s, rt::FRONT_Z + 0.002]);
        draw.scale([px, -px, px]);
        game.native_draw_text(draw, &Text::literal(text), [-width / 2.0, -4.0], 0xFF_FFFF, true, 0);
        draw.pop();
    }
    draw.pop();
    // No `draw.keep()`: the light is read from the block in front, which keep() would not
    // follow (native_block_entity_draw.rs:198-209).
}

/// Clicks remembered per player: the last right click on a drawer's front
/// (double clicks) and the last left click (a held button takes again only
/// every few ticks).
static FS_CLICKS: std::sync::Mutex<Vec<(u128, i64, i64)>> = std::sync::Mutex::new(Vec::new());

fn last_clicks(uuid: u128) -> (Option<i64>, Option<i64>) {
    let Ok(v) = FS_CLICKS.lock() else {
        return (None, None);
    };
    v.iter().find(|(u, _, _)| *u == uuid).map_or((None, None), |(_, r, l)| {
        (Some(*r).filter(|t| *t != i64::MIN), Some(*l).filter(|t| *t != i64::MIN))
    })
}

fn remember_click(uuid: u128, right: Option<i64>, left: Option<i64>) {
    let Ok(mut v) = FS_CLICKS.lock() else {
        return;
    };
    match v.iter_mut().find(|(u, _, _)| *u == uuid) {
        Some(e) => {
            if let Some(r) = right {
                e.1 = r;
            }
            if let Some(l) = left {
                e.2 = l;
            }
        }
        None => {
            if v.len() >= 256 {
                v.remove(0);
            }
            v.push((uuid, right.unwrap_or(i64::MIN), left.unwrap_or(i64::MIN)));
        }
    }
}

/// The slot the acting player aims at on the front, from their eyes and
/// look (the hook gives the face, not the point hit).
fn aimed_slot(game: &Game<'_>, pos: [i32; 3], size: rt::Size, facing: &str) -> usize {
    let Some(p) = game.native_acting_player() else {
        return 0;
    };
    let eye_height = if game.native_sneaking() {
        rt::EYE_SNEAKING
    } else {
        rt::EYE_STANDING
    };
    let eye = [p.position[0], p.position[1] + eye_height, p.position[2]];
    rt::front_hit(facing, pos, eye, rt::look_dir(p.yaw, p.pitch)).map_or(0, |(u, v)| rt::slot_at(size, u, v))
}

/// The item's name as the game translates it.
fn item_name(game: &Game<'_>, stack: &ItemStack) -> Text {
    if let Some(name) = stack.custom_name() {
        return Text::literal(name);
    }
    let id: &str = &stack.item;
    let (ns, path) = id.split_once(':').unwrap_or(("minecraft", id));
    let is_block = world_items!(game)
        .get(id)
        .and_then(|i| i.block)
        .is_some_and(|b| &*game.res.facts().get(b).name == id);
    let kind = if is_block { "block" } else { "item" };
    Text::translate(&format!("{kind}.{ns}.{path}"), &[])
}

fn tell(game: &mut Game<'_>, line: Text) {
    if let Some(p) = game.native_acting_player() {
        game.native_action_bar(p.uuid, line);
    }
}

/// "Gaveta 1x2: [1] 1.2K / 1K Pedregulho | [2] vazia · Cobre x8, Vazio".
fn info_line(game: &Game<'_>, d: &rt::Drawer<ItemStack>) -> Text {
    let st = St {
        items: world_items!(game),
    };
    let mut line = Text::literal(format!("Gaveta {}:", d.size.label()));
    for (i, slot) in d.slots.iter().enumerate() {
        if i > 0 {
            line = line.append(Text::literal(" |"));
        }
        if d.slots.len() > 1 {
            line = line.append(Text::literal(format!(" [{}]", i + 1)));
        }
        let cap = d.capacity(i, &st);
        match slot.proto.as_ref().filter(|_| slot.total > 0) {
            Some(p) => {
                line = line
                    .append(Text::literal(format!(" {} ", rt::amount_line(slot.total, cap))))
                    .append(item_name(game, p));
            }
            None => line = line.append(Text::literal(format!(" vazia (0 / {})", rt::format_big(cap)))),
        }
    }
    let mut ups: Vec<&str> = d.upgrades.iter().map(|u| u.label()).collect();
    if d.void {
        ups.push("Vazio");
    }
    if !ups.is_empty() {
        line = line.append(Text::literal(format!(" · {}", ups.join(", "))));
    }
    line
}

/// A right click on a drawer, before the block and the item act. Holding an
/// upgrade: it goes in. Sneaking with empty hands: one upgrade comes out.
/// On the front: the hand's stack goes in the slot aimed at; a second click
/// within 300 ms puts in every stack of that item from the inventory. Other
/// faces: the contents on the action bar. Sneaking with something in hand
/// is left to the game (a block placed against the drawer).
pub(super) fn right_click(game: &mut Game<'_>, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
    if *canceled {
        return;
    }
    let Some((size, items, nbt)) = snapshot(game, pos) else {
        return;
    };
    let sneaking = game.native_sneaking();
    let held: Option<ItemStack> = game.held_mut().clone().filter(|s| s.count > 0);
    let hands_full = held.is_some() || game.offhand.as_ref().is_some_and(|s| s.count > 0);
    if sneaking && hands_full {
        return;
    }
    *canceled = true;
    let mut d = {
        let st = St {
            items: world_items!(game),
        };
        read_drawer(size, &items, &nbt, &st)
    };

    if sneaking {
        let removed = {
            let st = St {
                items: world_items!(game),
            };
            d.remove_one(&st)
        };
        match removed {
            Ok(id) => {
                let uuid = game.native_acting_id();
                if let Some(back) = game.native_give_stack(uuid, ItemStack::new(id, 1)) {
                    // No room: it stays in the drawer.
                    let _ = back;
                    tell(game, Text::literal("Inventário cheio: o upgrade ficou na gaveta."));
                    return;
                }
                store(game, pos, &items, &nbt, &d);
                let line = info_line(game, &d);
                tell(game, line);
            }
            Err(rt::Removal::TooFull) => {
                tell(
                    game,
                    Text::literal("A gaveta tem itens demais para tirar esse upgrade."),
                );
            }
            Err(rt::Removal::None) => {
                let line = info_line(game, &d);
                tell(game, line);
            }
        }
        return;
    }

    if let Some(h) = &held {
        if let Some(up) = rt::upgrade_of(&h.item) {
            let result = {
                let st = St {
                    items: world_items!(game),
                };
                d.install(up, &st)
            };
            match result {
                rt::Install::Added | rt::Install::Replaced(_) => {
                    take_held(game, 1);
                    if let rt::Install::Replaced(old) = result {
                        let uuid = game.native_acting_id();
                        if let Some(back) = game.native_give_stack(uuid, ItemStack::new(old.id(), 1)) {
                            // No room for the old one: undo.
                            let _ = back;
                            give_held_back(game, h);
                            tell(
                                game,
                                Text::literal("Inventário cheio: libere espaço para o upgrade trocado."),
                            );
                            return;
                        }
                    }
                    store(game, pos, &items, &nbt, &d);
                    let line = info_line(game, &d);
                    tell(game, line);
                }
                rt::Install::Full => tell(game, Text::literal("A gaveta já tem 4 upgrades de armazenamento.")),
                rt::Install::TooSmall => {
                    tell(game, Text::literal("A gaveta tem itens demais para esse upgrade."));
                }
            }
            return;
        }
        if &*h.item == rt::VOID_UPGRADE {
            if d.install_void() {
                take_held(game, 1);
                store(game, pos, &items, &nbt, &d);
                let line = info_line(game, &d);
                tell(game, line);
            } else {
                tell(game, Text::literal("A gaveta já tem um upgrade de vazio."));
            }
            return;
        }
    }

    let facing = game.native_block_property(pos, "facing").unwrap_or_default();
    if rt::facing_normal(&facing) != Some(face) {
        let line = info_line(game, &d);
        tell(game, line);
        return;
    }
    let slot = aimed_slot(game, pos, size, &facing);
    let uuid = game.native_acting_id();
    let now = game.world.game_time;
    let (last_right, _) = last_clicks(uuid);
    let double = rt::is_double_click(last_right, now);
    remember_click(uuid, Some(now), None);

    if let Some(h) = &held {
        let n = {
            let st = St {
                items: world_items!(game),
            };
            d.insert(slot, h, u64::from(h.count), &st)
        };
        if n > 0 {
            take_held(game, n.min(u64::from(u32::MAX)) as u32);
            store(game, pos, &items, &nbt, &d);
            return;
        }
    }
    if double && d.slots.get(slot).is_some_and(|s| s.proto.is_some()) {
        let carried: Vec<Option<ItemStack>> = game.inventory.iter().take(36).cloned().collect();
        let mut taken: Vec<(usize, u32)> = Vec::new();
        {
            let st = St {
                items: world_items!(game),
            };
            for (k, s) in carried.iter().enumerate() {
                let Some(s) = s.as_ref().filter(|s| s.count > 0) else {
                    continue;
                };
                let n = d.insert(slot, s, u64::from(s.count), &st);
                if n > 0 {
                    taken.push((k, n.min(u64::from(s.count)) as u32));
                }
            }
        }
        if taken.is_empty() {
            return;
        }
        for (k, n) in taken {
            if let Some(slot_stack) = game.inventory.get_mut(k) {
                let left = slot_stack.as_ref().map_or(0, |s| s.count.saturating_sub(n));
                if left == 0 {
                    *slot_stack = None;
                } else if let Some(s) = slot_stack.as_mut() {
                    s.count = left;
                }
            }
        }
        store(game, pos, &items, &nbt, &d);
        return;
    }
    let line = info_line(game, &d);
    tell(game, line);
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

/// Puts one of `like` back in the acting hand (an upgrade swap undone).
fn give_held_back(game: &mut Game<'_>, like: &ItemStack) {
    let slot = game.held_mut();
    match slot.as_mut() {
        Some(s) if s.same_item_same_components(like) => s.count += 1,
        Some(_) => {}
        None => {
            let mut one = like.clone();
            one.count = 1;
            *slot = Some(one);
        }
    }
}

/// A left click on a drawer's front takes from the slot aimed at: one item,
/// a whole stack when sneaking. The block isn't dug from the front; other
/// faces dig as usual.
pub(super) fn left_click(game: &mut Game<'_>, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
    if *canceled {
        return;
    }
    let Some((size, items, nbt)) = snapshot(game, pos) else {
        return;
    };
    let facing = game.native_block_property(pos, "facing").unwrap_or_default();
    if rt::facing_normal(&facing) != Some(face) {
        return;
    }
    *canceled = true;
    let uuid = game.native_acting_id();
    let now = game.world.game_time;
    let (_, last_left) = last_clicks(uuid);
    if last_left.is_some_and(|t| now >= t && now - t < rt::LEFT_CLICK_TICKS) {
        return;
    }
    remember_click(uuid, None, Some(now));
    let slot = aimed_slot(game, pos, size, &facing);
    let whole = game.native_sneaking() == rt::SNEAK_TAKES_STACK;
    let mut d;
    let taken = {
        let st = St {
            items: world_items!(game),
        };
        d = read_drawer(size, &items, &nbt, &st);
        let want = match d.slots.get(slot).and_then(|s| s.proto.as_ref()) {
            Some(p) if whole => u64::from(rt::Stacks::max_stack(&st, p)),
            Some(_) => 1,
            None => 0,
        };
        d.extract(slot, want, &st)
    };
    let Some((proto, n)) = taken else {
        return;
    };
    let mut stack = proto.clone();
    stack.count = n as u32;
    let back = game.native_give_stack(uuid, stack);
    if let Some(rest) = back {
        let st = St {
            items: world_items!(game),
        };
        d.put_back(slot, &proto, u64::from(rest.count), &st);
    }
    store(game, pos, &items, &nbt, &d);
}

/// A player breaking a drawer: refused while it holds items (they would be
/// lost: this port doesn't keep them in the dropped drawer yet); its
/// upgrades go to the player first.
pub(super) fn destroy(game: &mut Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if *canceled {
        return;
    }
    let Some((size, items, nbt)) = snapshot(game, pos) else {
        return;
    };
    let mut d = {
        let st = St {
            items: world_items!(game),
        };
        read_drawer(size, &items, &nbt, &st)
    };
    if !d.is_empty() {
        *canceled = true;
        let total: u64 = d.slots.iter().map(|s| s.total).sum();
        tell(
            game,
            Text::literal(format!(
                "Esvazie a gaveta antes de quebrar ({} itens dentro).",
                rt::format_big(total)
            )),
        );
        return;
    }
    if !d.has_upgrades() {
        return;
    }
    let uuid = game.native_acting_id();
    let mut kept: Vec<rt::Upgrade> = Vec::new();
    for up in std::mem::take(&mut d.upgrades) {
        if game.native_give_stack(uuid, ItemStack::new(up.id(), 1)).is_some() {
            kept.push(up);
        }
    }
    if d.void
        && game
            .native_give_stack(uuid, ItemStack::new(rt::VOID_UPGRADE, 1))
            .is_none()
    {
        d.void = false;
    }
    d.upgrades = kept;
    if d.has_upgrades() {
        *canceled = true;
        tell(
            game,
            Text::literal("Inventário cheio: libere espaço para os upgrades da gaveta."),
        );
    }
    store(game, pos, &items, &nbt, &d);
}

/// An explosion spares a drawer that holds items or upgrades.
pub(super) fn explosion(game: &mut Game<'_>, dimension: &str, blocks: &mut Vec<[i32; 3]>) {
    if dimension != game.dim.id() {
        return;
    }
    let g: &Game<'_> = game;
    blocks.retain(|p| {
        let Some((size, items, nbt)) = snapshot(g, *p) else {
            return true;
        };
        let d = read_drawer(size, &items, &nbt, &Plain);
        d.is_empty() && !d.has_upgrades()
    });
}

// Pocket Storage for Horizon: the rules (`pocketstorage_rt`) over Horizon's
// item stacks and server menus (API 10), item use (API 13), pickups (API 40),
// item entities (API 42), commands (API 8), sneaking (API 351) and tooltips
// (API 12). Added to the game as `crate::game::pocketstorage_glue`; the hook
// helpers call it.
//
// A right click with a PSU asks to open it; the next server tick opens a
// chest menu of copies (each slot's item with its real count in the lore).
// Each click on it is answered here, with the hub in hand and no player's
// state swapped in: the PSU (its slots in `minecraft:custom_data`) and the
// player's inventory change together in one step, after checking that the
// PSU is still exactly where and what it was. Anything else closes the menu.
//
// The pickup runs in the player's own turn (their state swapped in): a PSU
// in the 36 inventory slots that already holds the item takes the whole
// stack (the excess is destroyed, as the original's "voids excess").

use super::Game;
use super::pocketstorage_rt as rt;
use horizon_client::first_person::Hand;
use horizon_client::items::ItemStack;
use horizon_client::text::Text;
use horizon_world::nbt::{JavaString, Tag};
use std::sync::Mutex;

/// Where the PSU sits in the player's inventory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PsuHolder {
    Slot(usize),
    OffHand,
}

impl PsuHolder {
    fn locked(self) -> Option<usize> {
        match self {
            PsuHolder::Slot(i) => Some(i),
            PsuHolder::OffHand => None,
        }
    }
}

struct PsuOpen {
    uuid: u128,
    window: i32,
    holder: PsuHolder,
    /// The PSU stack as this mod last left it: any other change closes the
    /// menu instead of acting on it.
    stack: ItemStack,
    page: usize,
}

static PSU_PENDING: Mutex<Vec<(u128, Hand)>> = Mutex::new(Vec::new());
static PSU_OPEN: Mutex<Vec<PsuOpen>> = Mutex::new(Vec::new());
/// Each player's search (`/psu buscar`), normalized.
static PSU_SEARCH: Mutex<Vec<(u128, String)>> = Mutex::new(Vec::new());
/// The players sneaking now (no pickup then, as the original).
static PSU_SNEAKING: Mutex<Vec<u128>> = Mutex::new(Vec::new());

fn psu_lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn search_of(uuid: u128) -> String {
    psu_lock(&PSU_SEARCH).iter().find(|(u, _)| *u == uuid).map(|(_, q)| q.clone()).unwrap_or_default()
}

fn set_search(uuid: u128, query: &str) {
    let mut all = psu_lock(&PSU_SEARCH);
    all.retain(|(u, _)| *u != uuid);
    if !query.is_empty() {
        if all.len() > 256 {
            all.clear();
        }
        all.push((uuid, query.to_owned()));
    }
}

// ------------------------------------------------------------------ stacks

/// No components at all: the only kind a PSU takes.
fn is_simple(s: &ItemStack) -> bool {
    let mut c = s.clone();
    c.tidy();
    c.damage == 0 && c.extra.is_none()
}

fn max_of(game: &Game<'_>, item: &str) -> u32 {
    game.res.items().get(item).map_or(64, |i| i.max_stack).clamp(1, 99)
}

/// The slots' text a PSU keeps in its `minecraft:custom_data`.
fn slots_text(s: &ItemStack) -> Option<String> {
    match s.data()?.other("minecraft:custom_data")?.get(rt::DATA_KEY)? {
        Tag::String(t) => Some(t.to_string_lossy()),
        _ => None,
    }
}

fn slots_of(s: &ItemStack, tier: &rt::Tier) -> rt::Slots {
    rt::decode(slots_text(s).as_deref(), tier)
}

/// One PSU holding `slots`, the rest of its `minecraft:custom_data` and its
/// other components kept; an empty PSU has no key (and no `custom_data` if
/// nothing else was there).
fn with_slots(s: &ItemStack, slots: &[Option<rt::Entry>]) -> ItemStack {
    let text = rt::encode(slots);
    let mut one = s.clone();
    one.count = 1;
    let mut map = match one.data().and_then(|d| d.other("minecraft:custom_data")) {
        Some(Tag::Compound(m)) => m.clone(),
        _ => Default::default(),
    };
    map.remove(&JavaString::from(rt::DATA_KEY));
    if !text.is_empty() {
        map.insert(JavaString::from(rt::DATA_KEY), Tag::String(JavaString::from(text.as_str())));
    }
    let data = one.data_mut();
    let value = if map.is_empty() { None } else { Some(Tag::Compound(map)) };
    data.set_other("minecraft:custom_data", value);
    data.wire = Default::default();
    one.tidy();
    one
}

fn psu_named(item: &str, count: u32, name: &str) -> ItemStack {
    let mut s = ItemStack::new(item, count);
    s.data_mut().custom_name = Some(Tag::String(name.into()));
    s.tidy();
    s
}

fn psu_filler() -> ItemStack {
    psu_named("minecraft:gray_stained_glass_pane", 1, " ")
}

fn with_lore(mut s: ItemStack, lines: &[String]) -> ItemStack {
    let list: Vec<Tag> = lines.iter().map(|l| Tag::String(JavaString::from(l.as_str()))).collect();
    s.data_mut().set_other("minecraft:lore", Some(Tag::List(list)));
    s.tidy();
    s
}

/// What a PSU slot shows: its item, at most 64 of it on the slot, the real
/// count in the lore.
fn shown(e: &rt::Entry, tier: &rt::Tier) -> ItemStack {
    with_lore(
        ItemStack::new(&e.item, rt::shown_count(e.count)),
        &[
            rt::stored_line(e.count, tier.capacity),
            "Clique: retira 1 | Shift+clique: retira 1 pack".to_owned(),
        ],
    )
}

fn inv_of(stacks: &[Option<ItemStack>]) -> Vec<Option<rt::InvStack>> {
    stacks
        .iter()
        .map(|s| {
            s.as_ref().map(|s| rt::InvStack { item: s.item.to_string(), count: s.count, simple: is_simple(s) })
        })
        .collect()
}

/// The inventory after the rules changed `after` (from `before`): a slot
/// left alone keeps its stack as it was, components and all.
fn inv_back(stacks: &[Option<ItemStack>], before: &[Option<rt::InvStack>], after: &[Option<rt::InvStack>]) -> Vec<Option<ItemStack>> {
    stacks
        .iter()
        .zip(before.iter().zip(after.iter()))
        .map(|(old, (b, a))| {
            if a == b {
                return old.clone();
            }
            match (a, old) {
                (None, _) => None,
                (Some(a), Some(old)) if !a.simple => {
                    let mut s = old.clone();
                    s.count = a.count;
                    Some(s)
                }
                (Some(a), _) => Some(ItemStack::new(&a.item, a.count)),
            }
        })
        .collect()
}

// -------------------------------------------------------------------- menu

/// The menu an open PSU shows now, at `page` (kept within its pages).
fn render(stack: &ItemStack, page: &mut usize, query: &str) -> Option<(u8, Vec<Option<ItemStack>>)> {
    let tier = rt::tier(&stack.item)?;
    let l = rt::layout(tier);
    let slots = slots_of(stack, tier);
    let list = rt::visible(&slots, query);
    let pages = rt::pages(&l, list.len());
    if *page >= pages {
        *page = pages - 1;
    }
    let mut out: Vec<Option<ItemStack>> = vec![Some(psu_filler()); l.size()];
    for (k, cell) in out.iter_mut().enumerate().take(l.per_page) {
        *cell = match list.get(*page * l.per_page + k) {
            Some(&i) => slots.get(i).cloned().flatten().map(|e| shown(&e, tier)),
            None if query.is_empty() => Some(psu_filler()),
            None => None,
        };
    }
    let c = l.control();
    if pages > 1 {
        out[c + rt::C_PREV] = Some(psu_named("minecraft:arrow", 1, &format!("< Pagina anterior ({}/{pages})", *page + 1)));
        out[c + rt::C_NEXT] = Some(psu_named("minecraft:spectral_arrow", 1, &format!("Proxima pagina > ({}/{pages})", *page + 1)));
    }
    out[c + rt::C_SEARCH] = Some(if query.is_empty() {
        with_lore(psu_named("minecraft:oak_sign", 1, "Sem busca"), &["Digite /psu buscar <texto> e abra o PSU de novo".to_owned()])
    } else {
        with_lore(
            psu_named("minecraft:name_tag", 1, &format!("Busca: {query}")),
            &[format!("{} encontrados - clique para limpar", list.len())],
        )
    });
    out[c + rt::C_INFO] = Some(with_lore(
        psu_named("minecraft:book", 1, tier.name),
        &[rt::info_line(tier), rt::used_line(&slots, tier)],
    ));
    Some((l.rows, out))
}

/// The PSU at `holder` of the guest, if it is a single PSU.
fn held(g: &horizon_client::lan::guest::Guest, holder: PsuHolder) -> Option<ItemStack> {
    let s = match holder {
        PsuHolder::Slot(i) => g.inventory.get(i).cloned().flatten(),
        PsuHolder::OffHand => g.offhand.clone(),
    }?;
    (rt::is_psu(&s.item) && s.count == 1).then_some(s)
}

/// `native_use_item`: a right click with a PSU is ours; it opens on the next
/// server tick, outside the player's swapped-in state.
pub(crate) fn use_item(game: &mut Game<'_>, stack: &ItemStack, claimed: &mut bool) {
    if !rt::is_psu(&stack.item) || !game.native_on(rt::ID) {
        return;
    }
    *claimed = true;
    if let Some(p) = game.native_acting_player() {
        let mut pending = psu_lock(&PSU_PENDING);
        pending.retain(|(u, _)| *u != p.uuid);
        pending.push((p.uuid, game.hand));
    }
}

/// `native_server_tick`: opens the PSUs asked for.
pub(crate) fn server_tick(game: &mut Game<'_>) {
    let pending = std::mem::take(&mut *psu_lock(&PSU_PENDING));
    for (uuid, hand) in pending {
        let found = game.lan.as_ref().and_then(|host| {
            let g = host.hub.guests.iter().find(|g| g.placed && !g.dead && g.uuid == uuid)?;
            let holder = match hand {
                Hand::Off => PsuHolder::OffHand,
                _ => PsuHolder::Slot(g.selected),
            };
            Some((holder, held(g, holder)?))
        });
        let Some((holder, stack)) = found else {
            continue;
        };
        let Some(tier) = rt::tier(&stack.item) else {
            continue;
        };
        let mut page = 0;
        let Some((rows, items)) = render(&stack, &mut page, &search_of(uuid)) else {
            continue;
        };
        if let Some(window) = game.native_open_menu(uuid, rows, Text::literal(tier.name), items) {
            let mut open = psu_lock(&PSU_OPEN);
            open.retain(|o| o.uuid != uuid);
            open.push(PsuOpen { uuid, window, holder, stack, page });
        }
    }
}

/// `native_menu_closed`.
pub(crate) fn menu_closed(uuid: u128, window: i32) {
    psu_lock(&PSU_OPEN).retain(|o| !(o.uuid == uuid && o.window == window));
}

/// `native_menu_clicked`: one click, answered whole or not at all.
pub(crate) fn menu_clicked(game: &mut Game<'_>, uuid: u128, window: i32, slot: i32, button: i32, mode: i32) {
    let mut open = psu_lock(&PSU_OPEN);
    let Some(at) = open.iter().position(|o| o.uuid == uuid && o.window == window) else {
        return;
    };
    let Some(mut host) = game.lan.take() else {
        return;
    };
    let keep = psu_click(game, &mut host, &mut open[at], slot, button, mode);
    game.lan = Some(host);
    if !keep {
        open.remove(at);
        drop(open);
        game.native_close_menu(uuid);
    }
}

/// False: the menu must close.
fn psu_click(game: &Game<'_>, host: &mut super::lan::LanHost, o: &mut PsuOpen, slot: i32, button: i32, mode: i32) -> bool {
    let Some(gi) = host.hub.guests.iter().position(|g| g.placed && !g.dead && g.uuid == o.uuid) else {
        return false;
    };
    let conn = host.hub.guests[gi].conn;
    match held(&host.hub.guests[gi], o.holder) {
        Some(s) if s == o.stack => {}
        _ => return false,
    }
    let Some(tier) = rt::tier(&o.stack.item) else {
        return false;
    };
    let l = rt::layout(tier);
    let mut slots = slots_of(&o.stack, tier);
    let mut query = search_of(o.uuid);
    let list = rt::visible(&slots, &query);
    let pages = rt::pages(&l, list.len());
    let locked = o.holder.locked();
    let g = &host.hub.guests[gi];
    let n = g.inventory.len().min(36);
    let stacks: Vec<Option<ItemStack>> = g.inventory[..n].to_vec();
    let before = inv_of(&stacks);
    let mut inv = before.clone();
    let mut changed = false;
    let mut redraw = false;
    match rt::target(&l, &list, o.page.min(pages - 1), slot) {
        rt::Target::Prev => {
            o.page = if o.page == 0 { pages - 1 } else { o.page - 1 };
            redraw = true;
        }
        rt::Target::Next => {
            o.page = if o.page + 1 >= pages { 0 } else { o.page + 1 };
            redraw = true;
        }
        rt::Target::Search => {
            if !query.is_empty() {
                set_search(o.uuid, "");
                query.clear();
                o.page = 0;
                redraw = true;
            }
        }
        rt::Target::Slot(i) => {
            if let Some(a) = rt::amount(button, mode)
                && let Some(Some(e)) = slots.get(i).cloned()
            {
                let max = max_of(game, &e.item);
                let want = match a {
                    rt::Amount::One => 1,
                    rt::Amount::Stack => max,
                };
                changed = rt::withdraw(&mut slots, i, want, &mut inv, max, locked) > 0;
            }
        }
        rt::Target::Inventory(k) if k < n => {
            if let Some(a) = rt::deposit_amount(button, mode) {
                changed = rt::deposit(&mut inv, k, a, &mut slots, tier.capacity, locked) > 0;
            }
        }
        _ => {}
    }
    if changed {
        let stack = with_slots(&o.stack, &slots);
        let back = inv_back(&stacks, &before, &inv);
        let g = &mut host.hub.guests[gi];
        g.inventory[..n].clone_from_slice(&back);
        match o.holder {
            PsuHolder::Slot(i) => g.inventory[i] = Some(stack.clone()),
            PsuHolder::OffHand => g.offhand = Some(stack.clone()),
        }
        o.stack = stack;
    }
    if changed || redraw {
        if let Some((_, items)) = render(&o.stack, &mut o.page, &query) {
            host.hub.set_native_menu(conn, items);
        }
    }
    true
}

// ------------------------------------------------------------------ pickup

/// `native_player_toggle_sneak`.
pub(crate) fn toggle_sneak(player: u128, sneaking: bool) {
    let mut all = psu_lock(&PSU_SNEAKING);
    all.retain(|u| *u != player);
    if sneaking {
        if all.len() > 256 {
            all.clear();
        }
        all.push(player);
    }
}

/// `native_item_picked_up`: an item one of the player's PSUs (the 36
/// inventory slots, in order) already holds goes into the first such PSU,
/// whole; the vanilla pickup is canceled. Not while sneaking or with a PSU
/// open (as the original).
pub(crate) fn item_picked_up(game: &mut Game<'_>, player: u128, item: &str, count: u32, canceled: &mut bool) {
    if *canceled || count == 0 || rt::refused(item) || !game.native_on(rt::ID) {
        return;
    }
    if psu_lock(&PSU_SNEAKING).contains(&player) || psu_lock(&PSU_OPEN).iter().any(|o| o.uuid == player) {
        return;
    }
    let Some(me) = game.native_acting_player() else {
        return;
    };
    if me.uuid != player || me.game_mode == 3 {
        return;
    }
    let mut chosen: Option<(usize, ItemStack, rt::Slots, &'static rt::Tier)> = None;
    for (i, s) in game.inventory.iter().enumerate().take(36) {
        if let Some(s) = s
            && s.count == 1
            && let Some(tier) = rt::tier(&s.item)
        {
            let slots = slots_of(s, tier);
            if rt::has(&slots, item) {
                chosen = Some((i, s.clone(), slots, tier));
                break;
            }
        }
    }
    let Some((at, pack, mut slots, tier)) = chosen else {
        return;
    };
    let [x, y, z] = me.position;
    let near = game.native_item_entities(&me.dimension, [x - 2.0, y - 1.0, z - 2.0], [x + 2.0, y + 3.0, z + 2.0]);
    let ints = horizon_client::lan::playerdata::uuid_ints(me.uuid);
    // The entity the pickup is about: the same simple item and count,
    // touching the player, takeable now. Another identical one is as good.
    let Some(e) = near.into_iter().find(|e| {
        e.pickup_delay == 0
            && &*e.stack.item == item
            && e.stack.count == count
            && is_simple(&e.stack)
            && (e.position[0] - x).abs() < 1.5
            && (e.position[2] - z).abs() < 1.5
            && e.position[1] > y - 0.6
            && e.position[1] < y + 2.4
            && game.dropped.iter().find(|d| d.id == e.id).is_some_and(|d| d.target.is_none_or(|t| t == ints))
    }) else {
        return;
    };
    if !rt::absorb(&mut slots, item, e.stack.count, tier.capacity) {
        return;
    }
    // The entity first: if it can't be changed, nothing is.
    let mut gone = e.stack.clone();
    gone.count = 0;
    if !game.native_set_item_entity(&me.dimension, e.id, gone) {
        return;
    }
    let new = with_slots(&pack, &slots);
    if let Some(slot) = game.inventory.get_mut(at) {
        *slot = Some(new);
    }
    *canceled = true;
}

// ---------------------------------------------------------------- commands

/// `native_register_commands`: `/psu buscar <texto>` and `/psu limpar`.
pub(crate) fn register_commands(commands: &mut Vec<horizon_client::mod_api::NativeCommand>) {
    commands.push(horizon_client::mod_api::NativeCommand::new(rt::COMMAND_SEARCH));
    commands.push(horizon_client::mod_api::NativeCommand::new(rt::COMMAND_CLEAR));
}

/// `native_perform_command`.
pub(crate) fn perform_command(
    game: &mut Game<'_>,
    player: &horizon_client::mod_api::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    if *ran {
        return;
    }
    let line = match rt::parse_command(command) {
        rt::Command::NotOurs => return,
        rt::Command::Search(q) => {
            set_search(player.uuid, &q);
            format!("PSU: busca \"{q}\". Abra o PSU para ver so o que combina.")
        }
        rt::Command::Clear => {
            set_search(player.uuid, "");
            "PSU: busca limpa.".to_owned()
        }
        rt::Command::Help => "Uso: /psu buscar <texto> | /psu limpar".to_owned(),
    };
    *ran = true;
    game.native_feedback(Text::literal(line));
}

// ----------------------------------------------------------------- tooltip

/// `native_tooltip`: the original's info line and how many kinds are used.
pub(crate) fn tooltip(stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    let Some(tier) = rt::tier(&stack.item) else {
        return;
    };
    lines.push((Text::literal(rt::info_line(tier)).with_color(0xAAAAAA), u32::MAX));
    let slots = slots_of(stack, tier);
    lines.push((Text::literal(rt::used_line(&slots, tier)).with_color(0xFFAA00), u32::MAX));
    lines.push((Text::literal("Pega sozinho o que ja guarda; o excesso some.").with_color(0xAAAAAA), u32::MAX));
}


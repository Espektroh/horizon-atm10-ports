// Iron Jetpacks for Horizon: the rules (`ironjetpacks_rt`) over Horizon's
// stacks and players. Added to the game as `crate::game::ironjetpacks_glue`;
// the hook helpers call it.
//
// Movement is the client's own (vanilla and Horizon alike: the server
// accepts the moves its client reports), so the flight is pushed on the
// window's player, as JetpackItem.inventoryTick does on the client side:
// each tick (`native_tick`, before the player moves) the push of the worn
// jetpack goes into the player's velocity (`native_push_player`, API 105).
// The keys it reads are last tick's (`native_movement_input`).
//
// The server keeps the jetpack itself: the client tells it the jump and
// sneak keys (`ironjetpacks_native:input`) and the toggles
// (`ironjetpacks_native:toggle`, mod messages, API 40). At the end of each
// player's tick (`native_player_tick`, their state swapped in) the worn
// jetpack (`armor[1]`, the chest) is toggled and drained, and while it flies
// the player's fall distance is put back to 0, as the mod does
// (`player.fallDistance = 0`). Off a single player world the floating check
// is reset too (`aboveGroundTickCount = 0`), in the server tick.
//
// A right click with a jetpack on a Mekanism energy cube or a Powah energy
// cell moves the energy into it (`native_right_click_block`, API 21).

use super::Game;
use super::ironjetpacks_rt as rt;
use horizon_client::first_person::Hand;
use horizon_client::items::ItemStack;
use horizon_client::text::Text;
use horizon_world::nbt::{Compound, JavaString, Tag};
use std::sync::Mutex;

/// The window's keys, as `native_movement_input` last read them.
#[derive(Clone, Copy, Default)]
struct Keys {
    forward: f32,
    strafe: f32,
    jump: bool,
    sneak: bool,
}
static CLIENT_KEYS: Mutex<Option<Keys>> = Mutex::new(None);
/// What the window last told the server, and the ticks since.
static CLIENT_SENT: Mutex<(u8, u32)> = Mutex::new((0, 0));

/// Server: each player's keys (`NET_INPUT`), toggles waiting for their tick,
/// and the players whose jetpack flew this tick.
static SERVER_KEYS: Mutex<Vec<(u128, u8)>> = Mutex::new(Vec::new());
static TOGGLES: Mutex<Vec<(u128, rt::Toggle)>> = Mutex::new(Vec::new());
static FLYING: Mutex<Vec<u128>> = Mutex::new(Vec::new());

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// ------------------------------------------------------------- the stack

fn number(map: &Compound, key: &str) -> Option<i64> {
    match map.get(&JavaString::from(key)) {
        Some(Tag::Byte(v)) => Some(i64::from(*v)),
        Some(Tag::Short(v)) => Some(i64::from(*v)),
        Some(Tag::Int(v)) => Some(i64::from(*v)),
        Some(Tag::Long(v)) => Some(*v),
        _ => None,
    }
}

/// The jetpack's state from its `minecraft:custom_data`.
fn state_of(stack: &ItemStack) -> rt::State {
    match stack.data().and_then(|d| d.other("minecraft:custom_data")) {
        Some(Tag::Compound(map)) => rt::State::read(
            number(map, rt::ENERGY_KEY),
            number(map, rt::ENGINE_KEY),
            number(map, rt::HOVER_KEY),
            number(map, rt::THROTTLE_KEY),
        ),
        _ => rt::State::default(),
    }
}

/// One jetpack holding `s`, the rest of its data kept. Defaults are left out,
/// so a new jetpack and an emptied one carry nothing.
fn with_state(stack: &ItemStack, s: rt::State) -> ItemStack {
    let mut one = stack.clone();
    one.count = 1;
    let mut map = match one.data().and_then(|d| d.other("minecraft:custom_data")) {
        Some(Tag::Compound(m)) => m.clone(),
        _ => Compound::new(),
    };
    for key in [rt::ENERGY_KEY, rt::ENGINE_KEY, rt::HOVER_KEY, rt::THROTTLE_KEY] {
        map.remove(&JavaString::from(key));
    }
    if s.energy > 0 {
        map.insert(JavaString::from(rt::ENERGY_KEY), Tag::Int(s.energy));
    }
    if s.engine {
        map.insert(JavaString::from(rt::ENGINE_KEY), Tag::Byte(1));
    }
    if s.hover {
        map.insert(JavaString::from(rt::HOVER_KEY), Tag::Byte(1));
    }
    if s.throttle != 100 {
        map.insert(JavaString::from(rt::THROTTLE_KEY), Tag::Int(s.throttle));
    }
    let value = if map.is_empty() { None } else { Some(Tag::Compound(map)) };
    let data = one.data_mut();
    data.set_other("minecraft:custom_data", value);
    data.wire = Default::default();
    one.tidy();
    one
}

/// The jetpack in the chest slot, with its tier and state.
fn worn(armor: &[Option<ItemStack>]) -> Option<(&'static rt::Tier, rt::State)> {
    let s = armor.get(1)?.as_ref()?;
    let tier = rt::tier_of(&s.item)?;
    Some((tier, state_of(s)))
}

// --------------------------------------------------------------- client

/// `native_movement_input` (the window): the keys of this tick, for the
/// next tick's push.
pub(crate) fn movement_input(forward: f32, strafe: f32, jump: bool, sneak: bool) {
    *lock(&CLIENT_KEYS) = Some(Keys { forward, strafe, jump, sneak });
}

/// `native_tick`, the window's game: tells the server the keys, then pushes
/// the player as the worn jetpack does.
pub(crate) fn tick(game: &mut Game<'_>) {
    if !game.native_window() || !game.native_on(rt::ID) {
        return;
    }
    // Taken, so keys the game stopped reading (a screen, a loading level)
    // don't keep the jetpack going.
    let keys = lock(&CLIENT_KEYS).take().unwrap_or_default();
    let jetpack = worn(&game.armor);
    let flags = match jetpack {
        Some((_, s)) if s.engine => rt::input_flags(keys.jump, keys.sneak),
        _ => 0,
    };
    {
        let mut sent = lock(&CLIENT_SENT);
        sent.1 = sent.1.saturating_add(1);
        // On a change, and every second while a key is held (a lost or early
        // message mends itself).
        if flags != sent.0 || (flags != 0 && sent.1 >= 20) {
            if game.native_net_send_to_server(rt::NET_INPUT, &[flags]).is_ok() {
                *sent = (flags, 0);
            }
        }
    }
    let Some((tier, s)) = jetpack else {
        return;
    };
    let yaw = match game.native_player() {
        Some(p) => p.yaw,
        None => return,
    };
    let creative = game.creative;
    let Some(p) = game.player.as_ref() else {
        return;
    };
    if p.spectator {
        return;
    }
    let body = rt::Body {
        velocity: [p.velocity.x, p.velocity.y, p.velocity.z],
        on_ground: p.ground.on_ground,
        in_water: p.fluids.was_touching_water,
        fall_flying: p.fall_flying,
        yaw,
        creative,
    };
    let input = rt::Input {
        up: keys.jump,
        down: keys.sneak,
        forward: keys.forward,
        strafe: keys.strafe,
        sprint: p.sprinting,
    };
    if let Some(v) = rt::push(tier, s, input, body) {
        game.native_push_player([v[0] as f32, v[1] as f32, v[2] as f32]);
    }
}

/// `native_key` (the window): the engine, hover and throttle keys ask the
/// server to change the worn jetpack; it answers on the action bar.
pub(crate) fn key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !pressed || !game.native_on(rt::ID) || !game.native_playing() {
        return;
    }
    if worn(&game.armor).is_none() {
        return;
    }
    for (name, _, default) in rt::KEYS {
        if key == game.native_mod_key(rt::ID, name, default) {
            if let Some(t) = rt::Toggle::of_key(name) {
                let _ = game.native_net_send_to_server(rt::NET_TOGGLE, &[t.code()]);
            }
        }
    }
}

/// `native_hud`: a box with the worn jetpack's energy and state.
pub(crate) fn hud(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    if !game.native_on(rt::ID) {
        return;
    }
    let Some((tier, s)) = worn(&game.armor) else {
        return;
    };
    use horizon_client::extras::Corner;
    let corner = if game.native_corner_shown(Corner::TopLeft) {
        Corner::TopLeft
    } else {
        Corner::BottomLeft
    };
    let lines = rt::hud_lines(tier, s);
    let width = lines
        .iter()
        .map(|l| game.native_text_width(l))
        .fold(0.0f32, f32::max)
        + 6.0;
    let height = 10.0 * lines.len() as f32 + 4.0;
    let (x, y) = corners.place(corner, width, height);
    game.native_fill(batch, [x, y, width, height], 0x7800_0000);
    let colors = [tier.color, 0x00E0_E0E0, 0x00E0_E0E0];
    for (k, line) in lines.iter().enumerate() {
        game.native_text(batch, line, [x + 3.0, y + 3.0 + 10.0 * k as f32], colors[k], true);
    }
}

/// `native_tooltip`: energy and state under the name.
pub(crate) fn tooltip(stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    let Some(tier) = rt::tier_of(&stack.item) else {
        return;
    };
    let at = lines.len().min(1);
    for (k, line) in rt::tooltip_lines(tier, state_of(stack)).iter().enumerate() {
        lines.insert(at + k, (Text::legacy(line), u32::MAX));
    }
}

/// `native_item_bar`: the energy bar.
pub(crate) fn item_bar(stack: &ItemStack, bar: &mut Option<(i32, u32)>) {
    if let Some(tier) = rt::tier_of(&stack.item) {
        *bar = rt::bar(tier, state_of(stack).energy);
    }
}

// --------------------------------------------------------------- server

/// `native_net_from_client`: a player's keys and toggles.
pub(crate) fn net_from_client(game: &mut Game<'_>, player: u128, channel: &str, data: &[u8]) {
    if !game.native_on(rt::ID) {
        return;
    }
    if channel == rt::NET_INPUT {
        let flags = data.first().copied().unwrap_or(0) & 3;
        let mut keys = lock(&SERVER_KEYS);
        keys.retain(|(u, _)| *u != player);
        if flags != 0 {
            if keys.len() > 512 {
                keys.clear();
            }
            keys.push((player, flags));
        }
    } else if channel == rt::NET_TOGGLE {
        if let Some(t) = data.first().copied().and_then(rt::Toggle::from_code) {
            let mut pending = lock(&TOGGLES);
            if pending.len() < 256 {
                pending.push((player, t));
            }
        }
    }
}

fn server_flags(uuid: u128) -> u8 {
    lock(&SERVER_KEYS).iter().find(|(u, _)| *u == uuid).map_or(0, |(_, f)| *f)
}

fn take_toggles(uuid: u128) -> Vec<rt::Toggle> {
    let mut pending = lock(&TOGGLES);
    let mine = pending.iter().filter(|(u, _)| *u == uuid).map(|(_, t)| *t).collect();
    pending.retain(|(u, _)| *u != uuid);
    mine
}

/// `native_player_tick`: the worn jetpack's toggles and energy, and no fall
/// damage while it flies.
pub(crate) fn player_tick(game: &mut Game<'_>) {
    if !game.native_on(rt::ID) {
        return;
    }
    let Some(me) = game.native_acting_player() else {
        return;
    };
    let toggles = take_toggles(me.uuid);
    if me.game_mode == 3 {
        return;
    }
    let Some(stack) = game.armor.get(1).cloned().flatten() else {
        return;
    };
    let Some(tier) = rt::tier_of(&stack.item) else {
        return;
    };
    let before = state_of(&stack);
    let mut s = before;
    let mut told = Vec::new();
    for t in toggles {
        s = t.apply(s);
        told.push(t.message(s));
    }
    let (on_ground, sprinting) = match game.player.as_ref() {
        Some(p) => (p.ground.on_ground, p.sprinting),
        None => return,
    };
    let (up, _down) = rt::flags_input(server_flags(me.uuid));
    if rt::thrusting(s, up, on_ground) {
        let (left, flies) = rt::drain(tier, s, sprinting, me.game_mode == 1);
        s.energy = left;
        if flies {
            if let Some(p) = game.player.as_mut() {
                p.fall_distance = 0.0;
            }
            let mut flying = lock(&FLYING);
            if !flying.contains(&me.uuid) && flying.len() < 512 {
                flying.push(me.uuid);
            }
        }
    }
    if s != before {
        if let Some(slot) = game.armor.get_mut(1) {
            *slot = Some(with_state(&stack, s));
        }
    }
    for line in told {
        game.native_action_bar(me.uuid, Text::literal(line));
    }
}

/// `native_server_tick`: the players whose jetpack flew are not floating
/// (`aboveGroundTickCount = 0`; a server without allow-flight would kick
/// them after 4 seconds).
pub(crate) fn server_tick(game: &mut Game<'_>) {
    let flying = std::mem::take(&mut *lock(&FLYING));
    if flying.is_empty() {
        return;
    }
    if let Some(host) = game.lan.as_mut() {
        for g in host.hub.guests.iter_mut() {
            if flying.contains(&g.uuid) {
                g.above_ground_ticks = 0;
            }
        }
    }
}

/// `native_right_click_block`: a jetpack in the main hand on an energy cube
/// or cell takes what it can of the block's energy.
pub(crate) fn right_click(game: &mut Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if *canceled || game.hand != Hand::Main || !game.native_on(rt::ID) {
        return;
    }
    let selected = game.selected;
    let Some(held) = game.inventory.get(selected).cloned().flatten() else {
        return;
    };
    let Some(tier) = rt::tier_of(&held.item) else {
        return;
    };
    let Some(kind) = rt::charger(&game.native_block(pos)) else {
        return;
    };
    // Ours: the block's own menu doesn't open.
    *canceled = true;
    if tier.creative || held.count != 1 {
        return;
    }
    let s = state_of(&held);
    let room = i64::from(tier.capacity) - i64::from(s.energy);
    if room <= 0 {
        return;
    }
    let gained = game
        .native_with_mod_block_entity(pos, |m| {
            let stored = number(&m.nbt, "energy").unwrap_or(0);
            let (fe, taken) = rt::charge(kind, stored, room);
            if fe > 0 {
                m.nbt.insert("energy".into(), Tag::Long(stored - taken));
            }
            fe
        })
        .unwrap_or(0);
    if gained <= 0 {
        return;
    }
    let energy = (i64::from(s.energy) + gained).min(i64::from(tier.capacity)) as i32;
    let charged = with_state(&held, rt::State { energy, ..s });
    if let Some(slot) = game.inventory.get_mut(selected) {
        *slot = Some(charged);
    }
    if let Some(me) = game.native_acting_player() {
        game.native_action_bar(me.uuid, Text::literal(rt::charge_message(tier, gained, energy)));
    }
}

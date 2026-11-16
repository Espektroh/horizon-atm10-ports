//! The glue over the stubs: what a hook does to the player, the jetpack and
//! the blocks. Run with --test-threads=1 (the glue keeps statics).
use super::Game;
use super::ironjetpacks_glue as glue;
use super::ironjetpacks_rt as rt;
use horizon_client::containers::{ModBlockEntity, ModBlockEntityFields};
use horizon_client::first_person::Hand;
use horizon_client::items::ItemStack;
use horizon_client::mod_api::NativeServerPlayer;
use horizon_client::motion::PlayerMotion;
use horizon_world::nbt::{Compound, JavaString, Tag};

fn game(window: bool) -> Game<'static> {
    Game {
        _p: std::marker::PhantomData,
        player: Some(PlayerMotion::default()),
        armor: [None, None, None, None],
        inventory: vec![None; 41],
        selected: 0,
        hand: Hand::Main,
        creative: false,
        yaw: 0.0,
        lan: None,
        window,
        acting: None,
        sent: Vec::new(),
        bar: Vec::new(),
        blocks: Vec::new(),
    }
}

fn player(uuid: u128, mode: i32) -> NativeServerPlayer {
    NativeServerPlayer {
        uuid,
        name: "Steve".into(),
        permission: 4,
        dimension: "minecraft:overworld".into(),
        position: [0.0, 64.0, 0.0],
        game_mode: mode,
        health: 20.0,
        yaw: 0.0,
        pitch: 0.0,
        entity_id: 1,
        address: String::new(),
    }
}

fn jetpack(item: &str, energy: i32, engine: bool, hover: bool) -> ItemStack {
    let mut map = Compound::new();
    if energy > 0 {
        map.insert(JavaString::from(rt::ENERGY_KEY), Tag::Int(energy));
    }
    if engine {
        map.insert(JavaString::from(rt::ENGINE_KEY), Tag::Byte(1));
    }
    if hover {
        map.insert(JavaString::from(rt::HOVER_KEY), Tag::Byte(1));
    }
    let mut s = ItemStack::new(item, 1);
    if !map.is_empty() {
        s.data_mut().set_other("minecraft:custom_data", Some(Tag::Compound(map)));
    }
    s
}

fn get(s: &ItemStack, key: &str) -> Option<Tag> {
    match s.data()?.other("minecraft:custom_data")? {
        Tag::Compound(m) => m.get(&JavaString::from(key)).cloned(),
        _ => None,
    }
}

fn energy(s: &ItemStack) -> i32 {
    match get(s, rt::ENERGY_KEY) {
        Some(Tag::Int(v)) => v,
        _ => 0,
    }
}

#[test]
fn client_flies_with_jump_held_and_tells_the_server() {
    let mut g = game(true);
    g.armor[1] = Some(jetpack("ironjetpacks:iron_jetpack", 100_000, true, false));
    // Nothing held: no push.
    g.native_movement_input(&mut 0.0, &mut 0.0, &mut false, &mut false);
    g.native_tick();
    assert_eq!(g.player.as_ref().unwrap().velocity.y, 0.0);
    // Jump held: the velocity goes to the iron speed (0.2), and the server hears it.
    g.player.as_mut().unwrap().velocity.y = -0.08;
    g.native_movement_input(&mut 0.0, &mut 0.0, &mut true, &mut false);
    g.native_tick();
    let v = g.player.as_ref().unwrap().velocity;
    assert!((v.y - 0.2).abs() < 1e-6, "{v:?}");
    assert!(g.sent.contains(&(rt::NET_INPUT.to_owned(), vec![1])));
    // The keys were taken: next tick without new input, no push.
    g.player.as_mut().unwrap().velocity.y = -0.08;
    g.native_tick();
    assert_eq!(g.player.as_ref().unwrap().velocity.y, -0.08);
    // Forward facing west (yaw 90): pushed toward -x.
    g.yaw = 90.0;
    g.native_movement_input(&mut 1.0, &mut 0.0, &mut true, &mut false);
    g.native_tick();
    assert!(g.player.as_ref().unwrap().velocity.x < -0.09);
}

#[test]
fn client_engine_off_or_empty_does_nothing_and_server_game_never_pushes() {
    let mut g = game(true);
    g.armor[1] = Some(jetpack("ironjetpacks:iron_jetpack", 100_000, false, false));
    g.native_movement_input(&mut 0.0, &mut 0.0, &mut true, &mut false);
    g.native_tick();
    assert_eq!(g.player.as_ref().unwrap().velocity.y, 0.0);
    g.armor[1] = Some(jetpack("ironjetpacks:iron_jetpack", 0, true, false));
    g.native_movement_input(&mut 0.0, &mut 0.0, &mut true, &mut false);
    g.native_tick();
    assert_eq!(g.player.as_ref().unwrap().velocity.y, 0.0);
    // Creative player: flies empty.
    g.creative = true;
    g.native_movement_input(&mut 0.0, &mut 0.0, &mut true, &mut false);
    g.native_tick();
    assert!(g.player.as_ref().unwrap().velocity.y > 0.1);
    // A server's game (not the window) never pushes.
    let mut s = game(false);
    s.armor[1] = Some(jetpack("ironjetpacks:iron_jetpack", 100_000, true, false));
    s.native_movement_input(&mut 0.0, &mut 0.0, &mut true, &mut false);
    s.native_tick();
    assert_eq!(s.player.as_ref().unwrap().velocity.y, 0.0);
    // Hover: falls slowly in the air without any key.
    let mut h = game(true);
    h.armor[1] = Some(jetpack("ironjetpacks:iron_jetpack", 100_000, true, true));
    h.player.as_mut().unwrap().velocity.y = -0.1;
    h.native_movement_input(&mut 0.0, &mut 0.0, &mut false, &mut false);
    h.native_tick();
    assert!((h.player.as_ref().unwrap().velocity.y + 0.08).abs() < 1e-6);
}

#[test]
fn server_drains_resets_the_fall_and_the_floating_check() {
    let uuid = 77;
    let mut g = game(false);
    g.acting = Some(player(uuid, 0));
    g.armor[1] = Some(jetpack("ironjetpacks:iron_jetpack", 1_000, true, false));
    g.player.as_mut().unwrap().fall_distance = 5.0;
    g.lan = Some(super::lan::LanHost {
        hub: horizon_client::lan::hub::Hub {
            guests: vec![horizon_client::lan::guest::Guest { conn: 1, uuid, above_ground_ticks: 60, placed: true }],
        },
    });
    // No keys: nothing happens.
    g.native_player_tick();
    assert_eq!(energy(g.armor[1].as_ref().unwrap()), 1_000);
    // Jump held: 120 FE a tick, no fall distance, not floating.
    g.native_net_from_client(uuid, rt::NET_INPUT, &[1]);
    g.native_player_tick();
    assert_eq!(energy(g.armor[1].as_ref().unwrap()), 880);
    assert_eq!(g.player.as_ref().unwrap().fall_distance, 0.0);
    g.native_server_tick();
    assert_eq!(g.lan.as_ref().unwrap().hub.guests[0].above_ground_ticks, 0);
    // Sprinting doubles the iron usage (2.0).
    g.player.as_mut().unwrap().sprinting = true;
    g.native_player_tick();
    assert_eq!(energy(g.armor[1].as_ref().unwrap()), 640);
    g.player.as_mut().unwrap().sprinting = false;
    // Down to empty: the last tick takes what is left and the fall is not reset.
    for _ in 0..5 {
        g.native_player_tick();
    }
    assert_eq!(energy(g.armor[1].as_ref().unwrap()), 40);
    g.player.as_mut().unwrap().fall_distance = 3.0;
    g.native_player_tick();
    assert_eq!(energy(g.armor[1].as_ref().unwrap()), 0);
    assert_eq!(get(g.armor[1].as_ref().unwrap(), rt::ENERGY_KEY), None);
    assert_eq!(g.player.as_ref().unwrap().fall_distance, 3.0);
    // Keys released.
    g.native_net_from_client(uuid, rt::NET_INPUT, &[0]);
    g.armor[1] = Some(jetpack("ironjetpacks:iron_jetpack", 500, true, false));
    g.native_player_tick();
    assert_eq!(energy(g.armor[1].as_ref().unwrap()), 500);
    // Creative mode: no drain.
    g.native_net_from_client(uuid, rt::NET_INPUT, &[1]);
    g.acting = Some(player(uuid, 1));
    g.native_player_tick();
    assert_eq!(energy(g.armor[1].as_ref().unwrap()), 500);
    g.native_net_from_client(uuid, rt::NET_INPUT, &[0]);
}

#[test]
fn server_toggles_and_tells_the_action_bar() {
    let uuid = 78;
    let mut g = game(false);
    g.acting = Some(player(uuid, 0));
    g.armor[1] = Some(jetpack("ironjetpacks:gold_jetpack", 0, false, false));
    g.native_net_from_client(uuid, rt::NET_TOGGLE, &[rt::Toggle::Engine.code()]);
    g.native_net_from_client(uuid, rt::NET_TOGGLE, &[rt::Toggle::Hover.code()]);
    g.native_net_from_client(uuid, rt::NET_TOGGLE, &[rt::Toggle::ThrottleDown.code()]);
    g.native_net_from_client(uuid, rt::NET_TOGGLE, &[99]);
    g.native_player_tick();
    let s = g.armor[1].as_ref().unwrap();
    assert_eq!(get(s, rt::ENGINE_KEY), Some(Tag::Byte(1)));
    assert_eq!(get(s, rt::HOVER_KEY), Some(Tag::Byte(1)));
    assert_eq!(get(s, rt::THROTTLE_KEY), Some(Tag::Int(80)));
    assert_eq!(g.bar, vec!["Motor: LIGADO", "Modo pairar: LIGADO", "Potência definida para 80%"]);
    // Off again: the keys go away.
    for t in [rt::Toggle::Engine, rt::Toggle::Hover, rt::Toggle::ThrottleUp] {
        g.native_net_from_client(uuid, rt::NET_TOGGLE, &[t.code()]);
    }
    g.native_player_tick();
    assert_eq!(g.armor[1].as_ref().unwrap().extra, None);
    // Not wearing a jetpack: the toggles are dropped.
    g.armor[1] = Some(ItemStack::new("minecraft:iron_chestplate", 1));
    g.native_net_from_client(uuid, rt::NET_TOGGLE, &[1]);
    g.native_player_tick();
    assert_eq!(g.armor[1].as_ref().unwrap().extra, None);
}

#[test]
fn client_keys_send_toggles_only_with_a_jetpack() {
    let mut g = game(true);
    g.native_key("key.keyboard.v", true);
    assert!(g.sent.is_empty());
    g.armor[1] = Some(jetpack("ironjetpacks:wood_jetpack", 0, false, false));
    g.native_key("key.keyboard.v", true);
    g.native_key("key.keyboard.v", false);
    g.native_key("key.keyboard.h", true);
    g.native_key("key.keyboard.period", true);
    g.native_key("key.keyboard.comma", true);
    g.native_key("key.keyboard.x", true);
    let toggles: Vec<u8> = g.sent.iter().filter(|(c, _)| c == rt::NET_TOGGLE).map(|(_, d)| d[0]).collect();
    assert_eq!(toggles, vec![1, 2, 3, 4]);
}

fn cube(name: &str, energy: Tag) -> ModBlockEntity {
    let mut nbt = Compound::new();
    nbt.insert(JavaString::from("energy"), energy);
    ModBlockEntity::new(ModBlockEntityFields { id: name.into(), nbt, items: vec![], data: vec![] })
}

#[test]
fn right_click_charges_from_a_cube_or_a_cell() {
    let mut g = game(false);
    g.acting = Some(player(79, 0));
    g.blocks.push(([0, 64, 0], "mekanism:basic_energy_cube".into(), cube("mekanism:basic_energy_cube", Tag::Long(4_000_000))));
    g.blocks.push(([1, 64, 0], "powah:energy_cell_basic".into(), cube("powah:energy_cell_basic", Tag::Long(50_000))));
    g.blocks.push(([2, 64, 0], "minecraft:chest".into(), cube("minecraft:chest", Tag::Long(9))));
    g.inventory[0] = Some(jetpack("ironjetpacks:iron_jetpack", 0, true, false));
    let mut canceled = false;
    g.native_right_click_block([0, 64, 0], [0, 1, 0], &mut canceled);
    assert!(canceled);
    assert_eq!(energy(g.inventory[0].as_ref().unwrap()), 120_000);
    assert_eq!(g.blocks[0].2.nbt.get(&JavaString::from("energy")), Some(&Tag::Long(3_700_000)));
    assert_eq!(get(g.inventory[0].as_ref().unwrap(), rt::ENGINE_KEY), Some(Tag::Byte(1)));
    assert_eq!(g.bar.last().unwrap(), "Jetpack de Ferro: +120.000 FE (100%)");
    // Full: canceled (the cube's menu stays shut) but nothing moves.
    let mut canceled = false;
    g.native_right_click_block([0, 64, 0], [0, 1, 0], &mut canceled);
    assert!(canceled);
    assert_eq!(g.blocks[0].2.nbt.get(&JavaString::from("energy")), Some(&Tag::Long(3_700_000)));
    // A Powah cell, FE for FE.
    g.inventory[0] = Some(jetpack("ironjetpacks:iron_jetpack", 100_000, false, false));
    let mut canceled = false;
    g.native_right_click_block([1, 64, 0], [0, 1, 0], &mut canceled);
    assert_eq!(energy(g.inventory[0].as_ref().unwrap()), 120_000);
    assert_eq!(g.blocks[1].2.nbt.get(&JavaString::from("energy")), Some(&Tag::Long(30_000)));
    // Another block, or no jetpack, or the off hand: left alone.
    let mut canceled = false;
    g.native_right_click_block([2, 64, 0], [0, 1, 0], &mut canceled);
    assert!(!canceled);
    g.inventory[0] = Some(ItemStack::new("minecraft:stick", 1));
    g.native_right_click_block([0, 64, 0], [0, 1, 0], &mut canceled);
    assert!(!canceled);
    g.inventory[0] = Some(jetpack("ironjetpacks:iron_jetpack", 0, false, false));
    g.hand = Hand::Off;
    g.native_right_click_block([0, 64, 0], [0, 1, 0], &mut canceled);
    assert!(!canceled);
}

#[test]
fn tooltip_bar_hud_and_list() {
    let g = game(true);
    let s = jetpack("ironjetpacks:iron_jetpack", 60_000, true, false);
    let mut lines = vec![(horizon_client::text::Text::literal("Jetpack de Ferro"), u32::MAX)];
    g.native_tooltip(&s, &mut lines);
    assert_eq!(lines[1].0.0, "§760.000 / 120.000 FE");
    assert_eq!(lines.len(), 6);
    let mut bar = Some((13, 0));
    g.native_item_bar(&s, &mut bar);
    assert_eq!(bar.unwrap().0, 7);
    let mut bar = Some((13, 0));
    g.native_item_bar(&ItemStack::new("minecraft:diamond_sword", 1), &mut bar);
    assert_eq!(bar, Some((13, 0)));
    g.native_item_bar(&jetpack("ironjetpacks:creative_jetpack", 0, false, false), &mut bar);
    assert_eq!(bar, None);
    let mut g = game(true);
    let mut batch = horizon_client::hud::HudBatch::default();
    let mut corners = horizon_client::extras_hud::Corners { width: 400.0, height: 300.0 };
    g.native_hud(&mut batch, &mut corners);
    assert!(batch.texts.is_empty());
    g.armor[1] = Some(s);
    g.native_hud(&mut batch, &mut corners);
    assert_eq!(batch.fills.len(), 1);
    assert_eq!(batch.texts[1].0, "Energia: 50% (60k FE)");
    let mut mods = Vec::new();
    g.native_mods(&mut mods);
    assert_eq!(mods[0].id, "ironjetpacks_native");
    assert_eq!(mods[0].keys.len(), 4);
    assert_eq!(mods[0].keys[0], ["engine".to_owned(), "Ligar/desligar motor".to_owned(), "key.keyboard.v".to_owned()]);
}

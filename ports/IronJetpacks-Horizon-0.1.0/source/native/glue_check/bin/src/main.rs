//! Type-checks the glue and the hook helpers against stubs copying the
//! exact signatures of Horizon 66f3981 they use. Never runs anything of Horizon.
#![allow(dead_code, unused)]
mod game {
    use horizon_client::items::ItemStack;
    pub mod lan {
        pub struct LanHost { pub hub: horizon_client::lan::hub::Hub }
    }
    pub struct Game<'a> {
        pub(crate) _p: std::marker::PhantomData<&'a ()>,
        player: Option<horizon_client::motion::PlayerMotion>,
        armor: [Option<ItemStack>; 4],
        inventory: Vec<Option<ItemStack>>,
        selected: usize,
        hand: horizon_client::first_person::Hand,
        creative: bool,
        yaw: f32,
        lan: Option<lan::LanHost>,
        // Stub-only state.
        pub(crate) window: bool,
        pub(crate) acting: Option<horizon_client::mod_api::NativeServerPlayer>,
        pub(crate) sent: Vec<(String, Vec<u8>)>,
        pub(crate) bar: Vec<String>,
        pub(crate) blocks: Vec<([i32; 3], String, horizon_client::containers::ModBlockEntity)>,
    }
    pub mod native_hooks {
        use super::Game;
        use horizon_client::items::ItemStack;
        use horizon_client::text::Text;
        use horizon_client::hud::HudBatch;
        use horizon_client::extras::Corner;
        use horizon_client::extras_hud::Corners;
        use horizon_client::containers::ModBlockEntity;
        pub(crate) use horizon_client::mod_api::NativeMod;
        pub struct NativePlayer { pub x: f64, pub y: f64, pub z: f64, pub yaw: f32, pub pitch: f32, pub dimension: &'static str }
        impl Game<'_> {
            pub(super) fn native_on(&self, mod_id: &str) -> bool { true }
            pub(super) fn native_playing(&self) -> bool { true }
            pub(super) fn native_window(&self) -> bool { self.window }
            pub(super) fn native_mod_key(&self, mod_id: &str, name: &str, default: &str) -> String { default.to_owned() }
            pub(super) fn native_player(&self) -> Option<NativePlayer> {
                let p = self.player.as_ref()?;
                Some(NativePlayer { x: 0.0, y: 0.0, z: 0.0, yaw: self.yaw, pitch: 0.0, dimension: "minecraft:overworld" })
            }
            pub(super) fn native_corner_shown(&self, corner: Corner) -> bool { true }
            pub(super) fn native_text_width(&self, text: &str) -> f32 { 6.0 * text.chars().count() as f32 }
            pub(super) fn native_fill(&self, batch: &mut HudBatch, rect: [f32; 4], argb: u32) { batch.fills.push((rect, argb)); }
            pub(super) fn native_text(&self, batch: &mut HudBatch, text: &str, at: [f32; 2], rgb: u32, shadow: bool) { batch.texts.push((text.to_owned(), at, rgb)); }
            pub(super) fn native_block(&self, pos: [i32; 3]) -> String {
                self.blocks.iter().find(|(p, _, _)| *p == pos).map_or("minecraft:air".to_owned(), |(_, n, _)| n.clone())
            }
            pub(super) fn native_with_mod_block_entity<R>(&mut self, pos: [i32; 3], change: impl FnOnce(&mut ModBlockEntity) -> R) -> Option<R> {
                let (_, _, m) = self.blocks.iter_mut().find(|(p, _, _)| *p == pos)?;
                Some(change(m))
            }
            pub(super) fn native_tick(&mut self) {
                // Each tick of the window's game: the worn jetpack pushes the player.
fn ironjetpacks_tick(game: &mut crate::game::Game<'_>) {
    crate::game::ironjetpacks_glue::tick(game);
}

                ironjetpacks_tick(&mut *self);
            }
            pub(super) fn native_movement_input(&self, forward: &mut f32, strafe: &mut f32, jump: &mut bool, sneak: &mut bool) {
                // The window's movement keys of this tick, for the jetpack's next push.
fn ironjetpacks_movement_input(forward: f32, strafe: f32, jump: bool, sneak: bool) {
    crate::game::ironjetpacks_glue::movement_input(forward, strafe, jump, sneak);
}

                ironjetpacks_movement_input(*forward, *strafe, *jump, *sneak);
            }
            pub(super) fn native_key(&mut self, key: &str, pressed: bool) {
                // The engine, hover and throttle keys (V, H, period, comma by default).
fn ironjetpacks_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::ironjetpacks_glue::key(game, key, pressed);
}

                ironjetpacks_key(&mut *self, key, pressed);
            }
            pub(super) fn native_hud(&mut self, batch: &mut HudBatch, corners: &mut Corners) {
                // The worn jetpack's energy and state on the HUD.
fn ironjetpacks_hud(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    crate::game::ironjetpacks_glue::hud(game, batch, corners);
}

                ironjetpacks_hud(&*self, &mut *batch, &mut *corners);
            }
            pub(super) fn native_tooltip(&self, stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
                // A jetpack's energy and state in its tooltip.
fn ironjetpacks_tooltip(stack: &horizon_client::items::ItemStack, lines: &mut Vec<(horizon_client::text::Text, u32)>) {
    crate::game::ironjetpacks_glue::tooltip(stack, lines);
}

                ironjetpacks_tooltip(stack, &mut *lines);
            }
            pub(super) fn native_item_bar(&self, stack: &ItemStack, bar: &mut Option<(i32, u32)>) {
                // A jetpack's energy bar under its icon.
fn ironjetpacks_item_bar(stack: &horizon_client::items::ItemStack, bar: &mut Option<(i32, u32)>) {
    crate::game::ironjetpacks_glue::item_bar(stack, bar);
}

                ironjetpacks_item_bar(stack, &mut *bar);
            }
            pub(super) fn native_net_from_client(&mut self, player: u128, channel: &str, data: &[u8]) {
                // A player's jetpack keys and toggles, from their client.
fn ironjetpacks_net_from_client(game: &mut crate::game::Game<'_>, player: u128, channel: &str, data: &[u8]) {
    crate::game::ironjetpacks_glue::net_from_client(game, player, channel, data);
}

                ironjetpacks_net_from_client(&mut *self, player, channel, data);
            }
            pub(super) fn native_player_tick(&mut self) {
                // The end of a player's server tick: the worn jetpack's toggles and energy.
fn ironjetpacks_player_tick(game: &mut crate::game::Game<'_>) {
    crate::game::ironjetpacks_glue::player_tick(game);
}

                ironjetpacks_player_tick(&mut *self);
            }
            pub(super) fn native_server_tick(&mut self) {
                // The server's tick: players flying a jetpack are not floating.
fn ironjetpacks_server_tick(game: &mut crate::game::Game<'_>) {
    crate::game::ironjetpacks_glue::server_tick(game);
}

                ironjetpacks_server_tick(&mut *self);
            }
            pub(super) fn native_right_click_block(&mut self, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
                // A right click with a jetpack on an energy cube or cell charges it.
fn ironjetpacks_right_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    crate::game::ironjetpacks_glue::right_click(game, pos, canceled);
}

                ironjetpacks_right_click(&mut *self, pos, &mut *canceled);
            }
            pub(super) fn native_mods(&mut self, mods: &mut Vec<NativeMod>) {
                // The mod's row in Options > Mods, with its keys.
fn ironjetpacks_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::ironjetpacks_rt as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: m::KEYS.iter().map(|(n, l, d)| [(*n).to_owned(), (*l).to_owned(), (*d).to_owned()]).collect(),
    });
}

                ironjetpacks_listed(&mut *mods);
            }
        }
    }
    mod native_motion {
        use super::Game;
        impl Game<'_> {
            pub(super) fn native_push_player(&mut self, kick: [f32; 3]) {
                if let Some(p) = self.player.as_mut() && !p.spectator {
                    p.velocity.x += f64::from(kick[0].clamp(-4.0, 4.0));
                    p.velocity.y += f64::from(kick[1].clamp(-4.0, 4.0));
                    p.velocity.z += f64::from(kick[2].clamp(-4.0, 4.0));
                    if kick[1] > 0.0 { p.ground.on_ground = false; }
                }
            }
        }
    }
    mod native_server {
        use super::Game;
        use horizon_client::text::Text;
        pub(crate) use horizon_client::mod_api::NativeServerPlayer;
        impl Game<'_> {
            pub(super) fn native_acting_player(&self) -> Option<NativeServerPlayer> { self.acting.clone() }
            pub(super) fn native_action_bar(&mut self, uuid: u128, line: Text) -> bool { self.bar.push(line.0); true }
        }
    }
    mod native_net {
        use super::Game;
        impl Game<'_> {
            pub(super) fn native_net_send_to_server(&mut self, channel: &str, data: &[u8]) -> Result<(), String> {
                self.sent.push((channel.to_owned(), data.to_vec()));
                Ok(())
            }
        }
    }
    #[cfg(test)]
    mod behaviour;
    #[path = "/home/claude/ports/IronJetpacks-Horizon-0.1.0/source/native/mixins/ironjetpacks_native/runtime.rs"]
    pub(crate) mod ironjetpacks_rt;
    #[path = "/home/claude/ports/IronJetpacks-Horizon-0.1.0/source/native/mixins/ironjetpacks_native/glue.rs"]
    pub(crate) mod ironjetpacks_glue;
}
fn main() {}

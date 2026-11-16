// The window's movement keys of this tick, for the jetpack's next push.
fn ironjetpacks_movement_input(forward: f32, strafe: f32, jump: bool, sneak: bool) {
    crate::game::ironjetpacks_glue::movement_input(forward, strafe, jump, sneak);
}

// Draws a Powah block's screen: energy, the orb's progress, fuel and lava.
fn powah_screen_draw(
    game: &crate::game::Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut horizon_client::hud::HudBatch,
    origin: [f32; 2],
) {
    crate::game::powah_glue::screen_draw(game, block, data, batch, origin);
}

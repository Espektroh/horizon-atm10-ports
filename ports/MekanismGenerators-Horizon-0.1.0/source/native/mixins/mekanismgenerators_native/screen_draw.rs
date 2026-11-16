// Draws a generator's screen: energy, generation, lava, sun, wind.
fn mekanismgenerators_screen_draw(
    game: &crate::game::Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut horizon_client::hud::HudBatch,
    origin: [f32; 2],
) {
    crate::game::mekanismgenerators_glue::screen_draw(game, block, data, batch, origin);
}

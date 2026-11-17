// Draws a machine's screen: energy, progress, usage.
fn mekmm_screen_draw(
    game: &crate::game::Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut horizon_client::hud::HudBatch,
    origin: [f32; 2],
) {
    crate::game::mekmm_glue::screen_draw(game, block, data, batch, origin);
}

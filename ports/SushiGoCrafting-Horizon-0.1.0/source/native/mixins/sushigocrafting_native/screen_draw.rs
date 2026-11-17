// Draws the tank, the work arrow and the cooker's fire.
fn sushigocrafting_screen_draw(
    game: &crate::game::Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut horizon_client::hud::HudBatch,
    origin: [f32; 2],
) {
    crate::game::sushigocrafting_glue::screen_draw(game, block, data, batch, origin);
}

// The worn jetpack's energy and state on the HUD.
fn ironjetpacks_hud(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    crate::game::ironjetpacks_glue::hud(game, batch, corners);
}

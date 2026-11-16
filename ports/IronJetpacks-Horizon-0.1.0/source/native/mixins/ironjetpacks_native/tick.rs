// Each tick of the window's game: the worn jetpack pushes the player.
fn ironjetpacks_tick(game: &mut crate::game::Game<'_>) {
    crate::game::ironjetpacks_glue::tick(game);
}

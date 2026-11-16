// The end of a player's server tick: the worn jetpack's toggles and energy.
fn ironjetpacks_player_tick(game: &mut crate::game::Game<'_>) {
    crate::game::ironjetpacks_glue::player_tick(game);
}

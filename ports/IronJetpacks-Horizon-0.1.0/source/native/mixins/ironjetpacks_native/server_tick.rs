// The server's tick: players flying a jetpack are not floating.
fn ironjetpacks_server_tick(game: &mut crate::game::Game<'_>) {
    crate::game::ironjetpacks_glue::server_tick(game);
}

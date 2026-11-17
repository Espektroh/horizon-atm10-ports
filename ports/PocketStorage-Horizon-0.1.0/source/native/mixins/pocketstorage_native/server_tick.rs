// Opens the PSUs players asked for, outside any player's swapped-in state.
fn pocketstorage_server_tick(game: &mut crate::game::Game<'_>) {
    crate::game::pocketstorage_glue::server_tick(game);
}

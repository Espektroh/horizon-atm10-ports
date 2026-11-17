// A player's `/psu` command: sets or clears their search.
fn pocketstorage_perform_command(
    game: &mut crate::game::Game<'_>,
    player: &horizon_client::mod_api::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    crate::game::pocketstorage_glue::perform_command(game, player, command, ran);
}

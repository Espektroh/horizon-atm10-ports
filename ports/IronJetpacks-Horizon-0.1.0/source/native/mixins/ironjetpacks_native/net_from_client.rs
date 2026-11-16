// A player's jetpack keys and toggles, from their client.
fn ironjetpacks_net_from_client(game: &mut crate::game::Game<'_>, player: u128, channel: &str, data: &[u8]) {
    crate::game::ironjetpacks_glue::net_from_client(game, player, channel, data);
}

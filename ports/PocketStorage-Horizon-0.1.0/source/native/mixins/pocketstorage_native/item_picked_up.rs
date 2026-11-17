// A player touches a dropped item: a PSU that already holds it takes it all
// (API 40; `canceled` leaves nothing for the vanilla pickup).
fn pocketstorage_item_picked_up(
    game: &mut crate::game::Game<'_>,
    player: u128,
    item: &str,
    count: u32,
    canceled: &mut bool,
) {
    crate::game::pocketstorage_glue::item_picked_up(game, player, item, count, canceled);
}

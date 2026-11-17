// A right click with a PSU opens it (on the next server tick).
fn pocketstorage_use_item(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    claimed: &mut bool,
) {
    crate::game::pocketstorage_glue::use_item(game, stack, claimed);
}

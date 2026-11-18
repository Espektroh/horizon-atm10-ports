// A Model Framework used on a mob becomes its Data Model.
fn hostilenetworks_interact_entity(
    game: &mut crate::game::Game<'_>,
    kind: &str,
    stack: Option<&horizon_client::items::ItemStack>,
    claimed: &mut bool,
) {
    crate::game::hostilenetworks_glue::interact_entity(game, kind, stack, claimed);
}

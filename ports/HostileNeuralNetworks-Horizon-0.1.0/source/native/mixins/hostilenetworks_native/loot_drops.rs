// A mob killed by a player trains the Data Models in their Deep Learners.
fn hostilenetworks_loot_drops(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLootDrops<'_>,
    drops: &mut Vec<horizon_client::items::ItemStack>,
) {
    crate::game::hostilenetworks_glue::loot_drops(game, context, drops);
}

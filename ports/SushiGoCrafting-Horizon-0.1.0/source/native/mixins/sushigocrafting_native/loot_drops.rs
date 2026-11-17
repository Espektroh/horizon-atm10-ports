// Seeds from grass, portions of the amount items, tobiko from fish.
fn sushigocrafting_loot_drops(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLootDrops<'_>,
    drops: &mut Vec<horizon_client::items::ItemStack>,
) {
    crate::game::sushigocrafting_glue::loot_drops(game, context, drops);
}

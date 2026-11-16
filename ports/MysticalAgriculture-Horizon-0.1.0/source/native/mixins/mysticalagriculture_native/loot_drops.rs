// Crop harvests by farmland tier, Inferium Essence from mobs, Agradditions' boss drops.
fn mysticalagriculture_loot_drops(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLootDrops<'_>,
    drops: &mut Vec<horizon_client::items::ItemStack>,
) {
    crate::game::mysticalagriculture_glue::loot_drops(game, context, drops);
}

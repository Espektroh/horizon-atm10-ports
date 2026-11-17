// Bone meal on rice, the avocado sapling and avocado leaves.
fn sushigocrafting_use_item_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    crate::game::sushigocrafting_glue::use_item_on(game, stack, pos, claimed);
}

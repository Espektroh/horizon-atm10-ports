// The bottle on a block: start or double the accelerator there.
fn tiab_use_item_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    crate::game::tiab_glue::use_item_on(game, stack, pos, claimed);
}

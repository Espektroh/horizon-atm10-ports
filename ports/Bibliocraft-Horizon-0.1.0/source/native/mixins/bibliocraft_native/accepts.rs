// What a hopper may put in a bookcase, a shelf or the sword pedestal.
fn bibliocraft_accepts(
    game: &crate::game::Game<'_>,
    entity: &horizon_client::containers::ModBlockEntity,
    slot: usize,
    stack: &horizon_client::items::ItemStack,
    cancel: &mut bool,
) {
    crate::game::bibliocraft_glue::accepts(game, entity, slot, stack, cancel);
}

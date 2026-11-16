// What a hopper may put in a drawer.
fn functionalstorage_accepts(
    game: &crate::game::Game<'_>,
    entity: &horizon_client::containers::ModBlockEntity,
    slot: usize,
    stack: &horizon_client::items::ItemStack,
    cancel: &mut bool,
) {
    crate::game::functionalstorage_glue::accepts(game, entity, slot, stack, cancel);
}

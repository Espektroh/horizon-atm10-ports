// Shift-click into the stamper and the presser: each item to its slot.
fn mekmm_quick_move(
    game: &mut crate::game::Game<'_>,
    entity: &mut horizon_client::containers::ModBlockEntity,
    container_slots: usize,
    slot: usize,
    stack: &mut horizon_client::items::ItemStack,
    handled: &mut bool,
) {
    crate::game::mekmm_glue::quick_move(game, entity, container_slots, slot, stack, handled);
}

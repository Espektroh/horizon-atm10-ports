// Shift-click into a Mekanism machine: osmium to the compressor's tank slot,
// infuse material to the infuser's side slot, the recipe's item to the input.
fn mekanism_quick_move(
    game: &mut crate::game::Game<'_>,
    entity: &mut horizon_client::containers::ModBlockEntity,
    container_slots: usize,
    slot: usize,
    stack: &mut horizon_client::items::ItemStack,
    handled: &mut bool,
) {
    crate::game::mekanism_glue::quick_move(game, entity, container_slots, slot, stack, handled);
}

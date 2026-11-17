// Shift-click into the cooker (rice, fuel) and the barrel (what a recipe takes).
fn sushigocrafting_quick_move(
    game: &mut crate::game::Game<'_>,
    entity: &mut horizon_client::containers::ModBlockEntity,
    container_slots: usize,
    slot: usize,
    stack: &mut horizon_client::items::ItemStack,
    handled: &mut bool,
) {
    crate::game::sushigocrafting_glue::quick_move(
        game,
        entity,
        container_slots,
        slot,
        stack,
        handled,
    );
}

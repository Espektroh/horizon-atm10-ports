// What a hopper may put in a machine.
fn hostilenetworks_accepts(
    entity: &horizon_client::containers::ModBlockEntity,
    slot: usize,
    stack: &horizon_client::items::ItemStack,
    cancel: &mut bool,
) {
    crate::game::hostilenetworks_glue::accepts(entity, slot, stack, cancel);
}

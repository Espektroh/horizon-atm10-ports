// Mega torch, dread lamp and feral flare lantern get a block entity.
fn torchmaster_block_entity(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    crate::game::torchmaster_glue::block_entity(block, entity);
}

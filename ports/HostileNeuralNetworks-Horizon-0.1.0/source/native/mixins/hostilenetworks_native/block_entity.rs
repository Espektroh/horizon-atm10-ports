// The Simulation Chamber and the Loot Fabricator get a block entity.
fn hostilenetworks_block_entity(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    crate::game::hostilenetworks_glue::block_entity(block, entity);
}

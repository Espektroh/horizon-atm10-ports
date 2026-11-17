// A furniture block placed: it gets its block entity.
fn bibliocraft_block_entity(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::bibliocraft_glue::block_entity(block, entity);
}

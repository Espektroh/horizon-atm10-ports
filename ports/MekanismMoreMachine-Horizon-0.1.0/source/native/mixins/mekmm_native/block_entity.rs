// A MoreMachine machine placed: it gets its block entity.
fn mekmm_block_entity(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::mekmm_glue::block_entity(block, entity);
}

// A Powah block placed: it gets its block entity.
fn powah_block_entity(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::powah_glue::block_entity(block, entity);
}

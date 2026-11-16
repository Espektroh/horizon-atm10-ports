// A drawer placed: it gets its block entity.
fn functionalstorage_block_entity(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::functionalstorage_glue::block_entity(block, entity);
}

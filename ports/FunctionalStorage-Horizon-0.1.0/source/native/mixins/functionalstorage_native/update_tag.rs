// What clients get of a drawer: each slot's item and count.
fn functionalstorage_update_tag(
    entity: &horizon_client::containers::ModBlockEntity,
    tag: &mut Option<horizon_world::nbt::Compound>,
) {
    crate::game::functionalstorage_glue::update_tag(entity, tag);
}

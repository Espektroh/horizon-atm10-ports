// What clients get of a furniture block: each slot's item.
fn bibliocraft_update_tag(
    entity: &horizon_client::containers::ModBlockEntity,
    tag: &mut Option<horizon_world::nbt::Compound>,
) {
    crate::game::bibliocraft_glue::update_tag(entity, tag);
}

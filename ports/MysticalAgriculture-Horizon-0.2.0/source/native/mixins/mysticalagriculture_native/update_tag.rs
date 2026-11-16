// 0.2.0: what clients get of an altar or a pedestal: the items it holds, to draw them.
fn mysticalagriculture_update_tag(
    entity: &horizon_client::containers::ModBlockEntity,
    tag: &mut Option<horizon_world::nbt::Compound>,
) {
    crate::game::mysticalagriculture_glue::update_tag(entity, tag);
}

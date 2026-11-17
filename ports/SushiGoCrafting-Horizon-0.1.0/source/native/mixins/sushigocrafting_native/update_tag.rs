// What clients get of a cutting board: its item.
fn sushigocrafting_update_tag(
    entity: &horizon_client::containers::ModBlockEntity,
    tag: &mut Option<horizon_world::nbt::Compound>,
) {
    crate::game::sushigocrafting_glue::update_tag(entity, tag);
}

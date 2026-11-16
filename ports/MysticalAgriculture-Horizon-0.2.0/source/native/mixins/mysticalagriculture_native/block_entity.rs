// 0.2.0: the Infusion Altar and the Infusion Pedestal get their block entity.
fn mysticalagriculture_block_entity(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    crate::game::mysticalagriculture_glue::block_entity(block, entity);
}

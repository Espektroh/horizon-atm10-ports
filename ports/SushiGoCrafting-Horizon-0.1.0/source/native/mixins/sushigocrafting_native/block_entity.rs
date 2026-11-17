// The cutting board, the rice cooker and the fermentation barrel get their block entities.
fn sushigocrafting_block_entity(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    crate::game::sushigocrafting_glue::block_entity(block, entity);
}

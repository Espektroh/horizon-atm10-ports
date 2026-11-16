// 0.2.0: draws the item floating over an altar or a pedestal.
fn mysticalagriculture_submit(
    game: &crate::game::Game<'_>,
    pos: [i32; 3],
    block: &str,
    entity: &horizon_client::containers::ModBlockEntity,
    draw: &mut crate::game::native_block_entity_draw::BlockEntityDraw,
) {
    crate::game::mysticalagriculture_glue::submit(game, pos, block, entity, draw);
}

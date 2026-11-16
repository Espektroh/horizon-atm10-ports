// Draws a drawer's items and counts on its front.
fn functionalstorage_submit(
    game: &crate::game::Game<'_>,
    pos: [i32; 3],
    block: &str,
    state: horizon_world::registry::BlockStateId,
    entity: &horizon_client::containers::ModBlockEntity,
    draw: &mut crate::game::native_block_entity_draw::BlockEntityDraw,
) {
    crate::game::functionalstorage_glue::submit(game, pos, block, state, entity, draw);
}

// One server tick of one of them.
fn torchmaster_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::torchmaster_glue::tick(game, pos, entity);
}

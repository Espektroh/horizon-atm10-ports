// One server tick of a furniture block entity.
fn bibliocraft_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::bibliocraft_glue::tick(game, pos, entity);
}

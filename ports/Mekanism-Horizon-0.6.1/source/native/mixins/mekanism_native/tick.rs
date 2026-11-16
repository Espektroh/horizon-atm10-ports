// One server tick of a Mekanism block entity.
fn mekanism_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::mekanism_glue::tick(game, pos, entity);
}

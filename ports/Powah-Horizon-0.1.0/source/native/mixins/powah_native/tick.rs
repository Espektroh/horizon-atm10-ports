// One server tick of a Powah block entity.
fn powah_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::powah_glue::tick(game, pos, entity);
}

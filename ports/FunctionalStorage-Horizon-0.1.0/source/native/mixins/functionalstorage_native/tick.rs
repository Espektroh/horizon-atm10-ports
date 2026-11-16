// One server tick of a drawer.
fn functionalstorage_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::functionalstorage_glue::tick(game, pos, entity);
}

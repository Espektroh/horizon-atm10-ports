// One server tick of the rice cooker or the fermentation barrel.
fn sushigocrafting_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::sushigocrafting_glue::tick(game, pos, entity);
}

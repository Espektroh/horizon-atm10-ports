// 0.2.0: one server tick of an Infusion Altar (redstone, recipe, 100 ticks, craft).
fn mysticalagriculture_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::mysticalagriculture_glue::altar_tick(game, pos, entity);
}

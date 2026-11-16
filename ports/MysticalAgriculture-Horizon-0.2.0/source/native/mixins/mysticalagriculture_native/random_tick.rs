// Agradditions' crops grow only over their crux.
fn mysticalagriculture_random_tick(
    level: &crate::game::GameLevel<'_>,
    pos: horizon_core::position::BlockPos,
    state: horizon_world::registry::BlockStateId,
    handled: &mut bool,
) {
    crate::game::mysticalagriculture_glue::random_tick(level, pos, state, handled);
}

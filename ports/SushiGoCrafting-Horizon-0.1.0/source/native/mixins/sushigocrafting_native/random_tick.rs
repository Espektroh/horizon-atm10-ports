// The crops grow (rice in water), the sapling grows an avocado tree, avocado leaves ripen.
fn sushigocrafting_random_tick(
    level: &mut crate::game::GameLevel<'_>,
    ticks: &mut horizon_world::level::Ticks,
    pos: horizon_core::position::BlockPos,
    state: horizon_world::registry::BlockStateId,
    handled: &mut bool,
) {
    crate::game::sushigocrafting_glue::random_tick(level, ticks, pos, state, handled);
}

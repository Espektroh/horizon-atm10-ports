// A neighbour of a Mekanism cable changed: the cable takes the matching variant.
fn mekanism_cable_shape(
    level: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    direction: horizon_core::geometry::Direction,
    neighbour: horizon_world::registry::BlockStateId,
    new: &mut Option<horizon_world::registry::BlockStateId>,
) {
    crate::game::mekanism_glue::cable_shape(level, state, direction, neighbour, new);
}

// Bone meal doesn't grow the mod's crops.
fn mysticalagriculture_bone_meal(
    level: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    valid: &mut Option<bool>,
) {
    crate::game::mysticalagriculture_glue::bonemeal_target(level, state, valid);
}

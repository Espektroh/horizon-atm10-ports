// Bone meal not from a player's click (a dispenser) doesn't turn rice into tall seagrass or the
// avocado sapling into an oak.
fn sushigocrafting_bone_meal(
    level: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    valid: &mut Option<bool>,
) {
    crate::game::sushigocrafting_glue::bonemeal_target(level, state, valid);
}

// Rice stands on dirt only, as the original's WaterCropBlock.
fn sushigocrafting_may_place_on(
    level: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    ground: horizon_world::registry::BlockStateId,
    may: &mut Option<bool>,
) {
    crate::game::sushigocrafting_glue::may_place_on(level, state, ground, may);
}

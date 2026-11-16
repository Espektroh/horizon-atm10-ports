// The level's clock: lights whose block entity stopped ticking are forgotten.
fn torchmaster_level_tick(dimension: &str) {
    crate::game::torchmaster_glue::level_tick(dimension);
}

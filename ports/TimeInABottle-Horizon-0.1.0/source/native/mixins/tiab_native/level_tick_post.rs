// The accelerators of this level give their blocks the extra ticks.
fn tiab_level_tick_post(game: &mut crate::game::Game<'_>, dimension: &str) {
    crate::game::tiab_glue::level_tick_post(game, dimension);
}

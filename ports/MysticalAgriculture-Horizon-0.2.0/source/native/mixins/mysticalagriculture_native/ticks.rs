// 0.2.0: the altar's block entity ticks.
fn mysticalagriculture_ticks(block: &str, ticks: &mut bool) {
    crate::game::mysticalagriculture_glue::ticks(block, ticks);
}

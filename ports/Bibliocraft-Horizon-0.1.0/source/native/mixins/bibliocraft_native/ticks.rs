// The furniture's block entities tick (to tell clients what changed).
fn bibliocraft_ticks(block: &str, ticks: &mut bool) {
    crate::game::bibliocraft_glue::ticks(block, ticks);
}

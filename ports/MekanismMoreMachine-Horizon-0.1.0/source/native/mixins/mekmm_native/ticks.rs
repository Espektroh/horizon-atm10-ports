// Every MoreMachine block entity ticks.
fn mekmm_ticks(block: &str, ticks: &mut bool) {
    crate::game::mekmm_glue::ticks(block, ticks);
}

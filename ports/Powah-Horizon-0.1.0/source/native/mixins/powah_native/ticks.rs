// Every Powah block entity ticks.
fn powah_ticks(block: &str, ticks: &mut bool) {
    crate::game::powah_glue::ticks(block, ticks);
}

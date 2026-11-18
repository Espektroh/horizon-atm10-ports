// Both machines tick.
fn hostilenetworks_ticks(block: &str, ticks: &mut bool) {
    crate::game::hostilenetworks_glue::ticks(block, ticks);
}

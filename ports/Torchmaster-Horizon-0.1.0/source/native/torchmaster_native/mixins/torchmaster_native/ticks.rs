// Their block entities tick.
fn torchmaster_ticks(block: &str, ticks: &mut bool) {
    crate::game::torchmaster_glue::ticks(block, ticks);
}

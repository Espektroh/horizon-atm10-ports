// Every Mekanism Generators block entity ticks.
fn mekanismgenerators_ticks(block: &str, ticks: &mut bool) {
    crate::game::mekanismgenerators_glue::ticks(block, ticks);
}

// The rice cooker and the fermentation barrel tick.
fn sushigocrafting_ticks(block: &str, ticks: &mut bool) {
    crate::game::sushigocrafting_glue::ticks(block, ticks);
}

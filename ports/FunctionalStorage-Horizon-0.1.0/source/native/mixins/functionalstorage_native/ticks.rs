// Every drawer's block entity ticks.
fn functionalstorage_ticks(block: &str, ticks: &mut bool) {
    crate::game::functionalstorage_glue::ticks(block, ticks);
}

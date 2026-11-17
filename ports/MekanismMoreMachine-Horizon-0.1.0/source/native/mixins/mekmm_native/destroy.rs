// A MoreMachine machine broken gives its upgrades back.
fn mekmm_destroy(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if !*canceled {
        crate::game::mekmm_glue::destroyed(game, pos);
    }
}

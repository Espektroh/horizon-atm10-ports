// A right click on a MoreMachine machine with an upgrade in hand puts it in.
fn mekmm_right_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    crate::game::mekmm_glue::right_click(game, pos, canceled);
}

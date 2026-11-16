// 0.2.0: a right click on an altar or a pedestal puts in or takes out (the wand switches on).
fn mysticalagriculture_right_click(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    canceled: &mut bool,
) {
    crate::game::mysticalagriculture_glue::right_click(game, pos, canceled);
}

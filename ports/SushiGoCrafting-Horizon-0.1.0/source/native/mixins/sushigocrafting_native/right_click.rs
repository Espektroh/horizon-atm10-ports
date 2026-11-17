// The cutting board's clicks, ripe avocado leaves, buckets into the cooker and the barrel.
fn sushigocrafting_right_click(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    canceled: &mut bool,
) {
    crate::game::sushigocrafting_glue::right_click(game, pos, canceled);
}

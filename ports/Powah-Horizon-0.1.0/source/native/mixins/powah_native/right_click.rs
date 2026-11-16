// A right click on the Energizing Orb puts an item in or takes the result;
// a lava bucket on a Magmator fills its tank.
fn powah_right_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    crate::game::powah_glue::right_click(game, pos, canceled);
}

// A right click on a shelf, the sword pedestal or a table: items in and out.
fn bibliocraft_right_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
    crate::game::bibliocraft_glue::right_click(game, pos, face, canceled);
}

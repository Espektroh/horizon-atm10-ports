// A left click on a drawer's front takes from it.
fn functionalstorage_left_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
    crate::game::functionalstorage_glue::left_click(game, pos, face, canceled);
}

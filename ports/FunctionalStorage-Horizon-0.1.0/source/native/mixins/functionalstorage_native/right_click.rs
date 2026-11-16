// A right click on a drawer: insert, insert all, upgrades, contents.
fn functionalstorage_right_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
    crate::game::functionalstorage_glue::right_click(game, pos, face, canceled);
}

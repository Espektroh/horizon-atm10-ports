// A drawer broken by a player: kept while it holds items; upgrades given back.
fn functionalstorage_destroy(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    crate::game::functionalstorage_glue::destroy(game, pos, canceled);
}

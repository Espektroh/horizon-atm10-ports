// A right click with a jetpack on an energy cube or cell charges it.
fn ironjetpacks_right_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    crate::game::ironjetpacks_glue::right_click(game, pos, canceled);
}

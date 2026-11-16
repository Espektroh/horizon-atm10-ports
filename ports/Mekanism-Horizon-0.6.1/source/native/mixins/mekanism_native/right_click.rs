// A right click on a Mekanism machine with an upgrade in hand puts it in.
fn mekanism_right_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    crate::game::mekanism_glue::right_click(game, pos, canceled);
}

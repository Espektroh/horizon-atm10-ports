// A Mekanism machine broken gives its upgrades back.
fn mekanism_destroy(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if !*canceled {
        crate::game::mekanism_glue::destroyed(game, pos);
    }
}

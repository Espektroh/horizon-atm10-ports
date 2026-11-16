// Explosions spare drawers that hold items or upgrades.
fn functionalstorage_explosion(game: &mut crate::game::Game<'_>, dimension: &str, blocks: &mut Vec<[i32; 3]>) {
    crate::game::functionalstorage_glue::explosion(game, dimension, blocks);
}

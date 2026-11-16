// A carried bottle gains time.
fn tiab_inventory_tick(game: &mut crate::game::Game<'_>, slot: i32) {
    crate::game::tiab_glue::inventory_tick(game, slot);
}

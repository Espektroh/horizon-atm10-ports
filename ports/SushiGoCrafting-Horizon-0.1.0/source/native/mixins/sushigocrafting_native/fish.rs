// Raw tuna and shrimp from a fishing rod's catch.
fn sushigocrafting_fish(game: &mut crate::game::Game<'_>, state: i32, cancel: &mut bool) {
    crate::game::sushigocrafting_glue::fish(game, state, cancel);
}

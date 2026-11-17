// A click on an open PSU: the stacks move here, checked, in one step.
fn pocketstorage_menu_clicked(
    game: &mut crate::game::Game<'_>,
    uuid: u128,
    window: i32,
    slot: i32,
    button: i32,
    mode: i32,
) {
    crate::game::pocketstorage_glue::menu_clicked(game, uuid, window, slot, button, mode);
}

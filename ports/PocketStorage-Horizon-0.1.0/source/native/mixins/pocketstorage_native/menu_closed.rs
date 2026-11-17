// A PSU menu closed.
fn pocketstorage_menu_closed(uuid: u128, window: i32) {
    crate::game::pocketstorage_glue::menu_closed(uuid, window);
}

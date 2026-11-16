// The drawers' item slots, for hoppers.
fn functionalstorage_container(block: &str, container: &mut Option<horizon_client::containers::ModContainer>) {
    crate::game::functionalstorage_glue::container(block, container);
}

// The Powah blocks' slots and menus.
fn powah_container(block: &str, container: &mut Option<horizon_client::containers::ModContainer>) {
    crate::game::powah_glue::container(block, container);
}

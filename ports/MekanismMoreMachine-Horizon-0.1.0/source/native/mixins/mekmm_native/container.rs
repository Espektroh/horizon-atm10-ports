// The machines' slots and menus.
fn mekmm_container(block: &str, container: &mut Option<horizon_client::containers::ModContainer>) {
    crate::game::mekmm_glue::container(block, container);
}

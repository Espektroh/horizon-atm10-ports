// The machines' slots and menus.
fn hostilenetworks_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    crate::game::hostilenetworks_glue::container(block, container);
}

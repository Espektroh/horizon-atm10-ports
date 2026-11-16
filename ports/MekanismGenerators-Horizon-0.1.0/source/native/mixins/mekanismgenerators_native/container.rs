// The generators' slots and menus.
fn mekanismgenerators_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    crate::game::mekanismgenerators_glue::container(block, container);
}

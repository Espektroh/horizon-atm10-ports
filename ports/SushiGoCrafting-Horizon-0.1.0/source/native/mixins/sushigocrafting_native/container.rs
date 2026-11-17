// The machines' slots and menus.
fn sushigocrafting_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    crate::game::sushigocrafting_glue::container(block, container);
}

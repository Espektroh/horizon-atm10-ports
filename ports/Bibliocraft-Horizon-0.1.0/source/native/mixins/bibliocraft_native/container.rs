// The furniture's item slots, hopper faces and menu types.
fn bibliocraft_container(block: &str, container: &mut Option<horizon_client::containers::ModContainer>) {
    crate::game::bibliocraft_glue::container(block, container);
}

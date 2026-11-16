// 0.2.0: the altar's input and output, the pedestal's slot (saved, spilled when broken).
fn mysticalagriculture_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    crate::game::mysticalagriculture_glue::container(block, container);
}

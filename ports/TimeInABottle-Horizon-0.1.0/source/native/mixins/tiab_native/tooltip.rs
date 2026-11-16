// The stored and total time under the bottle's name.
fn tiab_tooltip(
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    crate::game::tiab_glue::tooltip(stack, lines);
}

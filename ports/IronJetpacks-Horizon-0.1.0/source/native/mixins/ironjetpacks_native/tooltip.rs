// A jetpack's energy and state in its tooltip.
fn ironjetpacks_tooltip(stack: &horizon_client::items::ItemStack, lines: &mut Vec<(horizon_client::text::Text, u32)>) {
    crate::game::ironjetpacks_glue::tooltip(stack, lines);
}

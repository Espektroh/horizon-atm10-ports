// A Data Model's tier and data; a Deep Learner's models.
fn hostilenetworks_tooltip(
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    crate::game::hostilenetworks_glue::tooltip(stack, lines);
}

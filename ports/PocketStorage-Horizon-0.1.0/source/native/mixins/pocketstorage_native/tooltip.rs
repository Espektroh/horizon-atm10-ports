// The PSU's capacity and the kinds it holds, under its name.
fn pocketstorage_tooltip(
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    crate::game::pocketstorage_glue::tooltip(stack, lines);
}

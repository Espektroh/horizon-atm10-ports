// A jetpack's energy bar under its icon.
fn ironjetpacks_item_bar(stack: &horizon_client::items::ItemStack, bar: &mut Option<(i32, u32)>) {
    crate::game::ironjetpacks_glue::item_bar(stack, bar);
}

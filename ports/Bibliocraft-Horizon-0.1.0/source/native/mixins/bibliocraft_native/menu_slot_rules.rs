// A bookcase slot takes one book (`BookcaseBlockEntity.canPlaceItem`:
// `#bibliocraft:bookcase_books`, and `getMaxStackSize` 1). This runs in the
// horizon-client library, which doesn't have the game's modules: the list is
// `bibliocraft_rt::BOOKS`, written out again.
fn bibliocraft_menu_slot_rules(
    menu: &str,
    slot: usize,
    stack: &horizon_client::items::ItemStack,
    may_place: &mut bool,
    max_stack: &mut u32,
) {
    let _ = slot;
    if menu == "bibliocraft:bookcase" {
        const BIBLIOCRAFT_BOOKS: [&str; 5] = [
            "minecraft:book",
            "minecraft:writable_book",
            "minecraft:written_book",
            "minecraft:enchanted_book",
            "minecraft:knowledge_book",
        ];
        if !BIBLIOCRAFT_BOOKS.contains(&&*stack.item) {
            *may_place = false;
        }
        *max_stack = 1;
    }
}

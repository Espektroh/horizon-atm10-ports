// A tier upgrade on the crafting grid takes only an empty PSU, so the grid
// never eats one full of items (the original moved the contents; this port
// can't see the grid from the result hook).
fn pocketstorage_ingredient_test(
    kind: &str,
    stack: &horizon_client::items::ItemStack,
    accept: &mut bool,
) {
    if kind == "pocketstorage:empty_psu"
        && stack.data().is_some_and(|d| {
            d.other("minecraft:custom_data")
                .and_then(|t| t.get("pocketstorage_native:slots"))
                .is_some()
        })
    {
        *accept = false;
    }
}

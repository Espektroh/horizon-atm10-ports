// What each slot of the machines' menus takes. This runs in the horizon-client
// library, which doesn't have the game's modules: the ids are written out again.
fn hostilenetworks_menu_slot_rules(
    menu: &str,
    slot: usize,
    stack: &horizon_client::items::ItemStack,
    may_place: &mut bool,
    max_stack: &mut u32,
) {
    let item: &str = &stack.item;
    if menu == "hostilenetworks:sim_chamber" {
        if slot == 0 {
            *may_place = item.starts_with("hostilenetworks:data_model_");
            *max_stack = 1;
        } else if slot == 1 {
            *may_place = item == "hostilenetworks:prediction_matrix";
        }
    } else if menu == "hostilenetworks:loot_fabricator" && slot == 0 {
        *may_place = item.starts_with("hostilenetworks:prediction_")
            && item != "hostilenetworks:prediction_matrix";
    }
}

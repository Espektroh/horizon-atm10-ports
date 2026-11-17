// The layout of the rice cooker's and the barrel's screens (the furnace menu's slots).
fn sushigocrafting_screen(
    block: &str,
    screen: &mut Option<crate::game::native_hooks::NativeScreen>,
) {
    crate::game::sushigocrafting_glue::screen(block, screen);
}

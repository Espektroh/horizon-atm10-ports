// The layout of a Mekanism machine's screen.
fn mekanism_screen(block: &str, screen: &mut Option<crate::game::native_hooks::NativeScreen>) {
    crate::game::mekanism_glue::screen(block, screen);
}

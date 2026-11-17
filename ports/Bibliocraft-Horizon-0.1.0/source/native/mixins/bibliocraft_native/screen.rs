// The layout of the bookcase's and the shelf's screens.
fn bibliocraft_screen(block: &str, screen: &mut Option<crate::game::native_hooks::NativeScreen>) {
    crate::game::bibliocraft_glue::screen(block, screen);
}

// The layout of a Powah block's screen.
fn powah_screen(block: &str, screen: &mut Option<crate::game::native_hooks::NativeScreen>) {
    crate::game::powah_glue::screen(block, screen);
}

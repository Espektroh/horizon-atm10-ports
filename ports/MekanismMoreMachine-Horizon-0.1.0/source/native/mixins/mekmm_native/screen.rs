// The layout of a machine's screen.
fn mekmm_screen(block: &str, screen: &mut Option<crate::game::native_hooks::NativeScreen>) {
    crate::game::mekmm_glue::screen(block, screen);
}

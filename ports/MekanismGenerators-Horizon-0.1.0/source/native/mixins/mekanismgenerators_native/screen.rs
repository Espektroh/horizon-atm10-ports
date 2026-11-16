// The layout of a generator's screen.
fn mekanismgenerators_screen(
    block: &str,
    screen: &mut Option<crate::game::native_hooks::NativeScreen>,
) {
    crate::game::mekanismgenerators_glue::screen(block, screen);
}

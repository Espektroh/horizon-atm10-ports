// The layout of a machine's screen.
fn hostilenetworks_screen(
    block: &str,
    screen: &mut Option<crate::game::native_hooks::NativeScreen>,
) {
    crate::game::hostilenetworks_glue::screen(block, screen);
}

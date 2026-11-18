// A click on one of the Loot Fabricator's drops picks it.
fn hostilenetworks_screen_click(block: &str, data: &[i32], at: [f32; 2], button: &mut Option<i32>) {
    crate::game::hostilenetworks_glue::screen_click(block, data, at, button);
}

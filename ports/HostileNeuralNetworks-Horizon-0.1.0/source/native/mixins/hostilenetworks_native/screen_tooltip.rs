// Tooltips over the energy bar and the fabricator's drops.
fn hostilenetworks_screen_tooltip(
    block: &str,
    data: &[i32],
    at: [f32; 2],
    lines: &mut Vec<String>,
) {
    crate::game::hostilenetworks_glue::screen_tooltip(block, data, at, lines);
}

// A natural spawn inside a mega torch's (monsters) or dread lamp's (animals) cube.
fn torchmaster_potential_spawns(
    dimension: &str,
    category: &str,
    pos: [i32; 3],
    canceled: &mut bool,
) {
    crate::game::torchmaster_glue::potential_spawns(dimension, category, pos, canceled);
}

// A zombie siege whose spawn point is inside a mega torch's cube.
fn torchmaster_village_siege(dimension: &str, attempted_pos: [f64; 3], canceled: &mut bool) {
    crate::game::torchmaster_glue::village_siege(dimension, attempted_pos, canceled);
}

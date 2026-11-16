// A world opened or closed: forget every light.
fn torchmaster_lifecycle() {
    crate::game::torchmaster_glue::world_changed();
}

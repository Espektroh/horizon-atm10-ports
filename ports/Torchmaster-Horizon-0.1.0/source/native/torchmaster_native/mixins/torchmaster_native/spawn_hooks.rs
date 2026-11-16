// While a mega torch or dread lamp is registered, natural spawning asks the
// mods for its spawn list (otherwise it only does so with trait mods loaded).
fn torchmaster_spawn_hooks() -> std::ops::ControlFlow<bool> {
    if crate::game::torchmaster_glue::any_lights() {
        return std::ops::ControlFlow::Break(true);
    }
    std::ops::ControlFlow::Continue(())
}

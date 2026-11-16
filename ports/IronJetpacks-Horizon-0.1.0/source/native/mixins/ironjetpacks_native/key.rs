// The engine, hover and throttle keys (V, H, period, comma by default).
fn ironjetpacks_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::ironjetpacks_glue::key(game, key, pressed);
}

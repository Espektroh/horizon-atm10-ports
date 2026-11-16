// A player breaks a block: a lantern takes its invisible lights with it.
fn torchmaster_destroy(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    let was = *canceled;
    crate::game::torchmaster_glue::destroy_block(game, pos, was);
}

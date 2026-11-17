// A player started or stopped sneaking (no PSU pickup while sneaking).
fn pocketstorage_toggle_sneak(player: u128, sneaking: bool) {
    crate::game::pocketstorage_glue::toggle_sneak(player, sneaking);
}

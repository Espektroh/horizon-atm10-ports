// A world that starts has no accelerators left from another.
fn tiab_server_start(dedicated: bool) {
    let _ = dedicated;
    crate::game::tiab_glue::server_start();
}

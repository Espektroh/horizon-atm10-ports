// `/psu buscar <texto>` and `/psu limpar`.
fn pocketstorage_register_commands(commands: &mut Vec<horizon_client::mod_api::NativeCommand>) {
    crate::game::pocketstorage_glue::register_commands(commands);
}

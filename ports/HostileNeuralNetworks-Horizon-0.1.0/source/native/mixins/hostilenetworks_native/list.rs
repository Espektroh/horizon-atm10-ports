// The mod's row in Options > Mods.
fn hostilenetworks_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::hostilenetworks_native as h;
    mods.push(crate::game::native_hooks::NativeMod {
        id: h::ID.into(),
        title: format!("{} {}", h::NAME, h::VERSION),
        description: h::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}

// The mod's row in Options > Mods.
fn powah_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::powah_native as p;
    mods.push(crate::game::native_hooks::NativeMod {
        id: p::ID.into(),
        title: format!("{} {}", p::NAME, p::VERSION),
        description: p::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}

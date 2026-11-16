// The mod's row in Options > Mods, with its keys.
fn ironjetpacks_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::ironjetpacks_rt as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: m::KEYS.iter().map(|(n, l, d)| [(*n).to_owned(), (*l).to_owned(), (*d).to_owned()]).collect(),
    });
}

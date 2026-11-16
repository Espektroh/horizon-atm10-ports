// The mod's row in Options > Mods.
fn mekanismgenerators_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::mekanismgenerators_native as mg;
    mods.push(crate::game::native_hooks::NativeMod {
        id: mg::MG_ID.into(),
        title: format!("{} {}", mg::MG_NAME, mg::MG_VERSION),
        description: mg::MG_DESCRIPTION.into(),
        keys: Vec::new(),
    });
}

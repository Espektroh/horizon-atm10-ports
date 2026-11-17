// The mod's row in Options > Mods.
fn mekmm_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::mekmm_native as mm;
    mods.push(crate::game::native_hooks::NativeMod {
        id: mm::MM_ID.into(),
        title: format!("{} {}", mm::MM_NAME, mm::MM_VERSION),
        description: mm::MM_DESCRIPTION.into(),
        keys: Vec::new(),
    });
}

// The Infusion Crystal stays in the crafting grid one use more worn, and goes after 1000 uses
// (the Master Infusion Crystal's remainder is itself, from its content file). Self-contained:
// `crafting.rs` is also built into a crate without the game (keep the numbers in step with
// `mysticalagriculture_rt::INFUSION_CRYSTAL` and `INFUSION_CRYSTAL_USES`).
fn mysticalagriculture_remainders(
    recipes: &horizon_client::crafting::Recipes,
    recipe: &str,
    grid: &[Option<horizon_client::items::ItemStack>],
    size: usize,
) -> core::ops::ControlFlow<Vec<Option<ItemStack>>> {
    thread_local! {
        static MYSTICALAGRICULTURE_INSIDE: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
    }
    const CRYSTAL: &str = "mysticalagriculture:infusion_crystal";
    const USES: u32 = 1000;
    if MYSTICALAGRICULTURE_INSIDE.with(core::cell::Cell::get)
        || !grid.iter().flatten().any(|s| &*s.item == CRYSTAL)
    {
        return core::ops::ControlFlow::Continue(());
    }
    MYSTICALAGRICULTURE_INSIDE.with(|c| c.set(true));
    let mut out = recipes.remainders(recipe, grid, size);
    MYSTICALAGRICULTURE_INSIDE.with(|c| c.set(false));
    for (cell, slot) in grid.iter().zip(out.iter_mut()) {
        let Some(s) = cell else { continue };
        if &*s.item != CRYSTAL {
            continue;
        }
        let next = s.damage + 1;
        *slot = (next < USES).then(|| ItemStack {
            count: 1,
            damage: next,
            ..s.clone()
        });
    }
    core::ops::ControlFlow::Break(out)
}

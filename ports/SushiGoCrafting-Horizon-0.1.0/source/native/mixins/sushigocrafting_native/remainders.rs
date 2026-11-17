// The Roller stays in the crafting grid when a sushi is rolled (it is the original's machine; here
// the sushi recipes are shaped recipes with the Roller in them). Self-contained: `crafting.rs` is
// also built into a crate without the game.
fn sushigocrafting_remainders(
    recipes: &horizon_client::crafting::Recipes,
    recipe: &str,
    grid: &[Option<horizon_client::items::ItemStack>],
    size: usize,
) -> core::ops::ControlFlow<Vec<Option<ItemStack>>> {
    thread_local! {
        static SUSHIGOCRAFTING_INSIDE: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
    }
    const ROLLER: &str = "sushigocrafting:roller";
    if SUSHIGOCRAFTING_INSIDE.with(core::cell::Cell::get)
        || !grid.iter().flatten().any(|s| &*s.item == ROLLER)
    {
        return core::ops::ControlFlow::Continue(());
    }
    SUSHIGOCRAFTING_INSIDE.with(|c| c.set(true));
    let mut out = recipes.remainders(recipe, grid, size);
    SUSHIGOCRAFTING_INSIDE.with(|c| c.set(false));
    for (cell, slot) in grid.iter().zip(out.iter_mut()) {
        let Some(s) = cell else { continue };
        if &*s.item != ROLLER {
            continue;
        }
        *slot = Some(ItemStack {
            count: 1,
            ..s.clone()
        });
    }
    core::ops::ControlFlow::Break(out)
}

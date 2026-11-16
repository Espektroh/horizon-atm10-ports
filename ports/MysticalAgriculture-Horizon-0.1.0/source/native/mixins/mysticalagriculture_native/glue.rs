// Mystical Agriculture for Horizon: the rules (`mysticalagriculture_rt`) over the game. Added
// as `crate::game::mysticalagriculture_glue`; the hook helpers call it.

use super::mysticalagriculture_rt as rt;
use super::{Game, GameLevel};
use horizon_client::items::ItemStack;
use horizon_client::mod_api::NativeLootDrops;
use horizon_core::position::BlockPos;
use horizon_world::registry::BlockStateId;

/// Vanilla crops the Mystical Fertilizer ripens at once, with their last age.
fn vanilla_crop_max_age(block: &str) -> Option<u32> {
    match block {
        "minecraft:wheat" | "minecraft:carrots" | "minecraft:potatoes" => Some(7),
        "minecraft:beetroots" => Some(3),
        _ => None,
    }
}

/// `native_loot_drops`: a ripe crop's drops as the original rolls them (the farmland below
/// decides the second essence and seed; Inferium grows with the farmland's tier; resource crops
/// sometimes give Fertilized Essence), and Inferium Essence from mobs, and Agradditions' boss drops.
pub(crate) fn loot_drops(
    game: &mut Game<'_>,
    context: &NativeLootDrops<'_>,
    drops: &mut Vec<ItemStack>,
) {
    if context.used || !game.native_on(rt::ID) {
        return;
    }
    if let Some((pos, state)) = context.block {
        let name = game.res.facts().get(state).name.to_string();
        let Some(crop) = rt::crop_of_block(&name) else {
            return;
        };
        // Ripe: its table rolled its essence (the table gives it at age 7 only).
        let Some(at) = drops.iter().position(|s| &*s.item == crop.essence) else {
            return;
        };
        // The level in place is the one `game.world` holds; another one's farmland isn't read.
        let below = if context.dimension == game.native_dimension() {
            rt::farmland_tier(&game.native_block([pos[0], pos[1] - 1, pos[2]]))
        } else {
            None
        };
        let h = rt::harvest(crop, below, rt::roll);
        drops[at].count = h.essence;
        if h.seeds > 1 {
            let seeds = format!("{}_seeds", crop.block.trim_end_matches("_crop"));
            if let Some(s) = drops.iter_mut().find(|s| *s.item == *seeds) {
                s.count = h.seeds;
            }
        }
        if h.fertilized > 0 {
            drops.push(ItemStack::new(rt::FERTILIZED_ESSENCE, h.fertilized));
        }
        return;
    }
    if let Some((_, kind)) = context.entity {
        // An empty table: the mob_drops rule is off (or the mob has none, as the dragon).
        if !context.table.is_empty() && rt::drops_inferium(kind, rt::roll()) {
            drops.push(ItemStack::new(rt::INFERIUM_ESSENCE, 1));
        }
        if let Some((item, count)) = rt::boss_drop(kind, rt::roll()) {
            drops.push(ItemStack::new(item, count));
        }
    }
}

/// `native_use_item_on`: an essence on farmland makes essence farmland (keeping its moisture),
/// the Mystical Fertilizer ripens a crop at once, Fertilized Essence grows one of this mod's
/// crops as bone meal grows wheat. Anything else is left to the item's own use.
pub(crate) fn use_item_on(
    game: &mut Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    if *claimed || !game.native_on(rt::ID) {
        return;
    }
    let item: String = stack.item.to_string();
    if !item.starts_with("mysticalagr") {
        return;
    }
    let block = game.native_block(pos);
    let creative = game
        .native_acting_player()
        .is_some_and(|p| p.game_mode == 1);
    if item == rt::MYSTICAL_FERTILIZER || item == rt::FERTILIZED_ESSENCE {
        let ours = rt::crop_of_block(&block);
        let max = match ours {
            Some(crop) if !rt::fertilizable(crop) => {
                // Tier 6 crops take no fertilizer (fertilizableTier6Crops = false).
                *claimed = true;
                return;
            }
            Some(_) => 7,
            // Fertilized Essence on anything else is bone meal (its template).
            None if item == rt::FERTILIZED_ESSENCE => return,
            None => match vanilla_crop_max_age(&block) {
                Some(max) => max,
                None => return,
            },
        };
        let Some(age) = game
            .native_block_property(pos, "age")
            .and_then(|a| a.parse::<u32>().ok())
        else {
            return;
        };
        if age >= max {
            return;
        }
        let next = if item == rt::MYSTICAL_FERTILIZER {
            max
        } else {
            rt::fertilized_age(age, max, rt::roll())
        };
        *claimed = true;
        let flags = horizon_world::level::flags::UPDATE_CLIENTS;
        if game.native_set_block_property_with(pos, "age", &next.to_string(), flags) && !creative {
            game.native_take_items(&item, 1);
        }
        return;
    }
    let Some((to, refund)) = rt::convert_farmland(&block, &item) else {
        return;
    };
    let moisture = game.native_block_property(pos, "moisture");
    let Some(state) = game.pack.assets.registry.default_state(to) else {
        return;
    };
    let dimension = game.native_dimension();
    if !game.native_set_block(
        dimension,
        pos,
        state,
        horizon_world::level::flags::UPDATE_ALL,
    ) {
        return;
    }
    *claimed = true;
    if let Some(m) = moisture {
        let _ = game.native_set_block_property_with(
            pos,
            "moisture",
            &m,
            horizon_world::level::flags::UPDATE_CLIENTS,
        );
    }
    if !creative {
        game.native_take_items(&item, 1);
    }
    if let Some(essence) = refund {
        if rt::roll() < rt::FARMLAND_REFUND_CHANCE {
            game.native_give_items(essence, 1);
        }
    }
}

/// `native_random_tick`: an Agradditions crop doesn't grow without its crux two blocks below.
pub(super) fn random_tick(
    level: &GameLevel<'_>,
    pos: BlockPos,
    state: BlockStateId,
    handled: &mut bool,
) {
    if *handled {
        return;
    }
    let name: &str = &level.facts.get(state).name;
    let Some(crop) = rt::crop_of_block(name) else {
        return;
    };
    if crop.crux.is_none() {
        return;
    }
    let two_below = level.world.block(pos.x, pos.y - 2, pos.z);
    let two_below: &str = &level.facts.get(two_below).name;
    if !rt::can_grow(crop, two_below) {
        *handled = true;
    }
}

/// `native_is_valid_bonemeal_target`: bone meal doesn't grow this mod's crops (the original's
/// `isBonemealSuccess` is false); it is kept, as Inferium Lab does. Fertilizers go through
/// `use_item_on` instead.
pub(super) fn bonemeal_target(
    level: &GameLevel<'_>,
    state: BlockStateId,
    valid: &mut Option<bool>,
) {
    if valid.is_none() && rt::crop_of_block(&level.facts.get(state).name).is_some() {
        *valid = Some(false);
    }
}

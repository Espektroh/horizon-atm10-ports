// Mystical Agriculture for Horizon: the rules, with no game types (std only), so they are
// tested on their own (`cargo test --lib`). Added to the game as
// `crate::game::mysticalagriculture_rt`; the glue reads the world and calls these.
//
// Numbers from Mystical Agriculture 8.0.28 and Mystical Agradditions 8.0.14 (BlakeBr0, MIT):
// `MysticalCropBlock.getDrops`, `InferiumCropBlock.getDrops`, `Crop.getSecondaryChance`,
// `CropTier` (base secondary chance 0.1), `MobDropHandler` and `ModConfigs` defaults
// (inferiumDropChance 0.2, fertilizedEssenceChance 0.1, secondarySeedDrops true,
// witheringSoulChance 0.35, dragonScalesAmount 8, fertilizableTier6Crops false).

use std::sync::atomic::{AtomicU64, Ordering};

pub const ID: &str = "mysticalagriculture_native";
pub const NAME: &str = "Mystical Agriculture";
pub const VERSION: &str = "0.2.0";
pub const DESCRIPTION: &str = "Essence farmland bonus, Inferium from mobs, cruxes, fertilizers, \
farmland conversion, infusion crystals, the Infusion Altar with its pedestals, the wand and the \
watering cans for the Mystical Agriculture port.";

pub const INFERIUM_ESSENCE: &str = "mysticalagriculture:inferium_essence";
pub const FERTILIZED_ESSENCE: &str = "mysticalagriculture:fertilized_essence";
pub const MYSTICAL_FERTILIZER: &str = "mysticalagriculture:mystical_fertilizer";
pub const INFUSION_CRYSTAL: &str = "mysticalagriculture:infusion_crystal";
pub const WITHERING_SOUL: &str = "mysticalagradditions:withering_soul";
pub const DRAGON_SCALE: &str = "mysticalagradditions:dragon_scale";

/// `ModConfigs` defaults.
pub const INFERIUM_DROP_CHANCE: f64 = 0.2;
pub const FERTILIZED_ESSENCE_CHANCE: f64 = 0.1;
pub const BASE_SECONDARY_CHANCE: f64 = 0.1;
pub const EFFECTIVE_FARMLAND_BONUS: f64 = 0.1;
pub const WITHERING_SOUL_CHANCE: f64 = 0.35;
pub const DRAGON_SCALES: u32 = 8;
/// The basic Infusion Crystal's uses (`infusionCrystalUses`).
pub const INFUSION_CRYSTAL_USES: u32 = 1000;
/// Converting essence farmland to another tier gives back the old tier's essence this often.
pub const FARMLAND_REFUND_CHANCE: f64 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Crop {
    pub block: &'static str,
    pub essence: &'static str,
    pub tier: u8,
    /// The block two below the crop it needs to grow (Agradditions' cruxes).
    pub crux: Option<&'static str>,
}

const fn c(block: &'static str, essence: &'static str, tier: u8) -> Crop {
    Crop {
        block,
        essence,
        tier,
        crux: None,
    }
}

/// Every crop of the port (keep in step with make_content.py, which checks it).
pub const CROPS: &[Crop] = &[
    c(
        "mysticalagriculture:inferium_crop",
        "mysticalagriculture:inferium_essence",
        1,
    ),
    c(
        "mysticalagriculture:stone_crop",
        "mysticalagriculture:stone_essence",
        1,
    ),
    c(
        "mysticalagriculture:dirt_crop",
        "mysticalagriculture:dirt_essence",
        1,
    ),
    c(
        "mysticalagriculture:wood_crop",
        "mysticalagriculture:wood_essence",
        1,
    ),
    c(
        "mysticalagriculture:ice_crop",
        "mysticalagriculture:ice_essence",
        1,
    ),
    c(
        "mysticalagriculture:deepslate_crop",
        "mysticalagriculture:deepslate_essence",
        1,
    ),
    c(
        "mysticalagriculture:nature_crop",
        "mysticalagriculture:nature_essence",
        2,
    ),
    c(
        "mysticalagriculture:dye_crop",
        "mysticalagriculture:dye_essence",
        2,
    ),
    c(
        "mysticalagriculture:nether_crop",
        "mysticalagriculture:nether_essence",
        2,
    ),
    c(
        "mysticalagriculture:coal_crop",
        "mysticalagriculture:coal_essence",
        2,
    ),
    c(
        "mysticalagriculture:coral_crop",
        "mysticalagriculture:coral_essence",
        2,
    ),
    c(
        "mysticalagriculture:honey_crop",
        "mysticalagriculture:honey_essence",
        2,
    ),
    c(
        "mysticalagriculture:amethyst_crop",
        "mysticalagriculture:amethyst_essence",
        2,
    ),
    c(
        "mysticalagriculture:iron_crop",
        "mysticalagriculture:iron_essence",
        3,
    ),
    c(
        "mysticalagriculture:copper_crop",
        "mysticalagriculture:copper_essence",
        3,
    ),
    c(
        "mysticalagriculture:nether_quartz_crop",
        "mysticalagriculture:nether_quartz_essence",
        3,
    ),
    c(
        "mysticalagriculture:glowstone_crop",
        "mysticalagriculture:glowstone_essence",
        3,
    ),
    c(
        "mysticalagriculture:redstone_crop",
        "mysticalagriculture:redstone_essence",
        3,
    ),
    c(
        "mysticalagriculture:obsidian_crop",
        "mysticalagriculture:obsidian_essence",
        3,
    ),
    c(
        "mysticalagriculture:prismarine_crop",
        "mysticalagriculture:prismarine_essence",
        3,
    ),
    c(
        "mysticalagriculture:sculk_crop",
        "mysticalagriculture:sculk_essence",
        3,
    ),
    c(
        "mysticalagriculture:gold_crop",
        "mysticalagriculture:gold_essence",
        4,
    ),
    c(
        "mysticalagriculture:lapis_lazuli_crop",
        "mysticalagriculture:lapis_lazuli_essence",
        4,
    ),
    c(
        "mysticalagriculture:end_crop",
        "mysticalagriculture:end_essence",
        4,
    ),
    c(
        "mysticalagriculture:osmium_crop",
        "mysticalagriculture:osmium_essence",
        4,
    ),
    c(
        "mysticalagriculture:steel_crop",
        "mysticalagriculture:steel_essence",
        4,
    ),
    c(
        "mysticalagriculture:diamond_crop",
        "mysticalagriculture:diamond_essence",
        5,
    ),
    c(
        "mysticalagriculture:emerald_crop",
        "mysticalagriculture:emerald_essence",
        5,
    ),
    c(
        "mysticalagriculture:netherite_crop",
        "mysticalagriculture:netherite_essence",
        5,
    ),
    Crop {
        block: "mysticalagriculture:nether_star_crop",
        essence: "mysticalagriculture:nether_star_essence",
        tier: 6,
        crux: Some("mysticalagradditions:nether_star_crux"),
    },
    Crop {
        block: "mysticalagriculture:dragon_egg_crop",
        essence: "mysticalagriculture:dragon_egg_essence",
        tier: 6,
        crux: Some("mysticalagradditions:dragon_egg_crux"),
    },
];

/// The essence farmland of each tier, 1 (inferium) to 6 (insanium), and the essence that makes it.
pub const FARMLANDS: [(&str, &str); 6] = [
    (
        "mysticalagriculture:inferium_farmland",
        "mysticalagriculture:inferium_essence",
    ),
    (
        "mysticalagriculture:prudentium_farmland",
        "mysticalagriculture:prudentium_essence",
    ),
    (
        "mysticalagriculture:tertium_farmland",
        "mysticalagriculture:tertium_essence",
    ),
    (
        "mysticalagriculture:imperium_farmland",
        "mysticalagriculture:imperium_essence",
    ),
    (
        "mysticalagriculture:supremium_farmland",
        "mysticalagriculture:supremium_essence",
    ),
    (
        "mysticalagradditions:insanium_farmland",
        "mysticalagradditions:insanium_essence",
    ),
];

pub fn crop_of_block(name: &str) -> Option<&'static Crop> {
    if !name.starts_with("mysticalagriculture:") || !name.ends_with("_crop") {
        return None;
    }
    CROPS.iter().find(|c| c.block == name)
}

/// The essence farmland's tier (1 to 6), None for any other block.
pub fn farmland_tier(name: &str) -> Option<u8> {
    FARMLANDS
        .iter()
        .position(|(f, _)| *f == name)
        .map(|i| i as u8 + 1)
}

/// The tier of an essence that makes farmland (inferium 1 .. insanium 6).
pub fn essence_tier(item: &str) -> Option<u8> {
    FARMLANDS
        .iter()
        .position(|(_, e)| *e == item)
        .map(|i| i as u8 + 1)
}

/// `Crop.getSecondaryChance(below)`: 0.1 on any essence farmland, 0.1 more on the crop's own
/// tier's (its effective farmland); none for a tier without secondary drops (6).
pub fn secondary_chance(crop_tier: u8, farmland: Option<u8>) -> f64 {
    if crop_tier >= 6 {
        return 0.0;
    }
    let mut chance = 0.0;
    if let Some(t) = farmland {
        chance += BASE_SECONDARY_CHANCE;
        if t == crop_tier {
            chance += EFFECTIVE_FARMLAND_BONUS;
        }
    }
    f64::min(chance, 1.0)
}

/// `InferiumCropBlock.getDrops`: the essence a ripe Inferium crop gives on a farmland of
/// `tier` (none: 1). `(int)(0.5 * tier + 0.5)`, and on an even tier above 1 one more half the
/// time (`roll` < 0.5).
pub fn inferium_essence(farmland: Option<u8>, roll: f64) -> u32 {
    let Some(tier) = farmland else { return 1 };
    let tier = u32::from(tier);
    let mut n = (0.5 * f64::from(tier) + 0.5) as u32;
    if tier > 1 && tier % 2 == 0 && roll < 0.5 {
        n += 1;
    }
    n
}

/// What a ripe crop gives, as the original rolls it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Harvest {
    pub essence: u32,
    pub seeds: u32,
    pub fertilized: u32,
}

/// The drops of a ripe crop on `farmland` (an essence farmland's tier, or None), drawing each
/// chance from `roll` (0..1) in the original's order.
pub fn harvest(crop: &Crop, farmland: Option<u8>, mut roll: impl FnMut() -> f64) -> Harvest {
    let chance = secondary_chance(crop.tier, farmland);
    if crop.block == "mysticalagriculture:inferium_crop" {
        let essence = if farmland.is_some() {
            inferium_essence(farmland, roll())
        } else {
            1
        };
        let seeds = if roll() < chance { 2 } else { 1 };
        return Harvest {
            essence,
            seeds,
            fertilized: 0,
        };
    }
    let essence = if roll() < chance { 2 } else { 1 };
    let seeds = if roll() < chance { 2 } else { 1 };
    let fertilized = u32::from(roll() < FERTILIZED_ESSENCE_CHANCE);
    Harvest {
        essence,
        seeds,
        fertilized,
    }
}

/// Mobs that aren't vanilla's `PathfinderMob` (the original's mob drop needs one).
const NOT_PATHFINDERS: &[&str] = &[
    "minecraft:player",
    "minecraft:armor_stand",
    "minecraft:slime",
    "minecraft:magma_cube",
    "minecraft:ghast",
    "minecraft:phantom",
    "minecraft:bat",
    "minecraft:ender_dragon",
];

/// Whether a mob of `kind` drops Inferium Essence for this `roll` (`MobDropHandler`).
pub fn drops_inferium(kind: &str, roll: f64) -> bool {
    !NOT_PATHFINDERS.contains(&kind) && roll < INFERIUM_DROP_CHANCE
}

/// Agradditions' boss drops for a mob of `kind`: (item, count).
pub fn boss_drop(kind: &str, roll: f64) -> Option<(&'static str, u32)> {
    match kind {
        "minecraft:wither" if roll < WITHERING_SOUL_CHANCE => Some((WITHERING_SOUL, 1)),
        "minecraft:ender_dragon" if DRAGON_SCALES > 0 => Some((DRAGON_SCALE, DRAGON_SCALES)),
        _ => None,
    }
}

/// `MysticalCropBlock.canGrow`: a crop with a crux needs it two blocks below.
pub fn can_grow(crop: &Crop, two_below: &str) -> bool {
    crop.crux.is_none_or(|crux| crux == two_below)
}

/// What an essence in hand does to the farmland clicked (`FarmlandConverter.convert`): the
/// essence farmland it becomes, and whether the old one was an essence farmland (a quarter of
/// the time its essence comes back).
pub fn convert_farmland(
    block: &str,
    essence: &str,
) -> Option<(&'static str, Option<&'static str>)> {
    let tier = essence_tier(essence)?;
    let to = FARMLANDS[usize::from(tier) - 1].0;
    if block == "minecraft:farmland" {
        return Some((to, None));
    }
    let old = farmland_tier(block)?;
    (old != tier).then_some((to, Some(FARMLANDS[usize::from(old) - 1].1)))
}

/// Whether a fertilizer may grow this crop (`CropTier.isFertilizable`; tier 6 not by default).
pub fn fertilizable(crop: &Crop) -> bool {
    crop.tier < 6
}

/// Fertilized Essence on a crop: bone meal's growth (`CropBlock.getBonemealAgeIncrease`, 2 to
/// 5 ages) from `roll` (0..1), up to the crop's last age.
pub fn fertilized_age(age: u32, max: u32, roll: f64) -> u32 {
    let add = 2 + (roll.clamp(0.0, 0.999_999) * 4.0) as u32;
    (age + add).min(max)
}

/// A worn Infusion Crystal left in the grid: the next damage, or None when it is used up.
pub fn crystal_after_use(damage: u32) -> Option<u32> {
    let next = damage + 1;
    (next < INFUSION_CRYSTAL_USES).then_some(next)
}

// ---- 0.2.0: the Infusion Altar and its pedestals, the wand, the watering cans ----------------
//
// From `InfusionAltarTileEntity`, `InfusionAltarBlock`, `InfusionPedestalBlock`, `InfusionRecipe`,
// `WandItem`, `ModItems` (the cans' range and chance) and Cucumber's `BaseWateringCanItem` (MIT).

pub const INFUSION_ALTAR: &str = "mysticalagriculture:infusion_altar";
pub const INFUSION_PEDESTAL: &str = "mysticalagriculture:infusion_pedestal";
pub const WAND: &str = "mysticalagriculture:wand";
/// The recipe type the altar crafts (`data/*/recipe/**`, read with `native_recipes_of_type`).
pub const INFUSION_TYPE: &str = "mysticalagriculture:infusion";
/// Ticks of work before the altar crafts (`progress >= 100`).
pub const INFUSION_TICKS: u32 = 100;
/// `InfusionRecipe` takes 1 to 8 ingredients around the altar's own input.
pub const INFUSION_MAX_INGREDIENTS: usize = 8;
/// The altar's slots: its input and, after a craft, the output (`createInventoryHandler`).
pub const ALTAR_INPUT: usize = 0;
pub const ALTAR_OUTPUT: usize = 1;
pub const ALTAR_SLOTS: usize = 2;
pub const PEDESTAL_SLOTS: usize = 1;
/// The pedestals' places around the altar, in the original's order (`pedestalLocations`).
pub const PEDESTAL_OFFSETS: [[i32; 3]; 8] = [
    [3, 0, 0],
    [0, 0, 3],
    [-3, 0, 0],
    [0, 0, -3],
    [2, 0, 2],
    [2, 0, -2],
    [-2, 0, 2],
    [-2, 0, -2],
];
/// Saved in the altar's compound (the original's own keys).
pub const KEY_PROGRESS: &str = "Progress";
pub const KEY_ACTIVE: &str = "Active";
/// What clients get of an altar or a pedestal: one compound per slot (`id`, `components`).
pub const KEY_VIEW: &str = "mysticalagriculture_view";
/// The altar's slots as last sent to clients (item ids), so a hopper taking its output is seen.
pub const KEY_SHOWN: &str = "mysticalagriculture_shown";

/// What a client shows of slots holding these item ids ("" for an empty slot).
pub fn shown_signature<'a>(ids: impl Iterator<Item = &'a str>) -> String {
    ids.collect::<Vec<_>>().join("|")
}

/// The altar and the pedestal have a block entity of their own type, named after the block.
pub fn is_infusion_block(name: &str) -> bool {
    name == INFUSION_ALTAR || name == INFUSION_PEDESTAL
}

/// The block's item slots.
pub fn infusion_slots(name: &str) -> usize {
    if name == INFUSION_ALTAR {
        ALTAR_SLOTS
    } else {
        PEDESTAL_SLOTS
    }
}

/// Where the eight pedestals of the altar at `pos` stand.
pub fn pedestal_positions(pos: [i32; 3]) -> [[i32; 3]; 8] {
    PEDESTAL_OFFSETS.map(|o| [pos[0] + o[0], pos[1] + o[1], pos[2] + o[2]])
}

/// The slot whose item the block shows floating above it (the renderers: the altar shows its
/// output when it has one, else its input), and how high, in blocks.
pub fn shown_slot(name: &str, filled: &[bool]) -> Option<(usize, f32)> {
    let has = |i: usize| filled.get(i).copied().unwrap_or(false);
    if name == INFUSION_ALTAR {
        if has(ALTAR_OUTPUT) {
            Some((ALTAR_OUTPUT, 1.1))
        } else if has(ALTAR_INPUT) {
            Some((ALTAR_INPUT, 1.1))
        } else {
            None
        }
    } else if name == INFUSION_PEDESTAL && has(0) {
        Some((0, 1.2))
    } else {
        None
    }
}

/// The renderers' float: scale (0.95 for a block's item, 0.75 for others), bob and turn at
/// `ticks` (game time plus the partial tick). The original turns by the clock,
/// `millis / 800`; a tick is 50 ms.
pub fn float_pose(ticks: f64, is_block: bool) -> (f32, f32, f32) {
    let scale = if is_block { 0.95 } else { 0.75 };
    let t = ticks * 50.0 / 800.0;
    let bob = (t % std::f64::consts::TAU).sin() * 0.065;
    let yaw = (t * 40.0) % 360.0;
    (scale, bob as f32, yaw as f32)
}

/// `RecipeMatcher.findMatches`: gives each of `stacks` its own ingredient, every ingredient
/// used once, or None. `fits(i, s)`: ingredient `i` takes stack `s`. Augmenting paths, so an
/// order that a greedy pass would get wrong still matches.
pub fn match_shapeless(
    ingredients: usize,
    stacks: usize,
    fits: impl Fn(usize, usize) -> bool,
) -> Option<Vec<usize>> {
    if ingredients != stacks {
        return None;
    }
    let n = stacks;
    let fit: Vec<Vec<bool>> = (0..n)
        .map(|i| (0..n).map(|s| fits(i, s)).collect())
        .collect();
    // owner[s]: the ingredient that took stack s.
    let mut owner: Vec<Option<usize>> = vec![None; n];
    fn take(i: usize, fit: &[Vec<bool>], owner: &mut [Option<usize>], seen: &mut [bool]) -> bool {
        for s in 0..owner.len() {
            if fit[i][s] && !seen[s] {
                seen[s] = true;
                let free = match owner[s] {
                    None => true,
                    Some(j) => take(j, fit, owner, seen),
                };
                if free {
                    owner[s] = Some(i);
                    return true;
                }
            }
        }
        false
    }
    for i in 0..n {
        let mut seen = vec![false; n];
        if !take(i, &fit, &mut owner, &mut seen) {
            return None;
        }
    }
    owner.into_iter().collect()
}

/// `InfusionRecipe.matches`: the altar's input fits the recipe's `input`, and the pedestals
/// that hold an item hold exactly the recipe's ingredients, in any order. An empty pedestal
/// counts for nothing; a recipe with more or fewer ingredients than items doesn't match.
pub fn infusion_matches<I, S>(
    center: &I,
    ingredients: &[I],
    input: &S,
    pedestals: &[Option<S>],
    test: impl Fn(&I, &S) -> bool,
) -> bool {
    if ingredients.is_empty() || ingredients.len() > INFUSION_MAX_INGREDIENTS {
        return false;
    }
    let present: Vec<&S> = pedestals.iter().flatten().collect();
    if present.len() != ingredients.len() || !test(center, input) {
        return false;
    }
    match_shapeless(ingredients.len(), present.len(), |i, s| {
        test(&ingredients[i], present[s])
    })
    .is_some()
}

/// The altar's saved state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Altar {
    pub progress: u32,
    pub active: bool,
}

/// What one tick of the altar does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AltarStep<R> {
    /// Nothing to do (no input, not switched on, or no recipe: progress back to 0).
    Idle,
    /// A recipe matches and the altar is on: one more tick of progress.
    Working,
    /// The 100th tick: craft this.
    Craft(R),
}

/// `InfusionAltarTileEntity.tick`: with no input the altar resets; it switches on with a
/// redstone signal (`isActive`: `hasNeighborSignal`) or the wand, and stays on until it crafts or
/// its recipe stops matching. `find` looks the recipe up (asked only while the altar is on).
pub fn altar_step<R>(
    a: &mut Altar,
    has_input: bool,
    powered: bool,
    find: impl FnOnce() -> Option<R>,
) -> AltarStep<R> {
    if !has_input {
        *a = Altar::default();
        return AltarStep::Idle;
    }
    if !a.active {
        a.active = powered;
    }
    if !a.active {
        a.progress = 0;
        return AltarStep::Idle;
    }
    match find() {
        Some(r) => {
            a.progress += 1;
            if a.progress >= INFUSION_TICKS {
                *a = Altar::default();
                AltarStep::Craft(r)
            } else {
                AltarStep::Working
            }
        }
        None => {
            *a = Altar::default();
            AltarStep::Idle
        }
    }
}

/// An item stack as the altar's rules see it: how many, and that many fewer.
pub trait Counted: Clone {
    fn count(&self) -> u32;
    fn with_count(&self, n: u32) -> Self;
}

/// One of `s` used: what is left in its slot (`count - 1`, or its crafting remainder when it
/// was the last one).
fn used_one<T: Counted>(s: &T, remainder: &impl Fn(&T) -> Option<T>) -> Option<T> {
    if s.count() > 1 {
        Some(s.with_count(s.count() - 1))
    } else {
        remainder(s)
    }
}

/// The craft, as the original does it: one of each pedestal's item and of the altar's input is
/// used (their crafting remainders stay in their slots), and the result goes to the output.
/// False (and nothing changes) when the output is taken or the input empty.
pub fn infuse<T: Counted>(
    input: &mut Option<T>,
    output: &mut Option<T>,
    pedestals: &mut [Option<T>],
    result: T,
    remainder: impl Fn(&T) -> Option<T>,
) -> bool {
    if output.is_some() {
        return false;
    }
    let Some(i) = input.take() else {
        return false;
    };
    *input = used_one(&i, &remainder);
    for p in pedestals.iter_mut() {
        if let Some(s) = p.take() {
            *p = used_one(&s, &remainder);
        }
    }
    *output = Some(result);
    true
}

/// What a right click on the altar or a pedestal does (`useItemOn`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    /// The altar's output goes to the player.
    TakeOutput,
    /// The block's input (a pedestal's item) goes to the player.
    TakeInput,
    /// One of the held stack goes in.
    Insert,
    /// The wand switches the altar on (`WandItem.useOn` on an `IActivatable`).
    Activate,
    /// Empty hands on an empty block: nothing (this port shows the altar's state).
    Nothing,
}

/// `InfusionAltarBlock.useItemOn`: the output first; else an empty altar takes one of what is
/// held (the wand too, as in the original); else the input comes back, unless the wand is held.
pub fn altar_click(has_input: bool, has_output: bool, held: Option<&str>) -> Click {
    if has_output {
        Click::TakeOutput
    } else if !has_input && held.is_some() {
        Click::Insert
    } else if has_input {
        if held == Some(WAND) {
            Click::Activate
        } else {
            Click::TakeInput
        }
    } else {
        Click::Nothing
    }
}

/// `InfusionPedestalBlock.useItemOn`: an empty pedestal takes one of what is held; a full one
/// gives its item back.
pub fn pedestal_click(has_item: bool, holding: bool) -> Click {
    if has_item {
        Click::TakeInput
    } else if holding {
        Click::Insert
    } else {
        Click::Nothing
    }
}

/// A watering can: its area (`range` x `range`) and the chance each watering random-ticks the
/// plants in it (`ModItems`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WateringCan {
    pub item: &'static str,
    pub range: i32,
    pub chance: f64,
}

pub const WATERING_CANS: [WateringCan; 6] = [
    WateringCan {
        item: "mysticalagriculture:watering_can",
        range: 3,
        chance: 0.25,
    },
    WateringCan {
        item: "mysticalagriculture:inferium_watering_can",
        range: 3,
        chance: 0.25,
    },
    WateringCan {
        item: "mysticalagriculture:prudentium_watering_can",
        range: 5,
        chance: 0.30,
    },
    WateringCan {
        item: "mysticalagriculture:tertium_watering_can",
        range: 7,
        chance: 0.35,
    },
    WateringCan {
        item: "mysticalagriculture:imperium_watering_can",
        range: 9,
        chance: 0.40,
    },
    WateringCan {
        item: "mysticalagriculture:supremium_watering_can",
        range: 11,
        chance: 0.45,
    },
];

/// A player waters at most once every 5 ticks (`getThrottleTicks`, the cooldown it starts).
pub const WATERING_THROTTLE_TICKS: i64 = 5;
/// The plants a watering ticks are up to 3 blocks above and below the block watered.
pub const WATERING_HEIGHT: i32 = 3;

pub fn watering_can(item: &str) -> Option<&'static WateringCan> {
    WATERING_CANS.iter().find(|c| c.item == item)
}

/// Half the side of the can's area: `(range - 1) / 2`.
pub fn watering_radius(range: i32) -> i32 {
    (range - 1) / 2
}

/// Whether a watering may happen now, the last one at `last` (game time).
pub fn may_water(last: Option<i64>, now: i64) -> bool {
    last.is_none_or(|t| now < t || now - t >= WATERING_THROTTLE_TICKS)
}

/// Farmland the can wets (`FarmlandBlock`: vanilla's and the essence farmlands).
pub fn is_farmland(name: &str) -> bool {
    name == "minecraft:farmland" || farmland_tier(name).is_some()
}

/// Vanilla's `BonemealableBlock`s that tick randomly, mycelium and the chorus flower: what the
/// can random-ticks (`plantBlock instanceof BonemealableBlock || MYCELIUM || CHORUS_FLOWER`).
const GROWABLE: &[&str] = &[
    "minecraft:wheat",
    "minecraft:carrots",
    "minecraft:potatoes",
    "minecraft:beetroots",
    "minecraft:torchflower_crop",
    "minecraft:pitcher_crop",
    "minecraft:melon_stem",
    "minecraft:pumpkin_stem",
    "minecraft:sweet_berry_bush",
    "minecraft:cocoa",
    "minecraft:bamboo",
    "minecraft:bamboo_sapling",
    "minecraft:kelp",
    "minecraft:cave_vines",
    "minecraft:cave_vines_plant",
    "minecraft:twisting_vines",
    "minecraft:weeping_vines",
    "minecraft:grass_block",
    "minecraft:mycelium",
    "minecraft:chorus_flower",
    "minecraft:mangrove_propagule",
    "minecraft:brown_mushroom",
    "minecraft:red_mushroom",
    "minecraft:sea_pickle",
];

/// Whether the can random-ticks a block of this name: this mod's crops, vanilla's crops,
/// saplings and the other growing plants (`GROWABLE`).
pub fn is_growable(name: &str) -> bool {
    crop_of_block(name).is_some()
        || GROWABLE.contains(&name)
        || (name.starts_with("minecraft:") && name.ends_with("_sapling"))
}

static SEED: AtomicU64 = AtomicU64::new(0);

/// A number in 0..1 (as the original's `Math.random()`): splitmix64 over a seed taken from the
/// clock the first time.
pub fn roll() -> f64 {
    let mut s = SEED.load(Ordering::Relaxed);
    if s == 0 {
        s = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0x9E37_79B9_7F4A_7C15, |d| d.as_nanos() as u64)
            | 1;
    }
    s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
    SEED.store(s, Ordering::Relaxed);
    let mut z = s;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crop(name: &str) -> &'static Crop {
        crop_of_block(&format!("mysticalagriculture:{name}_crop")).unwrap()
    }

    #[test]
    fn tables_hold_together() {
        assert_eq!(CROPS.len(), 31);
        for c in CROPS {
            assert!((1..=6).contains(&c.tier), "{}", c.block);
            assert!(c.block.ends_with("_crop") && c.essence.ends_with("_essence"));
            assert_eq!(c.crux.is_some(), c.tier == 6);
        }
        assert!(crop_of_block("minecraft:wheat").is_none());
        assert!(crop_of_block("mysticalagriculture:inferium_farmland").is_none());
    }

    #[test]
    fn secondary_chance_by_farmland() {
        // Plain farmland: none. Any essence farmland: 10%. Its own tier's: 20%.
        assert_eq!(secondary_chance(3, None), 0.0);
        assert_eq!(secondary_chance(3, Some(1)), 0.1);
        assert_eq!(secondary_chance(3, Some(5)), 0.1);
        assert!((secondary_chance(3, Some(3)) - 0.2).abs() < 1e-12);
        assert!((secondary_chance(1, Some(1)) - 0.2).abs() < 1e-12);
        // Tier 6 has no secondary drops.
        assert_eq!(secondary_chance(6, Some(6)), 0.0);
    }

    #[test]
    fn inferium_essence_by_farmland_tier() {
        // (tier, low roll, high roll)
        let table = [
            (1, 1, 1),
            (2, 2, 1),
            (3, 2, 2),
            (4, 3, 2),
            (5, 3, 3),
            (6, 4, 3),
        ];
        for (tier, low, high) in table {
            assert_eq!(inferium_essence(Some(tier), 0.1), low, "tier {tier}");
            assert_eq!(inferium_essence(Some(tier), 0.9), high, "tier {tier}");
        }
        assert_eq!(inferium_essence(None, 0.0), 1);
    }

    #[test]
    fn inferium_harvest() {
        let inferium = crop("inferium");
        // Plain farmland: one essence, one seed, never fertilized essence.
        let h = harvest(inferium, None, || 0.0);
        assert_eq!(
            h,
            Harvest {
                essence: 1,
                seeds: 1,
                fertilized: 0
            }
        );
        // Supremium farmland: 3 essence; a low roll gives the second seed (20% on tier 1 only).
        let h = harvest(inferium, Some(5), || 0.05);
        assert_eq!(
            h,
            Harvest {
                essence: 3,
                seeds: 2,
                fertilized: 0
            }
        );
        let h = harvest(inferium, Some(4), || 0.6);
        assert_eq!(
            h,
            Harvest {
                essence: 2,
                seeds: 1,
                fertilized: 0
            }
        );
    }

    #[test]
    fn resource_harvest() {
        let iron = crop("iron");
        // No essence farmland: never a second essence or seed; fertilized essence 10%.
        let h = harvest(iron, None, || 0.05);
        assert_eq!(
            h,
            Harvest {
                essence: 1,
                seeds: 1,
                fertilized: 1
            }
        );
        // Tertium farmland (its own): a roll under 0.2 doubles.
        let h = harvest(iron, Some(3), || 0.15);
        assert_eq!(
            h,
            Harvest {
                essence: 2,
                seeds: 2,
                fertilized: 0
            }
        );
        // Inferium farmland: only under 0.1.
        let h = harvest(iron, Some(1), || 0.15);
        assert_eq!(
            h,
            Harvest {
                essence: 1,
                seeds: 1,
                fertilized: 0
            }
        );
        // Tier 6: no secondary drops on any farmland.
        let star = crop("nether_star");
        let h = harvest(star, Some(6), || 0.0);
        assert_eq!(
            h,
            Harvest {
                essence: 1,
                seeds: 1,
                fertilized: 1
            }
        );
    }

    #[test]
    fn harvest_rates_over_many_rolls() {
        // The expected essence per ripe iron crop on its own farmland: 1.2.
        let iron = crop("iron");
        let n = 200_000;
        let total: u32 = (0..n).map(|_| harvest(iron, Some(3), roll).essence).sum();
        let mean = f64::from(total) / f64::from(n);
        assert!((mean - 1.2).abs() < 0.01, "{mean}");
        // Inferium on imperium farmland: 2 or 3, 2.5 on average.
        let inferium = crop("inferium");
        let total: u32 = (0..n)
            .map(|_| harvest(inferium, Some(4), roll).essence)
            .sum();
        let mean = f64::from(total) / f64::from(n);
        assert!((mean - 2.5).abs() < 0.01, "{mean}");
    }

    #[test]
    fn mob_drops() {
        assert!(drops_inferium("minecraft:zombie", 0.19));
        assert!(!drops_inferium("minecraft:zombie", 0.2));
        assert!(drops_inferium("minecraft:cow", 0.0));
        assert!(!drops_inferium("minecraft:slime", 0.0));
        assert!(!drops_inferium("minecraft:player", 0.0));
        assert_eq!(
            boss_drop("minecraft:wither", 0.3),
            Some((WITHERING_SOUL, 1))
        );
        assert_eq!(boss_drop("minecraft:wither", 0.4), None);
        assert_eq!(
            boss_drop("minecraft:ender_dragon", 0.9),
            Some((DRAGON_SCALE, 8))
        );
        assert_eq!(boss_drop("minecraft:zombie", 0.0), None);
        let n = 100_000;
        let hits = (0..n)
            .filter(|_| drops_inferium("minecraft:skeleton", roll()))
            .count();
        let rate = hits as f64 / f64::from(n);
        assert!((rate - 0.2).abs() < 0.01, "{rate}");
    }

    #[test]
    fn cruxes() {
        let star = crop("nether_star");
        assert!(can_grow(star, "mysticalagradditions:nether_star_crux"));
        assert!(!can_grow(star, "mysticalagradditions:dragon_egg_crux"));
        assert!(!can_grow(star, "minecraft:dirt"));
        assert!(can_grow(crop("iron"), "minecraft:dirt"));
    }

    #[test]
    fn farmland_conversion() {
        let inferium = "mysticalagriculture:inferium_essence";
        let tertium = "mysticalagriculture:tertium_essence";
        assert_eq!(
            convert_farmland("minecraft:farmland", inferium),
            Some(("mysticalagriculture:inferium_farmland", None))
        );
        assert_eq!(
            convert_farmland("mysticalagriculture:inferium_farmland", tertium),
            Some(("mysticalagriculture:tertium_farmland", Some(inferium)))
        );
        assert_eq!(
            convert_farmland("mysticalagriculture:inferium_farmland", inferium),
            None
        );
        assert_eq!(convert_farmland("minecraft:dirt", inferium), None);
        assert_eq!(
            convert_farmland("minecraft:farmland", "mysticalagriculture:iron_essence"),
            None
        );
        assert_eq!(
            convert_farmland(
                "minecraft:farmland",
                "mysticalagradditions:insanium_essence"
            ),
            Some(("mysticalagradditions:insanium_farmland", None))
        );
        assert_eq!(
            farmland_tier("mysticalagradditions:insanium_farmland"),
            Some(6)
        );
    }

    #[test]
    fn fertilizers() {
        assert!(fertilizable(crop("diamond")));
        assert!(!fertilizable(crop("dragon_egg")));
        assert_eq!(fertilized_age(0, 7, 0.0), 2);
        assert_eq!(fertilized_age(0, 7, 0.99), 5);
        assert_eq!(fertilized_age(5, 7, 0.99), 7);
    }

    #[test]
    fn crystal_wear() {
        assert_eq!(crystal_after_use(0), Some(1));
        assert_eq!(crystal_after_use(998), Some(999));
        assert_eq!(crystal_after_use(999), None);
    }

    #[test]
    fn rolls_stay_in_range() {
        for _ in 0..10_000 {
            let r = roll();
            assert!((0.0..1.0).contains(&r));
        }
    }

    // ---- 0.2.0: altar, pedestals, cans ----

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct S(&'static str, u32);

    impl Counted for S {
        fn count(&self) -> u32 {
            self.1
        }
        fn with_count(&self, n: u32) -> Self {
            S(self.0, n)
        }
    }

    /// An ingredient as a recipe names it: an item, or a "tag" (any item in a list).
    #[derive(Clone, Debug)]
    enum Ing {
        Item(&'static str),
        Any(&'static [&'static str]),
    }

    fn fits(i: &Ing, s: &S) -> bool {
        match i {
            Ing::Item(id) => *id == s.0,
            Ing::Any(ids) => ids.contains(&s.0),
        }
    }

    const BASE: &str = "mysticalagriculture:prosperity_seed_base";
    const IRON: &str = "minecraft:iron_ingot";
    const TERTIUM: &str = "mysticalagriculture:tertium_essence";

    /// The iron seeds' recipe: material and essence alternating, 4 of each (seed/infusion/iron).
    fn iron_recipe() -> (Ing, Vec<Ing>) {
        let mut v = Vec::new();
        for _ in 0..4 {
            v.push(Ing::Item(IRON));
            v.push(Ing::Item(TERTIUM));
        }
        (Ing::Item(BASE), v)
    }

    fn eight(order: [&'static str; 8]) -> Vec<Option<S>> {
        order.iter().map(|id| Some(S(id, 1))).collect()
    }

    #[test]
    fn pedestals_stand_where_the_original_puts_them() {
        let p = pedestal_positions([10, 64, -5]);
        assert_eq!(p[0], [13, 64, -5]);
        assert_eq!(p[3], [10, 64, -8]);
        assert_eq!(p[7], [8, 64, -7]);
        // All on the altar's level, 8 different places, none on the altar.
        let mut seen = std::collections::HashSet::new();
        for q in p {
            assert_eq!(q[1], 64);
            assert!(q != [10, 64, -5]);
            assert!(seen.insert(q));
        }
    }

    #[test]
    fn eight_pedestals_match_in_any_order() {
        let (center, ings) = iron_recipe();
        let input = S(BASE, 1);
        // As the original lays them out, and every essence first: shapeless.
        let laid = eight([IRON, TERTIUM, IRON, TERTIUM, IRON, TERTIUM, IRON, TERTIUM]);
        let grouped = eight([TERTIUM, TERTIUM, TERTIUM, TERTIUM, IRON, IRON, IRON, IRON]);
        assert!(infusion_matches(&center, &ings, &input, &laid, fits));
        assert!(infusion_matches(&center, &ings, &input, &grouped, fits));
        // Seven items (one pedestal empty or missing): no.
        let mut seven = laid.clone();
        seven[5] = None;
        assert!(!infusion_matches(&center, &ings, &input, &seven, fits));
        assert!(!infusion_matches(&center, &ings, &input, &laid[..7], fits));
        // Five irons and three essences: no.
        let wrong = eight([IRON, IRON, IRON, TERTIUM, IRON, TERTIUM, IRON, TERTIUM]);
        assert!(!infusion_matches(&center, &ings, &input, &wrong, fits));
        // Another item in place of one: no.
        let other = eight([
            IRON,
            TERTIUM,
            IRON,
            TERTIUM,
            IRON,
            TERTIUM,
            "minecraft:gold_ingot",
            TERTIUM,
        ]);
        assert!(!infusion_matches(&center, &ings, &input, &other, fits));
        // The wrong seed base on the altar: no.
        assert!(!infusion_matches(
            &center,
            &ings,
            &S("minecraft:wheat_seeds", 1),
            &laid,
            fits
        ));
    }

    #[test]
    fn a_tag_ingredient_does_not_steal_a_stack_another_needs() {
        // "#logs" (oak or birch) and plain oak: a greedy pass giving the oak to the tag would
        // leave the plain ingredient nothing; the matcher swaps.
        let logs: &'static [&'static str] = &["minecraft:oak_log", "minecraft:birch_log"];
        let ings = vec![Ing::Any(logs), Ing::Item("minecraft:oak_log")];
        let stacks = [S("minecraft:oak_log", 1), S("minecraft:birch_log", 1)];
        let m = match_shapeless(2, 2, |i, s| fits(&ings[i], &stacks[s])).unwrap();
        assert_eq!(m, vec![1, 0]);
        assert!(match_shapeless(2, 2, |_, s| s == 0).is_none());
        assert!(match_shapeless(2, 3, |_, _| true).is_none());
    }

    #[test]
    fn the_altar_works_100_ticks_once_powered() {
        let mut a = Altar::default();
        // Not powered: nothing, the recipe isn't even looked up.
        assert_eq!(
            altar_step(&mut a, true, false, || -> Option<u8> { panic!("asked") }),
            AltarStep::Idle
        );
        // A pulse switches it on; it stays on without the signal.
        assert_eq!(
            altar_step(&mut a, true, true, || Some(7u8)),
            AltarStep::Working
        );
        for _ in 1..INFUSION_TICKS - 1 {
            assert_eq!(
                altar_step(&mut a, true, false, || Some(7u8)),
                AltarStep::Working
            );
        }
        assert_eq!(a.progress, INFUSION_TICKS - 1);
        assert_eq!(
            altar_step(&mut a, true, false, || Some(7u8)),
            AltarStep::Craft(7)
        );
        assert_eq!(a, Altar::default());
        // Then off again: the next craft needs another signal.
        assert_eq!(
            altar_step(&mut a, true, false, || Some(7u8)),
            AltarStep::Idle
        );
    }

    #[test]
    fn the_altar_resets_when_its_recipe_or_input_goes() {
        let mut a = Altar::default();
        altar_step(&mut a, true, true, || Some(()));
        altar_step(&mut a, true, false, || Some(()));
        assert_eq!(a.progress, 2);
        // A pedestal emptied: no recipe, off and back to 0.
        assert_eq!(
            altar_step(&mut a, true, false, || None::<()>),
            AltarStep::Idle
        );
        assert_eq!(a, Altar::default());
        // The wand (active saved) with the input taken: reset.
        a.active = true;
        a.progress = 50;
        assert_eq!(
            altar_step(&mut a, false, true, || Some(())),
            AltarStep::Idle
        );
        assert_eq!(a, Altar::default());
    }

    fn total(slots: &[Option<S>]) -> std::collections::BTreeMap<&'static str, u32> {
        let mut m = std::collections::BTreeMap::new();
        for s in slots.iter().flatten() {
            *m.entry(s.0).or_insert(0) += s.1;
        }
        m
    }

    #[test]
    fn a_craft_uses_nine_items_and_makes_one() {
        let mut input = Some(S(BASE, 1));
        let mut output = None;
        let mut peds = eight([IRON, TERTIUM, IRON, TERTIUM, IRON, TERTIUM, IRON, TERTIUM]);
        let result = S("mysticalagriculture:iron_seeds", 1);
        assert!(infuse(
            &mut input,
            &mut output,
            &mut peds,
            result.clone(),
            |_| None
        ));
        assert_eq!(input, None);
        assert_eq!(output, Some(result));
        assert!(peds.iter().all(Option::is_none));
    }

    #[test]
    fn a_craft_keeps_remainders_and_never_duplicates() {
        // A bucket of water on a pedestal leaves its bucket; two irons on one leave one.
        let mut input = Some(S(BASE, 1));
        let mut output = None;
        let mut peds = vec![
            Some(S("minecraft:water_bucket", 1)),
            Some(S(IRON, 2)),
            None,
            Some(S(TERTIUM, 1)),
        ];
        let before = total(&peds);
        let rem = |s: &S| (s.0 == "minecraft:water_bucket").then_some(S("minecraft:bucket", 1));
        assert!(infuse(
            &mut input,
            &mut output,
            &mut peds,
            S("x:result", 1),
            rem
        ));
        assert_eq!(peds[0], Some(S("minecraft:bucket", 1)));
        assert_eq!(peds[1], Some(S(IRON, 1)));
        assert_eq!(peds[2], None);
        assert_eq!(peds[3], None);
        // Each filled pedestal lost exactly one item (a remainder stands in for it).
        let after = total(&peds);
        assert_eq!(
            before.values().sum::<u32>() - 3 + 1,
            after.values().sum::<u32>()
        );
        // With the output taken nothing happens.
        let mut input = Some(S(BASE, 1));
        let mut output = Some(S("x:result", 1));
        let mut peds = eight([IRON; 8]);
        let snapshot = (input.clone(), output.clone(), peds.clone());
        assert!(!infuse(
            &mut input,
            &mut output,
            &mut peds,
            S("x:result", 1),
            |_| None
        ));
        assert_eq!((input, output, peds), snapshot);
        // Nor without an input.
        let mut input = None;
        let mut output = None;
        let mut peds = eight([IRON; 8]);
        assert!(!infuse(
            &mut input,
            &mut output,
            &mut peds,
            S("x:result", 1),
            |_| None
        ));
        assert_eq!(output, None);
        assert_eq!(total(&peds).get(IRON), Some(&8));
    }

    #[test]
    fn clicks_move_one_item_and_never_lose_one() {
        // Altar: the output first, then insert, then take back; the wand switches on.
        assert_eq!(altar_click(false, true, Some(IRON)), Click::TakeOutput);
        assert_eq!(altar_click(true, true, None), Click::TakeOutput);
        assert_eq!(altar_click(false, false, Some(BASE)), Click::Insert);
        assert_eq!(altar_click(true, false, Some(IRON)), Click::TakeInput);
        assert_eq!(altar_click(true, false, None), Click::TakeInput);
        assert_eq!(altar_click(true, false, Some(WAND)), Click::Activate);
        assert_eq!(altar_click(false, false, Some(WAND)), Click::Insert);
        assert_eq!(altar_click(false, false, None), Click::Nothing);
        // Pedestal.
        assert_eq!(pedestal_click(false, true), Click::Insert);
        assert_eq!(pedestal_click(true, true), Click::TakeInput);
        assert_eq!(pedestal_click(true, false), Click::TakeInput);
        assert_eq!(pedestal_click(false, false), Click::Nothing);
        // Insert then take: the player ends with what they had.
        let mut hand = Some(S(IRON, 5));
        let mut slot: Option<S> = None;
        if pedestal_click(slot.is_some(), hand.is_some()) == Click::Insert {
            let h = hand.take().unwrap();
            slot = Some(h.with_count(1));
            hand = Some(h.with_count(h.1 - 1));
        }
        assert_eq!(
            (hand.clone(), slot.clone()),
            (Some(S(IRON, 4)), Some(S(IRON, 1)))
        );
        if pedestal_click(slot.is_some(), hand.is_some()) == Click::TakeInput {
            let s = slot.take().unwrap();
            hand = hand.map(|h| h.with_count(h.1 + s.1));
        }
        assert_eq!((hand, slot), (Some(S(IRON, 5)), None));
    }

    #[test]
    fn shown_items_float_where_the_renderers_put_them() {
        assert_eq!(shown_slot(INFUSION_ALTAR, &[true, true]), Some((1, 1.1)));
        assert_eq!(shown_slot(INFUSION_ALTAR, &[true, false]), Some((0, 1.1)));
        assert_eq!(shown_slot(INFUSION_ALTAR, &[false, false]), None);
        assert_eq!(shown_slot(INFUSION_PEDESTAL, &[true]), Some((0, 1.2)));
        assert_eq!(shown_slot(INFUSION_PEDESTAL, &[]), None);
        assert_eq!(shown_slot("minecraft:stone", &[true]), None);
        let (s, bob, yaw) = float_pose(0.0, true);
        assert_eq!((s, bob, yaw), (0.95, 0.0, 0.0));
        let (s, bob, yaw) = float_pose(16.0, false);
        assert_eq!(s, 0.75);
        assert!(bob.abs() <= 0.065 && (yaw - 40.0).abs() < 1e-4);
        assert_eq!(infusion_slots(INFUSION_ALTAR), 2);
        assert_eq!(infusion_slots(INFUSION_PEDESTAL), 1);
        assert!(is_infusion_block(INFUSION_PEDESTAL) && !is_infusion_block(WAND));
        assert_eq!(shown_signature(["a:b", ""].into_iter()), "a:b|");
        assert_ne!(
            shown_signature(["a:b", ""].into_iter()),
            shown_signature(["", "a:b"].into_iter())
        );
    }

    #[test]
    fn watering_cans() {
        let base = watering_can("mysticalagriculture:watering_can").unwrap();
        assert_eq!((base.range, watering_radius(base.range)), (3, 1));
        let top = watering_can("mysticalagriculture:supremium_watering_can").unwrap();
        assert_eq!(
            (top.range, watering_radius(top.range), top.chance),
            (11, 5, 0.45)
        );
        assert!(watering_can("minecraft:bucket").is_none());
        assert!(may_water(None, 0));
        assert!(!may_water(Some(100), 104));
        assert!(may_water(Some(100), 105));
        assert!(may_water(Some(100), 50)); // a world reloaded with an earlier time
        assert!(is_farmland("minecraft:farmland"));
        assert!(is_farmland("mysticalagriculture:tertium_farmland"));
        assert!(!is_farmland("minecraft:dirt"));
        assert!(is_growable("mysticalagriculture:iron_crop"));
        assert!(is_growable("minecraft:wheat"));
        assert!(is_growable("minecraft:oak_sapling"));
        assert!(!is_growable("minecraft:oak_leaves"));
        assert!(!is_growable("minecraft:farmland"));
        assert!(!is_growable("minecraft:stone"));
    }
}

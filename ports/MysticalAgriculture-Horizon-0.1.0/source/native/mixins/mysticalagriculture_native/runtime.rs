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
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Essence farmland bonus, Inferium from mobs, cruxes, fertilizers, \
farmland conversion and infusion crystals for the Mystical Agriculture port.";

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
}

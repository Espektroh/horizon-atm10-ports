// Sushi Go Crafting for Horizon: the rules, with no game types (std only), so they are tested on
// their own (`cargo test --lib`). Added to the game as `crate::game::sushigocrafting_rt`; the
// glue reads the world and calls these.
//
// Numbers from Sushi Go Crafting 0.6.7 (Buuz135, MIT, branch 1.21): `SushiContent` (the amount
// items' grams), `FoodIngredient` (default amounts), `CustomCropBlock` / vanilla `CropBlock`
// (growth), `SaplingBlock`, `AvocadoLeavesBlock`, `SeedsLootModifier`, `CuttingBoardTile`,
// `RiceCookerTile`, `FermentationBarrelTile` and the `avocado_tree` configured feature.

use std::sync::atomic::{AtomicU64, Ordering};

pub const ID: &str = "sushigocrafting_native";
pub const NAME: &str = "Sushi Go Crafting";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Crops (rice in water), avocado trees, grass seeds, tobiko from fish, \
tuna and shrimp from fishing, the cutting board, the rice cooker, the fermentation barrel and the \
roller that stays in the grid, for the Sushi Go Crafting port.";

pub const CUTTING_BOARD: &str = "sushigocrafting:cutting_board";
pub const RICE_COOKER: &str = "sushigocrafting:rice_cooker";
pub const FERMENTATION_BARREL: &str = "sushigocrafting:fermentation_barrel";
pub const CLEAVER_KNIFE: &str = "sushigocrafting:cleaver_knife";
pub const RICE_CROP: &str = "sushigocrafting:rice_crop";
pub const AVOCADO_SAPLING: &str = "sushigocrafting:avocado_sapling";
pub const AVOCADO_LEAVES: &str = "sushigocrafting:avocado_leaves";
pub const AVOCADO_LOG: &str = "sushigocrafting:avocado_log";
pub const AVOCADO: &str = "sushigocrafting:avocado";
pub const RICE: &str = "sushigocrafting:rice";
pub const COOKED_RICE: &str = "sushigocrafting:cooked_rice";
pub const TOBIKO: &str = "sushigocrafting:tobiko";
pub const RAW_TUNA: &str = "sushigocrafting:raw_tuna";
pub const SHRIMP: &str = "sushigocrafting:shrimp";
pub const CUTTING_TYPE: &str = "sushigocrafting:cutting_board";
pub const FERMENTING_TYPE: &str = "sushigocrafting:fermenting_barrel";
pub const WATER: &str = "minecraft:water";
pub const MILK: &str = "minecraft:milk";

/// The crops that grow on farmland (template `minecraft:beetroots`, age 0..3).
pub const FARM_CROPS: [&str; 4] = [
    "sushigocrafting:cucumber_crop",
    "sushigocrafting:soy_crop",
    "sushigocrafting:wasabi_crop",
    "sushigocrafting:sesame_crop",
];
pub const MAX_AGE: u32 = 3;

pub fn is_crop(block: &str) -> bool {
    block == RICE_CROP || FARM_CROPS.contains(&block)
}

// ---------------------------------------------------------------- the random source

static SEED: AtomicU64 = AtomicU64::new(0);

/// A uniform number in [0, 1) (splitmix64 over a shared counter, seeded from the clock).
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

/// `Random.nextInt(n)` from a roll: 0..n-1 (0 when n is 0).
pub fn next_int(r: f64, n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    ((r * f64::from(n)) as u32).min(n - 1)
}

// ---------------------------------------------------------------- portions

/// The amount items (`AmountItem`): id, min grams, max grams (exclusive, as
/// `nextInt(max - min) + min`), the most one stack combines, and the grams one portion is
/// (`FoodIngredient.defaultAmount`, what the Roller takes at the highest weight).
pub const AMOUNTS: [(&str, u32, u32, u32, u32); 12] = [
    ("sushigocrafting:avocado_slices", 100, 500, 1000, 45),
    ("sushigocrafting:tuna_fillet", 1000, 3000, 6000, 30),
    ("sushigocrafting:salmon_fillet", 500, 2000, 4000, 30),
    ("sushigocrafting:cooked_rice", 50, 500, 2000, 30),
    ("sushigocrafting:cucumber_slices", 50, 200, 400, 15),
    ("sushigocrafting:imitation_crab", 50, 200, 400, 30),
    ("sushigocrafting:sesame_seed", 10, 100, 200, 6),
    ("sushigocrafting:tobiko", 10, 50, 100, 12),
    ("sushigocrafting:cheese", 50, 250, 500, 12),
    ("sushigocrafting:shrimp", 20, 50, 100, 30),
    ("sushigocrafting:soy_sauce", 10, 50, 100, 4),
    ("sushigocrafting:wasabi_paste", 10, 50, 100, 4),
];
/// A stack's size.
pub const MAX_PORTIONS: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Amount {
    pub min: u32,
    pub max: u32,
    pub combine: u32,
    pub portion: u32,
}

pub fn amount_of(item: &str) -> Option<Amount> {
    AMOUNTS.iter().find(|a| a.0 == item).map(|a| Amount {
        min: a.1,
        max: a.2,
        combine: a.3,
        portion: a.4,
    })
}

/// Grams as portions: rounded, at least 1, at most a stack.
pub fn portions_of(grams: u32, portion: u32) -> u32 {
    if portion == 0 {
        return 1;
    }
    ((grams + portion / 2) / portion).clamp(1, MAX_PORTIONS)
}

/// `AmountItem.random`: grams in [min, max), as portions.
pub fn roll_portions(a: Amount, r: f64) -> u32 {
    let grams = a.min + next_int(r, a.max.saturating_sub(a.min));
    portions_of(grams, a.portion)
}

/// The most portions one output slot holds (`getMaxCombineAmount`, a stack at most).
pub fn portion_cap(a: Amount) -> u32 {
    portions_of(a.combine, a.portion)
}

// ---------------------------------------------------------------- grass seeds

/// `SeedsLootModifier`: what a wheat seed from grass becomes, by weight.
pub const GRASS_SEEDS: [(&str, u32); 6] = [
    ("minecraft:wheat_seeds", 100),
    ("sushigocrafting:rice_seeds", 25),
    ("sushigocrafting:sesame_seeds", 10),
    ("sushigocrafting:wasabi_seeds", 25),
    ("sushigocrafting:soy_seeds", 25),
    ("sushigocrafting:cucumber_seeds", 25),
];

pub fn is_grass(block: &str) -> bool {
    matches!(
        block,
        "minecraft:short_grass"
            | "minecraft:fern"
            | "minecraft:tall_grass"
            | "minecraft:large_fern"
    )
}

/// `WeightedRandom.getRandomItem`.
pub fn grass_seed(r: f64) -> &'static str {
    let total: u32 = GRASS_SEEDS.iter().map(|s| s.1).sum();
    let mut n = next_int(r, total);
    for (id, w) in GRASS_SEEDS {
        if n < w {
            return id;
        }
        n -= w;
    }
    GRASS_SEEDS[0].0
}

/// `SushiGoCrafting`'s `LivingDropsEvent`: every `AbstractFish` drops tobiko 3 in 10 times.
pub const TOBIKO_CHANCE: f64 = 0.3;
pub fn is_fish(kind: &str) -> bool {
    matches!(
        kind,
        "minecraft:cod" | "minecraft:salmon" | "minecraft:pufferfish" | "minecraft:tropical_fish"
    )
}

/// Tuna and shrimp have no entities here: a fishing rod's catch brings one, this often.
pub const TUNA_FISHING_CHANCE: f64 = 0.15;
pub const SHRIMP_FISHING_CHANCE: f64 = 0.15;

// ---------------------------------------------------------------- crop growth

/// One of the 3x3 blocks under a crop, as `CropBlock.getGrowthSpeed` sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Soil {
    Other,
    Farmland,
    Moist,
}

/// `CropBlock.getGrowthSpeed`: `soil` row by row (x -1..1 outer, z -1..1 inner, centre 4);
/// `rows` is true when a neighbour east or west and one north or south are the same crop,
/// `diagonal` when a diagonal one is.
pub fn growth_speed(soil: &[Soil; 9], rows: bool, diagonal: bool) -> f32 {
    let mut f = 1.0f32;
    for (i, s) in soil.iter().enumerate() {
        let mut g = match s {
            Soil::Other => 0.0,
            Soil::Farmland => 1.0,
            Soil::Moist => 3.0,
        };
        if i != 4 {
            g /= 4.0;
        }
        f += g;
    }
    if diagonal || rows {
        f /= 2.0;
    }
    f
}

/// `CropBlock.randomTick`: a crop below its last age grows one age when the light is 9 or more
/// and `nextInt((int)(25 / f) + 1) == 0`.
pub fn crop_grows(age: u32, light: i32, speed: f32, r: f64) -> bool {
    if age >= MAX_AGE || light < 9 || speed <= 0.0 {
        return false;
    }
    let n = (25.0 / speed) as u32 + 1;
    next_int(r, n) == 0
}

/// `CropBlock.getBonemealAgeIncrease` / 3 (`CustomCropBlock`): `nextInt(2, 5) / 3`.
pub fn bonemeal_ages(r: f64) -> u32 {
    (2 + next_int(r, 4)) / 3
}

// ---------------------------------------------------------------- avocado

/// `SaplingBlock.randomTick`: light 9 or more above, then 1 in 7.
pub fn sapling_advances(light_above: i32, r: f64) -> bool {
    light_above >= 9 && next_int(r, 7) == 0
}
/// `SaplingBlock.isBonemealSuccess`.
pub const SAPLING_BONEMEAL_CHANCE: f64 = 0.45;

/// `AvocadoLeavesBlock.randomTick`: stage 1 ripens to 2, 1 in 3.
pub fn leaves_ripen(stage: u32, r: f64) -> bool {
    stage == 1 && next_int(r, 3) == 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreePart {
    Log,
    /// Leaves with their distance to the log (1..=6) and stage (0, 1 or 2).
    Leaves {
        distance: u32,
        stage: u32,
    },
}

/// The `avocado_tree` feature grown from a sapling at the origin: a straight trunk of
/// `2 + nextInt(2)` logs (`straight_trunk_placer` 2, 1, 0), and the blob foliage (radius 2, height
/// 1, offset 0): two 3x3 layers at the trunk's top and one above, each corner left out half the
/// time; leaves weighted 2:1:1 to stages 0, 1, 2. `rolls` gives the numbers in order.
pub fn avocado_tree(mut rolls: impl FnMut() -> f64) -> Vec<([i32; 3], TreePart)> {
    let height = 2 + next_int(rolls(), 2) as i32;
    let mut out: Vec<([i32; 3], TreePart)> =
        (0..height).map(|y| ([0, y, 0], TreePart::Log)).collect();
    // Foliage attachment: one above the last log (y = height); layers y = height and height - 1.
    for y in [height, height - 1] {
        for x in -1..=1i32 {
            for z in -1..=1i32 {
                if x == 0 && z == 0 && y < height {
                    continue; // the trunk
                }
                if x.abs() == 1 && z.abs() == 1 && next_int(rolls(), 2) == 0 {
                    continue;
                }
                let stage = match next_int(rolls(), 4) {
                    0 | 1 => 0,
                    2 => 1,
                    _ => 2,
                };
                out.push(([x, y, z], TreePart::Leaves { distance: 7, stage }));
            }
        }
    }
    // Distances: 1 beside a log, else one more than the nearest leaf (as `LeavesBlock` settles).
    let logs: Vec<[i32; 3]> = out
        .iter()
        .filter(|(_, p)| *p == TreePart::Log)
        .map(|(p, _)| *p)
        .collect();
    let near = |a: [i32; 3], b: [i32; 3]| {
        (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs() == 1
    };
    let mut dist: Vec<u32> = out
        .iter()
        .map(|(p, part)| match part {
            TreePart::Log => 0,
            TreePart::Leaves { .. } => {
                if logs.iter().any(|l| near(*l, *p)) {
                    1
                } else {
                    7
                }
            }
        })
        .collect();
    for _ in 0..6 {
        for i in 0..out.len() {
            if dist[i] == 0 {
                continue;
            }
            let best = (0..out.len())
                .filter(|&j| near(out[i].0, out[j].0))
                .map(|j| dist[j] + 1)
                .min()
                .unwrap_or(7);
            dist[i] = dist[i].min(best).min(7);
        }
    }
    for (i, (_, part)) in out.iter_mut().enumerate() {
        if let TreePart::Leaves { distance, .. } = part {
            *distance = dist[i].clamp(1, 7);
        }
    }
    out
}

// ---------------------------------------------------------------- cutting board

/// `CuttingBoardTile.onActivated`: the sixth knife click on a filled board cuts.
pub const CUTS: u32 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardClick {
    /// Puts the whole held stack on the empty board.
    Insert,
    /// One knife click; `true` when this one cuts.
    Knife(bool),
    /// Sneaking with an empty hand: the board's stack comes back.
    Take,
    Nothing,
}

/// What a click on the board does: `held` is what the hand holds (None: empty), `cuttable` whether
/// a recipe takes it, `filled` whether the board holds something, `clicks` its count so far.
pub fn board_click(
    held: Option<&str>,
    cuttable: bool,
    filled: bool,
    sneaking: bool,
    clicks: u32,
) -> BoardClick {
    match held {
        Some(item) if filled && item == CLEAVER_KNIFE => BoardClick::Knife(clicks + 1 >= CUTS),
        Some(_) if !filled && cuttable => BoardClick::Insert,
        Some(_) => BoardClick::Nothing,
        None if sneaking && filled => BoardClick::Take,
        None => BoardClick::Nothing,
    }
}

// ---------------------------------------------------------------- machines

/// `ProgressBarComponent(…, 100)`: 100 ticks of work.
pub const WORK_TICKS: i32 = 100;
pub const COOKER_SLOTS: usize = 3;
pub const INPUT: usize = 0;
pub const FUEL: usize = 1;
pub const OUTPUT: usize = 2;
pub const COOKER_WATER_CAPACITY: i32 = 8000;
pub const COOKER_WATER_PER_BATCH: i32 = 1000;
/// Four input slots of one rice each in the original: a batch takes up to four.
pub const COOKER_RICE_PER_BATCH: u32 = 4;
/// Grams of cooked rice per rice (`getSlotsFilled() * 50`).
pub const COOKER_GRAMS_PER_RICE: u32 = 50;
/// Fuel: burn time / 200 batches (`burnTime += getBurnTime / 200D`), here in fuel ticks.
pub const BURN_PER_BATCH: i32 = 200;
pub const BARREL_CAPACITY: i32 = 2000;
pub const BUCKET: i32 = 1000;

/// A slot as the rules see it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub item: String,
    pub count: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cooker {
    pub water: i32,
    /// Fuel ticks left (200 is a batch); may go below 0, as the original's double does.
    pub burn: i32,
    pub progress: i32,
}

/// Cooked rice portions for `rice` rice in one batch.
pub fn cooked_rice_portions(rice: u32) -> u32 {
    let a = amount_of(COOKED_RICE).expect("cooked rice is an amount item");
    portions_of(rice * COOKER_GRAMS_PER_RICE, a.portion)
}

/// One tick of the rice cooker (`RiceCookerTile.canStart` and the bar's finish).
/// `burn_time(item)`: the fuel's burn ticks (0: not a fuel); `remainder(item)`: what a used fuel
/// leaves (a bucket). True while it works.
pub fn cooker_tick(
    s: &mut Cooker,
    slots: &mut [Option<Slot>],
    burn_time: impl Fn(&str) -> i32,
    remainder: impl Fn(&str) -> Option<String>,
) -> bool {
    let rice = slots[INPUT]
        .as_ref()
        .filter(|r| r.item == RICE)
        .map_or(0, |r| r.count);
    let cap = amount_of(COOKED_RICE).map_or(MAX_PORTIONS, portion_cap);
    let out_ok = match &slots[OUTPUT] {
        None => true,
        Some(o) => o.item == COOKED_RICE && o.count < cap,
    };
    // canStart: refill the burn first, as the original does on every check.
    if s.burn < BURN_PER_BATCH
        && let Some(fuel) = slots[FUEL].clone()
    {
        let t = burn_time(&fuel.item);
        if t > 0 {
            s.burn += t;
            slots[FUEL] = match remainder(&fuel.item) {
                Some(rest) => Some(Slot {
                    item: rest,
                    count: 1,
                }),
                None if fuel.count > 1 => Some(Slot {
                    item: fuel.item,
                    count: fuel.count - 1,
                }),
                None => None,
            };
        }
    }
    let can = s.burn > 0 && rice > 0 && out_ok && s.water >= COOKER_WATER_PER_BATCH;
    if !can {
        return false;
    }
    s.progress += 1;
    if s.progress >= WORK_TICKS {
        let n = rice.min(COOKER_RICE_PER_BATCH);
        let made = cooked_rice_portions(n);
        let have = slots[OUTPUT].as_ref().map_or(0, |o| o.count);
        slots[OUTPUT] = Some(Slot {
            item: COOKED_RICE.into(),
            count: (have + made).min(cap),
        });
        slots[INPUT] = if rice > n {
            Some(Slot {
                item: RICE.into(),
                count: rice - n,
            })
        } else {
            None
        };
        s.water -= COOKER_WATER_PER_BATCH;
        s.burn -= BURN_PER_BATCH;
        s.progress = 0;
    }
    true
}

/// A fermenting recipe as the glue reads it: whether `input` matches (None: the recipe takes no
/// item, which matches an empty slot, as `Ingredient.EMPTY.test`), its fluid and amount, and
/// what it makes.
#[derive(Clone, Debug, PartialEq)]
pub struct Ferment {
    pub takes_item: bool,
    pub fluid: String,
    pub amount: i32,
    pub output: String,
    pub count: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Barrel {
    /// 0 empty, 1 water, 2 milk.
    pub fluid: u8,
    pub amount: i32,
    pub progress: i32,
}

pub fn fluid_code(id: &str) -> u8 {
    match id {
        WATER => 1,
        MILK => 2,
        _ => 0,
    }
}

/// What a bucket puts in a tank: the fluid's code (water and milk only).
pub fn bucket_fluid(item: &str) -> u8 {
    match item {
        "minecraft:water_bucket" => 1,
        "minecraft:milk_bucket" => 2,
        _ => 0,
    }
}

/// Whether a bucket of `fluid` goes into a tank holding `have` of `held` (`capacity` mB).
pub fn tank_takes(held: u8, have: i32, fluid: u8, capacity: i32) -> bool {
    fluid != 0 && (held == 0 || have <= 0 || held == fluid) && have + BUCKET <= capacity
}

/// Output room: an empty slot, or the same item under its cap (`canStack`).
fn barrel_room(out: &Option<Slot>, r: &Ferment) -> bool {
    match out {
        None => true,
        Some(o) => {
            o.item == r.output && o.count < amount_of(&r.output).map_or(MAX_PORTIONS, portion_cap)
        }
    }
}

/// One tick of the fermentation barrel. `recipes`: each recipe and whether the input slot fits
/// it (for one that takes an item). The output's count is rolled for an amount item.
pub fn barrel_tick(
    s: &mut Barrel,
    slots: &mut [Option<Slot>],
    recipes: &[(Ferment, bool)],
    r: f64,
) -> bool {
    let empty_input = slots[INPUT].is_none();
    let found = recipes.iter().find(|(f, fits)| {
        let item_ok = if f.takes_item { *fits } else { empty_input };
        let fluid_ok =
            f.fluid.is_empty() || (fluid_code(&f.fluid) == s.fluid && s.amount >= f.amount);
        item_ok && fluid_ok && barrel_room(&slots[OUTPUT], f)
    });
    let Some((f, _)) = found else {
        return false;
    };
    s.progress += 1;
    if s.progress < WORK_TICKS {
        return true;
    }
    s.progress = 0;
    if f.takes_item {
        slots[INPUT] = match slots[INPUT].take() {
            Some(i) if i.count > 1 => Some(Slot {
                item: i.item,
                count: i.count - 1,
            }),
            _ => None,
        };
    }
    if !f.fluid.is_empty() {
        s.amount -= f.amount;
        if s.amount <= 0 {
            s.amount = 0;
            s.fluid = 0;
        }
    }
    let (made, cap) = match amount_of(&f.output) {
        Some(a) => (roll_portions(a, r), portion_cap(a)),
        None => (f.count.max(1), MAX_PORTIONS),
    };
    let have = slots[OUTPUT].as_ref().map_or(0, |o| o.count);
    slots[OUTPUT] = Some(Slot {
        item: f.output.clone(),
        count: (have + made).min(cap),
    });
    true
}

/// Permille of `a` over `b` for a screen's bar.
pub fn permille(a: i32, b: i32) -> i32 {
    if b <= 0 {
        0
    } else {
        (i64::from(a) * 1000 / i64::from(b)).clamp(0, 1000) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(item: &str, count: u32) -> Option<Slot> {
        Some(Slot {
            item: item.into(),
            count,
        })
    }

    #[test]
    fn portions_follow_the_grams() {
        let salmon = amount_of("sushigocrafting:salmon_fillet").unwrap();
        assert_eq!(roll_portions(salmon, 0.0), 17); // 500 g / 30
        assert_eq!(roll_portions(salmon, 0.999_999), 64); // 1999 g, a stack at most
        let avocado = amount_of("sushigocrafting:avocado_slices").unwrap();
        assert_eq!(roll_portions(avocado, 0.0), 2); // 100 g / 45
        assert_eq!(roll_portions(avocado, 0.999_999), 11); // 499 g
        let shrimp = amount_of(SHRIMP).unwrap();
        assert_eq!(
            (roll_portions(shrimp, 0.0), roll_portions(shrimp, 0.999)),
            (1, 2)
        );
        let soy = amount_of("sushigocrafting:soy_sauce").unwrap();
        assert_eq!(roll_portions(soy, 0.0), 3); // 10 g / 4 rounds up
        assert_eq!(portion_cap(soy), 25);
        assert_eq!(portion_cap(amount_of(COOKED_RICE).unwrap()), 64);
        assert!(amount_of(RICE).is_none());
    }

    #[test]
    fn grass_seeds_by_weight() {
        let total = 210.0;
        assert_eq!(grass_seed(0.0), "minecraft:wheat_seeds");
        assert_eq!(grass_seed(99.5 / total), "minecraft:wheat_seeds");
        assert_eq!(grass_seed(100.5 / total), "sushigocrafting:rice_seeds");
        assert_eq!(grass_seed(125.5 / total), "sushigocrafting:sesame_seeds");
        assert_eq!(grass_seed(135.5 / total), "sushigocrafting:wasabi_seeds");
        assert_eq!(grass_seed(160.5 / total), "sushigocrafting:soy_seeds");
        assert_eq!(grass_seed(209.5 / total), "sushigocrafting:cucumber_seeds");
        let mut counts = [0u32; 6];
        for i in 0..21_000 {
            let s = grass_seed(f64::from(i) / 21_000.0);
            counts[GRASS_SEEDS.iter().position(|g| g.0 == s).unwrap()] += 1;
        }
        assert_eq!(counts, [10_000, 2_500, 1_000, 2_500, 2_500, 2_500]);
    }

    #[test]
    fn growth_speed_as_crop_block() {
        let none = [Soil::Other; 9];
        assert_eq!(growth_speed(&none, false, false), 1.0);
        let moist = [Soil::Moist; 9];
        assert_eq!(growth_speed(&moist, false, false), 10.0);
        assert_eq!(growth_speed(&moist, true, false), 5.0);
        let mut dry = [Soil::Other; 9];
        dry[4] = Soil::Farmland;
        assert_eq!(growth_speed(&dry, false, false), 2.0);
        // 25 / 10 = 2.5 -> nextInt(3): a third of the ticks.
        assert!(crop_grows(0, 15, 10.0, 0.0));
        assert!(!crop_grows(0, 15, 10.0, 0.5));
        // On dirt (rice): nextInt(26).
        assert!(crop_grows(1, 9, 1.0, 0.03));
        assert!(!crop_grows(1, 9, 1.0, 0.04));
        assert!(!crop_grows(0, 8, 10.0, 0.0), "dark");
        assert!(!crop_grows(3, 15, 10.0, 0.0), "ripe");
    }

    #[test]
    fn bonemeal_is_a_quarter_nothing() {
        assert_eq!(bonemeal_ages(0.0), 0); // 2 / 3
        assert_eq!(bonemeal_ages(0.3), 1); // 3 / 3
        assert_eq!(bonemeal_ages(0.99), 1); // 5 / 3
    }

    #[test]
    fn avocado_tree_shape() {
        // All zero rolls: height 2, every corner left out, stage 0.
        let t = avocado_tree(|| 0.0);
        let logs: Vec<_> = t.iter().filter(|p| p.1 == TreePart::Log).collect();
        assert_eq!(logs.len(), 2);
        let leaves: Vec<_> = t.iter().filter(|p| p.1 != TreePart::Log).collect();
        assert_eq!(leaves.len(), 5 + 4); // top layer: plus shape; trunk layer: 4 sides
        for (p, part) in &t {
            if let TreePart::Leaves { distance, stage } = part {
                assert_eq!(*stage, 0);
                assert!((1..=2).contains(distance), "{p:?} {distance}");
                if *p == [0, 2, 0] || p[1] == 1 {
                    assert_eq!(*distance, 1, "{p:?}");
                }
            }
        }
        // Rolls near one: height 3, every corner kept, stage 2.
        let t = avocado_tree(|| 0.99);
        assert_eq!(t.iter().filter(|p| p.1 == TreePart::Log).count(), 3);
        assert_eq!(t.len(), 3 + 9 + 8);
        let corner = t.iter().find(|p| p.0 == [1, 3, 1]).unwrap();
        assert_eq!(
            corner.1,
            TreePart::Leaves {
                distance: 3,
                stage: 2
            }
        );
    }

    #[test]
    fn board_clicks() {
        assert_eq!(
            board_click(Some("minecraft:salmon"), true, false, false, 0),
            BoardClick::Insert
        );
        assert_eq!(
            board_click(Some("minecraft:dirt"), false, false, false, 0),
            BoardClick::Nothing
        );
        assert_eq!(
            board_click(Some(CLEAVER_KNIFE), false, true, false, 0),
            BoardClick::Knife(false)
        );
        assert_eq!(
            board_click(Some(CLEAVER_KNIFE), false, true, false, 5),
            BoardClick::Knife(true)
        );
        assert_eq!(
            board_click(Some(CLEAVER_KNIFE), false, false, false, 0),
            BoardClick::Nothing
        );
        assert_eq!(board_click(None, false, true, true, 2), BoardClick::Take);
        assert_eq!(
            board_click(None, false, true, false, 2),
            BoardClick::Nothing
        );
    }

    #[test]
    fn rice_cooker_makes_rice_in_100_ticks() {
        let mut s = Cooker {
            water: 2000,
            ..Default::default()
        };
        let mut slots = vec![slot(RICE, 6), slot("minecraft:coal", 1), None];
        let burn = |i: &str| if i == "minecraft:coal" { 1600 } else { 0 };
        for t in 1..=99 {
            assert!(cooker_tick(&mut s, &mut slots, burn, |_| None), "tick {t}");
        }
        assert!(slots[OUTPUT].is_none());
        assert!(slots[FUEL].is_none(), "the coal went in at once");
        assert_eq!(s.burn, 1600);
        assert!(cooker_tick(&mut s, &mut slots, burn, |_| None));
        // 4 rice: 200 g / 30 = 6.67 -> 7 portions; 2 rice left, 1000 mB and one batch of fuel used.
        assert_eq!(slots[OUTPUT], slot(COOKED_RICE, 7));
        assert_eq!(slots[INPUT], slot(RICE, 2));
        assert_eq!((s.water, s.burn, s.progress), (1000, 1400, 0));
        for _ in 0..100 {
            cooker_tick(&mut s, &mut slots, burn, |_| None);
        }
        // 2 rice: 100 g -> 3 portions.
        assert_eq!(slots[OUTPUT], slot(COOKED_RICE, 10));
        assert!(slots[INPUT].is_none());
        assert_eq!(s.water, 0);
        slots[INPUT] = slot(RICE, 1);
        assert!(!cooker_tick(&mut s, &mut slots, burn, |_| None), "no water");
    }

    #[test]
    fn rice_cooker_needs_fuel_and_leaves_the_bucket() {
        let mut s = Cooker {
            water: 1000,
            ..Default::default()
        };
        let mut slots = vec![slot(RICE, 1), None, None];
        assert!(!cooker_tick(&mut s, &mut slots, |_| 0, |_| None));
        slots[FUEL] = slot("minecraft:lava_bucket", 1);
        let burn = |i: &str| {
            if i == "minecraft:lava_bucket" {
                20000
            } else {
                0
            }
        };
        let rest = |i: &str| (i == "minecraft:lava_bucket").then(|| "minecraft:bucket".to_owned());
        assert!(cooker_tick(&mut s, &mut slots, burn, rest));
        assert_eq!(slots[FUEL], slot("minecraft:bucket", 1));
        // A stick (100 ticks) is half a batch, and still runs one, as the original's double.
        let mut s = Cooker {
            water: 1000,
            ..Default::default()
        };
        let mut slots = vec![slot(RICE, 1), slot("minecraft:stick", 1), None];
        let burn = |i: &str| if i == "minecraft:stick" { 100 } else { 0 };
        for _ in 0..100 {
            assert!(cooker_tick(&mut s, &mut slots, burn, |_| None));
        }
        assert_eq!(slots[OUTPUT], slot(COOKED_RICE, 2)); // 50 g -> 1.67 -> 2
        assert_eq!(s.burn, -100);
    }

    fn soy() -> Ferment {
        Ferment {
            takes_item: true,
            fluid: WATER.into(),
            amount: 250,
            output: "sushigocrafting:soy_sauce".into(),
            count: 1,
        }
    }
    fn cheese() -> Ferment {
        Ferment {
            takes_item: false,
            fluid: MILK.into(),
            amount: 250,
            output: "sushigocrafting:cheese".into(),
            count: 1,
        }
    }

    #[test]
    fn barrel_ferments_soy_and_cheese() {
        let mut s = Barrel {
            fluid: 1,
            amount: 1000,
            progress: 0,
        };
        let mut slots = vec![slot("sushigocrafting:soy_bean", 2), None, None];
        let recipes = [(soy(), true), (cheese(), false)];
        for _ in 0..99 {
            assert!(barrel_tick(&mut s, &mut slots, &recipes, 0.0));
        }
        assert!(slots[OUTPUT].is_none());
        assert!(barrel_tick(&mut s, &mut slots, &recipes, 0.0));
        assert_eq!(slots[OUTPUT], slot("sushigocrafting:soy_sauce", 3));
        assert_eq!(slots[INPUT], slot("sushigocrafting:soy_bean", 1));
        assert_eq!(s.amount, 750);
        // Milk and an empty input: cheese.
        let mut s = Barrel {
            fluid: 2,
            amount: 250,
            progress: 99,
        };
        let mut slots = vec![None, None, None];
        let recipes = [(soy(), false), (cheese(), false)];
        assert!(barrel_tick(&mut s, &mut slots, &recipes, 0.0));
        assert_eq!(slots[OUTPUT], slot("sushigocrafting:cheese", 4)); // 50 g / 12
        assert_eq!((s.fluid, s.amount), (0, 0), "the tank emptied");
        // Milk with soy beans in: nothing (cheese wants an empty input).
        let mut s = Barrel {
            fluid: 2,
            amount: 1000,
            progress: 0,
        };
        let mut slots = vec![slot("sushigocrafting:soy_bean", 1), None, None];
        let recipes = [(soy(), true), (cheese(), false)];
        assert!(!barrel_tick(&mut s, &mut slots, &recipes, 0.0));
        // A full output stops it.
        let mut s = Barrel {
            fluid: 1,
            amount: 1000,
            progress: 0,
        };
        let mut slots = vec![
            slot("sushigocrafting:soy_bean", 1),
            None,
            slot("sushigocrafting:soy_sauce", 25),
        ];
        assert!(!barrel_tick(&mut s, &mut slots, &[(soy(), true)], 0.0));
    }

    #[test]
    fn buckets_fill_tanks() {
        assert!(tank_takes(0, 0, 1, BARREL_CAPACITY));
        assert!(tank_takes(1, 1000, 1, BARREL_CAPACITY));
        assert!(!tank_takes(1, 1500, 1, BARREL_CAPACITY));
        assert!(!tank_takes(1, 1000, 2, BARREL_CAPACITY), "no milk on water");
        assert!(
            tank_takes(2, 0, 1, BARREL_CAPACITY),
            "an emptied tank takes any"
        );
        assert!(!tank_takes(0, 0, 0, BARREL_CAPACITY));
        assert_eq!(bucket_fluid("minecraft:milk_bucket"), 2);
        assert_eq!(bucket_fluid("minecraft:lava_bucket"), 0);
        assert!(tank_takes(1, 7000, 1, COOKER_WATER_CAPACITY));
        assert!(!tank_takes(1, 7500, 1, COOKER_WATER_CAPACITY));
    }

    #[test]
    fn rolls_are_in_range() {
        for _ in 0..10_000 {
            let r = roll();
            assert!((0.0..1.0).contains(&r));
        }
        assert_eq!(next_int(0.999_999_9, 7), 6);
        assert_eq!(next_int(0.5, 0), 0);
    }
}

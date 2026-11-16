// Mekanism Generators for Horizon 0.1.0: the solar, advanced solar, heat and
// wind generators, feeding the Mekanism port's cables, cubes and machines.
// This file is both the mod's Cargo library (its tests) and the module the
// bundle adds to the game (`crate::game::mekanismgenerators_native`);
// `glue.rs` adapts it to Horizon. Only std: it builds and tests on its own.
//
// Port of Mekanism Generators 10.7 (MIT). Numbers are its defaults
// (GeneratorsConfig, GeneratorsStorageConfig), in joules (J).
//
// Energy goes out the way the Mekanism port (`mekanism_native` 0.6.x) moves
// it: the `energy` (Long) key of the block entity's saved compound, along
// its universal cables, to its machines and energy cubes. This module keeps
// its own copy of the port's cable tiers and acceptors (names, capacities),
// so it never calls `mekanism_native` and builds without it.

pub const MG_ID: &str = "mekanismgenerators_native";
pub const MG_NAME: &str = "Mekanism Generators";
pub const MG_VERSION: &str = "0.1.0";
pub const MG_DESCRIPTION: &str = "Mekanism Generators: Gerador Solar, Gerador Solar Avançado, Gerador a Calor e Gerador Eólico, que alimentam cabos, cubos e máquinas do Mekanism. Precisa dos mods mekanismgenerators (conteúdo), mekanism e mekanism_native.";

// ------------------------------------------------------------------ numbers

/// Solar Generator: peak J/t (`solarGeneration`).
pub const MG_SOLAR_GENERATION: i64 = 50;
/// Advanced Solar Generator: peak J/t (`advancedSolarGeneration`).
pub const MG_ADVANCED_SOLAR_GENERATION: i64 = 300;
/// Heat Generator: J/t while burning lava from its tank (`heatGeneration`).
pub const MG_HEAT_GENERATION: i64 = 200;
/// ... per side touching lava (`heatGenerationLava`).
pub const MG_HEAT_LAVA: i64 = 30;
/// ... in an ultra warm dimension, the Nether (`heatGenerationNether`).
pub const MG_HEAT_NETHER: i64 = 100;
/// Its lava tank, mB (`heatTankCapacity`: 24 buckets).
pub const MG_HEAT_TANK: i32 = 24_000;
/// Lava it burns per tick, mB (`heatGenerationFluidRate`).
pub const MG_HEAT_FLUID_RATE: i32 = 10;
/// A fuel item's burn time divided by this is the lava it gives, mB
/// (`FluidFuelInventorySlot.forFuel(.., burnTime / 20, ..)`): coal 80 mB.
pub const MG_HEAT_BURN_DIVISOR: i32 = 20;
/// Wind Generator (`wind_generator`): J/t at `minY` and at `maxY`.
pub const MG_WIND_MIN: i64 = 60;
pub const MG_WIND_MAX: i64 = 480;
pub const MG_WIND_MIN_Y: i32 = 24;
/// `DimensionType.MAX_Y`, the config's default `maxY` (clamped to the world).
pub const MG_WIND_MAX_Y: i32 = 2031;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MgKind {
    Solar,
    AdvancedSolar,
    Heat,
    Wind,
}

/// A side a generator gives energy from, relative to its `facing`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MgSide {
    Front,
    Bottom,
}

impl MgKind {
    pub const ALL: [MgKind; 4] = [
        MgKind::Solar,
        MgKind::AdvancedSolar,
        MgKind::Heat,
        MgKind::Wind,
    ];
    pub fn block(self) -> &'static str {
        match self {
            MgKind::Solar => "mekanismgenerators:solar_generator",
            MgKind::AdvancedSolar => "mekanismgenerators:advanced_solar_generator",
            MgKind::Heat => "mekanismgenerators:heat_generator",
            MgKind::Wind => "mekanismgenerators:wind_generator",
        }
    }
    pub fn of(block: &str) -> Option<MgKind> {
        MgKind::ALL.into_iter().find(|k| k.block() == block)
    }
    /// The block entity type id.
    pub fn entity(self) -> &'static str {
        self.block()
    }
    /// Joules stored (GeneratorsStorageConfig).
    pub fn capacity(self) -> i64 {
        match self {
            MgKind::Solar => 96_000,
            MgKind::AdvancedSolar => 200_000,
            MgKind::Heat => 160_000,
            MgKind::Wind => 200_000,
        }
    }
    /// The most it makes in a tick: the heat generator's is 7 lava sides,
    /// the Nether and burning (510 J/t), as `MAX_PRODUCTION`.
    pub fn max_production(self) -> i64 {
        match self {
            MgKind::Solar => MG_SOLAR_GENERATION,
            MgKind::AdvancedSolar => MG_ADVANCED_SOLAR_GENERATION,
            MgKind::Heat => MG_HEAT_LAVA * 7 + MG_HEAT_NETHER + MG_HEAT_GENERATION,
            MgKind::Wind => MG_WIND_MAX,
        }
    }
    /// What it gives out per tick at most: twice its production
    /// (`TileEntityGenerator.updateMaxOutputRaw`).
    pub fn output(self) -> i64 {
        self.max_production() * 2
    }
    /// The sides it gives from (`getEnergySides`).
    pub fn sides(self) -> &'static [MgSide] {
        match self {
            MgKind::Solar => &[MgSide::Bottom],
            MgKind::AdvancedSolar | MgKind::Wind => &[MgSide::Front, MgSide::Bottom],
            MgKind::Heat => &[MgSide::Front],
        }
    }
}

/// The six directions as offsets: down, up, north, south, west, east.
pub const MG_DIRS: [[i32; 3]; 6] = [
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, 0, 0],
    [1, 0, 0],
];

pub fn mg_offset(p: [i32; 3], d: [i32; 3]) -> [i32; 3] {
    [p[0] + d[0], p[1] + d[1], p[2] + d[2]]
}

/// The offset of a horizontal `facing` (the furnace template's).
pub fn mg_facing_offset(facing: &str) -> [i32; 3] {
    match facing {
        "south" => [0, 0, 1],
        "west" => [-1, 0, 0],
        "east" => [1, 0, 0],
        _ => [0, 0, -1],
    }
}

/// The offsets a generator facing `facing` gives through.
pub fn mg_side_offsets(kind: MgKind, facing: &str) -> Vec<[i32; 3]> {
    kind.sides()
        .iter()
        .map(|s| match s {
            MgSide::Front => mg_facing_offset(facing),
            MgSide::Bottom => [0, -1, 0],
        })
        .collect()
}

// ------------------------------------------------------------------ the sun

/// `DimensionType.timeOfDay` (vanilla's): the sun's angle as a fraction
/// of a turn, 0 at noon (day time 6000) and 0.5 at midnight.
pub fn mg_time_of_day(day_time: i64) -> f32 {
    let t = day_time.rem_euclid(24_000) as f64;
    let d = (t / 24_000.0 - 0.25).rem_euclid(1.0);
    let e = 0.5 - (d * std::f64::consts::PI).cos() / 2.0;
    ((d * 2.0 + e) / 3.0) as f32
}

/// `WorldUtils.getSunBrightness`: 1.0 around noon, 0.2 at night; rain and
/// thunder levels (0..1) dim it.
pub fn mg_sun_brightness(day_time: i64, rain: f32, thunder: f32) -> f32 {
    let f = mg_time_of_day(day_time);
    let mut f1 = 1.0 - ((f * std::f32::consts::TAU).cos() * 2.0 + 0.2);
    f1 = f1.clamp(0.0, 1.0);
    f1 = 1.0 - f1;
    f1 = (f64::from(f1) * (1.0 - f64::from(rain) * 5.0 / 16.0)) as f32;
    f1 = (f64::from(f1) * (1.0 - f64::from(thunder) * 5.0 / 16.0)) as f32;
    f1 * 0.8 + 0.2
}

/// Vanilla's `Level.getSkyDarken` (0 bright day .. 11 night), for the tests;
/// the glue asks the game's own.
pub fn mg_sky_darken(day_time: i64, rain: f32, thunder: f32) -> i32 {
    let f = mg_time_of_day(day_time);
    let mut g = 1.0 - ((f * std::f32::consts::TAU).cos() * 2.0 + 0.5);
    g = g.clamp(0.0, 1.0);
    g = 1.0 - g;
    g = (f64::from(g) * (1.0 - f64::from(rain) * 5.0 / 16.0)) as f32;
    g = (f64::from(g) * (1.0 - f64::from(thunder) * 5.0 / 16.0)) as f32;
    g = 1.0 - g;
    (g * 11.0) as i32
}

/// The biome where a solar panel stands, as `SolarCheck` reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MgClimate {
    pub temperature: f32,
    pub downfall: f32,
    /// The biome has precipitation (rain or snow): rain penalises it.
    pub precipitation: bool,
}

impl MgClimate {
    /// Plains (0.8, 0.4, rain): what the glue falls back to when it can't read a biome.
    pub const PLAINS: MgClimate = MgClimate {
        temperature: 0.8,
        downfall: 0.4,
        precipitation: true,
    };
    /// `SolarCheck.peakMultiplier`: 0.8 is the best temperature, hotter
    /// loses, cooler gains (30% scale); humidity drags it down (30% scale).
    pub fn peak(self) -> f32 {
        let temp_eff = 0.3f32 * (0.8f32 - self.temperature);
        let humidity_eff = if self.precipitation {
            -0.3f32 * self.downfall
        } else {
            0.0
        };
        1.0f32 + temp_eff + humidity_eff
    }
    /// `SolarCheck.getGenerationMultiplier`: 0 without sun, the peak, or a
    /// fifth of it while it rains (or thunders) where it can rain.
    pub fn generation_multiplier(self, sees_sun: bool, raining: bool) -> f32 {
        if !sees_sun {
            return 0.0;
        }
        if self.precipitation && raining {
            return self.peak() * 0.2f32;
        }
        self.peak()
    }
}

/// Java's `clampToLong(long * float)`: the float product, truncated.
fn mg_float_joules(config: i64, factor: f32) -> i64 {
    let v = config as f32 * factor;
    if v.is_nan() || v <= 0.0 { 0 } else { v as i64 }
}

/// What a solar generator makes this tick: `config` J/t at peak, times the
/// sun's brightness and the generation multiplier.
pub fn mg_solar_production(config: i64, brightness: f32, multiplier: f32) -> i64 {
    mg_float_joules(config, brightness * multiplier)
}

/// The nine points an advanced solar generator checks (its panel's 3x3,
/// centre first), as offsets from the point above its centre.
pub const MG_ADVANCED_POINTS: [[i32; 3]; 9] = [
    [0, 0, 0],
    [-1, 0, -1],
    [-1, 0, 0],
    [-1, 0, 1],
    [0, 0, -1],
    [0, 0, 1],
    [1, 0, -1],
    [1, 0, 0],
    [1, 0, 1],
];

/// The advanced solar generator over its nine points' sky (`seen`): whether
/// it "sees" the sun (5 of 9 or more) and the average multiplier.
pub fn mg_advanced_solar(seen: &[bool; 9], climate: MgClimate, raining: bool) -> (bool, f32) {
    let count = seen.iter().filter(|s| **s).count();
    let mut sum = 0.0f32;
    for s in seen {
        sum += climate.generation_multiplier(*s, raining);
    }
    (count > 4, sum / 9.0)
}

// ------------------------------------------------------------------ the wind

/// `TileEntityWindGenerator.getMultiplier` times `generationMin`: J/t for
/// a generator whose top (`y + 4`) is at `top_y` and sees the sky, in a world
/// from `min_y` and `height` blocks tall, `logical_height` as its dimension.
pub fn mg_wind_generation(top_y: i32, min_y: i32, height: i32, logical_height: i32) -> i64 {
    let max_build = min_y + height;
    let max_level = max_build.min(min_y + logical_height) - 1;
    let lo = MG_WIND_MIN_Y.max(min_y);
    let hi = MG_WIND_MAX_Y.min(max_level);
    if hi <= lo {
        return MG_WIND_MIN;
    }
    let clamped = hi.min(lo.max(top_y));
    let min_g = MG_WIND_MIN as f64;
    let slope = (MG_WIND_MAX - MG_WIND_MIN) as f64 / f64::from(hi - lo);
    let to_gen = min_g + slope * f64::from(clamped - lo);
    let multiplier = to_gen / min_g;
    (min_g * multiplier) as i64
}

// ------------------------------------------------------------------ the heat

/// The lava (mB) one fuel item puts in the heat generator's tank: its burn
/// time / 20. A lava bucket (20 000 ticks) is 1000 mB.
pub fn mg_fuel_lava(burn_time: i32) -> i32 {
    (burn_time / MG_HEAT_BURN_DIVISOR).max(0)
}

/// Whether `amount` more mB of lava fits the tank holding `lava`.
pub fn mg_heat_fits(lava: i32, amount: i32) -> bool {
    amount > 0 && lava.saturating_add(amount) <= MG_HEAT_TANK
}

/// The passive heat: 30 J/t per side touching lava, 100 in the Nether.
pub fn mg_heat_passive(lava_sides: u32, nether: bool) -> i64 {
    MG_HEAT_LAVA * i64::from(lava_sides.min(7)) + if nether { MG_HEAT_NETHER } else { 0 }
}

/// The saved numbers of a generator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MgState {
    pub energy: i64,
    /// The heat generator's lava tank, mB.
    pub lava: i32,
}

impl MgState {
    /// Adds up to `joules` within `kind`'s capacity; returns what went in.
    pub fn add(&mut self, kind: MgKind, joules: i64) -> i64 {
        let room = (kind.capacity() - self.energy).max(0);
        let took = joules.clamp(0, room);
        self.energy += took;
        took
    }
}

/// One tick of the heat generator after its fuel went in the tank: the
/// passive heat, then 10 mB of lava burnt for 200 J/t while it has room
/// (`getNeeded() > 0`). Returns (burning, J/t made).
pub fn mg_heat_tick(state: &mut MgState, lava_sides: u32, nether: bool) -> (bool, i64) {
    let kind = MgKind::Heat;
    state.lava = state.lava.clamp(0, MG_HEAT_TANK);
    let mut made = state.add(kind, mg_heat_passive(lava_sides, nether));
    let burning = state.energy < kind.capacity() && state.lava >= MG_HEAT_FLUID_RATE;
    if burning {
        state.lava -= MG_HEAT_FLUID_RATE;
        made += state.add(kind, MG_HEAT_GENERATION);
    }
    (burning, made)
}

// ------------------------------------------------------------------ the port's network

/// The Mekanism port's basic cable (Horizon's pipe block) and its 0.4.0
/// variants (`_m1`..`_m63`), and the other tiers: J/t they carry.
pub const MG_CABLE_BASIC: &str = "mekanism:basic_universal_cable";
pub const MG_CABLE_TIERS: [(&str, i64); 4] = [
    (MG_CABLE_BASIC, 8_000),
    ("mekanism:advanced_universal_cable", 128_000),
    ("mekanism:elite_universal_cable", 1_024_000),
    ("mekanism:ultimate_universal_cable", 8_192_000),
];
/// How far a network is followed, in cable blocks (the port's `CABLE_REACH`).
pub const MG_CABLE_REACH: usize = 512;
/// No cable between the generator and a target.
pub const MG_DIRECT: i64 = i64::MAX;

pub fn mg_cable_rate(name: &str) -> Option<i64> {
    if let Some(rest) = name.strip_prefix(MG_CABLE_BASIC) {
        if rest.is_empty() {
            return Some(8_000);
        }
        let n: u8 = rest.strip_prefix("_m")?.parse().ok()?;
        return (1..64).contains(&n).then_some(8_000);
    }
    MG_CABLE_TIERS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, r)| *r)
}

/// A block of the Mekanism port that takes energy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MgTarget {
    /// Joules it holds with no energy upgrades.
    pub capacity: i64,
    /// A machine: energy upgrades (`energy_upgrades`) multiply its buffer.
    pub machine: bool,
    /// An energy cube: what it takes in per tick at most; never through
    /// its front (its output face).
    pub cube_rate: Option<i64>,
}

/// The port's acceptors (`mekanism_native` 0.6.x `Kind::accepts`).
pub fn mg_target(name: &str) -> Option<MgTarget> {
    let machine = |capacity| MgTarget {
        capacity,
        machine: true,
        cube_rate: None,
    };
    let cube = |capacity, rate| MgTarget {
        capacity,
        machine: false,
        cube_rate: Some(rate),
    };
    Some(match name {
        "mekanism:enrichment_chamber"
        | "mekanism:crusher"
        | "mekanism:energized_smelter"
        | "mekanism:metallurgic_infuser" => machine(20_000),
        "mekanism:osmium_compressor" => machine(80_000),
        "mekanism:basic_energy_cube" => cube(4_000_000, 4_000),
        "mekanism:advanced_energy_cube" => cube(16_000_000, 16_000),
        "mekanism:elite_energy_cube" => cube(64_000_000, 64_000),
        "mekanism:ultimate_energy_cube" => cube(256_000_000, 256_000),
        _ => return None,
    })
}

/// The port's `upgrade_multiplier`: 10^(n/8), 8 upgrades at most.
pub fn mg_upgrade_multiplier(n: u8) -> f64 {
    10f64.powf(f64::from(n.min(8)) / 8.0)
}

impl MgTarget {
    /// Its capacity with `energy_upgrades` (the port's `capacity`).
    pub fn capacity_with(self, energy_upgrades: u8) -> i64 {
        if self.machine {
            (self.capacity as f64 * mg_upgrade_multiplier(energy_upgrades)).round() as i64
        } else {
            self.capacity
        }
    }
    /// What it takes now: free room, a cube at most its rate.
    pub fn room(self, energy: i64, energy_upgrades: u8) -> i64 {
        let free = (self.capacity_with(energy_upgrades) - energy).max(0);
        match self.cube_rate {
            Some(rate) => free.min(rate),
            None => free,
        }
    }
}

/// Where energy from `from` goes: the acceptors beside the sides it gives
/// through and those along cables reached from them, each with the slowest
/// cable tier on its way (`MG_DIRECT` beside the generator). `block` names
/// the block at a position; `accepts` says whether a block takes energy
/// coming in along a step. As the port's `acceptors_rated`.
pub fn mg_acceptors(
    from: [i32; 3],
    sides: &[[i32; 3]],
    block: &mut impl FnMut([i32; 3]) -> String,
    accepts: &mut impl FnMut([i32; 3], [i32; 3]) -> bool,
) -> Vec<([i32; 3], i64)> {
    use std::collections::{HashSet, VecDeque};
    let mut out = Vec::new();
    let mut found = HashSet::new();
    let mut seen = HashSet::new();
    let mut queue: VecDeque<([i32; 3], i64)> = VecDeque::new();
    seen.insert(from);
    let mut visit = |p: [i32; 3],
                     came: [i32; 3],
                     limit: i64,
                     queue: &mut VecDeque<([i32; 3], i64)>,
                     seen: &mut HashSet<[i32; 3]>| {
        if p == from {
            return;
        }
        if let Some(rate) = mg_cable_rate(&block(p)) {
            if seen.insert(p) {
                queue.push_back((p, limit.min(rate)));
            }
        } else if accepts(p, came) && found.insert(p) {
            out.push((p, limit));
        }
    };
    for &d in sides {
        visit(mg_offset(from, d), d, MG_DIRECT, &mut queue, &mut seen);
    }
    let mut cables = 0;
    while let Some((c, limit)) = queue.pop_front() {
        cables += 1;
        if cables > MG_CABLE_REACH {
            break;
        }
        for d in MG_DIRS {
            visit(mg_offset(c, d), d, limit, &mut queue, &mut seen);
        }
    }
    out
}

/// `amount` split over targets with `rooms` of free space: equal shares,
/// what a full target can't take passed on to the others.
pub fn mg_split(amount: i64, rooms: &[i64]) -> Vec<i64> {
    let mut given = vec![0i64; rooms.len()];
    let mut left = amount.max(0);
    loop {
        let open: Vec<usize> = (0..rooms.len())
            .filter(|&i| rooms[i] - given[i] > 0)
            .collect();
        if left <= 0 || open.is_empty() {
            break;
        }
        let share = (left / open.len() as i64).max(1);
        let mut moved = 0;
        for i in open {
            if left <= 0 {
                break;
            }
            let g = share.min(rooms[i] - given[i]).min(left);
            given[i] += g;
            left -= g;
            moved += g;
        }
        if moved == 0 {
            break;
        }
    }
    given
}

/// What a generator with `amount` this tick hands each target: equal
/// shares within each one's room and way, no more in all than the fastest
/// way carries (the port's `share_out`).
pub fn mg_share_out(amount: i64, rooms: &[i64], limits: &[i64]) -> Vec<i64> {
    let total = limits.iter().copied().max().map_or(0, |m| amount.min(m));
    let capped: Vec<i64> = rooms
        .iter()
        .zip(limits.iter().chain(std::iter::repeat(&0)))
        .map(|(r, l)| (*r).min(*l).max(0))
        .collect();
    mg_split(total, &capped)
}

// ------------------------------------------------------------------ the screen

/// Units the menu's 16-bit data slots count energy in.
pub fn mg_energy_unit(kind: MgKind) -> i64 {
    ((kind.capacity() + 29_999) / 30_000).max(1)
}

/// The menu's four data slots: energy (in `mg_energy_unit`s), J/t made this
/// tick, and two of the kind's own: the heat generator's lava (mB) and
/// whether it burns; the solar ones' sun (1/0) and brightness in
/// thousandths; the wind generator's height multiplier in thousandths and
/// whether it turns.
pub fn mg_data(kind: MgKind, state: &MgState, made: i64, extra: [i32; 2]) -> [i32; 4] {
    let energy = (state.energy / mg_energy_unit(kind)).clamp(0, 32_767) as i32;
    let made = made.clamp(0, 32_767) as i32;
    let a = match kind {
        MgKind::Heat => state.lava.clamp(0, MG_HEAT_TANK),
        _ => extra[0].clamp(0, 32_767),
    };
    [energy, made, a, extra[1].clamp(0, 32_767)]
}

/// "12.345 J", "1.234,5 kJ", "3,2 MJ", as Mekanism shows energy (pt-BR digits).
pub fn mg_joules(j: i64) -> String {
    fn group(n: i64) -> String {
        let s = n.abs().to_string();
        let mut out = String::new();
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (s.len() - i) % 3 == 0 {
                out.push('.');
            }
            out.push(c);
        }
        if n < 0 { format!("-{out}") } else { out }
    }
    if j.abs() < 100_000 {
        format!("{} J", group(j))
    } else if j.abs() < 10_000_000 {
        format!("{},{} kJ", group(j / 1000), (j % 1000).abs() / 100)
    } else {
        format!(
            "{},{} MJ",
            group(j / 1_000_000),
            (j % 1_000_000).abs() / 100_000
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_and_numbers_are_the_originals() {
        assert_eq!(
            MgKind::of("mekanismgenerators:wind_generator"),
            Some(MgKind::Wind)
        );
        assert_eq!(
            MgKind::of("mekanism:solar_generator"),
            None,
            "the port's own stays the port's"
        );
        assert_eq!(MgKind::Heat.max_production(), 510);
        assert_eq!(MgKind::Heat.output(), 1020);
        assert_eq!(MgKind::Solar.output(), 100);
        assert_eq!(MgKind::AdvancedSolar.output(), 600);
        assert_eq!(MgKind::Wind.output(), 960);
        assert_eq!(MgKind::Solar.capacity(), 96_000);
        for k in MgKind::ALL {
            assert!(k.capacity() / mg_energy_unit(k) <= 32_767);
        }
    }

    #[test]
    fn sides_follow_the_facing() {
        assert_eq!(mg_side_offsets(MgKind::Solar, "east"), vec![[0, -1, 0]]);
        assert_eq!(mg_side_offsets(MgKind::Heat, "south"), vec![[0, 0, 1]]);
        assert_eq!(
            mg_side_offsets(MgKind::Wind, "north"),
            vec![[0, 0, -1], [0, -1, 0]]
        );
    }

    #[test]
    fn sun_is_full_at_noon_dim_at_night() {
        assert!((mg_sun_brightness(6000, 0.0, 0.0) - 1.0).abs() < 1e-6);
        assert!((mg_sun_brightness(18_000, 0.0, 0.0) - 0.2).abs() < 1e-6);
        // still full for a while either side of noon
        assert!((mg_sun_brightness(4000, 0.0, 0.0) - 1.0).abs() < 1e-6);
        assert!(mg_sun_brightness(6000, 1.0, 0.0) < 0.8);
        assert_eq!(mg_sky_darken(6000, 0.0, 0.0), 0);
        assert!(mg_sky_darken(18_000, 0.0, 0.0) >= 4);
        assert!(
            mg_sky_darken(6000, 1.0, 0.0) < 4,
            "rain at noon is still day"
        );
        assert!(
            mg_sky_darken(6000, 1.0, 1.0) >= 4,
            "a thunderstorm is too dark for the panels"
        );
    }

    #[test]
    fn solar_in_plains_desert_and_rain() {
        // plains at noon, clear: 50 * 0.88 (float) = 44 J/t
        let plains = MgClimate::PLAINS;
        let noon = mg_sun_brightness(6000, 0.0, 0.0);
        let p = mg_solar_production(
            MG_SOLAR_GENERATION,
            noon,
            plains.generation_multiplier(true, false),
        );
        assert_eq!(p, 44);
        // desert (2.0, no rain): 50 * 0.64 = 32 J/t
        let desert = MgClimate {
            temperature: 2.0,
            downfall: 0.0,
            precipitation: false,
        };
        assert_eq!(
            mg_solar_production(50, noon, desert.generation_multiplier(true, false)),
            32
        );
        // a perfect 0.8 / dry biome: exactly the config
        let ideal = MgClimate {
            temperature: 0.8,
            downfall: 0.0,
            precipitation: false,
        };
        assert_eq!(
            mg_solar_production(50, noon, ideal.generation_multiplier(true, false)),
            50
        );
        assert_eq!(
            mg_solar_production(300, noon, ideal.generation_multiplier(true, false)),
            300
        );
        // raining in plains: a fifth, and the dimmer sun
        let rain = mg_solar_production(
            50,
            mg_sun_brightness(6000, 1.0, 0.0),
            plains.generation_multiplier(true, true),
        );
        assert!(rain > 0 && rain <= 9, "{rain}");
        // no sky
        assert_eq!(
            mg_solar_production(50, noon, plains.generation_multiplier(false, false)),
            0
        );
    }

    #[test]
    fn advanced_solar_needs_five_of_nine() {
        let plains = MgClimate::PLAINS;
        let mut seen = [true; 9];
        let (sees, m) = mg_advanced_solar(&seen, plains, false);
        assert!(sees);
        let noon = mg_sun_brightness(6000, 0.0, 0.0);
        assert_eq!(
            mg_solar_production(MG_ADVANCED_SOLAR_GENERATION, noon, m),
            264
        );
        for s in seen.iter_mut().take(5) {
            *s = false;
        }
        assert!(!mg_advanced_solar(&seen, plains, false).0);
    }

    #[test]
    fn wind_by_height() {
        // overworld: -64, 384 tall: maxY 319; top below 24 makes the minimum
        assert_eq!(mg_wind_generation(10, -64, 384, 384), 60);
        assert_eq!(mg_wind_generation(24, -64, 384, 384), 60);
        assert_eq!(mg_wind_generation(319, -64, 384, 384), 480);
        assert_eq!(mg_wind_generation(400, -64, 384, 384), 480);
        // a generator at y=60 (top 64): 60 + 420/295 * 40 = 116.9
        assert_eq!(mg_wind_generation(64, -64, 384, 384), 116);
        // at y=100 (top 104): 60 + 420/295*80 = 173.8
        assert_eq!(mg_wind_generation(104, -64, 384, 384), 173);
    }

    #[test]
    fn heat_burns_ten_mb_for_200_and_lava_adds() {
        assert_eq!(mg_fuel_lava(1600), 80, "coal");
        assert_eq!(mg_fuel_lava(20_000), 1000, "lava bucket");
        let mut s = MgState {
            energy: 0,
            lava: 80,
        };
        let mut total = 0;
        for _ in 0..10 {
            total += mg_heat_tick(&mut s, 0, false).1;
        }
        assert_eq!(total, 8 * 200, "one coal: 8 ticks");
        assert_eq!(s.lava, 0);
        let mut s = MgState::default();
        assert_eq!(mg_heat_tick(&mut s, 2, false), (false, 60));
        assert_eq!(mg_heat_tick(&mut s, 0, true), (false, 100));
        let mut s = MgState {
            energy: 0,
            lava: 1000,
        };
        assert_eq!(mg_heat_tick(&mut s, 1, false), (true, 230));
        // full: keeps its lava
        let mut s = MgState {
            energy: 160_000,
            lava: 1000,
        };
        assert_eq!(mg_heat_tick(&mut s, 1, false), (false, 0));
        assert_eq!(s.lava, 1000);
        assert!(mg_heat_fits(23_920, 80) && !mg_heat_fits(23_921, 80));
    }

    #[test]
    fn cables_and_targets_of_the_port() {
        assert_eq!(mg_cable_rate("mekanism:basic_universal_cable"), Some(8_000));
        assert_eq!(
            mg_cable_rate("mekanism:basic_universal_cable_m63"),
            Some(8_000)
        );
        assert_eq!(mg_cable_rate("mekanism:basic_universal_cable_m64"), None);
        assert_eq!(
            mg_cable_rate("mekanism:elite_universal_cable"),
            Some(1_024_000)
        );
        assert_eq!(mg_cable_rate("mekanism:crusher"), None);
        let cube = mg_target("mekanism:basic_energy_cube").unwrap();
        assert_eq!(cube.room(0, 0), 4_000);
        assert_eq!(cube.room(3_999_000, 8), 1_000);
        let crusher = mg_target("mekanism:crusher").unwrap();
        assert_eq!(crusher.room(0, 0), 20_000);
        assert_eq!(crusher.room(0, 8), 200_000);
        assert!(mg_target("mekanismgenerators:solar_generator").is_none());
        assert!(mg_target("mekanism:heat_generator").is_none());
    }

    #[test]
    fn energy_reaches_a_cube_and_machines_along_cables() {
        use std::collections::HashMap;
        // solar at 0,1,0 gives down: cable at 0,0,0 → 1,0,0 cable → crusher at 2,0,0; cube at 0,0,1 beside the cable
        let mut world: HashMap<[i32; 3], &str> = HashMap::new();
        world.insert([0, 1, 0], "mekanismgenerators:solar_generator");
        world.insert([0, 0, 0], "mekanism:basic_universal_cable");
        world.insert([1, 0, 0], "mekanism:advanced_universal_cable");
        world.insert([2, 0, 0], "mekanism:crusher");
        world.insert([0, 0, 1], "mekanism:basic_energy_cube");
        world.insert([0, 1, 1], "mekanism:elite_energy_cube"); // beside the solar, not below: not reached
        let sides = mg_side_offsets(MgKind::Solar, "north");
        let found = mg_acceptors(
            [0, 1, 0],
            &sides,
            &mut |p| world.get(&p).copied().unwrap_or("minecraft:air").to_owned(),
            &mut |p, _| world.get(&p).is_some_and(|b| mg_target(b).is_some()),
        );
        let mut at: Vec<[i32; 3]> = found.iter().map(|(p, _)| *p).collect();
        at.sort();
        assert_eq!(at, vec![[0, 0, 1], [2, 0, 0]]);
        assert!(
            found.iter().all(|(_, l)| *l == 8_000),
            "the basic cable limits both"
        );
        let shares = mg_share_out(43, &[4_000, 20_000], &[8_000, 8_000]);
        assert_eq!(shares.iter().sum::<i64>(), 43);
        // beside: no cable, no limit but the output
        let direct = mg_acceptors(
            [0, 1, 0],
            &[[0, -1, 0]],
            &mut |_| "mekanism:basic_energy_cube".to_owned(),
            &mut |_, _| true,
        );
        assert_eq!(direct, vec![([0, 0, 0], MG_DIRECT)]);
    }

    #[test]
    fn hundred_ticks_of_sun_fill_the_cube_below_by_4400() {
        // the test the LEIA-ME asks for: noon, clear, plains, a basic cube right below
        let mut generator = MgState::default();
        let mut cube = 0i64;
        let target = mg_target("mekanism:basic_energy_cube").unwrap();
        let noon = mg_sun_brightness(6000, 0.0, 0.0);
        for t in 0..100 {
            let b = mg_sun_brightness(6000 + t, 0.0, 0.0);
            assert!((b - noon).abs() < 1e-6);
            let made =
                mg_solar_production(50, b, MgClimate::PLAINS.generation_multiplier(true, false));
            generator.add(MgKind::Solar, made);
            let amount = generator.energy.min(MgKind::Solar.output());
            let g = mg_share_out(amount, &[target.room(cube, 0)], &[MG_DIRECT])[0];
            cube += g;
            generator.energy -= g;
        }
        assert_eq!(cube, 4_400);
        assert_eq!(generator.energy, 0);
    }

    #[test]
    fn data_and_text() {
        let s = MgState {
            energy: 96_000,
            lava: 0,
        };
        assert_eq!(
            mg_data(MgKind::Solar, &s, 43, [1, 1000]),
            [24_000, 43, 1, 1000]
        );
        let h = MgState {
            energy: 160_000,
            lava: 24_000,
        };
        assert_eq!(mg_data(MgKind::Heat, &h, 230, [0, 1])[2], 24_000);
        assert_eq!(mg_joules(4_300), "4.300 J");
        assert_eq!(mg_joules(160_000), "160,0 kJ");
    }
}

// Mekanism: MoreMachine for Horizon, the logic (std only, unit tested).
//
// Five item machines of MoreMachine 1.4.3 (MIT, lostmyself8 / Jerry):
// Recycler, CNC Stamper, CNC Lathe, CNC Rolling Mill and Presser. Numbers
// from `MoreMachineUsageConfig` / `MoreMachineStorageConfig` (50 J/t,
// 20 kJ) and the tiles' `BASE_TICKS_REQUIRED` (200 ticks).
//
// Energy: the Mekanism port's cubes and generators only give to the
// blocks they know, so these machines take it themselves: each tick a
// machine with room follows the universal cables from its six sides and
// draws from the givers it finds (the port's cubes and generators and
// Mekanism Generators' ones), through the faces they give from, at most
// their output per tick and the slowest cable on the way. The `energy`
// key (Long) of the giver's saved compound is what changes, as the port's
// own cables do.
//
// Injected into Horizon's `game.rs` as `crate::game::mekmm_native`;
// every public name carries `mm_`/`Mm`/`MM_` so it can't meet another mod's.

pub const MM_ID: &str = "mekmm_native";
pub const MM_NAME: &str = "Mekanism: MoreMachine";
pub const MM_VERSION: &str = "0.1.0";
pub const MM_DESCRIPTION: &str = "Mekanism MoreMachine: Reciclador, Estampador CNC, Torno CNC, Laminador CNC e Prensa, alimentados pelos cabos e cubos do Mekanism. Precisa dos mods mekmm (conteúdo), mekanism e mekanism_native.";

/// Joules per tick of work, every machine (`MoreMachineUsageConfig`).
pub const MM_USAGE: i64 = 50;
/// Joules stored, every machine (`MoreMachineStorageConfig`).
pub const MM_CAPACITY: i64 = 20_000;
/// Ticks per operation (`BASE_TICKS_REQUIRED = 10 * TICKS_PER_SECOND`).
pub const MM_TICKS: i32 = 200;
/// Upgrades of each kind a machine takes (Mekanism's maximum).
pub const MM_MAX_UPGRADES: u8 = 8;
pub const MM_SPEED_UPGRADE: &str = "mekanism:upgrade_speed";
pub const MM_ENERGY_UPGRADE: &str = "mekanism:upgrade_energy";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MmKind {
    Recycler,
    Stamper,
    Lathe,
    RollingMill,
    Presser,
}

impl MmKind {
    pub const ALL: [MmKind; 5] = [
        MmKind::Recycler,
        MmKind::Stamper,
        MmKind::Lathe,
        MmKind::RollingMill,
        MmKind::Presser,
    ];
    pub fn block(self) -> &'static str {
        match self {
            MmKind::Recycler => "mekmm:recycler",
            MmKind::Stamper => "mekmm:cnc_stamper",
            MmKind::Lathe => "mekmm:cnc_lathe",
            MmKind::RollingMill => "mekmm:cnc_rolling_mill",
            MmKind::Presser => "mekmm:presser",
        }
    }
    pub fn of(block: &str) -> Option<MmKind> {
        MmKind::ALL.into_iter().find(|k| k.block() == block)
    }
    /// The block entity type id.
    pub fn entity(self) -> &'static str {
        self.block()
    }
    /// The recipe type it runs (the original's ids).
    pub fn recipe_type(self) -> &'static str {
        match self {
            MmKind::Recycler => "mekmm:recycler",
            MmKind::Stamper => "mekmm:stamper",
            MmKind::Lathe => "mekmm:lathe",
            MmKind::RollingMill => "mekmm:rolling_mill",
            MmKind::Presser => "mekmm:pressing",
        }
    }
    /// Slots in its container: the furnace menu's three (input, mold or
    /// unused, output), the presser's own menu four (three inputs, output).
    pub fn slots(self) -> usize {
        match self {
            MmKind::Presser => 4,
            _ => 3,
        }
    }
    /// The slot its result goes to.
    pub fn output_slot(self) -> usize {
        self.slots() - 1
    }
    /// The slots a recipe takes from, in the recipe's order.
    pub fn input_slots(self) -> &'static [usize] {
        match self {
            MmKind::Stamper => &[0, 1],
            MmKind::Presser => &[0, 1, 2],
            _ => &[0],
        }
    }
}

/// `MekanismUtils`: each kind of upgrade multiplies by 10^(n/8).
pub fn mm_upgrade_multiplier(n: u8) -> f64 {
    10f64.powf(f64::from(n.min(MM_MAX_UPGRADES)) / 8.0)
}

/// Ticks per operation with `speed` upgrades (200 down to 20).
pub fn mm_ticks(speed: u8) -> i32 {
    (f64::from(MM_TICKS) / mm_upgrade_multiplier(speed))
        .round()
        .max(1.0) as i32
}

/// Joules per tick of work: speed costs its square, energy upgrades divide it.
pub fn mm_usage(speed: u8, energy: u8) -> i64 {
    let u = MM_USAGE as f64 * mm_upgrade_multiplier(speed).powi(2) / mm_upgrade_multiplier(energy);
    (u.round() as i64).max(1)
}

/// Joules stored: energy upgrades multiply the buffer.
pub fn mm_capacity(energy_upgrades: u8) -> i64 {
    (MM_CAPACITY as f64 * mm_upgrade_multiplier(energy_upgrades)).round() as i64
}

/// Units the menu's 16-bit data slots count energy in.
pub fn mm_energy_unit(energy_upgrades: u8) -> i64 {
    ((mm_capacity(energy_upgrades) + 29_999) / 30_000).max(1)
}

/// The saved numbers of a machine.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MmState {
    pub energy: i64,
    pub progress: i32,
    pub speed_upgrades: u8,
    pub energy_upgrades: u8,
    /// The recycler's chance, in thousandths, carried between operations:
    /// each one adds its recipe's chance and a scrap comes out each time it
    /// reaches 1000 (the original rolls a random number instead).
    pub chance: i32,
}

impl MmState {
    pub fn capacity(&self) -> i64 {
        mm_capacity(self.energy_upgrades)
    }
    /// Free room for energy.
    pub fn room(&self) -> i64 {
        (self.capacity() - self.energy).max(0)
    }
    /// The menu's four data slots: energy (in `mm_energy_unit`s), progress
    /// in thousandths, the upgrades packed (speed | energy << 4) and the
    /// recycler's chance carried.
    pub fn data(&self) -> [i32; 4] {
        let unit = mm_energy_unit(self.energy_upgrades);
        let energy = (self.energy / unit).clamp(0, 32_767) as i32;
        let ticks = i64::from(mm_ticks(self.speed_upgrades));
        let progress = (i64::from(self.progress) * 1000 / ticks).clamp(0, 1000) as i32;
        let upgrades =
            i32::from(self.speed_upgrades.min(15)) | (i32::from(self.energy_upgrades.min(15)) << 4);
        [energy, progress, upgrades, self.chance.clamp(0, 999)]
    }
    /// One `item` put in as an upgrade: false when it isn't one or there are
    /// the most of that kind already.
    pub fn upgrade(&mut self, item: &str) -> bool {
        let slot = match item {
            MM_SPEED_UPGRADE => &mut self.speed_upgrades,
            MM_ENERGY_UPGRADE => &mut self.energy_upgrades,
            _ => return false,
        };
        if *slot >= MM_MAX_UPGRADES {
            return false;
        }
        *slot += 1;
        true
    }
}

/// The upgrades packed in the data slots: (speed, energy).
pub fn mm_unpack_upgrades(data: &[i32]) -> (u8, u8) {
    let packed = data.get(2).copied().unwrap_or(0);
    ((packed & 15) as u8, ((packed >> 4) & 15) as u8)
}

/// What a machine needs from the game, over its item stacks `S`.
pub trait MmStacks<S> {
    fn count(&self, s: &S) -> u32;
    /// Whether `result` stacks onto `out` with room for all of it.
    fn fits(&self, out: &S, result: &S) -> bool;
    fn grow(&self, out: &mut S, result: &S);
    /// `n` taken from the stack; None when it is used up.
    fn take(&self, s: S, n: u32) -> Option<S>;
}

/// A recipe found for what the machine holds: per input slot (in
/// `MmKind::input_slots` order) how many it needs and whether it is used up
/// (the stamper's mold is not), the result, and the recycler's chance in
/// thousandths (1000: always).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MmRecipe<S> {
    pub inputs: Vec<(u32, bool)>,
    pub result: S,
    pub chance: i32,
}

/// One tick of a machine. Returns whether it worked (its `lit`).
pub fn mm_machine_tick<S: Clone, K: MmStacks<S>>(
    kind: MmKind,
    state: &mut MmState,
    slots: &mut [Option<S>],
    recipe: Option<MmRecipe<S>>,
    stacks: &K,
) -> bool {
    let out_slot = kind.output_slot();
    let input_slots = kind.input_slots();
    let Some(recipe) = recipe else {
        state.progress = 0;
        return false;
    };
    if slots.len() <= out_slot || recipe.inputs.len() != input_slots.len() {
        state.progress = 0;
        return false;
    }
    let enough = input_slots.iter().zip(&recipe.inputs).all(|(&i, &(n, _))| {
        slots[i]
            .as_ref()
            .is_some_and(|s| stacks.count(s) >= n.max(1))
    });
    let room = slots[out_slot]
        .as_ref()
        .is_none_or(|o| stacks.fits(o, &recipe.result));
    if !enough || !room {
        state.progress = 0;
        return false;
    }
    let usage = mm_usage(state.speed_upgrades, state.energy_upgrades);
    if state.energy < usage {
        return false;
    }
    state.energy -= usage;
    state.progress += 1;
    if state.progress >= mm_ticks(state.speed_upgrades) {
        state.progress = 0;
        let chance = recipe.chance.clamp(0, 1000);
        let make = if chance >= 1000 {
            true
        } else {
            state.chance = state.chance.max(0) + chance;
            if state.chance >= 1000 {
                state.chance -= 1000;
                true
            } else {
                false
            }
        };
        if make {
            match &mut slots[out_slot] {
                Some(o) => stacks.grow(o, &recipe.result),
                None => slots[out_slot] = Some(recipe.result.clone()),
            }
        }
        for (&i, &(n, used)) in input_slots.iter().zip(&recipe.inputs) {
            if used && let Some(s) = slots[i].take() {
                slots[i] = stacks.take(s, n.max(1));
            }
        }
    }
    true
}

/// The chance a recipe's JSON gives (`"chance": 0.17`), in thousandths.
pub fn mm_chance_permille(chance: Option<f64>) -> i32 {
    match chance {
        Some(c) if c.is_finite() => (c.clamp(0.0, 1.0) * 1000.0).round() as i32,
        _ => 1000,
    }
}

// ----------------------------------------------------------------- energy

/// The universal cable tiers of the Mekanism port (`CableTier`): block and
/// joules per tick. The basic one's 0.4.0 variants (`..._m1` to `_m63`) are
/// basic cables too.
pub const MM_CABLE_BASIC: &str = "mekanism:basic_universal_cable";
pub const MM_CABLE_TIERS: [(&str, i64); 4] = [
    (MM_CABLE_BASIC, 8_000),
    ("mekanism:advanced_universal_cable", 128_000),
    ("mekanism:elite_universal_cable", 1_024_000),
    ("mekanism:ultimate_universal_cable", 8_192_000),
];
/// How far a network is followed, in cable blocks (the port's `CABLE_REACH`).
pub const MM_CABLE_REACH: usize = 512;
/// No cable between: only the giver's output limits it.
pub const MM_DIRECT: i64 = i64::MAX;

pub fn mm_cable_rate(name: &str) -> Option<i64> {
    if let Some(rest) = name.strip_prefix(MM_CABLE_BASIC) {
        if rest.is_empty() {
            return Some(MM_CABLE_TIERS[0].1);
        }
        let n: u8 = rest.strip_prefix("_m")?.parse().ok()?;
        return (1..64).contains(&n).then_some(MM_CABLE_TIERS[0].1);
    }
    MM_CABLE_TIERS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, r)| *r)
}

/// The faces a giver gives energy from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MmFaces {
    /// Every face (the port's own generators).
    All,
    /// The face it points to (`facing`): the energy cubes, the heat generator.
    Front,
    /// Below: the solar generator.
    Bottom,
    /// Front and below: the advanced solar and wind generators.
    FrontBottom,
}

/// A block that gives energy: the most it gives per tick and from where.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MmSource {
    pub rate: i64,
    pub faces: MmFaces,
}

/// The givers these machines draw from: the Mekanism port 0.6.1's cubes
/// and generators (`Kind::output`, `give`) and Mekanism Generators 0.1.0's
/// (`MgKind::output`, `sides`).
pub fn mm_source(name: &str) -> Option<MmSource> {
    let s = |rate, faces| Some(MmSource { rate, faces });
    match name {
        "mekanism:basic_energy_cube" => s(4_000, MmFaces::Front),
        "mekanism:advanced_energy_cube" => s(16_000, MmFaces::Front),
        "mekanism:elite_energy_cube" => s(64_000, MmFaces::Front),
        "mekanism:ultimate_energy_cube" => s(256_000, MmFaces::Front),
        "mekanism:heat_generator" => s(400, MmFaces::All),
        "mekanism:solar_generator" => s(100, MmFaces::All),
        "mekanismgenerators:solar_generator" => s(100, MmFaces::Bottom),
        "mekanismgenerators:advanced_solar_generator" => s(600, MmFaces::FrontBottom),
        "mekanismgenerators:heat_generator" => s(1_020, MmFaces::Front),
        "mekanismgenerators:wind_generator" => s(960, MmFaces::FrontBottom),
        _ => None,
    }
}

/// The six directions as offsets: down, up, north, south, west, east.
pub const MM_DIRS: [[i32; 3]; 6] = [
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, 0, 0],
    [1, 0, 0],
];

pub fn mm_offset(p: [i32; 3], d: [i32; 3]) -> [i32; 3] {
    [p[0] + d[0], p[1] + d[1], p[2] + d[2]]
}

/// The offset of a horizontal `facing` (the furnace template's).
pub fn mm_facing_offset(facing: &str) -> [i32; 3] {
    match facing {
        "south" => [0, 0, 1],
        "west" => [-1, 0, 0],
        "east" => [1, 0, 0],
        _ => [0, 0, -1],
    }
}

/// Whether a giver facing `facing` gives through the face `face` (an
/// offset from it).
pub fn mm_gives_through(source: MmSource, facing: &str, face: [i32; 3]) -> bool {
    let front = mm_facing_offset(facing);
    let bottom = [0, -1, 0];
    match source.faces {
        MmFaces::All => true,
        MmFaces::Front => face == front,
        MmFaces::Bottom => face == bottom,
        MmFaces::FrontBottom => face == front || face == bottom,
    }
}

/// The givers a machine at `from` reaches: beside it, and along cables
/// from it (up to `MM_CABLE_REACH` cables), each with the slowest cable on
/// the way the search found it by (`MM_DIRECT` beside the machine).
/// `block` names the block at a position; `gives` says whether the block
/// at a position gives through its face `face` (the offset from it back
/// towards the machine). The order is the search's: nearest first.
pub fn mm_givers(
    from: [i32; 3],
    block: &mut impl FnMut([i32; 3]) -> String,
    gives: &mut impl FnMut([i32; 3], [i32; 3]) -> bool,
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
        let name = block(p);
        if let Some(rate) = mm_cable_rate(&name) {
            if seen.insert(p) {
                queue.push_back((p, limit.min(rate)));
            }
        } else if mm_source(&name).is_some()
            && gives(p, [-came[0], -came[1], -came[2]])
            && found.insert(p)
        {
            out.push((p, limit));
        }
    };
    for d in MM_DIRS {
        visit(mm_offset(from, d), d, MM_DIRECT, &mut queue, &mut seen);
    }
    let mut cables = 0;
    while let Some((c, limit)) = queue.pop_front() {
        cables += 1;
        if cables > MM_CABLE_REACH {
            break;
        }
        for d in MM_DIRS {
            visit(mm_offset(c, d), d, limit, &mut queue, &mut seen);
        }
    }
    out
}

/// What a giver still gives this tick to these machines: its energy, at
/// most its rate less what they already drew from it this tick.
/// `pulled` is (the game tick it was last drawn from, the joules drawn then).
pub fn mm_available(energy: i64, rate: i64, pulled: (i64, i64), now: i64) -> i64 {
    let used = if pulled.0 == now { pulled.1.max(0) } else { 0 };
    energy.max(0).min((rate - used).max(0))
}

/// How much to draw from each giver, nearest first, to fill `room`: each
/// at most what it has (`available`) and its way's limit.
pub fn mm_pull_plan(room: i64, givers: &[(i64, i64)]) -> Vec<i64> {
    let mut left = room.max(0);
    givers
        .iter()
        .map(|&(available, limit)| {
            let take = left.min(available.max(0)).min(limit.max(0));
            left -= take;
            take
        })
        .collect()
}

/// "12.345 J", "20,0 kJ", as Mekanism shows energy (pt-BR digits).
pub fn mm_joules(j: i64) -> String {
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

    /// Stacks as (item, count), max 64.
    struct T;
    type S = (&'static str, u32);
    impl MmStacks<S> for T {
        fn count(&self, s: &S) -> u32 {
            s.1
        }
        fn fits(&self, out: &S, result: &S) -> bool {
            out.0 == result.0 && out.1 + result.1 <= 64
        }
        fn grow(&self, out: &mut S, result: &S) {
            out.1 += result.1;
        }
        fn take(&self, mut s: S, n: u32) -> Option<S> {
            s.1 = s.1.saturating_sub(n);
            (s.1 > 0).then_some(s)
        }
    }

    fn run(
        kind: MmKind,
        state: &mut MmState,
        slots: &mut [Option<S>],
        recipe: &MmRecipe<S>,
        ticks: u32,
    ) {
        for _ in 0..ticks {
            mm_machine_tick(kind, state, slots, Some(recipe.clone()), &T);
        }
    }

    #[test]
    fn numbers_of_the_original() {
        assert_eq!(mm_ticks(0), 200);
        assert_eq!(mm_ticks(8), 20);
        assert_eq!(mm_usage(0, 0), 50);
        assert_eq!(mm_usage(8, 0), 5000);
        assert_eq!(mm_usage(0, 8), 5);
        assert_eq!(mm_capacity(0), 20_000);
        assert_eq!(mm_capacity(8), 200_000);
        for k in MmKind::ALL {
            assert_eq!(MmKind::of(k.block()), Some(k));
        }
        assert_eq!(MmKind::of("mekanism:crusher"), None);
    }

    #[test]
    fn lathe_makes_four_sticks_in_200_ticks_for_10000_joules() {
        let mut s = MmState {
            energy: 20_000,
            ..Default::default()
        };
        let mut slots = vec![Some(("minecraft:oak_planks", 2)), None, None];
        let r = MmRecipe {
            inputs: vec![(1, true)],
            result: ("minecraft:stick", 4),
            chance: 1000,
        };
        run(MmKind::Lathe, &mut s, &mut slots, &r, 199);
        assert_eq!(slots[2], None);
        run(MmKind::Lathe, &mut s, &mut slots, &r, 1);
        assert_eq!(slots[2], Some(("minecraft:stick", 4)));
        assert_eq!(slots[0], Some(("minecraft:oak_planks", 1)));
        assert_eq!(s.energy, 20_000 - 200 * 50);
        run(MmKind::Lathe, &mut s, &mut slots, &r, 200);
        assert_eq!(slots[2], Some(("minecraft:stick", 8)));
        assert_eq!(slots[0], None);
        assert_eq!(s.energy, 0);
    }

    #[test]
    fn input_count_and_no_energy() {
        // rolling mill: 3 iron -> 8 bars; 2 iron is not enough
        let mut s = MmState {
            energy: 20_000,
            ..Default::default()
        };
        let mut slots = vec![Some(("minecraft:iron_ingot", 2)), None, None];
        let r = MmRecipe {
            inputs: vec![(3, true)],
            result: ("minecraft:iron_bars", 8),
            chance: 1000,
        };
        assert!(!mm_machine_tick(
            MmKind::RollingMill,
            &mut s,
            &mut slots,
            Some(r.clone()),
            &T
        ));
        assert_eq!(s.energy, 20_000);
        slots[0] = Some(("minecraft:iron_ingot", 7));
        run(MmKind::RollingMill, &mut s, &mut slots, &r, 200);
        assert_eq!(slots[0], Some(("minecraft:iron_ingot", 4)));
        assert_eq!(slots[2], Some(("minecraft:iron_bars", 8)));
        // without energy it stops but keeps its progress
        let mut s = MmState {
            energy: 120,
            progress: 10,
            ..Default::default()
        };
        run(MmKind::RollingMill, &mut s, &mut slots, &r, 5);
        assert_eq!((s.energy, s.progress), (20, 12));
    }

    #[test]
    fn stamper_keeps_its_mold() {
        let mut s = MmState {
            energy: 20_000,
            ..Default::default()
        };
        let mut slots = vec![
            Some(("minecraft:iron_ingot", 3)),
            Some(("minecraft:heavy_weighted_pressure_plate", 1)),
            None,
        ];
        let r = MmRecipe {
            inputs: vec![(1, true), (1, false)],
            result: ("minecraft:heavy_weighted_pressure_plate", 1),
            chance: 1000,
        };
        run(MmKind::Stamper, &mut s, &mut slots, &r, 200);
        assert_eq!(slots[0], Some(("minecraft:iron_ingot", 2)));
        assert_eq!(
            slots[1],
            Some(("minecraft:heavy_weighted_pressure_plate", 1))
        );
        assert_eq!(
            slots[2],
            Some(("minecraft:heavy_weighted_pressure_plate", 1))
        );
        // no mold: nothing
        slots[1] = None;
        assert!(!mm_machine_tick(
            MmKind::Stamper,
            &mut s,
            &mut slots,
            Some(r),
            &T
        ));
    }

    #[test]
    fn presser_uses_all_three() {
        let mut s = MmState {
            energy: 20_000,
            ..Default::default()
        };
        let mut slots = vec![
            Some(("minecraft:book", 2)),
            Some(("minecraft:ink_sac", 1)),
            Some(("minecraft:feather", 5)),
            None,
        ];
        let r = MmRecipe {
            inputs: vec![(1, true), (1, true), (1, true)],
            result: ("minecraft:writable_book", 1),
            chance: 1000,
        };
        run(MmKind::Presser, &mut s, &mut slots, &r, 200);
        assert_eq!(slots[3], Some(("minecraft:writable_book", 1)));
        assert_eq!(slots[0], Some(("minecraft:book", 1)));
        assert_eq!(slots[1], None);
        assert_eq!(slots[2], Some(("minecraft:feather", 4)));
        // the ink is gone: it stops
        assert!(!mm_machine_tick(
            MmKind::Presser,
            &mut s,
            &mut slots,
            Some(r),
            &T
        ));
    }

    #[test]
    fn recycler_gives_one_scrap_every_six_stones_at_17_percent() {
        let mut s = MmState {
            energy: 1_000_000_000,
            ..Default::default()
        };
        let mut slots = vec![Some(("minecraft:stone", 64)), None, None];
        let r = MmRecipe {
            inputs: vec![(1, true)],
            result: ("mekmm:scrap", 1),
            chance: mm_chance_permille(Some(0.17)),
        };
        assert_eq!(r.chance, 170);
        run(MmKind::Recycler, &mut s, &mut slots, &r, 200 * 5);
        assert_eq!(slots[2], None);
        assert_eq!(slots[0], Some(("minecraft:stone", 59)));
        run(MmKind::Recycler, &mut s, &mut slots, &r, 200);
        assert_eq!(slots[2], Some(("mekmm:scrap", 1)));
        assert_eq!(s.chance, 20);
        // 100 stones -> 17 scrap
        let mut s = MmState {
            energy: 1_000_000_000,
            ..Default::default()
        };
        let mut slots = vec![Some(("minecraft:stone", 50)), None, None];
        run(MmKind::Recycler, &mut s, &mut slots, &r, 200 * 50);
        slots[0] = Some(("minecraft:stone", 50));
        run(MmKind::Recycler, &mut s, &mut slots, &r, 200 * 50);
        assert_eq!(slots[2], Some(("mekmm:scrap", 17)));
        assert_eq!(mm_chance_permille(None), 1000);
        assert_eq!(mm_chance_permille(Some(2.0)), 1000);
    }

    #[test]
    fn upgrades() {
        let mut s = MmState::default();
        for _ in 0..8 {
            assert!(s.upgrade(MM_SPEED_UPGRADE));
        }
        assert!(!s.upgrade(MM_SPEED_UPGRADE));
        assert!(s.upgrade(MM_ENERGY_UPGRADE));
        assert!(!s.upgrade("minecraft:stone"));
        assert_eq!(mm_unpack_upgrades(&s.data()), (8, 1));
        // 8 speed: 20 ticks, 5000 J/t
        let mut s = MmState {
            energy: 200_000,
            speed_upgrades: 8,
            energy_upgrades: 8,
            ..Default::default()
        };
        let mut slots = vec![Some(("minecraft:sugar_cane", 1)), None, None];
        let r = MmRecipe {
            inputs: vec![(1, true)],
            result: ("minecraft:paper", 2),
            chance: 1000,
        };
        run(MmKind::RollingMill, &mut s, &mut slots, &r, 20);
        assert_eq!(slots[2], Some(("minecraft:paper", 2)));
        assert_eq!(s.energy, 200_000 - 20 * 500);
    }

    #[test]
    fn cables_and_sources() {
        assert_eq!(mm_cable_rate("mekanism:basic_universal_cable"), Some(8_000));
        assert_eq!(
            mm_cable_rate("mekanism:basic_universal_cable_m63"),
            Some(8_000)
        );
        assert_eq!(mm_cable_rate("mekanism:basic_universal_cable_m64"), None);
        assert_eq!(
            mm_cable_rate("mekanism:ultimate_universal_cable"),
            Some(8_192_000)
        );
        assert_eq!(mm_cable_rate("mekanism:crusher"), None);
        let cube = mm_source("mekanism:basic_energy_cube").unwrap();
        assert!(mm_gives_through(cube, "east", [1, 0, 0]));
        assert!(!mm_gives_through(cube, "east", [-1, 0, 0]));
        let solar = mm_source("mekanismgenerators:solar_generator").unwrap();
        assert!(mm_gives_through(solar, "north", [0, -1, 0]));
        assert!(!mm_gives_through(solar, "north", [0, 0, -1]));
        assert!(mm_gives_through(
            mm_source("mekanism:heat_generator").unwrap(),
            "north",
            [0, 1, 0]
        ));
        assert_eq!(mm_source("mekmm:recycler"), None);
    }

    /// A little world: (pos, name, facing).
    fn world(blocks: &[([i32; 3], &str, &str)]) -> Vec<([i32; 3], i64)> {
        let name = |p: [i32; 3]| {
            blocks
                .iter()
                .find(|(q, _, _)| *q == p)
                .map_or(String::from("minecraft:air"), |(_, n, _)| n.to_string())
        };
        let facing = |p: [i32; 3]| {
            blocks
                .iter()
                .find(|(q, _, _)| *q == p)
                .map_or("north", |(_, _, f)| *f)
        };
        mm_givers([0, 0, 0], &mut |p| name(p), &mut |p, face| {
            mm_source(&name(p)).is_some_and(|s| mm_gives_through(s, facing(p), face))
        })
    }

    #[test]
    fn givers_found_through_their_output_face_only() {
        // cube at x=1 facing west (towards the machine): found, direct
        assert_eq!(
            world(&[([1, 0, 0], "mekanism:basic_energy_cube", "west")]),
            vec![([1, 0, 0], MM_DIRECT)]
        );
        // facing east (away): not found
        assert!(world(&[([1, 0, 0], "mekanism:basic_energy_cube", "east")]).is_empty());
        // cube at x=3 facing west, two basic cables between: limit 8 kJ/t
        assert_eq!(
            world(&[
                ([1, 0, 0], "mekanism:basic_universal_cable", ""),
                ([2, 0, 0], "mekanism:ultimate_universal_cable", ""),
                ([3, 0, 0], "mekanism:ultimate_energy_cube", "west"),
            ]),
            vec![([3, 0, 0], 8_000)]
        );
        // solar generator above a cable: gives down into it
        assert_eq!(
            world(&[
                ([1, 0, 0], "mekanism:basic_universal_cable", ""),
                ([1, 1, 0], "mekanismgenerators:solar_generator", "north"),
            ]),
            vec![([1, 1, 0], 8_000)]
        );
        // solar generator beside the machine: its side doesn't give
        assert!(world(&[([1, 0, 0], "mekanismgenerators:solar_generator", "north")]).is_empty());
    }

    #[test]
    fn drawing_limits() {
        // cube with 4 MJ gives at most 4 kJ per tick, shared by every machine
        assert_eq!(mm_available(4_000_000, 4_000, (0, 0), 7), 4_000);
        assert_eq!(mm_available(4_000_000, 4_000, (7, 3_000), 7), 1_000);
        assert_eq!(mm_available(4_000_000, 4_000, (6, 3_000), 7), 4_000);
        assert_eq!(mm_available(500, 4_000, (0, 0), 7), 500);
        assert_eq!(
            mm_pull_plan(20_000, &[(4_000, MM_DIRECT), (100, 8_000)]),
            vec![4_000, 100]
        );
        assert_eq!(
            mm_pull_plan(3_000, &[(4_000, MM_DIRECT), (100, 8_000)]),
            vec![3_000, 0]
        );
        assert_eq!(mm_pull_plan(20_000, &[(16_000, 8_000)]), vec![8_000]);
        // an empty machine beside a full basic cube: full in 5 ticks
        let mut s = MmState::default();
        let mut cube = 4_000_000i64;
        let mut ticks = 0;
        while s.room() > 0 {
            let take = mm_pull_plan(
                s.room(),
                &[(mm_available(cube, 4_000, (0, 0), 1), MM_DIRECT)],
            )[0];
            s.energy += take;
            cube -= take;
            ticks += 1;
        }
        assert_eq!((ticks, s.energy, cube), (5, 20_000, 3_980_000));
    }

    #[test]
    fn joules_text() {
        assert_eq!(mm_joules(12_345), "12.345 J");
        assert_eq!(mm_joules(200_000), "200,0 kJ");
        assert_eq!(mm_joules(20_000), "20.000 J");
    }
}

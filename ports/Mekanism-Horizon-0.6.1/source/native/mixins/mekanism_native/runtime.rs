// Mekanism for Horizon, milestone 1 (ore doubling): energy, generators,
// cables, the energy cube and three machines, generic over the game's item
// stacks so it builds and tests on its own. This file is both the mod's
// Cargo library (its tests) and the module the bundle adds to the game
// (`crate::game::mekanism_native`); `glue.rs` adapts it to Horizon.
//
// Port of Mekanism and Mekanism Generators (MIT). Numbers are Mekanism
// 10.7's defaults, in joules (J) as Mekanism counts them.

pub const ID: &str = "mekanism_native";
pub const NAME: &str = "Mekanism";
pub const VERSION: &str = "0.6.0";
pub const DESCRIPTION: &str = "Mekanism: energia, geradores, cabos e cubos de energia (básico a definitivo), Câmara de Enriquecimento, Triturador, Fundição Energizada, Infusor Metalúrgico e Compressor de Ósmio. Precisa do mod de conteúdo mekanism.";

/// Ticks a basic machine takes per operation.
pub const MACHINE_TICKS: i32 = 200;
/// What one cable network carries per tick (Basic Universal Cable).
pub const CABLE_RATE: i64 = 8_000;
/// How far a network is followed, in cable blocks.
pub const CABLE_REACH: usize = 512;
pub const CABLE: &str = "mekanism:basic_universal_cable";

/// The universal cable tiers (`CableTier`): the block and the joules it
/// carries per tick. The basic one's 0.4.0 variants (`CABLE_m<mask>`) are
/// basic cables too.
pub const CABLE_TIERS: [(&str, i64); 4] = [
    (CABLE, CABLE_RATE),
    ("mekanism:advanced_universal_cable", 128_000),
    ("mekanism:elite_universal_cable", 1_024_000),
    ("mekanism:ultimate_universal_cable", 8_192_000),
];

/// What a cable carries per tick; None for a block that isn't a cable.
pub fn cable_rate(name: &str) -> Option<i64> {
    if cable_mask(name).is_some() {
        return Some(CABLE_RATE);
    }
    CABLE_TIERS.iter().find(|(n, _)| *n == name).map(|(_, r)| *r)
}

/// No cable between a giver and a target: only the giver's output limits it.
pub const DIRECT: i64 = i64::MAX;

/// The cable is Horizon's pipe block (`CABLE`), which joins its neighbours
/// by itself. Version 0.4.0, on a Horizon without pipe blocks, made it 64
/// blocks, one per set of joined sides (bits in `DIRS` order: down, up,
/// north, south, west, east): `CABLE_m<mask>`. Those already placed stay
/// cables and keep their shapes up to date; one left with no sides becomes
/// the pipe block.
pub fn cable_name(mask: u8) -> String {
    let mask = mask & 63;
    if mask == 0 { CABLE.to_owned() } else { format!("{CABLE}_m{mask}") }
}

pub fn cable_mask(name: &str) -> Option<u8> {
    if name == CABLE {
        return Some(0);
    }
    let n: u8 = name.strip_prefix(CABLE)?.strip_prefix("_m")?.parse().ok()?;
    (1..64).contains(&n).then_some(n)
}

pub fn is_cable(name: &str) -> bool {
    cable_rate(name).is_some()
}

/// Whether a cable joins a block: another cable or any Mekanism block.
pub fn connects(name: &str) -> bool {
    is_cable(name) || Kind::of(name).is_some()
}

/// The cable's sides from its six neighbours' names, in `DIRS` order.
pub fn cable_mask_from(neighbours: &[&str; 6]) -> u8 {
    neighbours
        .iter()
        .enumerate()
        .filter(|(_, n)| connects(n))
        .fold(0, |m, (i, _)| m | 1 << i)
}

/// The mask after the neighbour on side `side` became `neighbour`.
pub fn cable_update(mask: u8, side: usize, neighbour: &str) -> u8 {
    if side >= 6 {
        return mask;
    }
    if connects(neighbour) { mask | 1 << side } else { mask & !(1 << side) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    EnrichmentChamber,
    Crusher,
    EnergizedSmelter,
    MetallurgicInfuser,
    HeatGenerator,
    SolarGenerator,
    EnergyCube,
    OsmiumCompressor,
    AdvancedEnergyCube,
    EliteEnergyCube,
    UltimateEnergyCube,
}

impl Kind {
    pub const ALL: [Kind; 11] = [
        Kind::EnrichmentChamber,
        Kind::Crusher,
        Kind::EnergizedSmelter,
        Kind::MetallurgicInfuser,
        Kind::HeatGenerator,
        Kind::SolarGenerator,
        Kind::EnergyCube,
        Kind::OsmiumCompressor,
        Kind::AdvancedEnergyCube,
        Kind::EliteEnergyCube,
        Kind::UltimateEnergyCube,
    ];
    pub fn block(self) -> &'static str {
        match self {
            Kind::EnrichmentChamber => "mekanism:enrichment_chamber",
            Kind::Crusher => "mekanism:crusher",
            Kind::EnergizedSmelter => "mekanism:energized_smelter",
            Kind::MetallurgicInfuser => "mekanism:metallurgic_infuser",
            Kind::HeatGenerator => "mekanism:heat_generator",
            Kind::SolarGenerator => "mekanism:solar_generator",
            Kind::EnergyCube => "mekanism:basic_energy_cube",
            Kind::OsmiumCompressor => "mekanism:osmium_compressor",
            Kind::AdvancedEnergyCube => "mekanism:advanced_energy_cube",
            Kind::EliteEnergyCube => "mekanism:elite_energy_cube",
            Kind::UltimateEnergyCube => "mekanism:ultimate_energy_cube",
        }
    }
    /// The energy cubes, every tier (`EnergyCubeTier`).
    pub fn is_cube(self) -> bool {
        matches!(
            self,
            Kind::EnergyCube | Kind::AdvancedEnergyCube | Kind::EliteEnergyCube | Kind::UltimateEnergyCube
        )
    }
    pub fn of(block: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.block() == block)
    }
    /// The block entity type id.
    pub fn entity(self) -> &'static str {
        self.block()
    }
    /// Joules stored (StorageConfig, GeneratorsStorageConfig, energy cube tier).
    pub fn capacity(self) -> i64 {
        match self {
            Kind::EnrichmentChamber | Kind::Crusher | Kind::EnergizedSmelter | Kind::MetallurgicInfuser => 20_000,
            Kind::HeatGenerator => 160_000,
            Kind::SolarGenerator => 96_000,
            Kind::EnergyCube => 4_000_000,
            Kind::OsmiumCompressor => 80_000,
            Kind::AdvancedEnergyCube => 16_000_000,
            Kind::EliteEnergyCube => 64_000_000,
            Kind::UltimateEnergyCube => 256_000_000,
        }
    }
    /// Joules a machine uses per tick of work (UsageConfig).
    pub fn usage(self) -> i64 {
        match self {
            Kind::EnrichmentChamber | Kind::Crusher | Kind::EnergizedSmelter | Kind::MetallurgicInfuser => 50,
            Kind::OsmiumCompressor => 100,
            _ => 0,
        }
    }
    pub fn is_machine(self) -> bool {
        self.usage() > 0
    }
    /// Gives energy to the network.
    pub fn is_source(self) -> bool {
        matches!(self, Kind::HeatGenerator | Kind::SolarGenerator) || self.is_cube()
    }
    /// Takes energy from the network (on `face` of its own, for the cube: not its output face).
    pub fn accepts(self) -> bool {
        self.is_machine() || self.is_cube()
    }
    /// Joules it gives the network per tick at most (a cube takes as much in).
    pub fn output(self) -> i64 {
        match self {
            Kind::HeatGenerator => 400,
            Kind::SolarGenerator => 100,
            Kind::EnergyCube => 4_000,
            Kind::AdvancedEnergyCube => 16_000,
            Kind::EliteEnergyCube => 64_000,
            Kind::UltimateEnergyCube => 256_000,
            _ => 0,
        }
    }
    /// The machine recipe type it runs (a mod type in the content mod, or vanilla smelting).
    pub fn recipe_type(self) -> Option<&'static str> {
        match self {
            Kind::EnrichmentChamber => Some("mekanism:enriching"),
            Kind::Crusher => Some("mekanism:crushing"),
            Kind::EnergizedSmelter => Some("minecraft:smelting"),
            Kind::MetallurgicInfuser => Some("mekanism:metallurgic_infusing"),
            Kind::OsmiumCompressor => Some("mekanism:compressing"),
            _ => None,
        }
    }
}

/// Heat Generator: joules per tick while burning, and per lava block beside it.
pub const HEAT_GENERATION: i64 = 200;
pub const HEAT_LAVA: i64 = 30;
/// Solar Generator: joules per tick in daylight under open sky.
pub const SOLAR_GENERATION: i64 = 50;

/// Slots of the machines' furnace menu: 0 input, 1 fuel (the heat
/// generator's only slot; hidden on machines), 2 output.
pub const INPUT: usize = 0;
pub const FUEL: usize = 1;
pub const OUTPUT: usize = 2;

/// Upgrades a machine takes of each kind (Mekanism's maximum).
pub const MAX_UPGRADES: u8 = 8;
pub const SPEED_UPGRADE: &str = "mekanism:upgrade_speed";
pub const ENERGY_UPGRADE: &str = "mekanism:upgrade_energy";

/// `MekanismUtils`: each kind of upgrade multiplies by 10^(n/8), so 8 make it 10x.
pub fn upgrade_multiplier(n: u8) -> f64 {
    10f64.powf(f64::from(n.min(MAX_UPGRADES)) / 8.0)
}

/// Ticks per operation with `speed` upgrades (200 down to 20).
pub fn machine_ticks(speed: u8) -> i32 {
    (f64::from(MACHINE_TICKS) / upgrade_multiplier(speed)).round().max(1.0) as i32
}

/// Joules per tick of work: speed costs its square, energy upgrades divide it.
pub fn machine_usage(kind: Kind, speed: u8, energy: u8) -> i64 {
    let u = kind.usage() as f64 * upgrade_multiplier(speed).powi(2) / upgrade_multiplier(energy);
    (u.round() as i64).max(if kind.usage() > 0 { 1 } else { 0 })
}

/// Joules stored: energy upgrades multiply a machine's buffer.
pub fn capacity(kind: Kind, energy_upgrades: u8) -> i64 {
    if kind.is_machine() {
        (kind.capacity() as f64 * upgrade_multiplier(energy_upgrades)).round() as i64
    } else {
        kind.capacity()
    }
}

/// Units the menu's 16-bit data slots count energy in.
pub fn energy_unit(kind: Kind, energy_upgrades: u8) -> i64 {
    ((capacity(kind, energy_upgrades) + 29_999) / 30_000).max(1)
}

/// The infuser's buffer holds one infuse type at a time, up to this much.
pub const INFUSE_CAPACITY: i32 = 1000;

/// An infuse type: 0 none, then these (kept as a number in the save; new
/// ones only go at the end).
pub const INFUSE_TYPES: [&str; 4] = ["carbon", "redstone", "diamond", "refined_obsidian"];

pub fn infuse_code(name: &str) -> u8 {
    INFUSE_TYPES.iter().position(|t| *t == name).map_or(0, |i| i as u8 + 1)
}

pub fn infuse_label(code: u8) -> &'static str {
    match code {
        1 => "Carbono",
        2 => "Redstone",
        3 => "Diamante",
        4 => "Obs. Refinada",
        _ => "Vazio",
    }
}

/// The osmium compressor's tank of liquid osmium (`MAX_GAS`), in mB.
pub const OSMIUM_CAPACITY: i32 = 210;
/// The chemical a compressing recipe needs; the only one the port has.
pub const OSMIUM: &str = "osmium";

/// The saved numbers of every Mekanism block entity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub energy: i64,
    pub progress: i32,
    pub burn_remaining: i32,
    pub burn_total: i32,
    pub infuse_type: u8,
    pub infuse_amount: i32,
    pub speed_upgrades: u8,
    pub energy_upgrades: u8,
    /// The osmium compressor's liquid osmium (mB), and what the operation
    /// under way has used of it.
    pub chemical: i32,
    pub chemical_used: i32,
}

impl State {
    pub fn capacity(&self, kind: Kind) -> i64 {
        capacity(kind, self.energy_upgrades)
    }
    pub fn add(&mut self, kind: Kind, joules: i64) -> i64 {
        let room = (self.capacity(kind) - self.energy).max(0);
        let took = joules.clamp(0, room);
        self.energy += took;
        took
    }
    /// The furnace menu's four data slots: energy (in `energy_unit`s), work
    /// progress and fuel burn, each in thousandths, and the generation in J/t.
    /// The infuser sends its buffer (amount, type) in the last two.
    pub fn data(&self, kind: Kind, generating: i64) -> [i32; 4] {
        let unit = energy_unit(kind, self.energy_upgrades);
        let permille = |a: i64, b: i64| if b > 0 { (a * 1000 / b).clamp(0, 1000) as i32 } else { 0 };
        let energy = (self.energy / unit).clamp(0, 32_767) as i32;
        let progress = permille(i64::from(self.progress), i64::from(machine_ticks(self.speed_upgrades)));
        let upgrades = i32::from(self.speed_upgrades.min(15)) | (i32::from(self.energy_upgrades.min(15)) << 4);
        match kind {
            // buffer amount, then type and upgrades packed: type | speed << 4 | energy << 8
            Kind::MetallurgicInfuser => [
                energy,
                progress,
                self.infuse_amount.clamp(0, INFUSE_CAPACITY),
                i32::from(self.infuse_type.min(15)) | (upgrades << 4),
            ],
            Kind::EnrichmentChamber | Kind::Crusher | Kind::EnergizedSmelter => [energy, progress, upgrades, 0],
            // its osmium tank, then the upgrades
            Kind::OsmiumCompressor => [energy, progress, self.chemical.clamp(0, OSMIUM_CAPACITY), upgrades],
            _ => [
                energy,
                progress,
                permille(i64::from(self.burn_remaining), i64::from(self.burn_total)),
                generating.clamp(0, 32_767) as i32,
            ],
        }
    }
    /// One `item` put into a machine as an upgrade: false when it isn't one,
    /// the block isn't a machine, or it has the most of that kind already.
    pub fn upgrade(&mut self, kind: Kind, item: &str) -> bool {
        if !kind.is_machine() {
            return false;
        }
        let slot = match item {
            SPEED_UPGRADE => &mut self.speed_upgrades,
            ENERGY_UPGRADE => &mut self.energy_upgrades,
            _ => return false,
        };
        if *slot >= MAX_UPGRADES {
            return false;
        }
        *slot += 1;
        true
    }
}

/// The upgrades packed in a machine's data slots: (speed, energy).
pub fn unpack_upgrades(kind: Kind, data: &[i32]) -> (u8, u8) {
    let packed = match kind {
        Kind::MetallurgicInfuser => data.get(3).copied().unwrap_or(0) >> 4,
        Kind::EnrichmentChamber | Kind::Crusher | Kind::EnergizedSmelter => data.get(2).copied().unwrap_or(0),
        Kind::OsmiumCompressor => data.get(3).copied().unwrap_or(0),
        _ => 0,
    };
    ((packed & 15) as u8, ((packed >> 4) & 15) as u8)
}

/// What a machine needs from the game, over its item stacks `S`.
pub trait Stacks<S> {
    fn count(&self, s: &S) -> u32;
    /// Whether `result` stacks onto `out` with room for all of it.
    fn fits(&self, out: &S, result: &S) -> bool;
    fn grow(&self, out: &mut S, result: &S);
    /// `n` taken from the stack; None when it is used up.
    fn take(&self, s: S, n: u32) -> Option<S>;
}

/// One tick of a machine with the recipe for its input (`count` of the
/// input makes `result`). Returns whether it worked (its `lit`).
pub fn machine_tick<S: Clone, K: Stacks<S>>(
    kind: Kind,
    state: &mut State,
    slots: &mut [Option<S>],
    recipe: Option<(u32, S)>,
    stacks: &K,
) -> bool {
    let Some((needed, result)) = recipe else {
        state.progress = 0;
        return false;
    };
    let enough = slots[INPUT].as_ref().is_some_and(|i| stacks.count(i) >= needed.max(1));
    let room = slots[OUTPUT].as_ref().is_none_or(|o| stacks.fits(o, &result));
    if !enough || !room {
        state.progress = 0;
        return false;
    }
    let usage = machine_usage(kind, state.speed_upgrades, state.energy_upgrades);
    if state.energy < usage {
        return false;
    }
    state.energy -= usage;
    state.progress += 1;
    if state.progress >= machine_ticks(state.speed_upgrades) {
        state.progress = 0;
        match &mut slots[OUTPUT] {
            Some(o) => stacks.grow(o, &result),
            None => slots[OUTPUT] = Some(result),
        }
        if let Some(i) = slots[INPUT].take() {
            slots[INPUT] = stacks.take(i, needed.max(1));
        }
    }
    true
}

/// One tick of the metallurgic infuser. `conversion` is what the item in
/// its infuse slot (`FUEL`) gives the buffer, one item at a time while it
/// fits and matches the type held; `recipe` is the recipe for its input:
/// count, result, infuse type and amount, used up when an operation ends.
/// Returns whether it worked.
pub fn infuser_tick<S: Clone, K: Stacks<S>>(
    state: &mut State,
    slots: &mut [Option<S>],
    recipe: Option<(u32, S, u8, i32)>,
    conversion: Option<(u8, i32)>,
    stacks: &K,
) -> bool {
    let kind = Kind::MetallurgicInfuser;
    if state.infuse_amount <= 0 {
        state.infuse_amount = 0;
        state.infuse_type = 0;
    }
    if let Some((t, amount)) = conversion {
        let same = state.infuse_type == 0 || state.infuse_type == t;
        if t != 0 && amount > 0 && same && state.infuse_amount + amount <= INFUSE_CAPACITY {
            if let Some(item) = slots[FUEL].take() {
                slots[FUEL] = stacks.take(item, 1);
                state.infuse_type = t;
                state.infuse_amount += amount;
            }
        }
    }
    let Some((needed, result, t, amount)) = recipe else {
        state.progress = 0;
        return false;
    };
    if state.infuse_type != t || state.infuse_amount < amount {
        state.progress = 0;
        return false;
    }
    let enough = slots[INPUT].as_ref().is_some_and(|i| stacks.count(i) >= needed.max(1));
    let room = slots[OUTPUT].as_ref().is_none_or(|o| stacks.fits(o, &result));
    if !enough || !room {
        state.progress = 0;
        return false;
    }
    let usage = machine_usage(kind, state.speed_upgrades, state.energy_upgrades);
    if state.energy < usage {
        return false;
    }
    state.energy -= usage;
    state.progress += 1;
    if state.progress >= machine_ticks(state.speed_upgrades) {
        state.progress = 0;
        state.infuse_amount -= amount;
        if state.infuse_amount <= 0 {
            state.infuse_amount = 0;
            state.infuse_type = 0;
        }
        match &mut slots[OUTPUT] {
            Some(o) => stacks.grow(o, &result),
            None => slots[OUTPUT] = Some(result),
        }
        if let Some(i) = slots[INPUT].take() {
            slots[INPUT] = stacks.take(i, needed.max(1));
        }
    }
    true
}

/// The osmium a compressing operation uses this tick: what is left of its
/// total (`per_tick` for each of the 200 base ticks, so 200 mB at 1 mB/t,
/// however many speed upgrades: `ChemicalUsageMultiplier.constantUse`)
/// spread over the ticks left.
pub fn compressor_use(state: &State, per_tick: i32) -> i32 {
    let total = per_tick.max(0).saturating_mul(MACHINE_TICKS);
    let left = (total - state.chemical_used).max(0);
    let ticks_left = (machine_ticks(state.speed_upgrades) - state.progress).max(1);
    (left + ticks_left - 1) / ticks_left
}

/// One tick of the osmium compressor. `conversion` is what the item in its
/// osmium slot (`FUEL`) puts in its tank (an osmium ingot: 200 mB), one item
/// at a time while it fits; `recipe` is the recipe for its input: count,
/// result and osmium per tick. Returns whether it worked.
pub fn compressor_tick<S: Clone, K: Stacks<S>>(
    state: &mut State,
    slots: &mut [Option<S>],
    recipe: Option<(u32, S, i32)>,
    conversion: Option<i32>,
    stacks: &K,
) -> bool {
    let kind = Kind::OsmiumCompressor;
    state.chemical = state.chemical.clamp(0, OSMIUM_CAPACITY);
    if let Some(amount) = conversion
        && amount > 0
        && state.chemical + amount <= OSMIUM_CAPACITY
        && let Some(item) = slots[FUEL].take()
    {
        slots[FUEL] = stacks.take(item, 1);
        state.chemical += amount;
    }
    let Some((needed, result, per_tick)) = recipe else {
        state.progress = 0;
        state.chemical_used = 0;
        return false;
    };
    let enough = slots[INPUT].as_ref().is_some_and(|i| stacks.count(i) >= needed.max(1));
    let room = slots[OUTPUT].as_ref().is_none_or(|o| stacks.fits(o, &result));
    if !enough || !room {
        state.progress = 0;
        state.chemical_used = 0;
        return false;
    }
    let usage = machine_usage(kind, state.speed_upgrades, state.energy_upgrades);
    let osmium = compressor_use(state, per_tick);
    // Out of energy or osmium: it waits, keeping its progress.
    if state.energy < usage || state.chemical < osmium {
        return false;
    }
    state.energy -= usage;
    state.chemical -= osmium;
    state.chemical_used += osmium;
    state.progress += 1;
    if state.progress >= machine_ticks(state.speed_upgrades) {
        state.progress = 0;
        state.chemical_used = 0;
        match &mut slots[OUTPUT] {
            Some(o) => stacks.grow(o, &result),
            None => slots[OUTPUT] = Some(result),
        }
        if let Some(i) = slots[INPUT].take() {
            slots[INPUT] = stacks.take(i, needed.max(1));
        }
    }
    true
}

/// One tick of a heat generator: it burns a fuel item (its vanilla burn
/// time, in ticks) for 200 J/t while it has room, and gets 30 J/t for each
/// lava block beside it; a full one pauses its fuel. `burn_time` gives a fuel's ticks (0: not a fuel),
/// `remainder` what a used fuel leaves. Returns (burning, J/t made).
pub fn heat_tick<S: Clone, K: Stacks<S>>(
    state: &mut State,
    fuel: &mut Option<S>,
    lava_beside: u32,
    burn_time: impl Fn(&S) -> i32,
    remainder: impl Fn(&S) -> Option<S>,
    stacks: &K,
) -> (bool, i64) {
    let kind = Kind::HeatGenerator;
    let mut made = state.add(kind, HEAT_LAVA * i64::from(lava_beside));
    let room = state.energy + HEAT_GENERATION <= kind.capacity();
    if state.burn_remaining <= 0 && room {
        if let Some(f) = fuel.take() {
            let ticks = burn_time(&f);
            if ticks > 0 {
                state.burn_remaining = ticks;
                state.burn_total = ticks;
                let rest = remainder(&f);
                *fuel = stacks.take(f, 1).or(rest);
            } else {
                *fuel = Some(f);
            }
        }
    }
    // A full generator pauses its fuel instead of wasting it.
    let burning = state.burn_remaining > 0 && state.energy < kind.capacity();
    if burning {
        state.burn_remaining -= 1;
        made += state.add(kind, HEAT_GENERATION);
    }
    (burning, made)
}

/// One tick of a solar generator. Returns the J/t made.
pub fn solar_tick(state: &mut State, sunlit: bool) -> i64 {
    if sunlit { state.add(Kind::SolarGenerator, SOLAR_GENERATION) } else { 0 }
}

/// Whether the sun is up at `day_time` (ticks; 0 is sunrise), as
/// `Level.isDay` reads it in clear weather.
pub fn is_day(day_time: i64) -> bool {
    let t = day_time.rem_euclid(24_000);
    !(12_542..23_460).contains(&t)
}

/// `amount` split over targets with `rooms` of free space: equal shares,
/// what a full target can't take passed on to the others. The sum given
/// never exceeds `amount` nor any room.
pub fn split(amount: i64, rooms: &[i64]) -> Vec<i64> {
    let mut given = vec![0i64; rooms.len()];
    let mut left = amount.max(0);
    loop {
        let open: Vec<usize> = (0..rooms.len()).filter(|&i| rooms[i] - given[i] > 0).collect();
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

/// The six directions as offsets: down, up, north, south, west, east.
pub const DIRS: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];

pub fn offset(p: [i32; 3], d: [i32; 3]) -> [i32; 3] {
    [p[0] + d[0], p[1] + d[1], p[2] + d[2]]
}

/// The offset of a horizontal `facing` (the furnace template's).
pub fn facing_offset(facing: &str) -> [i32; 3] {
    match facing {
        "south" => [0, 0, 1],
        "west" => [-1, 0, 0],
        "east" => [1, 0, 0],
        _ => [0, 0, -1],
    }
}

/// Where energy from `from` goes: the acceptors beside the sides it gives
/// through, and those along cables reached from them (up to
/// `CABLE_REACH` cables). `block` names the block at a position; `accepts`
/// says whether a block at a position takes energy from the direction it
/// came from. The source itself is never one.
pub fn acceptors(
    from: [i32; 3],
    sides: &[[i32; 3]],
    block: &mut impl FnMut([i32; 3]) -> String,
    accepts: &mut impl FnMut([i32; 3], [i32; 3]) -> bool,
) -> Vec<[i32; 3]> {
    acceptors_rated(from, sides, block, accepts).into_iter().map(|(p, _)| p).collect()
}

/// `acceptors` with what reaches each one per tick at most: the slowest
/// cable tier on the way the search found it by (`DIRECT` beside the giver,
/// with no cable between). Mekanism's tiers join each other; a basic cable
/// in an ultimate line limits what goes through it to 8 kJ/t.
pub fn acceptors_rated(
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
        if let Some(rate) = cable_rate(&block(p)) {
            if seen.insert(p) {
                queue.push_back((p, limit.min(rate)));
            }
        } else if accepts(p, came) && found.insert(p) {
            out.push((p, limit));
        }
    };
    for &d in sides {
        visit(offset(from, d), d, DIRECT, &mut queue, &mut seen);
    }
    let mut cables = 0;
    while let Some((c, limit)) = queue.pop_front() {
        cables += 1;
        if cables > CABLE_REACH {
            break;
        }
        for d in DIRS {
            visit(offset(c, d), d, limit, &mut queue, &mut seen);
        }
    }
    out
}

/// What a giver with `amount` this tick hands each target, given each one's
/// free room and the most its way carries (`acceptors_rated`): equal shares
/// as `split`, none past a target's way, and no more in all than the
/// fastest way carries.
pub fn share_out(amount: i64, rooms: &[i64], limits: &[i64]) -> Vec<i64> {
    let total = limits.iter().copied().max().map_or(0, |m| amount.min(m));
    let capped: Vec<i64> = rooms
        .iter()
        .zip(limits.iter().chain(std::iter::repeat(&0)))
        .map(|(r, l)| (*r).min(*l).max(0))
        .collect();
    split(total, &capped)
}

/// "12.345 J", "1.234,5 kJ", "3,2 MJ", as Mekanism shows energy (pt-BR digits).
pub fn joules(j: i64) -> String {
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
        format!("{},{} MJ", group(j / 1_000_000), (j % 1_000_000).abs() / 100_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Stack = (&'static str, u32);
    struct T;
    impl Stacks<Stack> for T {
        fn count(&self, s: &Stack) -> u32 {
            s.1
        }
        fn fits(&self, out: &Stack, r: &Stack) -> bool {
            out.0 == r.0 && out.1 + r.1 <= 64
        }
        fn grow(&self, out: &mut Stack, r: &Stack) {
            out.1 += r.1;
        }
        fn take(&self, s: Stack, n: u32) -> Option<Stack> {
            (s.1 > n).then_some((s.0, s.1 - n))
        }
    }

    #[test]
    fn mekanism_defaults() {
        assert_eq!(Kind::of("mekanism:enrichment_chamber"), Some(Kind::EnrichmentChamber));
        assert_eq!(Kind::EnrichmentChamber.capacity(), 20_000);
        assert_eq!(Kind::Crusher.usage(), 50);
        assert_eq!(Kind::EnergyCube.capacity(), 4_000_000);
        assert!(Kind::of(CABLE).is_none());
        assert!(Kind::EnergyCube.is_source() && Kind::EnergyCube.accepts());
        assert!(!Kind::HeatGenerator.accepts());
        // data slots stay 16-bit for every capacity, upgraded or not
        for k in Kind::ALL {
            for e in [0, 8] {
                assert!(capacity(k, e) / energy_unit(k, e) <= 32_767, "{k:?}");
            }
        }
    }

    #[test]
    fn enrichment_doubles_an_ore_in_200_ticks_for_10_kj() {
        let mut s = State { energy: 20_000, ..State::default() };
        let mut slots = vec![Some(("iron_ore", 2)), None, None];
        let recipe = || Some((1, ("dust_iron", 2)));
        for _ in 0..199 {
            assert!(machine_tick(Kind::EnrichmentChamber, &mut s, &mut slots, recipe(), &T));
        }
        assert_eq!(slots[OUTPUT], None);
        machine_tick(Kind::EnrichmentChamber, &mut s, &mut slots, recipe(), &T);
        assert_eq!(slots[OUTPUT], Some(("dust_iron", 2)));
        assert_eq!(slots[INPUT], Some(("iron_ore", 1)));
        assert_eq!(s.energy, 20_000 - 200 * 50);
    }

    #[test]
    fn raw_ore_takes_three_for_four() {
        let mut s = State { energy: 20_000, ..State::default() };
        let mut slots = vec![Some(("raw_iron", 2)), None, None];
        assert!(!machine_tick(Kind::EnrichmentChamber, &mut s, &mut slots, Some((3, ("dust_iron", 4))), &T));
        slots[INPUT] = Some(("raw_iron", 3));
        for _ in 0..200 {
            machine_tick(Kind::EnrichmentChamber, &mut s, &mut slots, Some((3, ("dust_iron", 4))), &T);
        }
        assert_eq!(slots[OUTPUT], Some(("dust_iron", 4)));
        assert_eq!(slots[INPUT], None);
    }

    #[test]
    fn machine_waits_without_energy_and_keeps_progress() {
        let mut s = State { energy: 120, ..State::default() };
        let mut slots = vec![Some(("iron_ore", 1)), None, None];
        let r = || Some((1, ("dust_iron", 2)));
        assert!(machine_tick(Kind::Crusher, &mut s, &mut slots, r(), &T));
        assert!(machine_tick(Kind::Crusher, &mut s, &mut slots, r(), &T));
        assert!(!machine_tick(Kind::Crusher, &mut s, &mut slots, r(), &T));
        assert_eq!(s.progress, 2);
        assert_eq!(s.energy, 20);
    }

    #[test]
    fn full_output_stops_the_machine() {
        let mut s = State { energy: 20_000, progress: 50, ..State::default() };
        let mut slots = vec![Some(("iron_ore", 1)), None, Some(("dust_iron", 63))];
        assert!(!machine_tick(Kind::EnrichmentChamber, &mut s, &mut slots, Some((1, ("dust_iron", 2))), &T));
        assert_eq!(s.progress, 0);
        assert_eq!(s.energy, 20_000);
    }

    #[test]
    fn infuser_makes_steel_from_iron_and_coal() {
        let c = infuse_code("carbon");
        assert_eq!(c, 1);
        let mut s = State { energy: 20_000, ..State::default() };
        let mut slots = vec![Some(("iron_ingot", 1)), Some(("coal", 1)), None];
        let recipe = || Some((1, ("enriched_iron", 1), c, 10));
        // the coal goes into the buffer as 10 carbon first
        for _ in 0..200 {
            infuser_tick(&mut s, &mut slots, recipe(), Some((c, 10)), &T);
        }
        assert_eq!(slots[OUTPUT], Some(("enriched_iron", 1)));
        assert_eq!(slots[FUEL], None);
        assert_eq!((s.infuse_type, s.infuse_amount), (0, 0), "an empty buffer forgets its type");
        assert_eq!(s.energy, 20_000 - 200 * 50);
    }

    #[test]
    fn infuser_keeps_one_type_and_its_capacity() {
        let (c, r) = (infuse_code("carbon"), infuse_code("redstone"));
        let mut s = State { energy: 20_000, infuse_type: c, infuse_amount: 995, ..State::default() };
        let mut slots = vec![None, Some(("redstone", 5)), None];
        infuser_tick(&mut s, &mut slots, None, Some((r, 10)), &T);
        assert_eq!(slots[FUEL], Some(("redstone", 5)), "carbon in the buffer: redstone waits");
        s.infuse_type = r;
        infuser_tick(&mut s, &mut slots, None, Some((r, 10)), &T);
        assert_eq!(s.infuse_amount, 995, "no room for 10 more");
        s.infuse_amount = 980;
        infuser_tick(&mut s, &mut slots, None, Some((r, 10)), &T);
        assert_eq!((s.infuse_amount, slots[FUEL]), (990, Some(("redstone", 4))));
    }

    #[test]
    fn infuser_needs_the_recipes_type_and_amount() {
        let (c, r) = (infuse_code("carbon"), infuse_code("redstone"));
        let mut s = State { energy: 20_000, infuse_type: c, infuse_amount: 15, ..State::default() };
        let mut slots = vec![Some(("iron_ingot", 1)), None, None];
        assert!(!infuser_tick(&mut s, &mut slots, Some((1, ("circuit", 1), r, 20)), None, &T));
        assert!(!infuser_tick(&mut s, &mut slots, Some((1, ("x", 1), c, 20)), None, &T));
        assert!(infuser_tick(&mut s, &mut slots, Some((1, ("x", 1), c, 10)), None, &T));
        assert_eq!(State { infuse_amount: 400, infuse_type: c, ..s }.data(Kind::MetallurgicInfuser, 0)[2..], [400, 1]);
    }

    #[test]
    fn heat_generator_burns_coal_for_200_jt() {
        let mut s = State::default();
        let mut fuel = Some(("coal", 2));
        let (burning, made) = heat_tick(&mut s, &mut fuel, 0, |f| if f.0 == "coal" { 1600 } else { 0 }, |_| None, &T);
        assert!(burning);
        assert_eq!(made, 200);
        assert_eq!(fuel, Some(("coal", 1)));
        for _ in 0..1599 {
            heat_tick(&mut s, &mut fuel, 0, |_| 1600, |_| None, &T);
        }
        // 160 kJ is full after 800 ticks: the coal's other 800 wait for room.
        assert_eq!(s.energy, Kind::HeatGenerator.capacity());
        assert_eq!(s.burn_remaining, 800);
        assert_eq!(fuel, Some(("coal", 1)), "the second coal waits for the first to burn out");
    }

    #[test]
    fn heat_generator_stops_burning_when_full_and_counts_lava() {
        let mut s = State { energy: Kind::HeatGenerator.capacity() - 100, ..State::default() };
        let mut fuel = Some(("coal", 1));
        let (burning, made) = heat_tick(&mut s, &mut fuel, 2, |_| 1600, |_| None, &T);
        assert!(!burning, "no room for 200 J: the coal stays");
        assert_eq!(fuel, Some(("coal", 1)));
        assert_eq!(made, 60);
    }

    #[test]
    fn solar_needs_daylight() {
        assert!(is_day(1000) && is_day(6000) && is_day(23_500));
        assert!(!is_day(13_000) && !is_day(18_000));
        let mut s = State::default();
        assert_eq!(solar_tick(&mut s, true), 50);
        assert_eq!(solar_tick(&mut s, false), 0);
        assert_eq!(s.energy, 50);
    }

    #[test]
    fn split_is_fair_and_bounded() {
        assert_eq!(split(300, &[1000, 1000, 1000]), vec![100, 100, 100]);
        assert_eq!(split(300, &[50, 1000, 1000]), vec![50, 125, 125]);
        assert_eq!(split(100, &[0, 0]), vec![0, 0]);
        assert_eq!(split(10, &[100, 100, 100]).iter().sum::<i64>(), 10);
        assert_eq!(split(8000, &[20, 30]), vec![20, 30]);
    }

    #[test]
    fn cables_carry_energy_to_every_machine_on_them() {
        use std::collections::HashMap;
        // generator at 0,0,0; cable east to x=3; machines at 4,0,0 and 2,1,0; a cube at 0,0,1
        let mut world: HashMap<[i32; 3], &str> = HashMap::new();
        world.insert([0, 0, 0], "mekanism:heat_generator");
        for x in 1..=3 {
            world.insert([x, 0, 0], CABLE);
        }
        world.insert([4, 0, 0], "mekanism:crusher");
        world.insert([2, 1, 0], "mekanism:enrichment_chamber");
        world.insert([0, 0, 1], "mekanism:basic_energy_cube");
        let mut block = |p: [i32; 3]| world.get(&p).copied().unwrap_or("minecraft:air").to_owned();
        let names = world.clone();
        let mut accepts = |p: [i32; 3], _from: [i32; 3]| {
            names.get(&p).and_then(|b| Kind::of(b)).is_some_and(|k| k.accepts())
        };
        let mut got = acceptors([0, 0, 0], &DIRS, &mut block, &mut accepts);
        got.sort();
        assert_eq!(got, vec![[0, 0, 1], [2, 1, 0], [4, 0, 0]]);
    }

    #[test]
    fn upgrades_follow_mekanism() {
        assert_eq!(machine_ticks(0), 200);
        assert_eq!(machine_ticks(8), 20);
        assert_eq!(machine_ticks(4), 63);
        assert_eq!(machine_usage(Kind::Crusher, 0, 0), 50);
        assert_eq!(machine_usage(Kind::Crusher, 8, 0), 5_000);
        assert_eq!(machine_usage(Kind::Crusher, 0, 8), 5);
        assert_eq!(machine_usage(Kind::Crusher, 8, 8), 500);
        assert_eq!(capacity(Kind::Crusher, 8), 200_000);
        assert_eq!(capacity(Kind::EnergyCube, 8), 4_000_000);
        // per operation: speed costs 10x the energy, energy upgrades give it back
        assert_eq!(machine_usage(Kind::Crusher, 8, 0) * i64::from(machine_ticks(8)), 100_000);
        let mut s = State::default();
        for _ in 0..8 {
            assert!(s.upgrade(Kind::Crusher, SPEED_UPGRADE));
        }
        assert!(!s.upgrade(Kind::Crusher, SPEED_UPGRADE), "8 at most");
        assert!(!s.upgrade(Kind::HeatGenerator, ENERGY_UPGRADE), "machines only");
        assert!(!s.upgrade(Kind::Crusher, "minecraft:dirt"));
    }

    #[test]
    fn speed_upgrades_make_a_machine_finish_sooner() {
        let mut s = State { energy: 200_000, speed_upgrades: 8, energy_upgrades: 8, ..State::default() };
        let mut slots = vec![Some(("iron_ore", 1)), None, None];
        for _ in 0..20 {
            machine_tick(Kind::EnrichmentChamber, &mut s, &mut slots, Some((1, ("dust_iron", 2))), &T);
        }
        assert_eq!(slots[OUTPUT], Some(("dust_iron", 2)));
        assert_eq!(s.energy, 200_000 - 20 * 500);
    }

    #[test]
    fn cable_variants() {
        assert_eq!(cable_name(0), CABLE);
        assert_eq!(cable_name(63), "mekanism:basic_universal_cable_m63");
        for m in 0..64u8 {
            assert_eq!(cable_mask(&cable_name(m)), Some(m));
        }
        assert_eq!(cable_mask("mekanism:basic_universal_cable_m64"), None);
        assert_eq!(cable_mask("mekanism:basic_universal_cable_m0"), None);
        assert!(!is_cable("mekanism:crusher"));
        let n = ["minecraft:stone", "mekanism:basic_universal_cable_m3", "minecraft:air", "mekanism:crusher", "minecraft:air", CABLE];
        assert_eq!(cable_mask_from(&n), 0b10_1010);
        assert_eq!(cable_update(0b10_1010, 0, "mekanism:heat_generator"), 0b10_1011);
        assert_eq!(cable_update(0b10_1010, 3, "minecraft:air"), 0b10_0010);
    }

    #[test]
    fn energy_flows_through_every_cable_variant() {
        use std::collections::HashMap;
        let mut world: HashMap<[i32; 3], String> = HashMap::new();
        world.insert([0, 0, 0], "mekanism:heat_generator".into());
        world.insert([1, 0, 0], cable_name(0b11_0000));
        world.insert([2, 0, 0], cable_name(0b11_0001));
        world.insert([2, -1, 0], cable_name(0b00_0010));
        world.insert([2, -2, 0], "mekanism:crusher".into());
        let names = world.clone();
        let mut block = |p: [i32; 3]| world.get(&p).cloned().unwrap_or_else(|| "minecraft:air".into());
        let mut accepts = |p: [i32; 3], _: [i32; 3]| names.get(&p).and_then(|b| Kind::of(b)).is_some_and(|k| k.accepts());
        assert_eq!(acceptors([0, 0, 0], &DIRS, &mut block, &mut accepts), vec![[2, -2, 0]]);
    }

    #[test]
    fn energy_text() {
        assert_eq!(joules(12_345), "12.345 J");
        assert_eq!(joules(160_000), "160,0 kJ");
        assert_eq!(joules(4_000_000), "4.000,0 kJ");
        assert_eq!(joules(12_500_000), "12,5 MJ");
    }

    #[test]
    fn data_slots() {
        let s = State { energy: 10_000, progress: 100, burn_remaining: 400, burn_total: 1600, ..State::default() };
        assert_eq!(s.data(Kind::EnrichmentChamber, 0), [10_000, 500, 0, 0]);
        assert_eq!(s.data(Kind::HeatGenerator, 0)[2], 250);
        assert_eq!(State { energy: 4_000_000, ..s }.data(Kind::EnergyCube, 0)[0], 4_000_000 / 134);
        let up = State { speed_upgrades: 3, energy_upgrades: 8, ..s };
        assert_eq!(unpack_upgrades(Kind::Crusher, &up.data(Kind::Crusher, 0)), (3, 8));
        let inf = State { infuse_type: 2, ..up };
        let d = inf.data(Kind::MetallurgicInfuser, 0);
        assert_eq!((d[3] & 15, unpack_upgrades(Kind::MetallurgicInfuser, &d)), (2, (3, 8)));
    }

    // ---- 0.6.0 ----

    #[test]
    fn tiers_follow_mekanism() {
        assert_eq!(Kind::of("mekanism:ultimate_energy_cube"), Some(Kind::UltimateEnergyCube));
        let cubes = [Kind::EnergyCube, Kind::AdvancedEnergyCube, Kind::EliteEnergyCube, Kind::UltimateEnergyCube];
        let caps: Vec<i64> = cubes.iter().map(|k| k.capacity()).collect();
        assert_eq!(caps, vec![4_000_000, 16_000_000, 64_000_000, 256_000_000]);
        let outs: Vec<i64> = cubes.iter().map(|k| k.output()).collect();
        assert_eq!(outs, vec![4_000, 16_000, 64_000, 256_000]);
        for k in cubes {
            assert!(k.is_cube() && k.is_source() && k.accepts() && !k.is_machine());
            assert!(connects(k.block()));
        }
        assert!(!Kind::OsmiumCompressor.is_cube());
        assert!(Kind::OsmiumCompressor.is_machine() && Kind::OsmiumCompressor.accepts());
        assert_eq!((Kind::OsmiumCompressor.capacity(), Kind::OsmiumCompressor.usage()), (80_000, 100));
        assert_eq!(capacity(Kind::OsmiumCompressor, 8), 800_000);
        assert_eq!(cable_rate(CABLE), Some(8_000));
        assert_eq!(cable_rate("mekanism:basic_universal_cable_m17"), Some(8_000));
        assert_eq!(cable_rate("mekanism:advanced_universal_cable"), Some(128_000));
        assert_eq!(cable_rate("mekanism:elite_universal_cable"), Some(1_024_000));
        assert_eq!(cable_rate("mekanism:ultimate_universal_cable"), Some(8_192_000));
        assert_eq!(cable_rate("mekanism:crusher"), None);
        assert!(is_cable("mekanism:elite_universal_cable") && connects("mekanism:elite_universal_cable"));
        // the new tiers are pipe blocks only: no 0.4.0 shape variants
        assert_eq!(cable_mask("mekanism:advanced_universal_cable"), None);
        // ids of 0.5.0 unchanged
        assert_eq!(Kind::EnergyCube.block(), "mekanism:basic_energy_cube");
        assert_eq!(Kind::ALL.len(), 11);
        for k in Kind::ALL {
            assert_eq!(Kind::of(k.block()), Some(k));
        }
    }

    #[test]
    fn infuse_codes_stay_and_grow_at_the_end() {
        assert_eq!(infuse_code("carbon"), 1);
        assert_eq!(infuse_code("redstone"), 2);
        assert_eq!(infuse_code("diamond"), 3);
        assert_eq!(infuse_code("refined_obsidian"), 4);
        assert_eq!(infuse_code("osmium"), 0, "osmium is the compressor's, not the infuser's");
        assert_eq!(infuse_label(3), "Diamante");
    }

    #[test]
    fn reinforced_alloy_takes_20_diamond() {
        let d = infuse_code("diamond");
        let mut s = State { energy: 20_000, ..State::default() };
        let mut slots = vec![Some(("alloy_infused", 1)), Some(("dust_diamond", 2)), None];
        // one dust a tick into the buffer: 20 diamond after the second, then 200 ticks
        for _ in 0..201 {
            infuser_tick(&mut s, &mut slots, Some((1, ("alloy_reinforced", 1), d, 20)), Some((d, 10)), &T);
        }
        assert_eq!(slots[OUTPUT], Some(("alloy_reinforced", 1)));
        assert_eq!(slots[FUEL], None);
        assert_eq!((s.infuse_type, s.infuse_amount), (0, 0));
    }

    #[test]
    fn compressor_makes_refined_obsidian_with_one_ingot_of_osmium() {
        let mut s = State { energy: 80_000, ..State::default() };
        let mut slots = vec![Some(("dust_refined_obsidian", 2)), Some(("ingot_osmium", 2)), None];
        let r = || Some((1, ("ingot_refined_obsidian", 1), 1));
        assert!(compressor_tick(&mut s, &mut slots, r(), Some(200), &T));
        assert_eq!(slots[FUEL], Some(("ingot_osmium", 1)));
        assert_eq!(s.chemical, 199);
        for _ in 0..199 {
            assert!(compressor_tick(&mut s, &mut slots, r(), Some(200), &T));
        }
        assert_eq!(slots[OUTPUT], Some(("ingot_refined_obsidian", 1)));
        assert_eq!(slots[INPUT], Some(("dust_refined_obsidian", 1)));
        assert_eq!(s.energy, 80_000 - 200 * 100);
        // 200 mB used for the one ingot; the second went in once the 210 mB tank had room for it
        assert_eq!((s.chemical, s.chemical_used, s.progress), (200, 0, 0));
        assert_eq!(slots[FUEL], None);
    }

    #[test]
    fn compressor_waits_for_osmium_and_keeps_progress() {
        let mut s = State { energy: 80_000, chemical: 3, ..State::default() };
        let mut slots = vec![Some(("glowstone_dust", 1)), None, None];
        let r = || Some((1, ("ingot_refined_glowstone", 1), 1));
        for _ in 0..3 {
            assert!(compressor_tick(&mut s, &mut slots, r(), None, &T));
        }
        assert!(!compressor_tick(&mut s, &mut slots, r(), None, &T));
        assert_eq!((s.progress, s.chemical, s.chemical_used), (3, 0, 3));
        assert_eq!(s.energy, 80_000 - 300);
        // more osmium: it goes on where it was
        s.chemical = 200;
        assert!(compressor_tick(&mut s, &mut slots, r(), None, &T));
        assert_eq!(s.progress, 4);
        // no recipe: progress and the osmium count of the operation restart
        assert!(!compressor_tick(&mut s, &mut slots, None, None, &T));
        assert_eq!((s.progress, s.chemical_used), (0, 0));
    }

    #[test]
    fn compressor_with_speed_upgrades_uses_the_same_osmium() {
        let mut s = State { energy: 800_000, speed_upgrades: 8, chemical: 200, ..State::default() };
        let mut slots = vec![Some(("glowstone_dust", 1)), None, None];
        for _ in 0..20 {
            assert!(compressor_tick(&mut s, &mut slots, Some((1, ("ingot_refined_glowstone", 1), 1)), None, &T));
        }
        assert_eq!(slots[OUTPUT], Some(("ingot_refined_glowstone", 1)));
        assert_eq!(s.chemical, 0, "200 mB in 20 ticks, 10 a tick");
        assert_eq!(s.energy, 800_000 - 20 * machine_usage(Kind::OsmiumCompressor, 8, 0));
        assert_eq!(compressor_use(&State::default(), 1), 1);
        assert_eq!(compressor_use(&State { progress: 199, chemical_used: 190, ..State::default() }, 1), 10);
    }

    #[test]
    fn compressor_data_slots() {
        let s = State { energy: 40_000, progress: 50, chemical: 150, speed_upgrades: 2, energy_upgrades: 5, ..State::default() };
        let d = s.data(Kind::OsmiumCompressor, 0);
        assert_eq!(d[1..3], [(50 * 1000 / machine_ticks(2)), 150]);
        assert_eq!(unpack_upgrades(Kind::OsmiumCompressor, &d), (2, 5));
        assert_eq!(i64::from(d[0]) * energy_unit(Kind::OsmiumCompressor, 5), 40_000 / energy_unit(Kind::OsmiumCompressor, 5) * energy_unit(Kind::OsmiumCompressor, 5));
        let mut u = State::default();
        assert!(u.upgrade(Kind::OsmiumCompressor, SPEED_UPGRADE));
        assert!(!u.upgrade(Kind::UltimateEnergyCube, SPEED_UPGRADE), "cubes take no upgrades");
    }

    #[test]
    fn the_slowest_cable_on_the_way_limits_it() {
        use std::collections::HashMap;
        // ultimate cube at 0,0,0 giving east; ultimate cable to x=2, a basic one at x=3, a cube at x=4;
        // a crusher beside the ultimate line at 1,1,0; a machine right beside the giver at 0,1,0.
        let mut world: HashMap<[i32; 3], &str> = HashMap::new();
        world.insert([0, 0, 0], "mekanism:ultimate_energy_cube");
        world.insert([1, 0, 0], "mekanism:ultimate_universal_cable");
        world.insert([2, 0, 0], "mekanism:ultimate_universal_cable");
        world.insert([3, 0, 0], CABLE);
        world.insert([4, 0, 0], "mekanism:elite_energy_cube");
        world.insert([1, 1, 0], "mekanism:crusher");
        world.insert([0, 1, 0], "mekanism:osmium_compressor");
        let names = world.clone();
        let mut block = |p: [i32; 3]| world.get(&p).copied().unwrap_or("minecraft:air").to_owned();
        let mut accepts = |p: [i32; 3], _: [i32; 3]| names.get(&p).and_then(|b| Kind::of(b)).is_some_and(|k| k.accepts());
        let mut got = acceptors_rated([0, 0, 0], &DIRS, &mut block, &mut accepts);
        got.sort();
        assert_eq!(got, vec![([0, 1, 0], DIRECT), ([1, 1, 0], 8_192_000), ([4, 0, 0], 8_000)]);
        // sharing 256 kJ: the far cube gets 8 kJ, the rest goes to the other two
        let shares = share_out(256_000, &[1_000_000, 1_000_000, 64_000], &[DIRECT, 8_192_000, 8_000]);
        assert_eq!(shares, vec![124_000, 124_000, 8_000]);
        // only a basic cable to reach: 8 kJ/t at most, whatever the giver has
        assert_eq!(share_out(256_000, &[1_000_000], &[8_000]), vec![8_000]);
        assert_eq!(share_out(100, &[50, 50], &[8_000, 8_000]).iter().sum::<i64>(), 100);
        assert_eq!(share_out(100, &[], &[]), Vec::<i64>::new());
    }

    #[test]
    fn old_networks_carry_what_they_did() {
        // 0.5.0: a heat generator (400 J/t) over basic cable to two machines
        let shares = share_out(400, &[20_000, 20_000], &[CABLE_RATE, CABLE_RATE]);
        assert_eq!(shares, split(400.min(CABLE_RATE), &[20_000, 20_000]));
    }
}

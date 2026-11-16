// Powah for Horizon 0.1.0: the Energizing Orb and Rods, the Furnator and
// Magmator, the Energy Cell and the Energy Cable (starter and basic tiers),
// generic over the game's item stacks so it builds and tests on its own.
// This file is both the mod's Cargo library (its tests) and the module the
// bundle adds to the game (`crate::game::powah_native`); `glue.rs` adapts it
// to Horizon.
//
// Port of Powah (owmii, Technici4n, shartte; LGPLv3). Numbers are Powah's
// default config (`DefaultEnergies`, `PowahConfig`), in FE.

pub const ID: &str = "powah_native";
pub const NAME: &str = "Powah";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Powah: Orbe Energizante e Hastes (aço energizado, cristais), Furnator, Magmator, Célula de Energia e Cabo de Energia (inicial e básico). Precisa do mod de conteúdo powah.";

/// Powah's tiers this port has (`Tier`), in its order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Starter,
    Basic,
}

impl Tier {
    pub fn name(self) -> &'static str {
        match self {
            Tier::Starter => "starter",
            Tier::Basic => "basic",
        }
    }
    /// `DefaultEnergies.baseScaling()`: starter 1, basic 4.
    fn scale(self) -> i64 {
        match self {
            Tier::Starter => 1,
            Tier::Basic => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Orb,
    Rod(Tier),
    Furnator(Tier),
    Magmator(Tier),
    Cell(Tier),
}

pub const ORB: &str = "powah:energizing_orb";

impl Kind {
    pub const ALL: [Kind; 9] = [
        Kind::Orb,
        Kind::Rod(Tier::Starter),
        Kind::Rod(Tier::Basic),
        Kind::Furnator(Tier::Starter),
        Kind::Furnator(Tier::Basic),
        Kind::Magmator(Tier::Starter),
        Kind::Magmator(Tier::Basic),
        Kind::Cell(Tier::Starter),
        Kind::Cell(Tier::Basic),
    ];
    pub fn block(self) -> &'static str {
        match self {
            Kind::Orb => ORB,
            Kind::Rod(Tier::Starter) => "powah:energizing_rod_starter",
            Kind::Rod(Tier::Basic) => "powah:energizing_rod_basic",
            Kind::Furnator(Tier::Starter) => "powah:furnator_starter",
            Kind::Furnator(Tier::Basic) => "powah:furnator_basic",
            Kind::Magmator(Tier::Starter) => "powah:magmator_starter",
            Kind::Magmator(Tier::Basic) => "powah:magmator_basic",
            Kind::Cell(Tier::Starter) => "powah:energy_cell_starter",
            Kind::Cell(Tier::Basic) => "powah:energy_cell_basic",
        }
    }
    pub fn of(block: &str) -> Option<Kind> {
        if !block.starts_with("powah:") {
            return None;
        }
        Kind::ALL.into_iter().find(|k| k.block() == block)
    }
    /// The block entity type id: the block's, as Horizon names them.
    pub fn entity(self) -> &'static str {
        self.block()
    }
    pub fn is_generator(self) -> bool {
        matches!(self, Kind::Furnator(_) | Kind::Magmator(_))
    }
    /// FE stored. Generators: `activeProduction * 1000`; cells:
    /// `reactorProduction * 4000`; rods: `energizingTransfer * 100`.
    pub fn capacity(self) -> i64 {
        match self {
            Kind::Orb => 0,
            Kind::Rod(t) => 10_000 * t.scale(),
            Kind::Furnator(t) | Kind::Magmator(t) => 20_000 * t.scale(),
            Kind::Cell(t) => 1_000_000 * t.scale(),
        }
    }
    /// FE a block moves per tick: what a generator or cell gives out (to
    /// each neighbour's way), what a cell or rod takes in from one giver,
    /// and what a rod gives the orb.
    pub fn transfer(self) -> i64 {
        match self {
            Kind::Orb => 0,
            Kind::Rod(t) => 100 * t.scale(),
            Kind::Furnator(t) | Kind::Magmator(t) => 80 * t.scale(),
            Kind::Cell(t) => 1_000 * t.scale(),
        }
    }
    /// FE a generator makes per tick while it burns (`activeProduction`).
    pub fn generation(self) -> i64 {
        match self {
            Kind::Furnator(t) | Kind::Magmator(t) => 20 * t.scale(),
            _ => 0,
        }
    }
    /// Gives energy to the blocks around it (all six sides, as Powah's
    /// generators and cells do with the default side config).
    pub fn is_source(self) -> bool {
        matches!(self, Kind::Furnator(_) | Kind::Magmator(_) | Kind::Cell(_))
    }
    /// Takes energy from a giver: cells from any side, rods from their base.
    pub fn accepts(self) -> bool {
        matches!(self, Kind::Rod(_) | Kind::Cell(_))
    }
}

/// Energy cables (`CableConfig`: `reactorProduction * 2`), FE per tick.
pub const CABLES: [(&str, i64); 2] = [("powah:energy_cable_starter", 500), ("powah:energy_cable_basic", 2_000)];

pub fn cable_rate(name: &str) -> Option<i64> {
    CABLES.iter().find(|(n, _)| *n == name).map(|(_, r)| *r)
}

/// How far a network is followed, in cable blocks.
pub const CABLE_REACH: usize = 512;
/// No cable between a giver and a target: only the giver's output limits it.
pub const DIRECT: i64 = i64::MAX;

/// `energizing_range`: rods link to an orb at most this far on each axis.
pub const ENERGIZING_RANGE: i32 = 4;
/// `energizing_energy_ratio`.
pub const ENERGIZING_RATIO: f64 = 1.0;
/// A rod's `coolDown` ticker: it warms up this many ticks before it fills the orb.
pub const ROD_WARMUP: i32 = 20;
/// Orb slots: 0 the result, 1 to 6 the ingredients.
pub const ORB_OUT: usize = 0;
pub const ORB_SLOTS: usize = 7;
/// `energy_per_fuel_tick`: FE a Furnator gets for each tick of a fuel's burn time.
pub const FE_PER_FUEL_TICK: i64 = 30;
/// Magmator: its tank (`bucketAmount * 4`), what it drains at a time, and
/// the FE lava gives per 100 mB (`magmator_fuel`: 10000).
pub const MAGMATOR_TANK: i32 = 4_000;
pub const MAGMATOR_DRAIN: i32 = 100;
pub const LAVA_FE_PER_DRAIN: i64 = 10_000;
pub const BUCKET_MB: i32 = 1_000;

/// Slots of the generators' furnace menu: 0 hidden, 1 fuel (a Furnator's
/// fuel, a Magmator's lava buckets), 2 the Magmator's empty buckets.
pub const FUEL: usize = 1;
pub const OUTPUT: usize = 2;

/// The energy cubes of the user's Mekanism for Horizon, with the joules
/// each gives out of its front per tick: a Powah cell or rod that front
/// faces draws from it. Mekanism counts 2.5 J to 1 FE.
pub const MEKANISM_CUBES: [(&str, i64); 4] = [
    ("mekanism:basic_energy_cube", 4_000),
    ("mekanism:advanced_energy_cube", 16_000),
    ("mekanism:elite_energy_cube", 64_000),
    ("mekanism:ultimate_energy_cube", 256_000),
];

pub fn mekanism_cube_output(name: &str) -> Option<i64> {
    MEKANISM_CUBES.iter().find(|(n, _)| *n == name).map(|(_, j)| *j)
}

/// FE drawn from a Mekanism cube holding `joules` and giving `output` J/t,
/// for a block with `room` FE free: (FE gained, J taken). Whole FE only,
/// 2 FE for every 5 J.
pub fn mekanism_draw(joules: i64, output: i64, room: i64) -> (i64, i64) {
    let fe = (joules.min(output).max(0) * 2 / 5).min(room.max(0));
    (fe, fe * 5 / 2 + (fe * 5) % 2)
}

/// The saved numbers of every Powah block entity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub energy: i64,
    /// Furnator: FE left of the fuel burning, and what it started with.
    pub carbon: i64,
    pub carbon_total: i64,
    /// Magmator: lava in its tank (mB); FE left of the lava drained, and
    /// what it started with.
    pub tank: i32,
    pub buffer: i64,
    pub buffer_total: i64,
    /// Rod: its warm-up, and the orb it fills (`linked`).
    pub cooldown: i32,
    pub linked: bool,
    pub orb: [i32; 3],
    /// Orb: the energy put in so far, what the recipe needs, which recipe
    /// (a hash of its id; 0 none), and whether it takes energy now.
    pub stored: i64,
    pub required: i64,
    pub recipe: i64,
    pub active: bool,
}

impl State {
    pub fn add(&mut self, kind: Kind, fe: i64) -> i64 {
        let room = (kind.capacity() - self.energy).max(0);
        let took = fe.clamp(0, room);
        self.energy += took;
        took
    }
    /// What it may take in from one giver this tick.
    pub fn room(&self, kind: Kind) -> i64 {
        if !kind.accepts() {
            return 0;
        }
        (kind.capacity() - self.energy).clamp(0, kind.transfer())
    }
    /// The furnace menu's four data slots: energy (in `energy_unit`s), then
    /// per kind: Furnator fuel left (thousandths) and FE/t made; Magmator
    /// tank (mB), lava left (thousandths) and FE/t made.
    pub fn data(&self, kind: Kind, generating: i64) -> [i32; 4] {
        let energy = (self.energy / energy_unit(kind)).clamp(0, 32_767) as i32;
        let made = generating.clamp(0, 32_767) as i32;
        match kind {
            Kind::Furnator(_) => [energy, 0, permille(self.carbon, self.carbon_total), made],
            Kind::Magmator(_) => [
                energy,
                self.tank.clamp(0, MAGMATOR_TANK),
                permille(self.buffer, self.buffer_total),
                made,
            ],
            _ => [energy, 0, 0, 0],
        }
    }
    /// The orb menu's five data slots: progress in thousandths, then the
    /// energy needed and stored, each in two 15-bit halves.
    pub fn orb_data(&self) -> [i32; 5] {
        let lo = |v: i64| (v.clamp(0, (1 << 30) - 1) & 0x7FFF) as i32;
        let hi = |v: i64| ((v.clamp(0, (1 << 30) - 1) >> 15) & 0x7FFF) as i32;
        [
            permille(self.stored, self.required),
            lo(self.required),
            hi(self.required),
            lo(self.stored),
            hi(self.stored),
        ]
    }
}

pub fn permille(a: i64, b: i64) -> i32 {
    if b > 0 {
        (a.max(0).saturating_mul(1000) / b).clamp(0, 1000) as i32
    } else {
        0
    }
}

/// Joins the orb's two 15-bit halves.
pub fn join15(lo: i32, hi: i32) -> i64 {
    i64::from(lo.clamp(0, 0x7FFF)) | (i64::from(hi.clamp(0, 0x7FFF)) << 15)
}

/// Units the menu's 16-bit data slots count energy in.
pub fn energy_unit(kind: Kind) -> i64 {
    ((kind.capacity() + 29_999) / 30_000).max(1)
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

/// One tick of a Furnator (`FurnatorTile.postTick`): with no fuel burning
/// it takes one item of fuel (its burn time × 30 FE), then makes up to its
/// generation while it has room. `burn_time` gives a fuel's ticks (0: not
/// a fuel), `remainder` what a used fuel leaves (a lava bucket's bucket).
/// Returns the FE made.
pub fn furnator_tick<S: Clone, K: Stacks<S>>(
    kind: Kind,
    state: &mut State,
    fuel: &mut Option<S>,
    burn_time: impl Fn(&S) -> i32,
    remainder: impl Fn(&S) -> Option<S>,
    stacks: &K,
) -> i64 {
    if state.carbon <= 0 {
        state.carbon = 0;
        if let Some(f) = fuel.take() {
            let ticks = burn_time(&f);
            if ticks > 0 {
                state.carbon = i64::from(ticks) * FE_PER_FUEL_TICK;
                state.carbon_total = state.carbon;
                let rest = remainder(&f);
                *fuel = stacks.take(f, 1).or(rest);
            } else {
                *fuel = Some(f);
            }
        }
    }
    if state.carbon <= 0 || state.energy >= kind.capacity() {
        return 0;
    }
    let made = (kind.capacity() - state.energy)
        .min(kind.generation())
        .min(state.carbon);
    state.energy += made;
    state.carbon -= made;
    made
}

/// One tick of a Magmator (`MagmatorTile.postTick`). A lava bucket in its
/// fuel slot goes into the tank when 1000 mB fit and its empty bucket fits
/// the output slot (`is_lava`, `bucket` tell and make them). With nothing
/// left of the last drain it drains up to 100 mB (10 000 FE each), and it
/// makes its generation when all of it fits. Returns the FE made.
pub fn magmator_tick<S: Clone, K: Stacks<S>>(
    kind: Kind,
    state: &mut State,
    slots: &mut [Option<S>],
    is_lava: impl Fn(&S) -> bool,
    bucket: S,
    stacks: &K,
) -> i64 {
    state.tank = state.tank.clamp(0, MAGMATOR_TANK);
    let lava_in = slots[FUEL].as_ref().is_some_and(&is_lava);
    let room_out = slots[OUTPUT].as_ref().is_none_or(|o| stacks.fits(o, &bucket));
    if lava_in && room_out && state.tank + BUCKET_MB <= MAGMATOR_TANK {
        if let Some(f) = slots[FUEL].take() {
            slots[FUEL] = stacks.take(f, 1);
            state.tank += BUCKET_MB;
            match &mut slots[OUTPUT] {
                Some(o) => stacks.grow(o, &bucket),
                None => slots[OUTPUT] = Some(bucket),
            }
        }
    }
    if state.buffer <= 0 && state.tank > 0 {
        let drained = state.tank.min(MAGMATOR_DRAIN);
        state.tank -= drained;
        state.buffer = i64::from(drained) * LAVA_FE_PER_DRAIN / i64::from(MAGMATOR_DRAIN);
        state.buffer_total = state.buffer;
    }
    let made = kind.generation().min(state.buffer);
    if made > 0 && kind.capacity() - state.energy >= made {
        state.energy += made;
        state.buffer -= made;
        made
    } else {
        0
    }
}

/// A lava bucket used on a Magmator: whether its tank takes the 1000 mB.
pub fn magmator_fill(state: &mut State) -> bool {
    if state.tank.clamp(0, MAGMATOR_TANK) + BUCKET_MB <= MAGMATOR_TANK {
        state.tank = state.tank.clamp(0, MAGMATOR_TANK) + BUCKET_MB;
        true
    } else {
        false
    }
}

/// `EnergizingRecipe.matches`: every non-empty ingredient slot takes the
/// first ingredient not yet used that it fits, and every ingredient is
/// used. `filled[i]` says whether slot `i` holds something; `fits(i, j)`
/// whether slot `i`'s item fits ingredient `j` of `count`.
pub fn energizing_matches(filled: &[bool], count: usize, mut fits: impl FnMut(usize, usize) -> bool) -> bool {
    if count == 0 {
        return false;
    }
    let mut used = vec![false; count];
    for (i, _) in filled.iter().enumerate().filter(|(_, f)| **f) {
        match (0..count).find(|&j| !used[j] && fits(i, j)) {
            Some(j) => used[j] = true,
            None => return false,
        }
    }
    used.iter().all(|u| *u)
}

/// `getScaledEnergy`: what a recipe needs, never under 1 FE.
pub fn scaled_energy(energy: i64) -> i64 {
    ((energy as f64 * ENERGIZING_RATIO) as i64).max(1)
}

/// A recipe id as the number the orb keeps (FNV-1a), never 0.
pub fn recipe_hash(id: &str) -> i64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in id.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    ((h >> 1) as i64).max(1)
}

/// One tick of the orb, with the recipe its ingredients match (hash, FE,
/// result). A different recipe (or none) starts over from 0 FE, as
/// Powah's `onSlotChanged` does. When the rods have filled it, one of each
/// ingredient is used and the result goes to slot 0. It takes energy
/// (`active`) only while there is a recipe and room for its result.
/// Returns whether it made something.
pub fn orb_tick<S: Clone, K: Stacks<S>>(
    state: &mut State,
    slots: &mut [Option<S>],
    recipe: Option<(i64, i64, S)>,
    stacks: &K,
) -> bool {
    let Some((hash, energy, result)) = recipe else {
        *state = State {
            energy: state.energy,
            ..State::default()
        };
        return false;
    };
    if state.recipe != hash {
        state.recipe = hash;
        state.stored = 0;
    }
    state.required = scaled_energy(energy);
    state.stored = state.stored.clamp(0, state.required);
    let room = slots[ORB_OUT].as_ref().is_none_or(|o| stacks.fits(o, &result));
    state.active = room && state.stored < state.required;
    if !room || state.stored < state.required {
        return false;
    }
    for slot in slots.iter_mut().take(ORB_SLOTS).skip(1) {
        if let Some(s) = slot.take() {
            *slot = stacks.take(s, 1);
        }
    }
    match &mut slots[ORB_OUT] {
        Some(o) => stacks.grow(o, &result),
        None => slots[ORB_OUT] = Some(result),
    }
    state.stored = 0;
    state.active = false;
    true
}

/// Energy a rod puts into an orb that needs `room` more.
pub fn orb_fill(state: &mut State, amount: i64) -> i64 {
    if !state.active {
        return 0;
    }
    let took = amount.clamp(0, (state.required - state.stored).max(0));
    state.stored += took;
    took
}

/// One tick of a rod's warm-up (`EnergizingRodTile.postTick`'s
/// `coolDown`): it warms while its orb takes energy and the rod has some,
/// cools otherwise. Returns what it may give the orb this tick.
pub fn rod_ready(kind: Kind, state: &mut State, orb_active: bool) -> i64 {
    if orb_active && state.energy > 0 {
        state.cooldown = (state.cooldown + 1).min(ROD_WARMUP);
    } else if state.cooldown > 0 {
        state.cooldown -= 1;
    }
    if state.cooldown >= ROD_WARMUP && orb_active {
        state.energy.min(kind.transfer())
    } else {
        0
    }
}

/// The orb nearest `pos` within the energizing range (ties: the first in
/// x, y, z order), `is_orb` saying where one is.
pub fn nearest_orb(pos: [i32; 3], mut is_orb: impl FnMut([i32; 3]) -> bool) -> Option<[i32; 3]> {
    let r = ENERGIZING_RANGE;
    let mut best: Option<([i32; 3], i32)> = None;
    for dx in -r..=r {
        for dy in -r..=r {
            for dz in -r..=r {
                if dx == 0 && dy == 0 && dz == 0 {
                    continue;
                }
                let d = dx * dx + dy * dy + dz * dz;
                if best.is_some_and(|(_, b)| b <= d) {
                    continue;
                }
                let p = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
                if is_orb(p) {
                    best = Some((p, d));
                }
            }
        }
    }
    best.map(|(p, _)| p)
}

pub fn in_range(a: [i32; 3], b: [i32; 3]) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() <= ENERGIZING_RANGE)
}

/// The six directions as offsets: down, up, north, south, west, east.
pub const DIRS: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];

pub fn offset(p: [i32; 3], d: [i32; 3]) -> [i32; 3] {
    [p[0] + d[0], p[1] + d[1], p[2] + d[2]]
}

/// The offset of a `facing` (the furnace's four, the end rod's six).
pub fn facing_offset(facing: &str) -> [i32; 3] {
    match facing {
        "up" => [0, 1, 0],
        "down" => [0, -1, 0],
        "south" => [0, 0, 1],
        "west" => [-1, 0, 0],
        "east" => [1, 0, 0],
        _ => [0, 0, -1],
    }
}

/// Where energy from `from` goes, with what reaches each one per tick at
/// most: the acceptors beside it and those along cables reached from it
/// (up to `CABLE_REACH` cables), limited by the slowest cable on the way
/// (`DIRECT` beside the giver). `block` names the block at a position;
/// `accepts` says whether a block at a position takes energy coming in
/// along `came` (the step from the giver to it). The source itself is
/// never one.
pub fn acceptors_rated(
    from: [i32; 3],
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
    for d in DIRS {
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

/// What a giver with `amount` this tick hands each target, given each
/// one's free room and the most its way carries: equal shares as `split`,
/// none past a target's way, and no more in all than the fastest way
/// carries.
pub fn share_out(amount: i64, rooms: &[i64], limits: &[i64]) -> Vec<i64> {
    let total = limits.iter().copied().max().map_or(0, |m| amount.min(m));
    let capped: Vec<i64> = rooms
        .iter()
        .zip(limits.iter().chain(std::iter::repeat(&0)))
        .map(|(r, l)| (*r).min(*l).max(0))
        .collect();
    split(total, &capped)
}

/// "12.345 FE", "1.234,5 kFE", "3,2 MFE" (pt-BR digits).
pub fn fe(v: i64) -> String {
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
    if v.abs() < 100_000 {
        format!("{} FE", group(v))
    } else if v.abs() < 10_000_000 {
        format!("{},{} kFE", group(v / 1000), (v % 1000).abs() / 100)
    } else {
        format!("{},{} MFE", group(v / 1_000_000), (v % 1_000_000).abs() / 100_000)
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
    fn powah_defaults() {
        let st = Tier::Starter;
        let b = Tier::Basic;
        assert_eq!(Kind::of("powah:energizing_orb"), Some(Kind::Orb));
        assert_eq!(Kind::of("powah:furnator_basic"), Some(Kind::Furnator(b)));
        assert_eq!(Kind::of("mekanism:crusher"), None);
        assert_eq!(
            Kind::of("powah:energy_cable_starter"),
            None,
            "cables have no block entity"
        );
        assert_eq!(
            (Kind::Furnator(st).generation(), Kind::Furnator(b).generation()),
            (20, 80)
        );
        assert_eq!(
            (Kind::Furnator(st).capacity(), Kind::Magmator(b).capacity()),
            (20_000, 80_000)
        );
        assert_eq!((Kind::Furnator(st).transfer(), Kind::Magmator(b).transfer()), (80, 320));
        assert_eq!(
            (Kind::Cell(st).capacity(), Kind::Cell(b).capacity()),
            (1_000_000, 4_000_000)
        );
        assert_eq!((Kind::Cell(st).transfer(), Kind::Cell(b).transfer()), (1_000, 4_000));
        assert_eq!((Kind::Rod(st).capacity(), Kind::Rod(b).capacity()), (10_000, 40_000));
        assert_eq!((Kind::Rod(st).transfer(), Kind::Rod(b).transfer()), (100, 400));
        assert_eq!(cable_rate("powah:energy_cable_starter"), Some(500));
        assert_eq!(cable_rate("powah:energy_cable_basic"), Some(2_000));
        for k in Kind::ALL {
            assert_eq!(Kind::of(k.block()), Some(k));
            assert!(k.capacity() / energy_unit(k) <= 32_767, "{k:?}");
        }
    }

    #[test]
    fn furnator_burns_coal_for_48_kfe_at_20_per_tick() {
        let k = Kind::Furnator(Tier::Starter);
        let mut s = State::default();
        let mut fuel = Some(("coal", 2));
        let made = furnator_tick(k, &mut s, &mut fuel, |_| 1600, |_| None, &T);
        assert_eq!(made, 20);
        assert_eq!(fuel, Some(("coal", 1)));
        assert_eq!(s.carbon_total, 48_000);
        for _ in 0..999 {
            furnator_tick(k, &mut s, &mut fuel, |_| 1600, |_| None, &T);
        }
        // 20 kFE is full after 1000 ticks: it stops, keeping the fuel's rest.
        assert_eq!(s.energy, 20_000);
        assert_eq!(furnator_tick(k, &mut s, &mut fuel, |_| 1600, |_| None, &T), 0);
        assert_eq!(s.carbon, 28_000);
        assert_eq!(fuel, Some(("coal", 1)), "the second coal waits");
        assert_eq!(s.data(k, 0)[2], 583);
    }

    #[test]
    fn furnator_leaves_the_bucket_of_lava_and_skips_non_fuel() {
        let k = Kind::Furnator(Tier::Basic);
        let mut s = State::default();
        let mut fuel = Some(("dirt", 5));
        assert_eq!(furnator_tick(k, &mut s, &mut fuel, |_| 0, |_| None, &T), 0);
        assert_eq!(fuel, Some(("dirt", 5)));
        let mut fuel = Some(("lava_bucket", 1));
        let made = furnator_tick(k, &mut s, &mut fuel, |_| 20_000, |_| Some(("bucket", 1)), &T);
        assert_eq!(made, 80);
        assert_eq!(fuel, Some(("bucket", 1)));
    }

    #[test]
    fn magmator_takes_a_bucket_and_makes_100_kfe() {
        let k = Kind::Magmator(Tier::Starter);
        let mut s = State::default();
        let mut slots = vec![None, Some(("lava_bucket", 2)), None];
        let lava = |x: &Stack| x.0 == "lava_bucket";
        assert_eq!(magmator_tick(k, &mut s, &mut slots, lava, ("bucket", 1), &T), 20);
        assert_eq!(slots[FUEL], Some(("lava_bucket", 1)));
        assert_eq!(slots[OUTPUT], Some(("bucket", 1)));
        assert_eq!((s.tank, s.buffer), (900, 9_980), "1000 mB in, 100 mB drained");
        magmator_tick(k, &mut s, &mut slots, lava, ("bucket", 1), &T);
        assert_eq!((s.tank, slots[FUEL], slots[OUTPUT]), (1_900, None, Some(("bucket", 2))));
        let mut total = 40;
        for _ in 0..998 {
            total += magmator_tick(k, &mut s, &mut slots, lava, ("bucket", 1), &T);
        }
        assert_eq!(total, 20_000);
        assert_eq!(s.energy, 20_000, "full: it stops");
        assert_eq!(magmator_tick(k, &mut s, &mut slots, lava, ("bucket", 1), &T), 0);
        // 2 buckets = 200 kFE in all
        let left = i64::from(s.tank) * 100 + s.buffer;
        assert_eq!(left + s.energy, 200_000);
    }

    #[test]
    fn magmator_tank_takes_whole_buckets_only() {
        let mut s = State {
            tank: 3_100,
            ..State::default()
        };
        assert!(!magmator_fill(&mut s));
        s.tank = 3_000;
        assert!(magmator_fill(&mut s));
        assert_eq!(s.tank, 4_000);
        let mut slots = vec![None, Some(("lava_bucket", 1)), Some(("dirt", 1))];
        let mut s = State::default();
        magmator_tick(
            Kind::Magmator(Tier::Basic),
            &mut s,
            &mut slots,
            |x: &Stack| x.0 == "lava_bucket",
            ("bucket", 1),
            &T,
        );
        assert_eq!(s.tank, 0, "no room for the empty bucket: the lava stays");
    }

    #[test]
    fn energizing_matching_is_powahs() {
        // iron + gold, in any slots
        let ings = ["iron", "gold"];
        let slots = [None, Some("gold"), None, Some("iron"), None, None];
        let filled: Vec<bool> = slots.iter().map(Option::is_some).collect();
        assert!(energizing_matches(&filled, 2, |i, j| slots[i] == Some(ings[j])));
        // a stray item breaks it, and so does a missing ingredient
        let slots = [Some("gold"), Some("iron"), Some("dirt"), None, None, None];
        let filled: Vec<bool> = slots.iter().map(Option::is_some).collect();
        assert!(!energizing_matches(&filled, 2, |i, j| slots[i] == Some(ings[j])));
        let slots = [Some("gold"), None, None, None, None, None];
        let filled: Vec<bool> = slots.iter().map(Option::is_some).collect();
        assert!(!energizing_matches(&filled, 2, |i, j| slots[i] == Some(ings[j])));
        // four blaze powder need four slots
        let filled = [true, true, true, false, false, false];
        assert!(!energizing_matches(&filled, 4, |_, _| true));
        let filled = [true, true, true, true, false, false];
        assert!(energizing_matches(&filled, 4, |_, _| true));
        assert!(!energizing_matches(&[false; 6], 0, |_, _| true));
    }

    #[test]
    fn orb_makes_energized_steel_when_filled() {
        let mut s = State::default();
        let mut slots: Vec<Option<Stack>> = vec![None, Some(("iron", 3)), Some(("gold", 1)), None, None, None, None];
        let r = || {
            Some((
                recipe_hash("powah:energizing/energized_steel"),
                10_000,
                ("steel_energized", 2),
            ))
        };
        assert!(!orb_tick(&mut s, &mut slots, r(), &T));
        assert!(s.active);
        assert_eq!(s.required, 10_000);
        assert_eq!(orb_fill(&mut s, 9_950), 9_950);
        assert_eq!(orb_fill(&mut s, 100), 50, "no more than the recipe needs");
        assert!(orb_tick(&mut s, &mut slots, r(), &T));
        assert_eq!(slots[ORB_OUT], Some(("steel_energized", 2)));
        assert_eq!((slots[1], slots[2]), (Some(("iron", 2)), None));
        assert_eq!((s.stored, s.active), (0, false));
        // the gold is gone: no recipe, nothing stored
        assert!(!orb_tick(&mut s, &mut slots, None, &T));
        assert_eq!(s, State::default());
    }

    #[test]
    fn orb_starts_over_on_another_recipe_and_waits_for_room() {
        let mut s = State {
            recipe: recipe_hash("a"),
            required: 100,
            stored: 60,
            active: true,
            ..State::default()
        };
        let mut slots: Vec<Option<Stack>> = vec![Some(("x", 64)), Some(("rod", 1)), None, None, None, None, None];
        assert!(!orb_tick(
            &mut s,
            &mut slots,
            Some((recipe_hash("b"), 120_000, ("crystal", 1))),
            &T
        ));
        assert_eq!(s.stored, 0, "another recipe: from 0");
        assert!(!s.active, "a full output slot takes no energy");
        assert_eq!(orb_fill(&mut s, 100), 0);
        assert_eq!(s.orb_data(), [0, 120_000 & 0x7FFF, 120_000 >> 15, 0, 0]);
        assert_eq!(join15(s.orb_data()[1], s.orb_data()[2]), 120_000);
        assert_eq!(join15(0x7FFF, 0x7FFF), (1 << 30) - 1);
    }

    #[test]
    fn rod_warms_up_for_a_second_then_gives_its_rate() {
        let k = Kind::Rod(Tier::Starter);
        let mut rod = State {
            energy: 10_000,
            ..State::default()
        };
        let mut orb = State {
            active: true,
            required: 10_000,
            ..State::default()
        };
        let mut ticks = 0;
        while orb.stored < orb.required {
            let give = rod_ready(k, &mut rod, orb.active);
            rod.energy -= orb_fill(&mut orb, give);
            ticks += 1;
            assert!(ticks < 1000);
        }
        // 19 ticks warming, then 100 FE a tick for 100 ticks
        assert_eq!(ticks, 119);
        assert_eq!(rod.energy, 0);
        assert_eq!(rod_ready(k, &mut rod, true), 0, "empty");
        assert_eq!(rod.cooldown, 19, "it cools again");
        let mut basic = State {
            energy: 40_000,
            cooldown: 20,
            ..State::default()
        };
        assert_eq!(rod_ready(Kind::Rod(Tier::Basic), &mut basic, true), 400);
        assert_eq!(rod_ready(Kind::Rod(Tier::Basic), &mut basic, false), 0);
    }

    #[test]
    fn rod_finds_the_nearest_orb_in_range() {
        let orbs = [[3, 0, 0], [1, 1, 0], [10, 0, 0]];
        assert_eq!(nearest_orb([0, 0, 0], |p| orbs.contains(&p)), Some([1, 1, 0]));
        assert_eq!(nearest_orb([6, 0, 0], |p| orbs.contains(&p)), Some([3, 0, 0]));
        assert_eq!(nearest_orb([20, 0, 0], |p| orbs.contains(&p)), None);
        assert!(in_range([0, 0, 0], [4, -4, 4]) && !in_range([0, 0, 0], [5, 0, 0]));
    }

    #[test]
    fn cables_carry_to_cells_and_rod_bases() {
        // furnator at 0, cable 1..=3 east, cell at 4, rod above the cable at x=2 (its base: the cable)
        let block = |p: [i32; 3]| -> String {
            match p {
                [1..=3, 0, 0] => "powah:energy_cable_starter".into(),
                [4, 0, 0] => "powah:energy_cell_starter".into(),
                [2, 1, 0] => "powah:energizing_rod_starter".into(),
                [0, 1, 0] => "powah:energizing_rod_starter".into(),
                _ => "minecraft:air".into(),
            }
        };
        let accepts = |p: [i32; 3], came: [i32; 3]| match Kind::of(&block(p)) {
            Some(Kind::Rod(_)) => came == [0, 1, 0], // placed on top: facing up
            Some(k) => k.accepts(),
            None => false,
        };
        let mut b = block;
        let mut a = accepts;
        let mut got = acceptors_rated([0, 0, 0], &mut b, &mut a);
        got.sort();
        assert_eq!(got, vec![([0, 1, 0], DIRECT), ([2, 1, 0], 500), ([4, 0, 0], 500)]);
    }

    #[test]
    fn sharing_respects_rooms_and_ways() {
        assert_eq!(share_out(80, &[100, 100], &[DIRECT, 500]), vec![40, 40]);
        assert_eq!(share_out(80, &[10, 100], &[DIRECT, 500]), vec![10, 70]);
        assert_eq!(share_out(5_000, &[4_000, 4_000], &[500, 500]), vec![250, 250]);
        assert_eq!(share_out(100, &[], &[]), Vec::<i64>::new());
        let s = State {
            energy: 9_950,
            ..State::default()
        };
        assert_eq!(s.room(Kind::Rod(Tier::Starter)), 50);
        assert_eq!(State::default().room(Kind::Furnator(Tier::Starter)), 0);
    }

    #[test]
    fn mekanism_cubes_give_2_fe_for_5_joules() {
        assert_eq!(mekanism_cube_output("mekanism:basic_energy_cube"), Some(4_000));
        assert_eq!(mekanism_cube_output("mekanism:crusher"), None);
        assert_eq!(mekanism_draw(1_000_000, 4_000, 1_000), (1_000, 2_500));
        assert_eq!(mekanism_draw(1_000_000, 4_000, 5_000), (1_600, 4_000));
        assert_eq!(mekanism_draw(7, 4_000, 1_000), (2, 5));
        assert_eq!(mekanism_draw(3, 4_000, 1_000).0, 1);
        assert!(mekanism_draw(3, 4_000, 1_000).1 <= 3);
        assert_eq!(mekanism_draw(0, 4_000, 1_000), (0, 0));
    }

    #[test]
    fn texts() {
        assert_eq!(fe(12_345), "12.345 FE");
        assert_eq!(fe(1_234_567), "1.234,5 kFE");
        assert_eq!(fe(20_000_000), "20,0 MFE");
        assert_eq!(permille(5, 0), 0);
        assert_ne!(recipe_hash("powah:energizing/energized_steel"), 0);
    }
}

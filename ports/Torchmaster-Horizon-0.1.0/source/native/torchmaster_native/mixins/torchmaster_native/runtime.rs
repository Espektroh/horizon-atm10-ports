// Torchmaster for Horizon: the rules, free of Horizon types (only std), so
// they are unit tested on their own. The glue (`glue.rs`) runs them over the
// game. Numbers and rules follow Torchmaster 21.1.12 (MIT, by Xalcon):
// `IDistanceLogic.Cubic`, `VanillaCompat`, `TorchmasterNeoforgeConfig` and
// `FeralFlareLanternBlockEntity`.

pub const ID: &str = "torchmaster_native";
pub const NAME: &str = "Torchmaster (Horizon)";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Mega Tocha (sem monstros num cubo de 64), Lampada do Medo (sem animais) e Lanterna Feroz (luzes invisiveis).";

pub const MEGA_TORCH: &str = "torchmaster:megatorch";
pub const DREAD_LAMP: &str = "torchmaster:dreadlamp";
pub const FERAL_FLARE: &str = "torchmaster:feral_flare_lantern";
pub const INVISIBLE_LIGHT: &str = "torchmaster:invisible_light";

/// The block entities' ids (one per block, named after it).
pub const ENTITY_MEGA_TORCH: &str = "torchmaster:megatorch";
pub const ENTITY_DREAD_LAMP: &str = "torchmaster:dreadlamp";
pub const ENTITY_FERAL_FLARE: &str = "torchmaster:feral_flare_lantern";

/// `megaTorchRadius` and `dreadLampRadius` (default 64): blocks in each
/// direction, a cube with the torch at its centre.
pub const MEGA_TORCH_RADIUS: i32 = 64;
pub const DREAD_LAMP_RADIUS: i32 = 64;
/// `feralFlareRadius` (16), `feralFlareTickRate` (5), `feralFlareMinLightLevel`
/// (10) and `feralFlareLanternLightCountHardcap` (255).
pub const FLARE_RADIUS: i32 = 16;
pub const FLARE_TICK_RATE: i32 = 5;
pub const FLARE_MIN_LIGHT: u8 = 10;
pub const FLARE_HARDCAP: usize = 255;

/// A light that has not ticked for this many ticks of its level is taken
/// out: its block was broken, or its chunk unloaded.
pub const STALE_TICKS: u64 = 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    MegaTorch,
    DreadLamp,
}

impl Kind {
    pub fn of_block(block: &str) -> Option<Kind> {
        match block {
            MEGA_TORCH => Some(Kind::MegaTorch),
            DREAD_LAMP => Some(Kind::DreadLamp),
            _ => None,
        }
    }
    pub fn radius(self) -> i32 {
        match self {
            Kind::MegaTorch => MEGA_TORCH_RADIUS,
            Kind::DreadLamp => DREAD_LAMP_RADIUS,
        }
    }
    /// `VanillaCompat`: the mega torch blocks every entity type whose
    /// category is not friendly (`monster`); the dread lamp every friendly
    /// one but `misc` (animals, bats, squid, fish, axolotls...).
    pub fn blocks_category(self, category: &str) -> bool {
        match self {
            Kind::MegaTorch => category == "monster",
            Kind::DreadLamp => matches!(
                category,
                "creature"
                    | "ambient"
                    | "axolotls"
                    | "underground_water_creature"
                    | "water_creature"
                    | "water_ambient"
            ),
        }
    }
}

/// The block entity a block gets, if it is one of ours.
pub fn entity_for(block: &str) -> Option<&'static str> {
    match block {
        MEGA_TORCH => Some(ENTITY_MEGA_TORCH),
        DREAD_LAMP => Some(ENTITY_DREAD_LAMP),
        FERAL_FLARE => Some(ENTITY_FERAL_FLARE),
        _ => None,
    }
}

/// `IDistanceLogic.Cubic`: `pos` (an entity position) is inside the cube
/// from `torch - range` to `torch + range + 1` on every axis, bounds
/// included. Y counts as X and Z do: the torch protects 64 blocks above and
/// below it, not the whole column.
pub fn in_cube(torch: [i32; 3], range: i32, pos: [f64; 3]) -> bool {
    (0..3).all(|i| {
        let min = f64::from(torch[i]) - f64::from(range);
        let max = f64::from(torch[i]) + f64::from(range) + 1.0;
        min <= pos[i] && max >= pos[i]
    })
}

/// Where a mob spawned at block `pos` stands (`NaturalSpawner` puts it at the
/// block's centre, feet on its floor).
pub fn spawn_position(pos: [i32; 3]) -> [f64; 3] {
    [
        f64::from(pos[0]) + 0.5,
        f64::from(pos[1]),
        f64::from(pos[2]) + 0.5,
    ]
}

#[derive(Clone, Debug, PartialEq)]
pub struct Light {
    pub dimension: String,
    pub pos: [i32; 3],
    pub kind: Kind,
    /// The level's tick count when its block entity last ticked.
    pub seen: u64,
}

/// The mega torches and dread lamps whose block entities are ticking, by
/// level, as Torchmaster's `EntityBlockingManager` keeps them.
#[derive(Debug, Default)]
pub struct Registry {
    pub lights: Vec<Light>,
    /// Each level's tick count (`native_level_tick_pre`).
    pub ticks: Vec<(String, u64)>,
}

impl Registry {
    pub const fn new() -> Self {
        Registry {
            lights: Vec::new(),
            ticks: Vec::new(),
        }
    }
    pub fn now(&self, dimension: &str) -> u64 {
        self.ticks
            .iter()
            .find(|(d, _)| d == dimension)
            .map_or(0, |(_, t)| *t)
    }
    /// One tick of level `dimension`; stale lights of that level go.
    pub fn tick_level(&mut self, dimension: &str) {
        let now = match self.ticks.iter_mut().find(|(d, _)| d == dimension) {
            Some((_, t)) => {
                *t += 1;
                *t
            }
            None => {
                self.ticks.push((dimension.to_owned(), 1));
                1
            }
        };
        self.lights
            .retain(|l| l.dimension != dimension || now.saturating_sub(l.seen) <= STALE_TICKS);
    }
    /// A light's block entity ticked.
    pub fn seen(&mut self, dimension: &str, pos: [i32; 3], kind: Kind) {
        let now = self.now(dimension);
        match self
            .lights
            .iter_mut()
            .find(|l| l.dimension == dimension && l.pos == pos)
        {
            Some(l) => {
                l.kind = kind;
                l.seen = now;
            }
            None => self.lights.push(Light {
                dimension: dimension.to_owned(),
                pos,
                kind,
                seen: now,
            }),
        }
    }
    pub fn remove(&mut self, dimension: &str, pos: [i32; 3]) {
        self.lights
            .retain(|l| !(l.dimension == dimension && l.pos == pos));
    }
    pub fn clear(&mut self) {
        self.lights.clear();
        self.ticks.clear();
    }
    pub fn is_empty(&self) -> bool {
        self.lights.is_empty()
    }
    /// Whether a natural spawn of `category` at block `pos` is blocked.
    pub fn blocks_spawn(&self, dimension: &str, category: &str, pos: [i32; 3]) -> bool {
        let at = spawn_position(pos);
        self.lights.iter().any(|l| {
            l.dimension == dimension
                && l.kind.blocks_category(category)
                && in_cube(l.pos, l.kind.radius(), at)
        })
    }
    /// `blockVillageSieges` (default true): a mega torch stops a zombie
    /// siege whose spawn point is in its cube.
    pub fn blocks_siege(&self, dimension: &str, at: [f64; 3]) -> bool {
        self.lights.iter().any(|l| {
            l.dimension == dimension
                && l.kind == Kind::MegaTorch
                && in_cube(l.pos, l.kind.radius(), at)
        })
    }
}

/// `FeralFlareLanternBlockEntity.encodePosition`: an offset from the lantern
/// in one int, a signed byte per axis.
pub fn encode_offset(origin: [i32; 3], target: [i32; 3]) -> i32 {
    let x = (target[0] - origin[0]) & 0xFF;
    let y = (target[1] - origin[1]) & 0xFF;
    let z = (target[2] - origin[2]) & 0xFF;
    (x << 16) + (y << 8) + z
}

pub fn decode_offset(origin: [i32; 3], encoded: i32) -> [i32; 3] {
    let x = i32::from(((encoded >> 16) & 0xFF) as u8 as i8);
    let y = i32::from(((encoded >> 8) & 0xFF) as u8 as i8);
    let z = i32::from((encoded & 0xFF) as u8 as i8);
    [origin[0] + x, origin[1] + y, origin[2] + z]
}

/// A small deterministic random (xorshift64*), kept in the lantern's saved
/// data; Java's `level.random` is not reachable from a block entity tick.
pub fn next_random(state: &mut u64) -> u64 {
    if *state == 0 {
        *state = 0x9E37_79B9_7F4A_7C15;
    }
    let mut x = *state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *state = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// `level.random.nextInt(bound)` for a positive bound.
pub fn next_int(state: &mut u64, bound: i32) -> i32 {
    if bound <= 0 {
        return 0;
    }
    ((next_random(state) >> 33) % (bound as u64)) as i32
}

/// The seed a lantern starts with, from its position.
pub fn seed_for(pos: [i32; 3]) -> u64 {
    let mut s = (pos[0] as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (pos[1] as i64 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ (pos[2] as i64 as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
    if s == 0 {
        s = 1;
    }
    s
}

/// The lantern's state between ticks (saved in its block entity).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Flare {
    pub lights: Vec<[i32; 3]>,
    pub ticks: i32,
    pub check: usize,
    pub seed: u64,
}

/// What one tick of a lantern asks the world.
pub enum FlareStep {
    /// Nothing to place this tick.
    Wait,
    /// Try `pos` (after the surface clamp): place a light there when it is
    /// air with block light under `FLARE_MIN_LIGHT`.
    Try([i32; 3]),
}

impl Flare {
    /// `tick()`'s timing and the random target in the cube of
    /// `FLARE_RADIUS` around `center` (`radius - nextInt(diameter)` per axis).
    pub fn step(&mut self, center: [i32; 3]) -> FlareStep {
        self.ticks += 1;
        if self.ticks % FLARE_TICK_RATE != 0 {
            return FlareStep::Wait;
        }
        if self.lights.len() > FLARE_HARDCAP {
            return FlareStep::Wait;
        }
        self.ticks = 0;
        let d = FLARE_RADIUS * 2;
        let x = FLARE_RADIUS - next_int(&mut self.seed, d) + center[0];
        let y = FLARE_RADIUS - next_int(&mut self.seed, d) + center[1];
        let z = FLARE_RADIUS - next_int(&mut self.seed, d) + center[2];
        FlareStep::Try([x, y, z])
    }
    /// The child light to check this tick (`checkNextChildLight`), if any.
    pub fn next_check(&mut self) -> Option<usize> {
        if self.lights.is_empty() {
            self.check = 0;
            return None;
        }
        if self.check >= self.lights.len() {
            self.check = 0;
        }
        Some(self.check)
    }
    /// The checked light is still there: move on to the next.
    pub fn checked_ok(&mut self) {
        if !self.lights.is_empty() {
            self.check = (self.check + 1) % self.lights.len();
        }
    }
    /// The checked light is gone: forget it.
    pub fn checked_gone(&mut self, i: usize) {
        if i < self.lights.len() {
            self.lights.remove(i);
        }
    }
}

/// "limit height - upper bounds": a target more than 4 above the column's
/// `WORLD_SURFACE` height goes down to 4 above it; then inside the world.
pub fn clamp_target(
    mut target: [i32; 3],
    surface: Option<i32>,
    min_y: i32,
    max_y: i32,
) -> [i32; 3] {
    if let Some(surface) = surface
        && target[1] > surface + 4
    {
        target[1] = surface + 4;
    }
    target[1] = target[1].clamp(min_y, max_y);
    target
}

/// Whether a block name is air for the lantern (`level.isEmptyBlock`).
pub fn is_air(block: &str) -> bool {
    matches!(
        block,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_bounds_like_java() {
        let t = [0, 64, 0];
        // Inclusive from torch - 64 to torch + 64 + 1.
        assert!(in_cube(t, 64, [-64.0, 64.0, 0.0]));
        assert!(in_cube(t, 64, [65.0, 64.0, 65.0]));
        assert!(!in_cube(t, 64, [-64.01, 64.0, 0.0]));
        assert!(!in_cube(t, 64, [65.01, 64.0, 0.0]));
        // Radius 0 is the torch's own block.
        assert!(in_cube([3, 3, 3], 0, [3.5, 3.0, 3.5]));
        assert!(!in_cube([3, 3, 3], 0, [4.5, 3.0, 3.5]));
    }

    #[test]
    fn height_rule() {
        let t = [10, 70, 10];
        // 64 below and 65 above (the + 1 of the cube's top) count.
        assert!(in_cube(t, 64, spawn_position([10, 6, 10])));
        assert!(!in_cube(t, 64, spawn_position([10, 5, 10])));
        assert!(in_cube(t, 64, spawn_position([10, 135, 10])));
        assert!(!in_cube(t, 64, spawn_position([10, 136, 10])));
    }

    #[test]
    fn spawn_position_is_block_centre() {
        assert_eq!(spawn_position([1, 2, -3]), [1.5, 2.0, -2.5]);
    }

    #[test]
    fn horizontal_edge_with_block_centre() {
        let t = [0, 64, 0];
        // Block x = 64 spawns at 64.5 <= 65: blocked; x = 65 at 65.5: not.
        assert!(in_cube(t, 64, spawn_position([64, 64, 0])));
        assert!(!in_cube(t, 64, spawn_position([65, 64, 0])));
        // Below: x = -64 spawns at -63.5: blocked; x = -65 at -64.5: not.
        assert!(in_cube(t, 64, spawn_position([-64, 64, 0])));
        assert!(!in_cube(t, 64, spawn_position([-65, 64, 0])));
    }

    #[test]
    fn categories() {
        assert!(Kind::MegaTorch.blocks_category("monster"));
        assert!(!Kind::MegaTorch.blocks_category("creature"));
        assert!(!Kind::MegaTorch.blocks_category("misc"));
        assert!(Kind::DreadLamp.blocks_category("creature"));
        assert!(Kind::DreadLamp.blocks_category("water_ambient"));
        assert!(Kind::DreadLamp.blocks_category("ambient"));
        assert!(!Kind::DreadLamp.blocks_category("monster"));
        assert!(!Kind::DreadLamp.blocks_category("misc"));
    }

    #[test]
    fn registry_blocks_by_dimension_and_kind() {
        let mut r = Registry::new();
        r.tick_level("minecraft:overworld");
        r.seen("minecraft:overworld", [0, 64, 0], Kind::MegaTorch);
        assert!(r.blocks_spawn("minecraft:overworld", "monster", [10, 60, -10]));
        assert!(!r.blocks_spawn("minecraft:overworld", "creature", [10, 60, -10]));
        assert!(!r.blocks_spawn("minecraft:the_nether", "monster", [10, 60, -10]));
        assert!(!r.blocks_spawn("minecraft:overworld", "monster", [100, 60, 0]));
        assert!(r.blocks_siege("minecraft:overworld", [5.0, 64.0, 5.0]));
        r.seen("minecraft:overworld", [500, 64, 0], Kind::DreadLamp);
        assert!(r.blocks_spawn("minecraft:overworld", "creature", [510, 70, 0]));
        assert!(!r.blocks_siege("minecraft:overworld", [510.0, 70.0, 0.0]));
    }

    #[test]
    fn registry_forgets_stale_lights() {
        let mut r = Registry::new();
        r.tick_level("minecraft:overworld");
        r.seen("minecraft:overworld", [0, 64, 0], Kind::MegaTorch);
        for _ in 0..STALE_TICKS {
            r.tick_level("minecraft:overworld");
        }
        assert!(!r.is_empty());
        // Another level ticking does not age it.
        for _ in 0..1000 {
            r.tick_level("minecraft:the_nether");
        }
        assert!(!r.is_empty());
        r.tick_level("minecraft:overworld");
        assert!(r.is_empty());
    }

    #[test]
    fn registry_refresh_and_remove() {
        let mut r = Registry::new();
        r.seen("minecraft:overworld", [0, 64, 0], Kind::MegaTorch);
        r.seen("minecraft:overworld", [0, 64, 0], Kind::MegaTorch);
        assert_eq!(r.lights.len(), 1);
        r.remove("minecraft:overworld", [0, 64, 0]);
        assert!(r.is_empty());
    }

    #[test]
    fn offsets_round_trip() {
        let o = [100, 64, -200];
        for t in [
            [100, 64, -200],
            [116, 48, -216],
            [84, 80, -184],
            [99, 65, -201],
        ] {
            assert_eq!(decode_offset(o, encode_offset(o, t)), t);
        }
    }

    #[test]
    fn flare_steps_every_fifth_tick_inside_cube() {
        let mut f = Flare {
            seed: seed_for([0, 64, 0]),
            ..Default::default()
        };
        let mut tries = 0;
        for i in 1..=100 {
            match f.step([0, 64, 0]) {
                FlareStep::Wait => assert!(i % FLARE_TICK_RATE != 0),
                FlareStep::Try(p) => {
                    tries += 1;
                    for (a, c) in p.iter().zip([0, 64, 0]) {
                        assert!(*a > c - FLARE_RADIUS && *a <= c + FLARE_RADIUS);
                    }
                }
            }
        }
        assert_eq!(tries, 20);
    }

    #[test]
    fn flare_stops_past_hardcap() {
        let mut f = Flare {
            lights: vec![[0, 0, 0]; FLARE_HARDCAP + 1],
            ..Default::default()
        };
        for _ in 0..50 {
            assert!(matches!(f.step([0, 0, 0]), FlareStep::Wait));
        }
    }

    #[test]
    fn flare_checks_cycle() {
        let mut f = Flare {
            lights: vec![[1, 1, 1], [2, 2, 2]],
            ..Default::default()
        };
        assert_eq!(f.next_check(), Some(0));
        f.checked_ok();
        assert_eq!(f.next_check(), Some(1));
        f.checked_gone(1);
        assert_eq!(f.next_check(), Some(0));
        f.checked_gone(0);
        assert_eq!(f.next_check(), None);
    }

    #[test]
    fn clamp_to_surface_and_world() {
        assert_eq!(clamp_target([0, 120, 0], Some(70), -64, 319), [0, 74, 0]);
        assert_eq!(clamp_target([0, 72, 0], Some(70), -64, 319), [0, 72, 0]);
        assert_eq!(clamp_target([0, -80, 0], Some(70), -64, 319), [0, -64, 0]);
        assert_eq!(clamp_target([0, 400, 0], None, -64, 319), [0, 319, 0]);
    }

    #[test]
    fn entities_and_air() {
        assert_eq!(entity_for(MEGA_TORCH), Some(ENTITY_MEGA_TORCH));
        assert_eq!(entity_for("minecraft:torch"), None);
        assert!(is_air("minecraft:cave_air"));
        assert!(!is_air(INVISIBLE_LIGHT));
    }
}

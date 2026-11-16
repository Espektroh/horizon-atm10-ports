// Iron Jetpacks for Horizon: the rules, std only (no Horizon type), so they
// are tested on their own (`cargo test` in source/native). Added to the game
// as `crate::game::ironjetpacks_rt`; `ironjetpacks_glue` applies them.
//
// The numbers are All the Mods 10's (its config/ironjetpacks/jetpacks/*.json,
// which overrides the mod's defaults); the flight is JetpackItem.inventoryTick
// of Iron Jetpacks 8.0.x (BlakeBr0, MIT), as the client applies it.

pub const ID: &str = "ironjetpacks_native";
pub const NAME: &str = "Iron Jetpacks (Horizon)";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Jetpacks do Iron Jetpacks (numeros do ATM10). Precisa do mod de conteudo ironjetpacks.";

/// Client to server: the keys the flight reads (bit 0 jump, bit 1 sneak).
pub const NET_INPUT: &str = "ironjetpacks_native:input";
/// Client to server: a key that changes the worn jetpack (`Toggle`).
pub const NET_TOGGLE: &str = "ironjetpacks_native:toggle";

/// The mod's keys: name, label (Options > Mods) and default.
pub const KEYS: [(&str, &str, &str); 4] = [
    ("engine", "Ligar/desligar motor", "key.keyboard.v"),
    ("hover", "Ligar/desligar pairar", "key.keyboard.h"),
    ("throttle_up", "Aumentar potencia", "key.keyboard.period"),
    ("throttle_down", "Diminuir potencia", "key.keyboard.comma"),
];

/// Where a jetpack keeps its state, in `minecraft:custom_data`. The energy
/// is Horizon's own item energy int (`native_item_energy.rs`, API 307).
pub const ENERGY_KEY: &str = "horizon_energy";
pub const ENGINE_KEY: &str = "ironjetpacks_engine";
pub const HOVER_KEY: &str = "ironjetpacks_hover";
pub const THROTTLE_KEY: &str = "ironjetpacks_throttle";

/// One jetpack tier (`Jetpack` and its JSON), ATM10's values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tier {
    /// `iron`: items `ironjetpacks:iron_jetpack`, `_cell`, `_thruster`, `_capacitor`.
    pub name: &'static str,
    pub jetpack: &'static str,
    pub label: &'static str,
    pub tier: i32,
    pub color: u32,
    pub armor: i32,
    pub capacity: i32,
    pub usage: i32,
    pub speed_vert: f64,
    pub accel_vert: f64,
    pub speed_side: f64,
    pub hover_ascend: f64,
    pub hover_descend: f64,
    pub hover_slow: f64,
    pub sprint_speed: f64,
    pub sprint_speed_vert: f64,
    pub sprint_fuel: f64,
    pub creative: bool,
}

const fn t(
    name: &'static str,
    jetpack: &'static str,
    label: &'static str,
    tier: i32,
    color: u32,
    armor: i32,
    capacity: i32,
    usage: i32,
    speeds: [f64; 9],
) -> Tier {
    Tier {
        name,
        jetpack,
        label,
        tier,
        color,
        armor,
        capacity,
        usage,
        speed_vert: speeds[0],
        accel_vert: speeds[1],
        speed_side: speeds[2],
        hover_ascend: speeds[3],
        hover_descend: speeds[4],
        hover_slow: speeds[5],
        sprint_speed: speeds[6],
        sprint_speed_vert: speeds[7],
        sprint_fuel: speeds[8],
        creative: false,
    }
}

/// speeds: vertical, vertical acceleration, sideways, hover ascend, hover
/// descend, hover, sprint, sprint vertical, sprint fuel.
pub static TIERS: [Tier; 12] = [
    t("wood", "ironjetpacks:wood_jetpack", "Madeira", 0, 0x83663C, 1, 20_000, 32,
      [0.15, 0.10, 0.05, 0.15, 0.25, 0.10, 1.0, 1.0, 1.0]),
    t("stone", "ironjetpacks:stone_jetpack", "Pedra", 1, 0x7F7F7F, 2, 50_000, 64,
      [0.15, 0.11, 0.06, 0.20, 0.25, 0.10, 1.0, 1.0, 1.0]),
    t("copper", "ironjetpacks:copper_jetpack", "Cobre", 1, 0xEE825B, 2, 75_000, 80,
      [0.16, 0.11, 0.07, 0.21, 0.25, 0.10, 1.05, 1.025, 1.2]),
    t("iron", "ironjetpacks:iron_jetpack", "Ferro", 2, 0xD8D8D8, 3, 120_000, 120,
      [0.20, 0.12, 0.10, 0.25, 0.25, 0.08, 1.1, 1.05, 2.0]),
    t("gold", "ironjetpacks:gold_jetpack", "Ouro", 3, 0xF9EB59, 2, 400_000, 300,
      [0.30, 0.13, 0.14, 0.30, 0.25, 0.03, 1.25, 1.1, 3.0]),
    t("steel", "ironjetpacks:steel_jetpack", "Aço", 3, 0x565656, 3, 500_000, 310,
      [0.31, 0.13, 0.14, 0.32, 0.25, 0.03, 1.25, 1.125, 3.2]),
    t("diamond", "ironjetpacks:diamond_jetpack", "Diamante", 4, 0x4CF4E0, 4, 5_000_000, 600,
      [0.35, 0.15, 0.16, 0.35, 0.25, 0.02, 1.3, 1.2, 3.5]),
    t("emerald", "ironjetpacks:emerald_jetpack", "Esmeralda", 5, 0x4DD979, 4, 12_500_000, 900,
      [0.40, 0.16, 0.17, 0.40, 0.25, 0.01, 1.35, 1.3, 4.0]),
    t("allthemodium", "ironjetpacks:allthemodium_jetpack", "Allthemodium", 6, 0xFFEF0E, 9, 60_000_000, 1600,
      [0.45, 0.17, 0.18, 0.45, 0.35, 0.0025, 1.5, 1.4, 4.5]),
    t("vibranium", "ironjetpacks:vibranium_jetpack", "Vibranium", 7, 0x26DE88, 12, 120_000_000, 2700,
      [0.50, 0.18, 0.19, 0.50, 0.40, 0.0, 1.6, 1.45, 5.0]),
    t("unobtainium", "ironjetpacks:unobtainium_jetpack", "Unobtainium", 8, 0xD152E3, 15, 180_000_000, 4000,
      [0.55, 0.20, 0.21, 0.55, 0.45, 0.0, 1.7, 1.5, 5.5]),
    Tier {
        creative: true,
        ..t("creative", "ironjetpacks:creative_jetpack", "Criativo", -1, 0xCF1AE9, 8, 0, 0,
            [0.6, 0.2, 0.25, 0.7, 0.6, 0.0, 2.0, 1.6, 0.0])
    },
];

/// The tier of jetpack item `item`; none for anything else.
pub fn tier_of(item: &str) -> Option<&'static Tier> {
    TIERS.iter().find(|t| t.jetpack == item)
}

pub fn is_jetpack(item: &str) -> bool {
    tier_of(item).is_some()
}

/// A jetpack's own state (its `custom_data`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub energy: i32,
    pub engine: bool,
    pub hover: bool,
    /// Percent, 20 to 100 in steps of 20 (`JETPACK_THROTTLE`, 0.2 to 1.0).
    pub throttle: i32,
}

impl Default for State {
    /// A new jetpack: empty, engine and hover off, full throttle (the
    /// components' defaults).
    fn default() -> Self {
        Self { energy: 0, engine: false, hover: false, throttle: 100 }
    }
}

impl State {
    /// From the stored values, each optional; out-of-range ones are mended.
    pub fn read(energy: Option<i64>, engine: Option<i64>, hover: Option<i64>, throttle: Option<i64>) -> Self {
        Self {
            energy: energy.unwrap_or(0).clamp(0, i64::from(i32::MAX)) as i32,
            engine: engine.unwrap_or(0) != 0,
            hover: hover.unwrap_or(0) != 0,
            throttle: throttle_clamped(throttle.unwrap_or(100)),
        }
    }
    pub fn throttle_f(self) -> f64 {
        f64::from(self.throttle) / 100.0
    }
}

fn throttle_clamped(p: i64) -> i32 {
    // The nearest step of 20 in 20..=100.
    let p = p.clamp(20, 100) as i32;
    ((p + 10) / 20 * 20).clamp(20, 100)
}

/// What a key asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggle {
    Engine,
    Hover,
    ThrottleUp,
    ThrottleDown,
}

impl Toggle {
    pub fn code(self) -> u8 {
        match self {
            Toggle::Engine => 1,
            Toggle::Hover => 2,
            Toggle::ThrottleUp => 3,
            Toggle::ThrottleDown => 4,
        }
    }
    pub fn from_code(c: u8) -> Option<Self> {
        match c {
            1 => Some(Toggle::Engine),
            2 => Some(Toggle::Hover),
            3 => Some(Toggle::ThrottleUp),
            4 => Some(Toggle::ThrottleDown),
            _ => None,
        }
    }
    /// Which key (by its `KEYS` name).
    pub fn of_key(name: &str) -> Option<Self> {
        match name {
            "engine" => Some(Toggle::Engine),
            "hover" => Some(Toggle::Hover),
            "throttle_up" => Some(Toggle::ThrottleUp),
            "throttle_down" => Some(Toggle::ThrottleDown),
            _ => None,
        }
    }
    /// The state after it (`toggleEngine`, `toggleHover`,
    /// `incrementThrottle`, `decrementThrottle`: steps of 0.2, 0.2 to 1).
    pub fn apply(self, s: State) -> State {
        let mut s = s;
        match self {
            Toggle::Engine => s.engine = !s.engine,
            Toggle::Hover => s.hover = !s.hover,
            Toggle::ThrottleUp => s.throttle = (s.throttle + 20).min(100),
            Toggle::ThrottleDown => s.throttle = (s.throttle - 20).max(20),
        }
        s
    }
    /// The action bar line after it (the mod's "Toggled Engine ON").
    pub fn message(self, s: State) -> String {
        let on = |b: bool| if b { "LIGADO" } else { "DESLIGADO" };
        match self {
            Toggle::Engine => format!("Motor: {}", on(s.engine)),
            Toggle::Hover => format!("Modo pairar: {}", on(s.hover)),
            Toggle::ThrottleUp | Toggle::ThrottleDown => format!("Potência definida para {}%", s.throttle),
        }
    }
}

/// The keys the flight reads, as the client sends them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Input {
    /// Jump held (`isHoldingUp`).
    pub up: bool,
    /// Sneak held (`isHoldingDown`; also crouching).
    pub down: bool,
    /// The movement keys: forward > 0 is W, < 0 S; strafe > 0 is A, < 0 D.
    pub forward: f32,
    pub strafe: f32,
    /// The player sprints.
    pub sprint: bool,
}

pub fn input_flags(up: bool, down: bool) -> u8 {
    u8::from(up) | u8::from(down) << 1
}

pub fn flags_input(flags: u8) -> (bool, bool) {
    (flags & 1 != 0, flags & 2 != 0)
}

/// The player's body, as the flight reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Body {
    /// Blocks a tick.
    pub velocity: [f64; 3],
    pub on_ground: bool,
    pub in_water: bool,
    pub fall_flying: bool,
    /// Degrees (`yRot`).
    pub yaw: f32,
    /// Creative game mode (`player.isCreative()`).
    pub creative: bool,
}

/// The jetpack is working this tick: engine on, and jump held or hovering
/// off the ground (the condition of `inventoryTick`).
pub fn thrusting(s: State, up: bool, on_ground: bool) -> bool {
    s.engine && (up || (s.hover && !on_ground))
}

/// The energy one tick takes (`usage`, times `sprintFuel` while sprinting,
/// truncated as `(int) usage` is).
pub fn usage(t: &Tier, sprinting: bool) -> i32 {
    if sprinting {
        (f64::from(t.usage) * t.sprint_fuel) as i32
    } else {
        t.usage
    }
}

/// One thrusting tick on the server: the energy left after it, and whether
/// the jetpack still flies (it takes its usage first, then flies only with
/// energy left, in creative, or as the creative jetpack).
pub fn drain(t: &Tier, s: State, sprinting: bool, creative_player: bool) -> (i32, bool) {
    if creative_player || t.creative {
        return (s.energy, true);
    }
    let left = (s.energy - usage(t, sprinting)).max(0);
    (left, left > 0)
}

/// Whether the client should fly with what its copy of the jetpack holds:
/// the server's drain is a tick ahead, so the client flies while its copy
/// has more than one tick's usage (the server stops it the tick after).
pub fn client_powered(t: &Tier, s: State, sprinting: bool, creative_player: bool) -> bool {
    creative_player || t.creative || s.energy > usage(t, sprinting)
}

/// The vertical velocity the tick sets (`fly(player, y)`).
pub fn vertical(t: &Tier, s: State, i: Input, b: Body) -> f64 {
    let vy = b.velocity[1];
    let hover_speed = if i.down { t.hover_descend } else { t.hover_slow };
    let accel = t.accel_vert * if vy < 0.3 { 2.5 } else { 1.0 };
    let speed = t.speed_vert * if b.in_water { 0.4 } else { 1.0 };
    let throttle = s.throttle_f();
    let sprint_vert = if vy >= 0.0 && i.sprint { t.sprint_speed_vert } else { 1.0 };
    if i.up {
        if !s.hover {
            (vy + accel).min(speed) * throttle * sprint_vert
        } else if i.down {
            (vy + accel).min(-t.hover_slow)
        } else {
            (vy + accel).min(t.hover_ascend) * throttle * sprint_vert
        }
    } else {
        (vy + accel).min(-hover_speed)
    }
}

/// `Entity.getInputVector(input, 1, yRot)` for an input shorter than 1.
fn relative(x: f64, z: f64, yaw: f32) -> [f64; 2] {
    if x * x + z * z < 1.0E-7 {
        return [0.0, 0.0];
    }
    let r = f64::from(yaw) * std::f64::consts::PI / 180.0;
    let (sin, cos) = (r.sin(), r.cos());
    [x * cos - z * sin, z * cos + x * sin]
}

/// The horizontal push the tick adds (`moveRelative` for each held key);
/// none while gliding.
pub fn horizontal(t: &Tier, s: State, i: Input, b: Body) -> [f64; 2] {
    if b.fall_flying {
        return [0.0, 0.0];
    }
    let throttle = s.throttle_f();
    let side = if i.down { t.speed_side * 0.5 } else { t.speed_side } * throttle;
    let forward = if i.sprint { side * t.sprint_speed } else { side } * throttle;
    let mut out = [0.0, 0.0];
    let mut add = |x: f64, z: f64| {
        let v = relative(x, z, b.yaw);
        out[0] += v[0];
        out[1] += v[1];
    };
    if i.forward > 0.0 {
        add(0.0, forward);
    }
    if i.forward < 0.0 {
        add(0.0, -side * 0.8);
    }
    if i.strafe > 0.0 {
        add(side, 0.0);
    }
    if i.strafe < 0.0 {
        add(-side, 0.0);
    }
    out
}

/// The push the client adds to its own velocity this tick, if the jetpack
/// flies: the vertical one replaces the velocity (`fly` sets it), the
/// horizontal one adds to it.
pub fn push(t: &Tier, s: State, i: Input, b: Body) -> Option<[f64; 3]> {
    if !thrusting(s, i.up, b.on_ground) || !client_powered(t, s, i.sprint, b.creative) {
        return None;
    }
    let vy = vertical(t, s, i, b);
    let [x, z] = horizontal(t, s, i, b);
    Some([x, vy - b.velocity[1], z])
}

/// `Mth.hsvToRgb`.
pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> u32 {
    let h = ((hue * 6.0) as i32).rem_euclid(6);
    let f = hue * 6.0 - (hue * 6.0).floor();
    let p = value * (1.0 - saturation);
    let q = value * (1.0 - f * saturation);
    let t = value * (1.0 - (1.0 - f) * saturation);
    let (r, g, b) = match h {
        0 => (value, t, p),
        1 => (q, value, p),
        2 => (p, value, t),
        3 => (p, q, value),
        4 => (t, p, value),
        _ => (value, p, q),
    };
    let c = |v: f32| ((v * 255.0) as i32).clamp(0, 255) as u32;
    c(r) << 16 | c(g) << 8 | c(b)
}

/// The item bar (`getBarWidth`, `getBarColor`): none for the creative one.
pub fn bar(t: &Tier, energy: i32) -> Option<(i32, u32)> {
    if t.creative || t.capacity <= 0 {
        return None;
    }
    let cap = t.capacity as f32;
    let missing = (t.capacity - energy.clamp(0, t.capacity)) as f32;
    let width = (13.0 - missing * 13.0 / cap).round() as i32;
    let f = (energy.max(0) as f32 / cap).max(0.0);
    Some((width.clamp(0, 13), hsv_to_rgb(f / 3.0, 1.0, 1.0)))
}

/// The HUD's energy text (`getFuelComponent`).
pub fn fuel(energy: i32) -> String {
    let e = energy.max(0);
    if e >= 1_000_000_000 {
        let big = e / 1_000_000_000;
        let small = (e - big * 1_000_000_000) / 100_000_000;
        if small != 0 { format!("{big}.{small}G FE") } else { format!("{big}G FE") }
    } else if e >= 1_000_000 {
        let big = e / 1_000_000;
        let small = (e - big * 1_000_000) / 100_000;
        if small != 0 { format!("{big}.{small}M FE") } else { format!("{big}M FE") }
    } else if e >= 1000 {
        format!("{}k FE", e / 1000)
    } else {
        format!("{e} FE")
    }
}

/// `1234567` as `1.234.567` (the tooltip's numbers, pt_BR).
pub fn grouped(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (k, c) in digits.chars().enumerate() {
        if k > 0 && (digits.len() - k) % 3 == 0 {
            out.push('.');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// The percentage a jetpack holds (0 to 100).
pub fn percent(t: &Tier, energy: i32) -> i32 {
    if t.creative || t.capacity <= 0 {
        return 100;
    }
    (i64::from(energy.clamp(0, t.capacity)) * 100 / i64::from(t.capacity)) as i32
}

/// The HUD's lines: name, energy, state.
pub fn hud_lines(t: &Tier, s: State) -> [String; 3] {
    let on = |b: bool| if b { "LIG" } else { "DESL" };
    let energy = if t.creative {
        "Energia: Infinita".to_owned()
    } else {
        format!("Energia: {}% ({})", percent(t, s.energy), fuel(s.energy))
    };
    [
        format!("Jetpack de {}", t.label),
        energy,
        format!("Motor {} | Pairar {} | {}%", on(s.engine), on(s.hover), s.throttle),
    ]
}

/// The lines the tooltip adds under the name, with `§` colour codes.
pub fn tooltip_lines(t: &Tier, s: State) -> Vec<String> {
    let mut out = Vec::new();
    if t.creative {
        out.push("§7Infinito FE".to_owned());
    } else {
        out.push(format!("§7{} / {} FE", grouped(i64::from(s.energy)), grouped(i64::from(t.capacity))));
    }
    let tier = if t.creative { "C".to_owned() } else { t.tier.to_string() };
    let c = |b: bool| if b { "§a" } else { "§c" };
    out.push(format!("§fNÍVEL {tier} §7| {}MOTOR §7| {}PAIRAR", c(s.engine), c(s.hover)));
    out.push(format!("§7Potência: {}%", s.throttle));
    out.push(format!("§7Consumo: {} FE/t (correndo {} FE/t)", t.usage, usage(t, true)));
    out.push(format!(
        "§7Vel. vertical {} · horizontal {} · pairar {}",
        t.speed_vert, t.speed_side, t.hover_slow
    ));
    out
}

// ------------------------------------------------------------- charging

/// A block a jetpack is charged from with a right click: its id, the unit
/// its block entity keeps (`energy`) and the most one click takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Charger {
    /// Mekanism's energy cubes (joules; 2 FE for every 5 J).
    MekanismCube,
    /// Powah's energy cells (FE).
    PowahCell,
}

pub fn charger(block: &str) -> Option<Charger> {
    match block {
        "mekanism:basic_energy_cube"
        | "mekanism:advanced_energy_cube"
        | "mekanism:elite_energy_cube"
        | "mekanism:ultimate_energy_cube"
        | "mekanism:creative_energy_cube" => Some(Charger::MekanismCube),
        "powah:energy_cell_starter" | "powah:energy_cell_basic" | "powah:energy_cell_hardened"
        | "powah:energy_cell_blazing" | "powah:energy_cell_niotic" | "powah:energy_cell_spirited"
        | "powah:energy_cell_nitro" | "powah:energy_cell_creative" => Some(Charger::PowahCell),
        _ => None,
    }
}

/// A click: from a block holding `stored` (its own unit) into a jetpack with
/// `room` FE free, the FE it gains and the block's units it takes. Whole FE
/// only; a Mekanism cube gives 2 FE for 5 J (what it takes rounds up).
pub fn charge(c: Charger, stored: i64, room: i64) -> (i64, i64) {
    let stored = stored.max(0);
    let room = room.max(0);
    match c {
        Charger::MekanismCube => {
            let fe = (stored * 2 / 5).min(room);
            let joules = (fe * 5 + 1) / 2;
            (fe, joules.min(stored))
        }
        Charger::PowahCell => {
            let fe = stored.min(room);
            (fe, fe)
        }
    }
}

/// The action bar after a charge.
pub fn charge_message(t: &Tier, gained: i64, energy: i32) -> String {
    format!(
        "Jetpack de {}: +{} FE ({}%)",
        t.label,
        grouped(gained),
        percent(t, energy)
    )
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    fn tier(name: &str) -> &'static Tier {
        TIERS.iter().find(|t| t.name == name).unwrap()
    }
    fn on() -> State {
        State { energy: 100_000, engine: true, hover: false, throttle: 100 }
    }
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn tiers_are_atm10s_numbers() {
        assert_eq!(TIERS.len(), 12);
        let iron = tier("iron");
        assert_eq!((iron.capacity, iron.usage, iron.armor, iron.tier), (120_000, 120, 3, 2));
        assert_eq!((iron.speed_vert, iron.accel_vert, iron.speed_side), (0.20, 0.12, 0.10));
        let unob = tier("unobtainium");
        assert_eq!((unob.capacity, unob.usage, unob.tier), (180_000_000, 4000, 8));
        assert!(tier("creative").creative);
        assert_eq!(tier_of("ironjetpacks:steel_jetpack").unwrap().usage, 310);
        assert!(tier_of("ironjetpacks:iron_cell").is_none());
        // Every jetpack id is unique and matches its name.
        for t in &TIERS {
            assert_eq!(t.jetpack, format!("ironjetpacks:{}_jetpack", t.name));
            assert!(t.capacity >= 0 && t.capacity <= i32::MAX);
        }
    }

    #[test]
    fn usage_and_sprint_fuel() {
        assert_eq!(usage(tier("wood"), false), 32);
        assert_eq!(usage(tier("wood"), true), 32);
        assert_eq!(usage(tier("iron"), true), 240);
        assert_eq!(usage(tier("steel"), true), 992); // 310 * 3.2
        assert_eq!(usage(tier("copper"), true), 96); // 80 * 1.2
        assert_eq!(usage(tier("unobtainium"), true), 22_000);
    }

    #[test]
    fn drain_takes_usage_and_stops_at_empty() {
        let iron = tier("iron");
        assert_eq!(drain(iron, on(), false, false), (99_880, true));
        assert_eq!(drain(iron, State { energy: 200, ..on() }, false, false), (80, true));
        assert_eq!(drain(iron, State { energy: 100, ..on() }, false, false), (0, false));
        assert_eq!(drain(iron, State { energy: 0, ..on() }, false, false), (0, false));
        // Creative players and the creative jetpack never use energy.
        assert_eq!(drain(iron, State { energy: 0, ..on() }, false, true), (0, true));
        assert_eq!(drain(tier("creative"), State { energy: 0, ..on() }, true, false), (0, true));
        // A full iron jetpack flies 1000 ticks (50 s) of held jump, 500 sprinting.
        let mut s = State { energy: iron.capacity, ..on() };
        let mut ticks = 0;
        loop {
            let (left, flies) = drain(iron, s, false, false);
            s.energy = left;
            if !flies {
                break;
            }
            ticks += 1;
        }
        assert_eq!(ticks, 999);
    }

    #[test]
    fn thrusting_needs_engine_and_jump_or_hover_in_air() {
        let s = on();
        assert!(thrusting(s, true, true));
        assert!(!thrusting(s, false, false));
        assert!(!thrusting(State { engine: false, ..s }, true, false));
        let h = State { hover: true, ..s };
        assert!(thrusting(h, false, false));
        assert!(!thrusting(h, false, true));
    }

    #[test]
    fn vertical_accelerates_to_the_tiers_speed() {
        let iron = tier("iron");
        let i = Input { up: true, ..Input::default() };
        // From rest: 0 + 0.12 * 2.5 = 0.3, capped at 0.2.
        assert!(close(vertical(iron, on(), i, Body::default()), 0.2));
        // Falling fast: -1.0 + 0.3 = -0.7 (the push brakes the fall).
        let falling = Body { velocity: [0.0, -1.0, 0.0], ..Body::default() };
        assert!(close(vertical(iron, on(), i, falling), -0.7));
        // Wood from -0.08 (gravity's first tick): -0.08 + 0.25 = 0.17, capped at 0.15.
        let b = Body { velocity: [0.0, -0.08, 0.0], ..Body::default() };
        assert!(close(vertical(tier("wood"), on(), i, b), 0.15));
        // Diamond: from 0.1, +0.375 = 0.475, capped at 0.35.
        let b = Body { velocity: [0.0, 0.1, 0.0], ..Body::default() };
        assert!(close(vertical(tier("diamond"), on(), i, b), 0.35));
        // At 0.3 or more the acceleration is the plain one.
        let gold = tier("gold");
        let b = Body { velocity: [0.0, 0.3, 0.0], ..Body::default() };
        assert!(close(vertical(gold, on(), i, b), 0.3)); // min(0.43, 0.30)
        // In water the speed is 40%.
        let b = Body { in_water: true, ..Body::default() };
        assert!(close(vertical(iron, on(), i, b), 0.2 * 0.4));
        // Throttle 60% and sprinting (vertical multiplier 1.05).
        let s = State { throttle: 60, ..on() };
        let sprint = Input { sprint: true, ..i };
        assert!(close(vertical(iron, s, sprint, Body::default()), 0.2 * 0.6 * 1.05));
    }

    #[test]
    fn hover_holds_a_slow_fall() {
        let iron = tier("iron");
        let s = State { hover: true, ..on() };
        let none = Input::default();
        // Not holding anything, falling at -0.5: brakes by 0.3 a tick; from -0.1 it holds -0.08 (hover speed).
        let b = Body { velocity: [0.0, -0.5, 0.0], ..Body::default() };
        assert!(close(vertical(iron, s, none, b), -0.2));
        let b = Body { velocity: [0.0, -0.1, 0.0], ..Body::default() };
        assert!(close(vertical(iron, s, none, b), -0.08));
        // Sneaking: descends at the hover descend speed (0.25).
        let down = Input { down: true, ..none };
        let b = Body { velocity: [0.0, -0.3, 0.0], ..Body::default() };
        assert!(close(vertical(iron, s, down, b), -0.25));
        // Jump: rises at the hover ascend speed.
        let up = Input { up: true, ..none };
        assert!(close(vertical(iron, s, up, Body::default()), 0.25));
        // Jump and sneak: -hover speed.
        let both = Input { up: true, down: true, ..none };
        assert!(close(vertical(iron, s, both, Body::default()), -0.08));
        // Vibranium hovers in place (hover speed 0).
        let vib = tier("vibranium");
        assert!(close(vertical(vib, s, none, Body::default()), 0.0));
        // Hovering on the ground does nothing.
        let ground = Body { on_ground: true, ..Body::default() };
        assert_eq!(push(iron, s, none, ground), None);
        assert!(push(iron, s, none, Body::default()).is_some());
    }

    #[test]
    fn horizontal_follows_the_look() {
        let iron = tier("iron");
        let fwd = Input { up: true, forward: 1.0, ..Input::default() };
        // Facing south (yaw 0): +z.
        let [x, z] = horizontal(iron, on(), fwd, Body::default());
        assert!(close(x, 0.0) && close(z, 0.10));
        // Facing west (yaw 90): -x.
        let b = Body { yaw: 90.0, ..Body::default() };
        let [x, z] = horizontal(iron, on(), fwd, b);
        assert!(close(x, -0.10) && z.abs() < 1e-9);
        // Backwards is 80%; left is +x facing south.
        let back = Input { forward: -1.0, ..fwd };
        assert!(close(horizontal(iron, on(), back, Body::default())[1], -0.08));
        let left = Input { forward: 0.0, strafe: 1.0, ..fwd };
        assert!(close(horizontal(iron, on(), left, Body::default())[0], 0.10));
        // Sprinting forward: speed * sprint (1.1); sneaking halves the sideways speed.
        let sprint = Input { sprint: true, ..fwd };
        assert!(close(horizontal(iron, on(), sprint, Body::default())[1], 0.11));
        let sneak = Input { down: true, ..fwd };
        assert!(close(horizontal(iron, on(), sneak, Body::default())[1], 0.05));
        // Throttle counts twice forward (side * t, then * t), as in the mod.
        let s = State { throttle: 60, ..on() };
        assert!(close(horizontal(iron, s, fwd, Body::default())[1], 0.10 * 0.6 * 0.6));
        // Gliding: nothing sideways.
        let glide = Body { fall_flying: true, ..Body::default() };
        assert_eq!(horizontal(iron, on(), fwd, glide), [0.0, 0.0]);
    }

    #[test]
    fn push_is_the_change_of_velocity() {
        let iron = tier("iron");
        let i = Input { up: true, ..Input::default() };
        let b = Body { velocity: [0.05, -0.08, 0.0], ..Body::default() };
        let p = push(iron, on(), i, b).unwrap();
        assert!(close(p[1], 0.2 + 0.08));
        assert!(close(p[0], 0.0));
        // Empty: no push; one tick's worth or less left: no push either.
        assert_eq!(push(iron, State { energy: 0, ..on() }, i, b), None);
        assert_eq!(push(iron, State { energy: 120, ..on() }, i, b), None);
        assert!(push(iron, State { energy: 121, ..on() }, i, b).is_some());
        // Creative player flies empty.
        let c = Body { creative: true, ..b };
        assert!(push(iron, State { energy: 0, ..on() }, i, c).is_some());
        // Engine off: nothing.
        assert_eq!(push(iron, State { engine: false, ..on() }, i, b), None);
    }

    #[test]
    fn state_reads_and_toggles() {
        assert_eq!(State::read(None, None, None, None), State::default());
        let s = State::read(Some(-5), Some(1), Some(0), Some(57));
        assert_eq!(s, State { energy: 0, engine: true, hover: false, throttle: 60 });
        assert_eq!(State::read(Some(i64::MAX), None, None, Some(500)).energy, i32::MAX);
        assert_eq!(State::read(None, None, None, Some(500)).throttle, 100);
        assert_eq!(State::read(None, None, None, Some(0)).throttle, 20);
        let s = State::default();
        assert!(Toggle::Engine.apply(s).engine);
        assert!(Toggle::Hover.apply(s).hover);
        assert_eq!(Toggle::ThrottleUp.apply(s).throttle, 100);
        let mut d = s;
        for _ in 0..10 {
            d = Toggle::ThrottleDown.apply(d);
        }
        assert_eq!(d.throttle, 20);
        assert_eq!(Toggle::ThrottleUp.apply(d).throttle, 40);
        for t in [Toggle::Engine, Toggle::Hover, Toggle::ThrottleUp, Toggle::ThrottleDown] {
            assert_eq!(Toggle::from_code(t.code()), Some(t));
        }
        assert_eq!(Toggle::from_code(9), None);
        assert_eq!(Toggle::Engine.message(Toggle::Engine.apply(s)), "Motor: LIGADO");
        assert_eq!(flags_input(input_flags(true, false)), (true, false));
        assert_eq!(flags_input(input_flags(false, true)), (false, true));
        for (name, _, _) in KEYS {
            assert!(Toggle::of_key(name).is_some());
        }
    }

    #[test]
    fn bar_and_texts() {
        let iron = tier("iron");
        assert_eq!(bar(iron, 120_000), Some((13, 0x00FF00)));
        assert_eq!(bar(iron, 0).unwrap().0, 0);
        assert_eq!(bar(iron, 60_000).unwrap().0, 7); // 6.5 rounds up
        assert_eq!(bar(iron, 0).unwrap().1, 0xFF0000);
        assert_eq!(bar(tier("creative"), 0), None);
        assert_eq!(hsv_to_rgb(1.0 / 6.0, 1.0, 1.0), 0xFFFF00);
        assert_eq!(fuel(999), "999 FE");
        assert_eq!(fuel(54_321), "54k FE");
        assert_eq!(fuel(12_500_000), "12.5M FE");
        assert_eq!(fuel(5_000_000), "5M FE");
        assert_eq!(grouped(1_234_567), "1.234.567");
        assert_eq!(grouped(0), "0");
        assert_eq!(percent(iron, 60_000), 50);
        let lines = hud_lines(iron, State { energy: 60_000, engine: true, hover: false, throttle: 80 });
        assert_eq!(lines[0], "Jetpack de Ferro");
        assert_eq!(lines[1], "Energia: 50% (60k FE)");
        assert_eq!(lines[2], "Motor LIG | Pairar DESL | 80%");
        let tip = tooltip_lines(iron, State::default());
        assert_eq!(tip[0], "§70 / 120.000 FE");
        assert!(tip[1].contains("NÍVEL 2"));
    }

    #[test]
    fn charging_from_cubes_and_cells() {
        assert_eq!(charger("mekanism:basic_energy_cube"), Some(Charger::MekanismCube));
        assert_eq!(charger("powah:energy_cell_basic"), Some(Charger::PowahCell));
        assert_eq!(charger("minecraft:stone"), None);
        // A cube with 4 MJ gives 1.6M FE; an iron jetpack takes 120k of it (300k J).
        assert_eq!(charge(Charger::MekanismCube, 4_000_000, 120_000), (120_000, 300_000));
        // Room larger than the cube: everything, in whole FE.
        assert_eq!(charge(Charger::MekanismCube, 7, 1_000), (2, 5));
        assert_eq!(charge(Charger::MekanismCube, 3, 1_000), (1, 3));
        assert_eq!(charge(Charger::MekanismCube, 2, 1_000), (0, 0));
        // An odd FE count takes the joules rounded up, never more than stored.
        assert_eq!(charge(Charger::MekanismCube, 1_000, 1), (1, 3));
        assert_eq!(charge(Charger::PowahCell, 50_000, 120_000), (50_000, 50_000));
        assert_eq!(charge(Charger::PowahCell, -5, 10), (0, 0));
        assert_eq!(charge(Charger::PowahCell, 50, 0), (0, 0));
        assert_eq!(charge_message(tier("iron"), 120_000, 120_000), "Jetpack de Ferro: +120.000 FE (100%)");
    }
}

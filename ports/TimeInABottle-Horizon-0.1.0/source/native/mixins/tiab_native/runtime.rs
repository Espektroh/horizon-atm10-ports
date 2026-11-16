// Time In A Bottle for Horizon: the rules, std only (tested with `cargo test`).
// Added to the game as `crate::game::tiab_rt`. Numbers and rules from haoict's
// Time In A Bottle (MIT, branch master 4.0.0: AbstractTiabItem, TimeInABottleItem,
// TimeAcceleratorEntity, TiabConfig defaults).

pub const ID: &str = "tiab_native";
pub const NAME: &str = "Time In A Bottle";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Tempo na Garrafa: acumula tempo e acelera blocos (x2 a x256).";

/// The item (content mod `tiab`).
pub const ITEM: &str = "tiab:time_in_a_bottle";
/// Keys in the stack's `minecraft:custom_data` (the original's NBT keys), in ticks.
pub const KEY_STORED: &str = "storedTime";
pub const KEY_TOTAL: &str = "totalAccumulatedTime";

/// `Constants.TICK_CONST`: ticks per second.
pub const TICK_CONST: i32 = 20;
/// `TiabConfig` defaults (ATM10 keeps them: no tiab config in its repository).
pub const EACH_USE_DURATION_S: i32 = 30;
pub const MAX_TIME_RATE_POWER: u32 = 8;
pub const MAX_STORED_TIME: i32 = 622_080_000;
pub const AVERAGE_UPDATE_RANDOM_TICK: u64 = 1365;

/// `getEachUseDuration`: 600 ticks.
pub const fn each_use_duration() -> i32 {
    TICK_CONST * EACH_USE_DURATION_S
}

/// The highest `timeRate` (extra ticks per game tick): 2^(8-1) = 128, shown as x256.
pub const fn max_rate() -> i32 {
    1 << (MAX_TIME_RATE_POWER - 1)
}

/// What the player sees: the original draws "x" + 2 * timeRate.
pub const fn shown_rate(rate: i32) -> i32 {
    2 * rate
}

/// `getEnergyCost(timeRate)`, in ticks of stored time.
pub fn energy_cost(rate: i32) -> i32 {
    if rate <= 1 {
        each_use_duration()
    } else {
        rate / 2 * each_use_duration()
    }
}

/// A running accelerator (the original's `TimeAcceleratorEntity`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Accel {
    pub rate: i32,
    pub remaining: i32,
}

/// What a right click on a block does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UseOutcome {
    /// Start or speed up: the accelerator becomes `accel`, `cost` ticks are spent
    /// (nothing in creative).
    Apply { accel: Accel, cost: i32 },
    /// Already at x256: nothing happens (the original answers SUCCESS and spends nothing).
    AtMax,
    /// Not enough stored time for `cost`.
    NotEnough { cost: i32 },
}

/// `AbstractTiabItem.useOn`, after the target was found acceptable.
pub fn plan_use(existing: Option<Accel>, stored: i32, creative: bool) -> UseOutcome {
    let can = |cost: i32| creative || stored >= cost;
    match existing {
        None => {
            let cost = energy_cost(1);
            if !can(cost) {
                return UseOutcome::NotEnough { cost };
            }
            UseOutcome::Apply {
                accel: Accel {
                    rate: 1,
                    remaining: each_use_duration(),
                },
                cost: if creative { 0 } else { cost },
            }
        }
        Some(a) => {
            if a.rate >= max_rate() {
                return UseOutcome::AtMax;
            }
            let next = a.rate * 2;
            let used = each_use_duration() - a.remaining;
            let cost = energy_cost(next);
            if !can(cost) {
                return UseOutcome::NotEnough { cost };
            }
            UseOutcome::Apply {
                accel: Accel {
                    rate: next,
                    remaining: a.remaining + used / 2,
                },
                cost: if creative { 0 } else { cost },
            }
        }
    }
}

/// `TimeInABottleItem.inventoryTick` every 20 game ticks: +20 to both counters, capped.
/// `None` when it is not such a tick.
pub fn accumulate(game_time: i64, stored: i32, total: i32) -> Option<(i32, i32)> {
    if game_time.rem_euclid(TICK_CONST as i64) != 0 {
        return None;
    }
    let up = |v: i32| {
        if v < MAX_STORED_TIME {
            (v.max(0) + TICK_CONST).min(MAX_STORED_TIME)
        } else {
            v
        }
    };
    Some((up(stored), up(total)))
}

/// Every 10 s the original empties a bottle when another one in the same
/// inventory holds more time.
pub fn duplicate_check_tick(game_time: i64) -> bool {
    game_time.rem_euclid((TICK_CONST * 10) as i64) == 0
}

/// The stored time after the duplicate check: 0 if any other bottle holds more.
pub fn after_duplicate_check(mine: i32, others: &[i32]) -> i32 {
    if others.iter().any(|o| mine < *o) {
        0
    } else {
        mine
    }
}

/// Stored time after spending `cost` (`applyDamage` -> `setStoredEnergy`, min with the cap).
pub fn spend(stored: i32, cost: i32) -> i32 {
    (stored - cost).min(MAX_STORED_TIME)
}

/// Ticks as the tooltip shows them: ("HH", "MM", "SS").
pub fn hms(ticks: i32) -> (String, String, String) {
    let s = ticks.max(0) / TICK_CONST;
    (
        format!("{:02}", s / 3600),
        format!("{:02}", (s % 3600) / 60),
        format!("{:02}", s % 60),
    )
}

/// The harp note the original plays for the new rate (`AbstractTiabItem.playSound`).
pub fn note_pitch(rate: i32) -> f32 {
    const PITCH: [f32; 11] = [
        0.707107, 0.793701, 0.890899, 0.943874, 1.059463, 1.189207, 1.334840, 1.414214, 1.587401,
        1.781797, 1.887749,
    ];
    let i = match rate {
        1 => 0,
        2 => 1,
        4 => 2,
        8 => 3,
        16 => 4,
        32 => 5,
        64 => 6,
        128 => 7,
        256 => 8,
        512 => 9,
        _ => 10,
    };
    PITCH[i]
}

/// A small deterministic generator (xorshift64*), seeded per tick and block.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut s = seed ^ 0x9E37_79B9_7F4A_7C15;
        if s == 0 {
            s = 1;
        }
        Rng(s)
    }
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// A seed from the game time and the block's position.
pub fn seed(game_time: i64, pos: [i32; 3]) -> u64 {
    let mut h = game_time as u64;
    for v in pos {
        h = h.wrapping_mul(0x100_0000_01B3) ^ (v as u32 as u64);
        h = h.rotate_left(17);
    }
    h
}

/// How many random ticks a randomly ticking block gets this game tick: each of
/// the `rate` extra ticks has a 1 in 1365 chance (`averageUpdateRandomTick`),
/// which is vanilla's random tick rate (3/4096 a tick), so x2N is N times vanilla.
pub fn random_ticks(rate: i32, rng: &mut Rng) -> u32 {
    let mut n = 0;
    for _ in 0..rate.max(0) {
        if rng.next() % AVERAGE_UPDATE_RANDOM_TICK == 0 {
            n += 1;
        }
    }
    n
}

/// The running accelerators, by dimension and block (kept in memory only).
pub struct Accelerators {
    pub list: Vec<(String, [i32; 3], Accel)>,
}

impl Accelerators {
    pub const fn new() -> Self {
        Accelerators { list: Vec::new() }
    }
    pub fn get(&self, dim: &str, pos: [i32; 3]) -> Option<Accel> {
        self.list
            .iter()
            .find(|(d, p, _)| d == dim && *p == pos)
            .map(|(_, _, a)| *a)
    }
    pub fn set(&mut self, dim: &str, pos: [i32; 3], accel: Accel) {
        match self.list.iter_mut().find(|(d, p, _)| d == dim && *p == pos) {
            Some(e) => e.2 = accel,
            None => {
                // A bound, so a broken world can't grow it forever.
                if self.list.len() >= 4096 {
                    self.list.remove(0);
                }
                self.list.push((dim.to_owned(), pos, accel));
            }
        }
    }
    pub fn remove(&mut self, dim: &str, pos: [i32; 3]) {
        self.list.retain(|(d, p, _)| !(d == dim && *p == pos));
    }
    /// Those of `dim`, as (pos, rate).
    pub fn of(&self, dim: &str) -> Vec<([i32; 3], i32)> {
        self.list
            .iter()
            .filter(|(d, _, _)| d == dim)
            .map(|(_, p, a)| (*p, a.rate))
            .collect()
    }
    /// One game tick of the accelerator at `pos` went by (`remainingTime -= 1`);
    /// it goes when it runs out.
    pub fn count_down(&mut self, dim: &str, pos: [i32; 3]) {
        if let Some(e) = self.list.iter_mut().find(|(d, p, _)| d == dim && *p == pos) {
            e.2.remaining -= 1;
        }
        self.list.retain(|(_, _, a)| a.remaining > 0);
    }
    pub fn clear(&mut self) {
        self.list.clear();
    }
}

impl Default for Accelerators {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        assert_eq!(each_use_duration(), 600);
        assert_eq!(max_rate(), 128);
        assert_eq!(shown_rate(max_rate()), 256);
        assert_eq!(energy_cost(1), 600);
        assert_eq!(energy_cost(2), 600);
        assert_eq!(energy_cost(4), 1200);
        assert_eq!(energy_cost(128), 38400);
    }

    #[test]
    fn first_use() {
        assert_eq!(
            plan_use(None, 600, false),
            UseOutcome::Apply {
                accel: Accel {
                    rate: 1,
                    remaining: 600
                },
                cost: 600
            }
        );
        assert_eq!(
            plan_use(None, 599, false),
            UseOutcome::NotEnough { cost: 600 }
        );
        assert!(matches!(
            plan_use(None, 0, true),
            UseOutcome::Apply { cost: 0, .. }
        ));
    }

    #[test]
    fn speed_up_adds_half_of_used_time() {
        // 100 ticks went by at x2: x4 costs 600 and gives back 50 ticks.
        let a = Accel {
            rate: 1,
            remaining: 500,
        };
        assert_eq!(
            plan_use(Some(a), 10_000, false),
            UseOutcome::Apply {
                accel: Accel {
                    rate: 2,
                    remaining: 550
                },
                cost: 600
            }
        );
        // Clicked at once: still 600 left.
        let b = Accel {
            rate: 2,
            remaining: 600,
        };
        assert_eq!(
            plan_use(Some(b), 1200, false),
            UseOutcome::Apply {
                accel: Accel {
                    rate: 4,
                    remaining: 600
                },
                cost: 1200
            }
        );
        assert_eq!(
            plan_use(
                Some(Accel {
                    rate: 128,
                    remaining: 10
                }),
                1 << 30,
                false
            ),
            UseOutcome::AtMax
        );
    }

    #[test]
    fn full_chain_cost() {
        // x2, x4 ... x256 clicked at once: 600+600+1200+2400+4800+9600+19200+38400.
        let mut a = None;
        let mut spent = 0;
        for _ in 0..8 {
            match plan_use(a, i32::MAX, false) {
                UseOutcome::Apply { accel, cost } => {
                    a = Some(accel);
                    spent += cost;
                }
                o => panic!("{o:?}"),
            }
        }
        assert_eq!(a.unwrap().rate, 128);
        assert_eq!(spent, 76_800);
        assert_eq!(plan_use(a, i32::MAX, false), UseOutcome::AtMax);
    }

    #[test]
    fn accumulation() {
        assert_eq!(accumulate(19, 0, 0), None);
        assert_eq!(accumulate(40, 0, 100), Some((20, 120)));
        assert_eq!(
            accumulate(20, MAX_STORED_TIME - 5, MAX_STORED_TIME),
            Some((MAX_STORED_TIME, MAX_STORED_TIME))
        );
        assert!(duplicate_check_tick(200));
        assert!(!duplicate_check_tick(220));
        assert_eq!(after_duplicate_check(100, &[200]), 0);
        assert_eq!(after_duplicate_check(300, &[200, 300]), 300);
        assert_eq!(spend(1000, 600), 400);
    }

    #[test]
    fn tooltip_time() {
        assert_eq!(
            hms(20 * 3725),
            ("01".to_string(), "02".to_string(), "05".to_string())
        );
        assert_eq!(hms(0), ("00".into(), "00".into(), "00".into()));
    }

    #[test]
    fn random_tick_rate_matches_vanilla_times_rate() {
        // x256 (rate 128) over 30 s: about 600 * 128 / 1365 = 56 random ticks.
        let mut total = 0;
        for t in 0..600 {
            let mut r = Rng::new(seed(t, [10, 64, -3]));
            total += random_ticks(128, &mut r);
        }
        assert!((35..=80).contains(&total), "{total}");
        let mut r = Rng::new(1);
        assert_eq!(random_ticks(0, &mut r), 0);
    }

    #[test]
    fn registry() {
        let mut r = Accelerators::new();
        r.set(
            "minecraft:overworld",
            [1, 2, 3],
            Accel {
                rate: 1,
                remaining: 2,
            },
        );
        r.set(
            "minecraft:the_nether",
            [1, 2, 3],
            Accel {
                rate: 4,
                remaining: 9,
            },
        );
        assert_eq!(r.of("minecraft:overworld"), vec![([1, 2, 3], 1)]);
        r.count_down("minecraft:overworld", [1, 2, 3]);
        assert!(r.get("minecraft:overworld", [1, 2, 3]).is_some());
        r.count_down("minecraft:overworld", [1, 2, 3]);
        assert!(r.get("minecraft:overworld", [1, 2, 3]).is_none());
        assert_eq!(r.get("minecraft:the_nether", [1, 2, 3]).unwrap().rate, 4);
        r.remove("minecraft:the_nether", [1, 2, 3]);
        assert!(r.list.is_empty());
    }

    #[test]
    fn pitches() {
        assert_eq!(note_pitch(1), 0.707107);
        assert_eq!(note_pitch(128), 1.414214);
    }
}

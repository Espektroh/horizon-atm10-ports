// Hostile Neural Networks for Horizon 0.1.0: the rules of the Data Models
// (tiers, data per kill, accuracy), the Simulation Chamber and the Loot
// Fabricator, with no dependency on the game so it builds and tests on its own.
// This file is both the mod's Cargo library (its tests) and the module the
// bundle adds to the game (`crate::game::hostilenetworks_native`); `glue.rs`
// adapts it to Horizon.
//
// Port of Hostile Neural Networks 6.5.1 (Shadows_of_Fire, MIT). Numbers are
// the original's default config (`HostileConfig`) and data files
// (`data/hostilenetworks/model_tiers/`, `data/hostilenetworks/data_models/`).

use std::sync::atomic::{AtomicU64, Ordering};

pub const ID: &str = "hostilenetworks_native";
pub const NAME: &str = "Hostile Neural Networks";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Hostile Neural Networks: Deep Learner, Data Models treinados por abate (5 tiers), Câmara de Simulação (FE) e Fabricador de Loot, 24 mobs vanilla. Precisa do mod de conteúdo hostilenetworks.";

pub const DEEP_LEARNER: &str = "hostilenetworks:deep_learner";
pub const BLANK_DATA_MODEL: &str = "hostilenetworks:blank_data_model";
pub const PREDICTION_MATRIX: &str = "hostilenetworks:prediction_matrix";
pub const SIM_CHAMBER: &str = "hostilenetworks:sim_chamber";
pub const LOOT_FABRICATOR: &str = "hostilenetworks:loot_fabricator";
pub const SIM_MENU: &str = "hostilenetworks:sim_chamber";
pub const FAB_MENU: &str = "hostilenetworks:loot_fabricator";
const DATA_MODEL_PREFIX: &str = "hostilenetworks:data_model_";
const PREDICTION_PREFIX: &str = "hostilenetworks:prediction_";

/// Keys kept in a Data Model's `minecraft:custom_data` (the original's
/// `hostilenetworks:data` and `hostilenetworks:iterations` components).
pub const KEY_DATA: &str = "data";
pub const KEY_ITERS: &str = "iterations";

/// Data Models a Deep Learner holds (`DeepLearnerItem`: 4 slots).
pub const LEARNER_SLOTS: usize = 4;

/// `HostileConfig` defaults.
pub const SIM_POWER_CAP: i64 = 2_000_000;
pub const FAB_POWER_CAP: i64 = 1_000_000;
pub const FAB_POWER_COST: i64 = 256;
/// Ticks of one Loot Fabricator run (`LootFabTileEntity`: `runtime >= 60`).
pub const FAB_RUNTIME: i32 = 60;
/// `SimMode.INFERENCE`: `round(300 / 1.25)` ticks, cost multiplier 1.
pub const SIM_RUNTIME: i32 = 240;

/// The Simulation Chamber's slots (`SimItemHandler`).
pub const SIM_MODEL: usize = 0;
pub const SIM_INPUT: usize = 1;
pub const SIM_BASE_OUT: usize = 2;
pub const SIM_PRED_OUT: usize = 3;
pub const SIM_SLOTS: usize = 4;
/// The Loot Fabricator's slots (`FabItemHandler`): the prediction, then 16 outputs.
pub const FAB_INPUT: usize = 0;
pub const FAB_SLOTS: usize = 17;

/// One model tier (`data/hostilenetworks/model_tiers/*.json`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tier {
    pub name: &'static str,
    pub label_pt: &'static str,
    pub required: i32,
    pub gained: i32,
    pub accuracy: f32,
    pub can_sim: bool,
    /// The tier's colour (`color`), as RGB.
    pub rgb: u32,
}

pub const TIERS: [Tier; 5] = [
    Tier {
        name: "faulty",
        label_pt: "Defeituoso",
        required: 0,
        gained: 1,
        accuracy: 0.0,
        can_sim: false,
        rgb: 0x555555,
    },
    Tier {
        name: "basic",
        label_pt: "Básico",
        required: 6,
        gained: 4,
        accuracy: 0.05,
        can_sim: true,
        rgb: 0x55FF55,
    },
    Tier {
        name: "advanced",
        label_pt: "Avançado",
        required: 54,
        gained: 10,
        accuracy: 0.22,
        can_sim: true,
        rgb: 0x5555FF,
    },
    Tier {
        name: "superior",
        label_pt: "Superior",
        required: 354,
        gained: 18,
        accuracy: 0.65,
        can_sim: true,
        rgb: 0xFF55FF,
    },
    Tier {
        name: "self_aware",
        label_pt: "Autoconsciente",
        required: 1254,
        gained: 0,
        accuracy: 1.0,
        can_sim: true,
        rgb: 0xFFAA00,
    },
];

/// One entity data model (`data/hostilenetworks/data_models/<name>.json`):
/// the mob and its vanilla variants, the FE/t it costs to simulate, the
/// generalized prediction it always gives, the Loot Fabricator's choices (the
/// non-optional ones) and its own data per kill, by tier, when it has one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Model {
    pub name: &'static str,
    pub entity: &'static str,
    pub variants: &'static [&'static str],
    pub sim_cost: i64,
    pub base_drop: &'static str,
    pub fab: &'static [(&'static str, u32)],
    pub gained: Option<[i32; 4]>,
    pub color: u32,
}

pub const MODELS: &[Model] = &[
    Model {
        name: "zombie",
        entity: "minecraft:zombie",
        variants: &["minecraft:husk", "minecraft:zombie_villager"],
        sim_cost: 128,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[
            ("minecraft:rotten_flesh", 64),
            ("minecraft:iron_ingot", 8),
            ("minecraft:carrot", 16),
            ("minecraft:potato", 16),
        ],
        gained: None,
        color: 0x3B622F,
    },
    Model {
        name: "skeleton",
        entity: "minecraft:skeleton",
        variants: &["minecraft:stray", "minecraft:bogged"],
        sim_cost: 128,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[
            ("minecraft:arrow", 32),
            ("minecraft:bone", 24),
            ("minecraft:skeleton_skull", 4),
        ],
        gained: None,
        color: 0xBCBCBC,
    },
    Model {
        name: "creeper",
        entity: "minecraft:creeper",
        variants: &[],
        sim_cost: 128,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[("minecraft:gunpowder", 32), ("minecraft:creeper_head", 4)],
        gained: None,
        color: 0x70DC5D,
    },
    Model {
        name: "spider",
        entity: "minecraft:spider",
        variants: &["minecraft:cave_spider"],
        sim_cost: 128,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[
            ("minecraft:string", 32),
            ("minecraft:spider_eye", 16),
            ("minecraft:cobweb", 4),
        ],
        gained: None,
        color: 0x322B26,
    },
    Model {
        name: "enderman",
        entity: "minecraft:enderman",
        variants: &[],
        sim_cost: 512,
        base_drop: "hostilenetworks:end_prediction",
        fab: &[("minecraft:ender_pearl", 16), ("minecraft:end_crystal", 1)],
        gained: None,
        color: 0x161616,
    },
    Model {
        name: "blaze",
        entity: "minecraft:blaze",
        variants: &[],
        sim_cost: 256,
        base_drop: "hostilenetworks:nether_prediction",
        fab: &[("minecraft:blaze_rod", 16)],
        gained: None,
        color: 0xFFD528,
    },
    Model {
        name: "slime",
        entity: "minecraft:slime",
        variants: &[],
        sim_cost: 128,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[("minecraft:slime_ball", 32), ("minecraft:slime_block", 3)],
        gained: None,
        color: 0x7BCA62,
    },
    Model {
        name: "wither_skeleton",
        entity: "minecraft:wither_skeleton",
        variants: &[],
        sim_cost: 768,
        base_drop: "hostilenetworks:nether_prediction",
        fab: &[
            ("minecraft:bone", 24),
            ("minecraft:coal", 32),
            ("minecraft:wither_skeleton_skull", 3),
        ],
        gained: None,
        color: 0x343434,
    },
    Model {
        name: "witch",
        entity: "minecraft:witch",
        variants: &[],
        sim_cost: 256,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[
            ("minecraft:redstone", 16),
            ("minecraft:glowstone_dust", 16),
            ("minecraft:sugar", 32),
            ("minecraft:stick", 32),
            ("minecraft:spider_eye", 16),
            ("minecraft:glass_bottle", 16),
        ],
        gained: None,
        color: 0x361758,
    },
    Model {
        name: "ghast",
        entity: "minecraft:ghast",
        variants: &[],
        sim_cost: 256,
        base_drop: "hostilenetworks:nether_prediction",
        fab: &[("minecraft:ghast_tear", 16), ("minecraft:gunpowder", 32)],
        gained: None,
        color: 0xF0F0F0,
    },
    Model {
        name: "magma_cube",
        entity: "minecraft:magma_cube",
        variants: &[],
        sim_cost: 256,
        base_drop: "hostilenetworks:nether_prediction",
        fab: &[("minecraft:magma_cream", 32)],
        gained: None,
        color: 0xCC2200,
    },
    Model {
        name: "guardian",
        entity: "minecraft:guardian",
        variants: &[],
        sim_cost: 256,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[("minecraft:cod", 24), ("minecraft:prismarine_crystals", 16)],
        gained: None,
        color: 0x6A9583,
    },
    Model {
        name: "drowned",
        entity: "minecraft:drowned",
        variants: &[],
        sim_cost: 128,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[
            ("minecraft:rotten_flesh", 64),
            ("minecraft:copper_ingot", 8),
        ],
        gained: None,
        color: 0x3B6269,
    },
    Model {
        name: "zombified_piglin",
        entity: "minecraft:zombified_piglin",
        variants: &["minecraft:piglin"],
        sim_cost: 256,
        base_drop: "hostilenetworks:nether_prediction",
        fab: &[("minecraft:rotten_flesh", 64), ("minecraft:gold_ingot", 8)],
        gained: None,
        color: 0x3B992F,
    },
    Model {
        name: "shulker",
        entity: "minecraft:shulker",
        variants: &[],
        sim_cost: 512,
        base_drop: "hostilenetworks:end_prediction",
        fab: &[("minecraft:shulker_shell", 8), ("minecraft:end_rod", 4)],
        gained: None,
        color: 0x976797,
    },
    Model {
        name: "phantom",
        entity: "minecraft:phantom",
        variants: &[],
        sim_cost: 128,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[("minecraft:phantom_membrane", 8)],
        gained: None,
        color: 0x0AB405,
    },
    Model {
        name: "evoker",
        entity: "minecraft:evoker",
        variants: &[],
        sim_cost: 1024,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[("minecraft:totem_of_undying", 2), ("minecraft:emerald", 16)],
        gained: None,
        color: 0xBCA341,
    },
    Model {
        name: "vindicator",
        entity: "minecraft:vindicator",
        variants: &[],
        sim_cost: 512,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[("minecraft:emerald", 8), ("minecraft:iron_axe", 1)],
        gained: None,
        color: 0x325566,
    },
    Model {
        name: "hoglin",
        entity: "minecraft:hoglin",
        variants: &["minecraft:zoglin"],
        sim_cost: 256,
        base_drop: "hostilenetworks:nether_prediction",
        fab: &[("minecraft:porkchop", 32), ("minecraft:leather", 16)],
        gained: None,
        color: 0xE8A074,
    },
    Model {
        name: "breeze",
        entity: "minecraft:breeze",
        variants: &[],
        sim_cost: 256,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[("minecraft:breeze_rod", 16)],
        gained: None,
        color: 0x3D4167,
    },
    Model {
        name: "elder_guardian",
        entity: "minecraft:elder_guardian",
        variants: &[],
        sim_cost: 1024,
        base_drop: "hostilenetworks:overworld_prediction",
        fab: &[
            ("minecraft:cod", 16),
            ("minecraft:salmon", 2),
            ("minecraft:pufferfish", 2),
            ("minecraft:tropical_fish", 2),
            ("minecraft:prismarine_crystals", 8),
            ("minecraft:prismarine_shard", 24),
            ("minecraft:wet_sponge", 32),
        ],
        gained: Some([3, 12, 30, 45]),
        color: 0xCBB1A5,
    },
    Model {
        name: "wither",
        entity: "minecraft:wither",
        variants: &[],
        sim_cost: 2560,
        base_drop: "hostilenetworks:end_prediction",
        fab: &[("minecraft:nether_star", 1)],
        gained: None,
        color: 0x343434,
    },
    Model {
        name: "ender_dragon",
        entity: "minecraft:ender_dragon",
        variants: &[],
        sim_cost: 4096,
        base_drop: "hostilenetworks:end_prediction",
        fab: &[("minecraft:dragon_breath", 16), ("minecraft:dragon_egg", 1)],
        gained: Some([3, 12, 30, 45]),
        color: 0xCC00CC,
    },
    Model {
        name: "warden",
        entity: "minecraft:warden",
        variants: &[],
        sim_cost: 2560,
        base_drop: "hostilenetworks:end_prediction",
        fab: &[("minecraft:echo_shard", 2), ("minecraft:sculk_catalyst", 4)],
        gained: None,
        color: 0x05343F,
    },
];

pub fn model_index(name: &str) -> Option<usize> {
    MODELS.iter().position(|m| m.name == name)
}

/// The model a mob of `kind` (`minecraft:husk`) trains.
pub fn model_for_kind(kind: &str) -> Option<usize> {
    let kind = if kind.contains(':') {
        kind.to_owned()
    } else {
        format!("minecraft:{kind}")
    };
    MODELS
        .iter()
        .position(|m| m.entity == kind || m.variants.contains(&kind.as_str()))
}

/// `hostilenetworks:data_model_zombie` for the zombie's model.
pub fn data_model_item(i: usize) -> String {
    format!("{DATA_MODEL_PREFIX}{}", MODELS[i].name)
}

pub fn prediction_item(i: usize) -> String {
    format!("{PREDICTION_PREFIX}{}", MODELS[i].name)
}

/// The model an attuned Data Model item is for.
pub fn model_of_data_item(item: &str) -> Option<usize> {
    item.strip_prefix(DATA_MODEL_PREFIX).and_then(model_index)
}

/// The model a mob Prediction item is from.
pub fn model_of_prediction(item: &str) -> Option<usize> {
    item.strip_prefix(PREDICTION_PREFIX).and_then(model_index)
}

pub fn is_ours(block: &str) -> bool {
    block == SIM_CHAMBER || block == LOOT_FABRICATOR
}

/// `ModelTierRegistry.getByData`: the highest tier whose required data is met.
pub fn tier_of(data: i32) -> usize {
    (0..TIERS.len())
        .rev()
        .find(|&t| data >= TIERS[t].required)
        .unwrap_or(0)
}

/// `DataModel.getDataGained(tier)`: the model's own value, else the tier's.
pub fn gained(model: usize, tier: usize) -> i32 {
    match (MODELS[model].gained, tier) {
        (Some(g), t) if t < 4 => g[t],
        _ => TIERS[tier].gained,
    }
}

/// The data after one kill of the model's mob with the model in a Deep
/// Learner (`HostileEvents.updateModels`): the tier's gain is added (the
/// self aware tier gains nothing).
pub fn data_after_kill(model: usize, data: i32) -> i32 {
    let data = data.max(0);
    data.saturating_add(gained(model, tier_of(data)))
}

/// `DataModelInstance.getAccuracy` with `continuousAccuracy` on: the tier's
/// accuracy, rising linearly towards the next tier's as data comes in.
pub fn accuracy(data: i32) -> f32 {
    let t = tier_of(data);
    if t + 1 >= TIERS.len() {
        return TIERS[t].accuracy;
    }
    let (cur, next) = (TIERS[t], TIERS[t + 1]);
    let diff = (next.required - cur.required) as f32;
    let done = (data - cur.required) as f32;
    cur.accuracy + (next.accuracy - cur.accuracy) * done / diff
}

/// Kills still needed for the next tier (`getActionsNeeded`); None at the top.
pub fn kills_to_next(model: usize, data: i32) -> Option<i32> {
    let t = tier_of(data);
    if t + 1 >= TIERS.len() {
        return None;
    }
    let g = gained(model, t).max(1);
    let left = TIERS[t + 1].required - data;
    Some((left + g - 1) / g)
}

/// `rollPredictions`: each whole 100% is one prediction, the rest a chance
/// of one more. `r` is a number in 0..1.
pub fn roll_predictions(accuracy: f32, r: f32) -> i32 {
    let floor = accuracy.floor();
    floor as i32 + i32::from(r < accuracy - floor)
}

/// Why the Simulation Chamber is not running (`FailureState`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fail {
    None = 0,
    Output = 1,
    Energy = 2,
    Input = 3,
    Model = 4,
    Faulty = 5,
    EnergyMidCycle = 6,
}

impl Fail {
    pub fn from_i32(v: i32) -> Fail {
        match v {
            1 => Fail::Output,
            2 => Fail::Energy,
            3 => Fail::Input,
            4 => Fail::Model,
            5 => Fail::Faulty,
            6 => Fail::EnergyMidCycle,
            _ => Fail::None,
        }
    }
    /// Short enough for the chamber's screen.
    pub fn short_pt(self) -> &'static str {
        match self {
            Fail::None => "Pronto",
            Fail::Output => "Saídas cheias",
            Fail::Energy => "Sem energia",
            Fail::Input => "Sem Matrix",
            Fail::Model => "Sem modelo",
            Fail::Faulty => "Defeituoso",
            Fail::EnergyMidCycle => "Pausado",
        }
    }
}

/// The Simulation Chamber's saved numbers.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SimState {
    pub energy: i64,
    pub runtime: i32,
    /// Predictions this run gives (0: the prediction failed).
    pub success: i32,
    pub fail: i32,
    /// The model the run is for, index + 1 (0: none).
    pub model: i32,
}

/// What the chamber sees in its slots this tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimSlots {
    /// The Data Model in slot 0: its model and data.
    pub model: Option<(usize, i32)>,
    /// A Prediction Matrix is in slot 1.
    pub matrix: bool,
    /// Slot 2 can take one more of the model's generalized prediction.
    pub base_room: bool,
    /// Slot 3 can take one more of the model's mob prediction.
    pub pred_room: bool,
}

/// What a tick did to the slots.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SimDone {
    /// A run started: one Prediction Matrix is used up.
    pub consumed: bool,
    /// A run ended: one generalized prediction out, and this many mob
    /// predictions, and the model's iteration count goes up.
    pub finished: bool,
    pub predictions: i32,
}

/// One server tick of the Simulation Chamber in Inference mode
/// (`SimChamberTileEntity.serverTick`). Redstone control and Training mode
/// are not part of this port.
pub fn sim_tick(s: &mut SimState, slots: &SimSlots, roll: impl FnOnce() -> f32) -> SimDone {
    let mut done = SimDone::default();
    let Some((model, data)) = slots.model else {
        s.fail = Fail::Model as i32;
        s.runtime = 0;
        s.model = 0;
        return done;
    };
    if s.model != model as i32 + 1 {
        s.model = model as i32 + 1;
        s.runtime = 0;
    }
    let tier = tier_of(data);
    if !TIERS[tier].can_sim {
        s.fail = Fail::Faulty as i32;
        s.runtime = 0;
        return done;
    }
    let cost = MODELS[model].sim_cost;
    if s.runtime == 0 {
        if !slots.matrix {
            s.fail = Fail::Input as i32;
        } else if !(slots.base_room && slots.pred_room) {
            s.fail = Fail::Output as i32;
        } else if s.energy < cost {
            s.fail = Fail::Energy as i32;
        } else {
            s.fail = Fail::None as i32;
            s.runtime = SIM_RUNTIME;
            s.success = roll_predictions(accuracy(data), roll());
            done.consumed = true;
        }
    } else if s.energy >= cost {
        s.fail = Fail::None as i32;
        s.runtime -= 1;
        if s.runtime == 0 {
            done.finished = true;
            done.predictions = s.success.max(0);
        } else {
            s.energy -= cost;
        }
    } else {
        s.fail = Fail::EnergyMidCycle as i32;
    }
    done
}

/// The Loot Fabricator's saved numbers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FabState {
    pub energy: i64,
    pub runtime: i32,
    /// The drop the run is for, as `selection + 1` (0: none), and the model.
    pub current: i32,
    pub model: i32,
}

/// What a Loot Fabricator tick asks of the slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FabStep {
    Idle,
    Working,
    /// Put the drop `selection` of the model in the outputs and use up one
    /// prediction, if it fits (`insert_plan`); `fab_finish` then resets.
    Output(usize, usize),
}

/// One tick of the Loot Fabricator (`LootFabTileEntity.serverTick`):
/// `input` is the model of the prediction in slot 0 and `selection` the drop
/// chosen for that model (or None).
pub fn fab_tick(s: &mut FabState, input: Option<usize>, selection: Option<usize>) -> FabStep {
    let Some(model) = input else {
        s.runtime = 0;
        s.model = 0;
        return FabStep::Idle;
    };
    let sel = selection.filter(|&i| i < MODELS[model].fab.len());
    let cur = sel.map_or(0, |i| i as i32 + 1);
    if s.current != cur || s.model != model as i32 + 1 {
        s.current = cur;
        s.model = model as i32 + 1;
        s.runtime = 0;
        return FabStep::Idle;
    }
    let Some(sel) = sel else {
        s.runtime = 0;
        return FabStep::Idle;
    };
    if s.runtime >= FAB_RUNTIME {
        return FabStep::Output(model, sel);
    }
    if s.energy < FAB_POWER_COST {
        return FabStep::Idle;
    }
    s.energy -= FAB_POWER_COST;
    s.runtime += 1;
    FabStep::Working
}

/// An output slot as `insert_plan` sees it: empty, or holding `count` of
/// the same item with `max` per stack, or something else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Out {
    Empty,
    Same { count: u32, max: u32 },
    Other,
}

/// Where `count` items go among `outs` (`insertInOutput`: slot by slot,
/// onto the same item first in slot order, then into empty slots as they
/// come): (slot, amount) pairs, or None when they don't all fit.
pub fn insert_plan(outs: &[Out], count: u32, max: u32) -> Option<Vec<(usize, u32)>> {
    let mut left = count;
    let mut plan = Vec::new();
    for (i, o) in outs.iter().enumerate() {
        if left == 0 {
            break;
        }
        let room = match *o {
            Out::Empty => max,
            Out::Same { count, max } => max.saturating_sub(count),
            Out::Other => 0,
        };
        let n = room.min(left);
        if n > 0 {
            plan.push((i, n));
            left -= n;
        }
    }
    (left == 0).then_some(plan)
}

/// The energy cubes of the user's Mekanism for Horizon, with the joules each
/// gives out of its front per tick (as Powah's port reads them). Mekanism
/// counts 2.5 J to 1 FE.
pub const MEKANISM_CUBES: [(&str, i64); 4] = [
    ("mekanism:basic_energy_cube", 4_000),
    ("mekanism:advanced_energy_cube", 16_000),
    ("mekanism:elite_energy_cube", 64_000),
    ("mekanism:ultimate_energy_cube", 256_000),
];

pub fn mekanism_cube_output(name: &str) -> Option<i64> {
    MEKANISM_CUBES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, j)| *j)
}

/// FE drawn from a Mekanism cube holding `joules` and giving `output` J/t,
/// for a block with `room` FE free: (FE gained, J taken). 2 FE for every 5 J.
pub fn mekanism_draw(joules: i64, output: i64, room: i64) -> (i64, i64) {
    let fe = (joules.min(output).max(0) * 2 / 5).min(room.max(0));
    (fe, fe * 5 / 2 + (fe * 5) % 2)
}

/// Powah's energy cells (its port keeps `energy` in FE), with the FE each
/// gives per tick.
pub const POWAH_CELLS: [(&str, i64); 2] = [
    ("powah:energy_cell_starter", 1_000),
    ("powah:energy_cell_basic", 4_000),
];

pub fn powah_cell_output(name: &str) -> Option<i64> {
    POWAH_CELLS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, fe)| *fe)
}

pub const DIRS: [[i32; 3]; 6] = [
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, 0, 0],
    [1, 0, 0],
];

pub fn offset(p: [i32; 3], d: [i32; 3]) -> [i32; 3] {
    [p[0] + d[0], p[1] + d[1], p[2] + d[2]]
}

/// The offset of a furnace-style `facing`.
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

/// A number split in two 15-bit data slots (the menu sends 16-bit values).
pub fn split15(v: i64) -> (i32, i32) {
    let v = v.clamp(0, (1 << 30) - 1);
    ((v & 0x7FFF) as i32, (v >> 15) as i32)
}

pub fn join15(lo: i32, hi: i32) -> i64 {
    i64::from(lo.clamp(0, 0x7FFF)) | (i64::from(hi.clamp(0, 0x7FFF)) << 15)
}

/// FE with dots between thousands (`2.000.000`).
pub fn fe(v: i64) -> String {
    let s = v.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push('.');
        }
        out.push(c);
    }
    if v < 0 { format!("-{out}") } else { out }
}

/// Mob names in Portuguese for the screens and messages.
pub fn mob_pt(name: &str) -> &'static str {
    match name {
        "zombie" => "Zumbi",
        "skeleton" => "Esqueleto",
        "creeper" => "Creeper",
        "spider" => "Aranha",
        "enderman" => "Enderman",
        "blaze" => "Blaze",
        "slime" => "Slime",
        "wither_skeleton" => "Esqueleto Wither",
        "witch" => "Bruxa",
        "ghast" => "Ghast",
        "magma_cube" => "Cubo de Magma",
        "guardian" => "Guardião",
        "drowned" => "Afogado",
        "zombified_piglin" => "Piglin Zumbificado",
        "shulker" => "Shulker",
        "phantom" => "Phantom",
        "evoker" => "Invocador",
        "vindicator" => "Vingador",
        "hoglin" => "Hoglin",
        "breeze" => "Breeze",
        "elder_guardian" => "Guardião Ancião",
        "wither" => "Wither",
        "ender_dragon" => "Dragão do Ender",
        "warden" => "Warden",
        _ => "?",
    }
}

/// A one-line summary of a Data Model: tier, data, accuracy, kills left.
pub fn model_line(model: usize, data: i32) -> String {
    let t = tier_of(data);
    let tier = TIERS[t];
    let acc = accuracy(data) * 100.0;
    match kills_to_next(model, data) {
        Some(k) => format!(
            "{} {}: {}/{} dados, precisão {:.1}%, faltam {} abates",
            tier.label_pt,
            mob_pt(MODELS[model].name),
            data,
            TIERS[t + 1].required,
            acc,
            k
        ),
        None => format!(
            "{} {}: {} dados, precisão {:.1}%",
            tier.label_pt,
            mob_pt(MODELS[model].name),
            data,
            acc
        ),
    }
}

static SEED: AtomicU64 = AtomicU64::new(0);

/// A number in 0..1: splitmix64 over a seed taken from the clock the first time.
pub fn roll() -> f32 {
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
    ((z >> 40) as f32) / ((1u64 << 24) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idx(n: &str) -> usize {
        model_index(n).unwrap()
    }

    #[test]
    fn kinds_and_variants() {
        assert_eq!(model_for_kind("minecraft:zombie"), Some(idx("zombie")));
        assert_eq!(model_for_kind("minecraft:husk"), Some(idx("zombie")));
        assert_eq!(model_for_kind("stray"), Some(idx("skeleton")));
        assert_eq!(model_for_kind("minecraft:cave_spider"), Some(idx("spider")));
        assert_eq!(model_for_kind("minecraft:cow"), None);
        assert_eq!(MODELS.len(), 24);
    }

    #[test]
    fn item_names_round_trip() {
        for i in 0..MODELS.len() {
            assert_eq!(model_of_data_item(&data_model_item(i)), Some(i));
            assert_eq!(model_of_prediction(&prediction_item(i)), Some(i));
        }
        assert_eq!(model_of_data_item(BLANK_DATA_MODEL), None);
        assert_eq!(
            model_of_prediction("hostilenetworks:overworld_prediction"),
            None
        );
    }

    #[test]
    fn tiers_by_data() {
        assert_eq!(tier_of(0), 0);
        assert_eq!(tier_of(5), 0);
        assert_eq!(tier_of(6), 1);
        assert_eq!(tier_of(54), 2);
        assert_eq!(tier_of(353), 2);
        assert_eq!(tier_of(354), 3);
        assert_eq!(tier_of(1254), 4);
        assert_eq!(tier_of(99_999), 4);
    }

    #[test]
    fn kills_per_tier_match_the_original() {
        // Faulty -> Basic: 6 kills of 1; Basic -> Advanced: 12 of 4;
        // Advanced -> Superior: 30 of 10; Superior -> Self Aware: 50 of 18.
        let z = idx("zombie");
        let mut data = 0;
        let mut kills = [0; 4];
        while tier_of(data) < 4 {
            kills[tier_of(data)] += 1;
            data = data_after_kill(z, data);
        }
        assert_eq!(kills, [6, 12, 30, 50]);
        assert_eq!(data_after_kill(z, 1254), 1254);
        assert_eq!(kills_to_next(z, 0), Some(6));
        assert_eq!(kills_to_next(z, 6), Some(12));
        assert_eq!(kills_to_next(z, 1254), None);
    }

    #[test]
    fn bosses_use_their_own_gain() {
        let w = idx("elder_guardian");
        assert_eq!(gained(w, 0), 3);
        assert_eq!(data_after_kill(w, 6), 18);
        assert_eq!(gained(idx("ender_dragon"), 3), 45);
        assert_eq!(gained(idx("ender_dragon"), 4), 0);
    }

    #[test]
    fn accuracy_is_continuous() {
        assert_eq!(accuracy(0), 0.0);
        assert!((accuracy(6) - 0.05).abs() < 1e-6);
        assert!((accuracy(30) - (0.05 + 0.17 * 0.5)).abs() < 1e-6);
        assert!((accuracy(354) - 0.65).abs() < 1e-6);
        assert_eq!(accuracy(1254), 1.0);
        assert_eq!(roll_predictions(1.0, 0.99), 1);
        assert_eq!(roll_predictions(0.25, 0.2), 1);
        assert_eq!(roll_predictions(0.25, 0.3), 0);
    }

    fn slots(model: usize, data: i32) -> SimSlots {
        SimSlots {
            model: Some((model, data)),
            matrix: true,
            base_room: true,
            pred_room: true,
        }
    }

    #[test]
    fn sim_run_costs_and_outputs() {
        let z = idx("zombie");
        let mut s = SimState {
            energy: SIM_POWER_CAP,
            ..Default::default()
        };
        let start = sim_tick(&mut s, &slots(z, 1254), || 0.5);
        assert!(start.consumed);
        assert_eq!(s.runtime, SIM_RUNTIME);
        assert_eq!(s.success, 1);
        let mut ticks = 1;
        loop {
            let d = sim_tick(&mut s, &slots(z, 1254), || 0.5);
            ticks += 1;
            if d.finished {
                assert_eq!(d.predictions, 1);
                break;
            }
        }
        assert_eq!(ticks, 241);
        assert_eq!(s.energy, SIM_POWER_CAP - 239 * 128);
        assert_eq!(s.fail, Fail::None as i32);
    }

    #[test]
    fn sim_refuses_faulty_missing_input_full_output_and_power() {
        let z = idx("zombie");
        let mut s = SimState {
            energy: SIM_POWER_CAP,
            ..Default::default()
        };
        assert!(!sim_tick(&mut s, &slots(z, 0), || 0.0).consumed);
        assert_eq!(Fail::from_i32(s.fail), Fail::Faulty);
        let mut no = slots(z, 6);
        no.matrix = false;
        sim_tick(&mut s, &no, || 0.0);
        assert_eq!(Fail::from_i32(s.fail), Fail::Input);
        let mut full = slots(z, 6);
        full.pred_room = false;
        sim_tick(&mut s, &full, || 0.0);
        assert_eq!(Fail::from_i32(s.fail), Fail::Output);
        s.energy = 127;
        sim_tick(&mut s, &slots(z, 6), || 0.0);
        assert_eq!(Fail::from_i32(s.fail), Fail::Energy);
        let none = SimSlots {
            model: None,
            matrix: true,
            base_room: true,
            pred_room: true,
        };
        sim_tick(&mut s, &none, || 0.0);
        assert_eq!(Fail::from_i32(s.fail), Fail::Model);
    }

    #[test]
    fn sim_pauses_without_power_mid_cycle_and_fails_prediction_roll() {
        let z = idx("zombie");
        let mut s = SimState {
            energy: 128 * 3,
            ..Default::default()
        };
        let d = sim_tick(&mut s, &slots(z, 6), || 0.9);
        assert!(d.consumed);
        assert_eq!(s.success, 0);
        for _ in 0..3 {
            sim_tick(&mut s, &slots(z, 6), || 0.9);
        }
        assert_eq!(s.energy, 0);
        sim_tick(&mut s, &slots(z, 6), || 0.9);
        assert_eq!(Fail::from_i32(s.fail), Fail::EnergyMidCycle);
        assert_eq!(s.runtime, SIM_RUNTIME - 3);
    }

    #[test]
    fn swapping_the_model_restarts() {
        let mut s = SimState {
            energy: SIM_POWER_CAP,
            ..Default::default()
        };
        sim_tick(&mut s, &slots(idx("zombie"), 6), || 0.0);
        sim_tick(&mut s, &slots(idx("zombie"), 6), || 0.0);
        assert_eq!(s.runtime, SIM_RUNTIME - 1);
        let d = sim_tick(&mut s, &slots(idx("blaze"), 6), || 0.0);
        assert!(d.consumed);
        assert_eq!(s.runtime, SIM_RUNTIME);
    }

    #[test]
    fn fabricator_takes_60_ticks_of_256() {
        let z = idx("zombie");
        let mut s = FabState {
            energy: FAB_POWER_CAP,
            ..Default::default()
        };
        assert_eq!(fab_tick(&mut s, Some(z), Some(0)), FabStep::Idle);
        for _ in 0..FAB_RUNTIME {
            assert_eq!(fab_tick(&mut s, Some(z), Some(0)), FabStep::Working);
        }
        assert_eq!(fab_tick(&mut s, Some(z), Some(0)), FabStep::Output(z, 0));
        assert_eq!(s.energy, FAB_POWER_CAP - 60 * 256);
        assert_eq!(fab_tick(&mut s, Some(z), None), FabStep::Idle);
        assert_eq!(fab_tick(&mut s, Some(z), Some(9)), FabStep::Idle);
        assert_eq!(s.runtime, 0);
        let mut low = FabState {
            energy: 255,
            current: 1,
            model: z as i32 + 1,
            runtime: 0,
        };
        assert_eq!(fab_tick(&mut low, Some(z), Some(0)), FabStep::Idle);
        assert_eq!(low.energy, 255);
    }

    #[test]
    fn output_plan() {
        let outs = [
            Out::Other,
            Out::Same { count: 60, max: 64 },
            Out::Empty,
            Out::Empty,
        ];
        assert_eq!(insert_plan(&outs, 64, 64), Some(vec![(1, 4), (2, 60)]));
        assert_eq!(
            insert_plan(&[Out::Other, Out::Same { count: 64, max: 64 }], 1, 64),
            None
        );
    }

    #[test]
    fn energy_helpers() {
        assert_eq!(mekanism_draw(10_000, 4_000, 1_000_000), (1_600, 4_000));
        assert_eq!(mekanism_draw(10_000, 4_000, 100), (100, 250));
        assert_eq!(
            join15(split15(1_999_999).0, split15(1_999_999).1),
            1_999_999
        );
        assert_eq!(fe(2_000_000), "2.000.000");
        assert_eq!(powah_cell_output("powah:energy_cell_basic"), Some(4_000));
        let r = roll();
        assert!((0.0..1.0).contains(&r));
    }
}

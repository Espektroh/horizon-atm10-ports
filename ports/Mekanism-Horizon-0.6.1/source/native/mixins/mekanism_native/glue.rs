// Mekanism for Horizon: the runtime (`mekanism_native`) over Horizon's item
// stacks, recipes, fuels and block entities, and the machines' screens.
// Added to the game as `crate::game::mekanism_glue`; the hook helpers call it.

use super::Game;
use super::mekanism_native as rt;
use super::native_hooks::NativeScreen;
use horizon_client::containers::{ModBlockEntity, ModContainer};
use horizon_client::hud::HudBatch;
use horizon_client::items::{ItemStack, Items};
use horizon_world::nbt::{Compound, JavaString, Tag};
use rt::Kind;

struct Stacks<'a> {
    items: &'a Items,
}

impl rt::Stacks<ItemStack> for Stacks<'_> {
    fn count(&self, s: &ItemStack) -> u32 {
        s.count
    }
    fn fits(&self, out: &ItemStack, result: &ItemStack) -> bool {
        let max = self.items.get(&result.item).map_or(64, |i| i.max_stack);
        out.same_item_same_components(result) && out.count + result.count <= max
    }
    fn grow(&self, out: &mut ItemStack, result: &ItemStack) {
        out.count += result.count;
    }
    fn take(&self, mut s: ItemStack, n: u32) -> Option<ItemStack> {
        s.count = s.count.saturating_sub(n);
        (s.count > 0).then_some(s)
    }
}

fn remainder(fuel: &ItemStack) -> Option<ItemStack> {
    match &*fuel.item {
        "minecraft:lava_bucket" | "minecraft:water_bucket" | "minecraft:milk_bucket" => {
            Some(ItemStack::new("minecraft:bucket", 1))
        }
        _ => None,
    }
}

fn get(nbt: &Compound, key: &str) -> i64 {
    match nbt.get(&JavaString::from(key)) {
        Some(Tag::Long(v)) => *v,
        Some(Tag::Int(v)) => i64::from(*v),
        Some(Tag::Short(v)) => i64::from(*v),
        Some(Tag::Byte(v)) => i64::from(*v),
        _ => 0,
    }
}

fn read(nbt: &Compound) -> rt::State {
    rt::State {
        energy: get(nbt, "energy"),
        progress: get(nbt, "progress") as i32,
        burn_remaining: get(nbt, "burn_remaining") as i32,
        burn_total: get(nbt, "burn_total") as i32,
        infuse_type: get(nbt, "infuse_type").clamp(0, 255) as u8,
        infuse_amount: get(nbt, "infuse_amount") as i32,
        speed_upgrades: get(nbt, "speed_upgrades").clamp(0, i64::from(rt::MAX_UPGRADES)) as u8,
        energy_upgrades: get(nbt, "energy_upgrades").clamp(0, i64::from(rt::MAX_UPGRADES)) as u8,
        chemical: get(nbt, "chemical").clamp(0, i64::from(rt::OSMIUM_CAPACITY)) as i32,
        chemical_used: get(nbt, "chemical_used").clamp(0, i64::from(i32::MAX)) as i32,
    }
}

fn write(nbt: &mut Compound, s: &rt::State) {
    nbt.insert("energy".into(), Tag::Long(s.energy));
    nbt.insert("progress".into(), Tag::Int(s.progress));
    nbt.insert("burn_remaining".into(), Tag::Int(s.burn_remaining));
    nbt.insert("burn_total".into(), Tag::Int(s.burn_total));
    nbt.insert("infuse_type".into(), Tag::Byte(s.infuse_type as i8));
    nbt.insert("infuse_amount".into(), Tag::Int(s.infuse_amount));
    nbt.insert("speed_upgrades".into(), Tag::Byte(s.speed_upgrades as i8));
    nbt.insert("energy_upgrades".into(), Tag::Byte(s.energy_upgrades as i8));
    // 0.6.0, the osmium compressor's tank: written only where there is one,
    // so the other blocks' saves stay as 0.5.0 wrote them.
    if s.chemical != 0 || s.chemical_used != 0 || nbt.get(&JavaString::from("chemical")).is_some() {
        nbt.insert("chemical".into(), Tag::Int(s.chemical));
        nbt.insert("chemical_used".into(), Tag::Int(s.chemical_used));
    }
}

/// Every Mekanism block but the cable gets a block entity of its own type.
pub(super) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none()
        && let Some(kind) = Kind::of(block)
    {
        *entity = Some((kind.entity().to_owned(), Compound::new()));
    }
}

pub(super) fn ticks(block: &str, ticks: &mut bool) {
    if Kind::of(block).is_some() {
        *ticks = true;
    }
}

/// Machines: input from the top and sides, output from below. The heat
/// generator: its fuel slot from every side. The cube: a menu for its
/// screen, no slots used. The solar generator has no menu.
pub(super) fn container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() {
        return;
    }
    let Some(kind) = Kind::of(block) else {
        return;
    };
    let menu = Some("minecraft:furnace".to_owned());
    *container = match kind {
        Kind::EnrichmentChamber | Kind::Crusher | Kind::EnergizedSmelter => Some(ModContainer {
            slots: 3,
            faces: [
                vec![rt::OUTPUT],
                vec![rt::INPUT],
                vec![rt::INPUT],
                vec![rt::INPUT],
                vec![rt::INPUT],
                vec![rt::INPUT],
            ],
            inputs: vec![rt::INPUT],
            outputs: vec![rt::OUTPUT],
            menu,
        }),
        Kind::MetallurgicInfuser | Kind::OsmiumCompressor => Some(ModContainer {
            slots: 3,
            // down: output, up: input, sides: what fills the infuse buffer
            // (the compressor's: osmium for its tank)
            faces: [
                vec![rt::OUTPUT],
                vec![rt::INPUT],
                vec![rt::FUEL],
                vec![rt::FUEL],
                vec![rt::FUEL],
                vec![rt::FUEL],
            ],
            inputs: vec![rt::INPUT, rt::FUEL],
            outputs: vec![rt::OUTPUT],
            menu,
        }),
        Kind::HeatGenerator => Some(ModContainer {
            slots: 3,
            faces: std::array::from_fn(|_| vec![rt::FUEL]),
            inputs: vec![rt::FUEL],
            outputs: Vec::new(),
            menu,
        }),
        Kind::EnergyCube | Kind::AdvancedEnergyCube | Kind::EliteEnergyCube | Kind::UltimateEnergyCube => {
            Some(ModContainer {
            slots: 3,
            faces: Default::default(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            menu,
        })
        }
        Kind::SolarGenerator => None,
    };
}

/// The recipe a machine runs for `input`: how many it takes, and what it makes.
fn machine_recipe(game: &Game<'_>, kind: Kind, input: Option<&ItemStack>) -> Option<(u32, ItemStack)> {
    let input = input?;
    let kind_type = kind.recipe_type()?;
    if kind_type == "minecraft:smelting" {
        let c = world_recipes!(game).cook(kind_type, input)?;
        return Some((1, c.result.clone()));
    }
    for r in game.native_recipes_of_type(kind_type) {
        let Some(ingredient) = game.native_recipe_ingredient(&r.json["input"]) else {
            continue;
        };
        if !ingredient.test(input) {
            continue;
        }
        let Some(result) = game.native_recipe_result(&r.json["output"]) else {
            continue;
        };
        let count = r.json["input_count"].as_u64().unwrap_or(1).clamp(1, 64) as u32;
        return Some((count, result));
    }
    None
}

/// The infusing recipe for `input`, preferring one of the type the buffer
/// holds: count, result, infuse type and amount.
fn infuser_recipe(game: &Game<'_>, input: Option<&ItemStack>, held: u8) -> Option<(u32, ItemStack, u8, i32)> {
    let input = input?;
    let mut first = None;
    for r in game.native_recipes_of_type("mekanism:metallurgic_infusing") {
        let Some(ingredient) = game.native_recipe_ingredient(&r.json["input"]) else {
            continue;
        };
        if !ingredient.test(input) {
            continue;
        }
        let Some(result) = game.native_recipe_result(&r.json["output"]) else {
            continue;
        };
        let count = r.json["input_count"].as_u64().unwrap_or(1).clamp(1, 64) as u32;
        let t = rt::infuse_code(r.json["infuse_type"].as_str().unwrap_or(""));
        let amount = r.json["infuse_amount"].as_i64().unwrap_or(10).clamp(1, i64::from(rt::INFUSE_CAPACITY)) as i32;
        let found = (count, result, t, amount);
        if t == held {
            return Some(found);
        }
        first.get_or_insert(found);
    }
    first
}

/// What an item gives the infuser's buffer: type and amount.
fn infuse_conversion(game: &Game<'_>, item: Option<&ItemStack>) -> Option<(u8, i32)> {
    let item = item?;
    for r in game.native_recipes_of_type("mekanism:infuse_conversion") {
        let Some(ingredient) = game.native_recipe_ingredient(&r.json["input"]) else {
            continue;
        };
        if ingredient.test(item) {
            let t = rt::infuse_code(r.json["infuse_type"].as_str().unwrap_or(""));
            let amount = r.json["amount"].as_i64().unwrap_or(0).clamp(0, i64::from(rt::INFUSE_CAPACITY)) as i32;
            return (t != 0 && amount > 0).then_some((t, amount));
        }
    }
    None
}

/// The compressing recipe for `input`: how many it takes, what it makes,
/// and the osmium it uses per tick of work (1 mB in Mekanism's).
fn compressor_recipe(game: &Game<'_>, input: Option<&ItemStack>) -> Option<(u32, ItemStack, i32)> {
    let input = input?;
    for r in game.native_recipes_of_type("mekanism:compressing") {
        if r.json["chemical"].as_str().unwrap_or(rt::OSMIUM) != rt::OSMIUM {
            continue;
        }
        let Some(ingredient) = game.native_recipe_ingredient(&r.json["input"]) else {
            continue;
        };
        if !ingredient.test(input) {
            continue;
        }
        let Some(result) = game.native_recipe_result(&r.json["output"]) else {
            continue;
        };
        let count = r.json["input_count"].as_u64().unwrap_or(1).clamp(1, 64) as u32;
        let per_tick = r.json["chemical_per_tick"].as_i64().unwrap_or(1).clamp(1, 10) as i32;
        return Some((count, result, per_tick));
    }
    None
}

/// What an item puts in the compressor's osmium tank, in mB.
fn osmium_conversion(game: &Game<'_>, item: Option<&ItemStack>) -> Option<i32> {
    let item = item?;
    for r in game.native_recipes_of_type("mekanism:chemical_conversion") {
        if r.json["chemical"].as_str() != Some(rt::OSMIUM) {
            continue;
        }
        let Some(ingredient) = game.native_recipe_ingredient(&r.json["input"]) else {
            continue;
        };
        if ingredient.test(item) {
            let amount = r.json["amount"].as_i64().unwrap_or(0).clamp(0, i64::from(rt::OSMIUM_CAPACITY)) as i32;
            return (amount > 0).then_some(amount);
        }
    }
    None
}

/// The sun on a solar generator: daytime in the overworld under open sky.
fn sunlit(game: &Game<'_>, pos: [i32; 3]) -> bool {
    if !matches!(game.dim, super::Dim::Overworld) || !rt::is_day(game.world.day_time) {
        return false;
    }
    (pos[1] + 1..pos[1] + 256).all(|y| {
        let state = game.world.block(pos[0], y, pos[2]);
        !game.res.facts().get(state).full_collision
    })
}

/// Whether the block at `p` takes energy coming in along `came` (the step
/// from the giver to it).
fn accepts(game: &Game<'_>, p: [i32; 3], came: [i32; 3]) -> bool {
    let Some(kind) = Kind::of(&game.native_block(p)) else {
        return false;
    };
    if !kind.accepts() {
        return false;
    }
    if kind.is_cube() {
        // Not through its own output face (the one it faces).
        let front = rt::facing_offset(&game.native_block_property(p, "facing").unwrap_or_default());
        let face = [-came[0], -came[1], -came[2]];
        return face != front;
    }
    true
}

/// Energy from the source at `pos` out to its acceptors, along cables.
fn give(game: &mut Game<'_>, pos: [i32; 3], kind: Kind, state: &mut rt::State) {
    let amount = state.energy.min(kind.output());
    if amount <= 0 {
        return;
    }
    let sides: Vec<[i32; 3]> = if kind.is_cube() {
        vec![rt::facing_offset(&game.native_block_property(pos, "facing").unwrap_or_default())]
    } else {
        rt::DIRS.to_vec()
    };
    let targets = {
        let g: &Game<'_> = game;
        rt::acceptors_rated(pos, &sides, &mut |p| g.native_block(p), &mut |p, came| accepts(g, p, came))
    };
    if targets.is_empty() {
        return;
    }
    let limits: Vec<i64> = targets.iter().map(|(_, l)| *l).collect();
    let targets: Vec<[i32; 3]> = targets.iter().map(|(p, _)| *p).collect();
    let mut rooms = Vec::with_capacity(targets.len());
    for &t in &targets {
        let kind_t = Kind::of(&game.native_block(t));
        let room = game
            .native_with_mod_block_entity(t, |m| read(&m.nbt))
            .zip(kind_t)
            .map_or(0, |(st, k)| {
                let free = (st.capacity(k) - st.energy).max(0);
                if k.is_cube() { free.min(k.output()) } else { free }
            });
        rooms.push(room);
    }
    let shares = rt::share_out(amount, &rooms, &limits);
    for (&t, &g) in targets.iter().zip(&shares) {
        if g <= 0 {
            continue;
        }
        let given = game.native_with_mod_block_entity(t, |m| {
            let mut s = read(&m.nbt);
            s.energy += g;
            write(&mut m.nbt, &s);
            g
        });
        state.energy -= given.unwrap_or(0);
    }
}

/// One server tick of a Mekanism block entity at `pos`.
pub(super) fn tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let Some(kind) = Kind::of(&entity.id) else {
        return;
    };
    if kind != Kind::SolarGenerator && entity.items.len() < 3 {
        entity.items.resize(3, None);
    }
    let mut state = read(&entity.nbt);
    let mut generating = 0;
    let active = match kind {
        Kind::EnrichmentChamber | Kind::Crusher | Kind::EnergizedSmelter => {
            let recipe = machine_recipe(game, kind, entity.items[rt::INPUT].as_ref());
            let stacks = Stacks { items: world_items!(game) };
            rt::machine_tick(kind, &mut state, &mut entity.items, recipe, &stacks)
        }
        Kind::MetallurgicInfuser => {
            let recipe = infuser_recipe(game, entity.items[rt::INPUT].as_ref(), state.infuse_type);
            let conversion = infuse_conversion(game, entity.items[rt::FUEL].as_ref());
            let stacks = Stacks { items: world_items!(game) };
            rt::infuser_tick(&mut state, &mut entity.items, recipe, conversion, &stacks)
        }
        Kind::OsmiumCompressor => {
            let recipe = compressor_recipe(game, entity.items[rt::INPUT].as_ref());
            let conversion = osmium_conversion(game, entity.items[rt::FUEL].as_ref());
            let stacks = Stacks { items: world_items!(game) };
            rt::compressor_tick(&mut state, &mut entity.items, recipe, conversion, &stacks)
        }
        Kind::HeatGenerator => {
            let lava = rt::DIRS
                .iter()
                .filter(|d| game.native_block(rt::offset(pos, **d)) == "minecraft:lava")
                .count() as u32;
            let items = world_items!(game);
            let providers = game.res.providers();
            let stacks = Stacks { items };
            let (burning, made) = rt::heat_tick(
                &mut state,
                &mut entity.items[rt::FUEL],
                lava,
                |f| providers.burn_time(items, f, false),
                remainder,
                &stacks,
            );
            generating = made;
            burning
        }
        Kind::SolarGenerator => {
            // The sky is checked once a second and kept.
            let lit = if game.world.game_time % 20 == 0 || entity.nbt.get(&JavaString::from("sunlit")).is_none() {
                let lit = sunlit(game, pos);
                entity.nbt.insert("sunlit".into(), Tag::Byte(i8::from(lit)));
                lit
            } else {
                get(&entity.nbt, "sunlit") != 0
            };
            generating = rt::solar_tick(&mut state, lit);
            generating > 0
        }
        Kind::EnergyCube | Kind::AdvancedEnergyCube | Kind::EliteEnergyCube | Kind::UltimateEnergyCube => {
            state.energy > 0
        }
    };
    if kind.is_source() {
        give(game, pos, kind, &mut state);
    }
    write(&mut entity.nbt, &state);
    entity.data = state.data(kind, generating).to_vec();
    if kind != Kind::SolarGenerator && !kind.is_cube() {
        game.native_set_block_property(pos, "lit", if active { "true" } else { "false" });
    }
}

/// The screens: the furnace menu's slots where a Mekanism machine has them.
pub(super) fn screen(block: &str, screen: &mut Option<NativeScreen>) {
    let Some(kind) = Kind::of(block) else {
        return;
    };
    let slots = match kind {
        Kind::EnrichmentChamber | Kind::Crusher | Kind::EnergizedSmelter => {
            vec![Some([56.0, 35.0]), None, Some([116.0, 35.0])]
        }
        Kind::MetallurgicInfuser | Kind::OsmiumCompressor => {
            vec![Some([56.0, 35.0]), Some([26.0, 35.0]), Some([116.0, 35.0])]
        }
        Kind::HeatGenerator => vec![None, Some([56.0, 35.0]), None],
        Kind::EnergyCube | Kind::AdvancedEnergyCube | Kind::EliteEnergyCube | Kind::UltimateEnergyCube => {
            vec![None, None, None]
        }
        Kind::SolarGenerator => return,
    };
    *screen = Some(NativeScreen {
        width: 176.0,
        height: 166.0,
        slots,
        inventory_y: 84.0,
        inventory_x: 8.0,
        own_labels: false,
        own_background: false,
    });
}

const DARK: u32 = 0x40_4040;
const BAR_BG: u32 = 0xFF_2B_2B_2B;
const ENERGY: u32 = 0xFF_3C_D2_3C;
const WORK: u32 = 0xFF_3C_96_E6;
const FIRE: u32 = 0xFF_F0_8C_28;
const CARBON: u32 = 0xFF_5A_5A_5A;
const REDSTONE: u32 = 0xFF_C8_1E_1E;
const DIAMOND: u32 = 0xFF_5A_DC_E6;
const REFINED_OBSIDIAN: u32 = 0xFF_7C_4C_A8;
const OSMIUM_COLOR: u32 = 0xFF_9C_B4_D2;

/// Draws a machine's screen: the energy bar on the right, progress or
/// burn, and the stored energy as text.
pub(super) fn screen_draw(game: &Game<'_>, block: &str, data: &[i32], batch: &mut HudBatch, origin: [f32; 2]) {
    let Some(kind) = Kind::of(block) else {
        return;
    };
    let d = |i: usize| data.get(i).copied().unwrap_or(0).max(0);
    let [ox, oy] = origin;
    let (speed, energy_up) = rt::unpack_upgrades(kind, data);
    let energy = i64::from(d(0)) * rt::energy_unit(kind, energy_up);
    let cap = rt::capacity(kind, energy_up);
    // Energy bar: x 160..168, y 14..74.
    let (bx, by, bw, bh) = (ox + 158.0, oy + 14.0, 10.0, 60.0);
    game.native_fill(batch, [bx, by, bw, bh], BAR_BG);
    let fill = (bh * (energy as f32 / cap as f32)).clamp(0.0, bh);
    game.native_fill(batch, [bx + 1.0, by + bh - fill, bw - 2.0, fill], ENERGY);
    let text = format!("{} / {}", rt::joules(energy), rt::joules(cap));
    game.native_text(batch, &text, [ox + 8.0, oy + 62.0], DARK, false);
    match kind {
        Kind::EnrichmentChamber | Kind::Crusher | Kind::EnergizedSmelter => {
            let (ax, ay, aw, ah) = (ox + 79.0, oy + 39.0, 24.0, 8.0);
            game.native_fill(batch, [ax, ay, aw, ah], BAR_BG);
            game.native_fill(batch, [ax, ay, aw * d(1) as f32 / 1000.0, ah], WORK);
            let line = format!("{} J/t", rt::machine_usage(kind, speed, energy_up));
            game.native_text(batch, &line, [ox + 8.0, oy + 18.0], DARK, false);
            upgrades_line(game, batch, origin, speed, energy_up);
        }
        Kind::MetallurgicInfuser => {
            let (ax, ay, aw, ah) = (ox + 79.0, oy + 39.0, 24.0, 8.0);
            game.native_fill(batch, [ax, ay, aw, ah], BAR_BG);
            game.native_fill(batch, [ax, ay, aw * d(1) as f32 / 1000.0, ah], WORK);
            // The infuse buffer: a bar left of the infuse slot, and its type and amount.
            let amount = d(2).min(rt::INFUSE_CAPACITY);
            let t = d(3).clamp(0, 255) as u8;
            let (ix, iy, iw, ih) = (ox + 8.0, oy + 14.0, 8.0, 44.0);
            game.native_fill(batch, [ix, iy, iw, ih], BAR_BG);
            let h = ih * amount as f32 / rt::INFUSE_CAPACITY as f32;
            let color = match t {
                2 => REDSTONE,
                3 => DIAMOND,
                4 => REFINED_OBSIDIAN,
                _ => CARBON,
            };
            game.native_fill(batch, [ix + 1.0, iy + ih - h, iw - 2.0, h], color);
            let line = format!("{}: {} / {}", rt::infuse_label(t), amount, rt::INFUSE_CAPACITY);
            game.native_text(batch, &line, [ox + 50.0, oy + 18.0], DARK, false);
            upgrades_line(game, batch, origin, speed, energy_up);
        }
        Kind::HeatGenerator => {
            let (fx, fy, fw, fh) = (ox + 80.0, oy + 37.0, 50.0, 12.0);
            game.native_fill(batch, [fx, fy, fw, fh], BAR_BG);
            game.native_fill(batch, [fx, fy, fw * d(2) as f32 / 1000.0, fh], FIRE);
            let line = format!("Gerando {} J/t", d(3));
            game.native_text(batch, &line, [ox + 8.0, oy + 18.0], DARK, false);
        }
        Kind::OsmiumCompressor => {
            let (ax, ay, aw, ah) = (ox + 79.0, oy + 39.0, 24.0, 8.0);
            game.native_fill(batch, [ax, ay, aw, ah], BAR_BG);
            game.native_fill(batch, [ax, ay, aw * d(1) as f32 / 1000.0, ah], WORK);
            // The osmium tank: a bar left of the osmium slot, and its amount.
            let amount = d(2).min(rt::OSMIUM_CAPACITY);
            let (ix, iy, iw, ih) = (ox + 8.0, oy + 14.0, 8.0, 44.0);
            game.native_fill(batch, [ix, iy, iw, ih], BAR_BG);
            let h = ih * amount as f32 / rt::OSMIUM_CAPACITY as f32;
            game.native_fill(batch, [ix + 1.0, iy + ih - h, iw - 2.0, h], OSMIUM_COLOR);
            let line = format!("Ósmio: {}/{} mB", amount, rt::OSMIUM_CAPACITY);
            game.native_text(batch, &line, [ox + 50.0, oy + 18.0], DARK, false);
            let usage = format!("{} J/t", rt::machine_usage(kind, speed, energy_up));
            game.native_text(batch, &usage, [ox + 50.0, oy + 28.0], DARK, false);
            upgrades_line(game, batch, origin, speed, energy_up);
        }
        Kind::EnergyCube | Kind::AdvancedEnergyCube | Kind::EliteEnergyCube | Kind::UltimateEnergyCube => {
            game.native_text(batch, "Saída pela frente, entrada pelos lados", [ox + 8.0, oy + 22.0], DARK, false);
            let line = format!("Até {}/t", rt::joules(kind.output()));
            game.native_text(batch, &line, [ox + 8.0, oy + 34.0], DARK, false);
        }
        Kind::SolarGenerator => {}
    }
}

/// "Vel. 3/8 · Ener. 8/8" above the energy text, when a machine has upgrades.
fn upgrades_line(game: &Game<'_>, batch: &mut HudBatch, origin: [f32; 2], speed: u8, energy: u8) {
    if speed == 0 && energy == 0 {
        return;
    }
    let line = format!("Vel. {speed}/{} · Ener. {energy}/{}", rt::MAX_UPGRADES, rt::MAX_UPGRADES);
    game.native_text(batch, &line, [origin[0] + 50.0, origin[1] + 52.0], DARK, false);
}

/// A right click on a machine holding a speed or energy upgrade puts one in
/// (as Mekanism's upgrade slot does), instead of opening its screen.
pub(super) fn right_click(game: &mut Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    let Some(kind) = Kind::of(&game.native_block(pos)) else {
        return;
    };
    let Some((item, _, plain)) = game.native_held() else {
        return;
    };
    if !kind.is_machine() || !plain || (item != rt::SPEED_UPGRADE && item != rt::ENERGY_UPGRADE) {
        return;
    }
    *canceled = true;
    let added = game
        .native_with_mod_block_entity(pos, |m| {
            let mut s = read(&m.nbt);
            let ok = s.upgrade(kind, &item);
            write(&mut m.nbt, &s);
            ok
        })
        .unwrap_or(false);
    if added {
        game.native_take_items(&item, 1);
    }
}

/// A machine broken gives its upgrades back to the player who broke it.
pub(super) fn destroyed(game: &mut Game<'_>, pos: [i32; 3]) {
    let Some(kind) = Kind::of(&game.native_block(pos)) else {
        return;
    };
    if !kind.is_machine() {
        return;
    }
    let Some(s) = game.native_with_mod_block_entity(pos, |m| read(&m.nbt)) else {
        return;
    };
    for (item, n) in [(rt::SPEED_UPGRADE, s.speed_upgrades), (rt::ENERGY_UPGRADE, s.energy_upgrades)] {
        if n > 0 {
            game.native_give_items(item, u32::from(n));
        }
    }
}

/// A neighbour of a cable changed (`updateShape`): the cable becomes the
/// variant with that side joined or cut. Read-only level, as vanilla's.
pub(super) fn cable_shape(
    level: &super::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    direction: horizon_core::geometry::Direction,
    neighbour: horizon_world::registry::BlockStateId,
    new: &mut Option<horizon_world::registry::BlockStateId>,
) {
    if new.is_some() {
        return;
    }
    // Only 0.4.0's variants: the pipe block (`rt::CABLE`, mask 0) joins by itself.
    let Some(mask) = rt::cable_mask(&level.facts.get(state).name).filter(|m| *m != 0) else {
        return;
    };
    let side = direction as usize;
    let next = rt::cable_update(mask, side, &level.facts.get(neighbour).name);
    if next != mask
        && let Some(id) = level.registry.default_state(&rt::cable_name(next))
    {
        *new = Some(id);
    }
}

/// 0.6.1: where a stack shift-clicked from the player's inventory goes in a
/// machine whose second slot is not fuel (`native_menu_quick_move`). The
/// furnace menu's own routing would put it in the first free slot, so the
/// osmium ingot could land in the compressor's input. Left unhandled (the
/// game's routing) for the other machines, for the machine's own slots and
/// when nothing fits.
pub(super) fn quick_move(
    game: &mut Game<'_>,
    entity: &mut ModBlockEntity,
    container_slots: usize,
    slot: usize,
    stack: &mut ItemStack,
    handled: &mut bool,
) {
    if *handled || slot < container_slots || stack.count == 0 {
        return;
    }
    let Some(kind) = Kind::of(&entity.id) else {
        return;
    };
    let target = match kind {
        Kind::OsmiumCompressor => {
            if osmium_conversion(game, Some(&*stack)).is_some() {
                rt::FUEL
            } else if compressor_recipe(game, Some(&*stack)).is_some() {
                rt::INPUT
            } else {
                return;
            }
        }
        Kind::MetallurgicInfuser => {
            if infuse_conversion(game, Some(&*stack)).is_some() {
                rt::FUEL
            } else {
                rt::INPUT
            }
        }
        _ => return,
    };
    if target >= container_slots {
        return;
    }
    if entity.items.len() < 3 {
        entity.items.resize(3, None);
    }
    let max = world_items!(game).get(&stack.item).map_or(64, |i| i.max_stack);
    let moved = match entity.items[target].as_mut() {
        None => stack.count.min(max),
        Some(have) if have.same_item_same_components(stack) => max.saturating_sub(have.count).min(stack.count),
        Some(_) => 0,
    };
    if moved == 0 {
        return;
    }
    match entity.items[target].as_mut() {
        Some(have) => have.count += moved,
        None => {
            let mut put = stack.clone();
            put.count = moved;
            entity.items[target] = Some(put);
        }
    }
    stack.count -= moved;
    *handled = true;
}

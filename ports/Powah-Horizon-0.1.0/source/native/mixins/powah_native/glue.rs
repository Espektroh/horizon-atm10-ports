// Powah for Horizon: the runtime (`powah_native`) over Horizon's item stacks,
// recipes, fuels and block entities, and the blocks' screens. Added to the
// game as `crate::game::powah_glue`; the hook helpers call it. Only uses what
// the user's mekanism_native 0.6.x glue already uses (it builds on 66f3981).

use super::Game;
use super::native_hooks::NativeScreen;
use super::powah_native as rt;
use horizon_client::containers::{ModBlockEntity, ModContainer};
use horizon_client::hud::HudBatch;
use horizon_client::items::{ItemStack, Items};
use horizon_world::nbt::{Compound, JavaString, Tag};
use rt::Kind;

/// The recipe type the orb runs (`data/powah/recipe/energizing/`).
const ENERGIZING: &str = "powah:energizing";
const LAVA_BUCKET: &str = "minecraft:lava_bucket";
const BUCKET: &str = "minecraft:bucket";
/// The orb's menu type (`data/powah/horizon/menu/energizing_orb.json`).
const ORB_MENU: &str = "powah:energizing_orb";

struct PowahStacks<'a> {
    items: &'a Items,
}

impl rt::Stacks<ItemStack> for PowahStacks<'_> {
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
        LAVA_BUCKET | "minecraft:water_bucket" | "minecraft:milk_bucket" => Some(ItemStack::new(BUCKET, 1)),
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
    let int = |k: &str| get(nbt, k).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
    rt::State {
        energy: get(nbt, "energy").max(0),
        carbon: get(nbt, "carbon").max(0),
        carbon_total: get(nbt, "carbon_total").max(0),
        tank: int("tank").clamp(0, rt::MAGMATOR_TANK),
        buffer: get(nbt, "buffer").max(0),
        buffer_total: get(nbt, "buffer_total").max(0),
        cooldown: int("cooldown").clamp(0, rt::ROD_WARMUP),
        linked: get(nbt, "linked") != 0,
        orb: [int("orb_x"), int("orb_y"), int("orb_z")],
        stored: get(nbt, "stored").max(0),
        required: get(nbt, "required").max(0),
        recipe: get(nbt, "recipe"),
        active: get(nbt, "active") != 0,
    }
}

fn write(nbt: &mut Compound, s: &rt::State) {
    nbt.insert("energy".into(), Tag::Long(s.energy));
    nbt.insert("carbon".into(), Tag::Long(s.carbon));
    nbt.insert("carbon_total".into(), Tag::Long(s.carbon_total));
    nbt.insert("tank".into(), Tag::Int(s.tank));
    nbt.insert("buffer".into(), Tag::Long(s.buffer));
    nbt.insert("buffer_total".into(), Tag::Long(s.buffer_total));
    nbt.insert("cooldown".into(), Tag::Int(s.cooldown));
    nbt.insert("linked".into(), Tag::Byte(i8::from(s.linked)));
    nbt.insert("orb_x".into(), Tag::Int(s.orb[0]));
    nbt.insert("orb_y".into(), Tag::Int(s.orb[1]));
    nbt.insert("orb_z".into(), Tag::Int(s.orb[2]));
    nbt.insert("stored".into(), Tag::Long(s.stored));
    nbt.insert("required".into(), Tag::Long(s.required));
    nbt.insert("recipe".into(), Tag::Long(s.recipe));
    nbt.insert("active".into(), Tag::Byte(i8::from(s.active)));
}

/// Every Powah block but the cables and storage blocks gets a block entity.
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

/// The orb: its own menu (`powah:energizing_orb`, 7 slots), ingredients in
/// from the top and sides, the result out from below. Furnator: fuel from
/// every side. Magmator: lava buckets in from the top and sides, empty
/// buckets out from below. Cell: a menu for its screen, no slots used.
/// Rods: none (the end rod template opens nothing).
pub(super) fn container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() {
        return;
    }
    let Some(kind) = Kind::of(block) else {
        return;
    };
    let furnace = Some("minecraft:furnace".to_owned());
    *container = match kind {
        Kind::Orb => {
            let inputs: Vec<usize> = (1..rt::ORB_SLOTS).collect();
            Some(ModContainer {
                slots: rt::ORB_SLOTS,
                faces: [
                    vec![rt::ORB_OUT],
                    inputs.clone(),
                    inputs.clone(),
                    inputs.clone(),
                    inputs.clone(),
                    inputs.clone(),
                ],
                inputs,
                outputs: vec![rt::ORB_OUT],
                menu: Some(ORB_MENU.to_owned()),
            })
        }
        Kind::Furnator(_) => Some(ModContainer {
            slots: 3,
            faces: std::array::from_fn(|_| vec![rt::FUEL]),
            inputs: vec![rt::FUEL],
            outputs: Vec::new(),
            menu: furnace,
        }),
        Kind::Magmator(_) => Some(ModContainer {
            slots: 3,
            faces: [
                vec![rt::OUTPUT],
                vec![rt::FUEL],
                vec![rt::FUEL],
                vec![rt::FUEL],
                vec![rt::FUEL],
                vec![rt::FUEL],
            ],
            inputs: vec![rt::FUEL],
            outputs: vec![rt::OUTPUT],
            menu: furnace,
        }),
        Kind::Cell(_) => Some(ModContainer {
            slots: 3,
            faces: Default::default(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            menu: furnace,
        }),
        Kind::Rod(_) => None,
    };
}

/// The energizing recipe the orb's ingredients (slots 1 to 6) match: its
/// id's hash, its FE and its result.
fn orb_recipe(game: &Game<'_>, slots: &[Option<ItemStack>]) -> Option<(i64, i64, ItemStack)> {
    let inputs: Vec<Option<&ItemStack>> = (1..rt::ORB_SLOTS)
        .map(|i| slots.get(i).and_then(|s| s.as_ref()))
        .collect();
    if inputs.iter().all(|s| s.is_none()) {
        return None;
    }
    let filled: Vec<bool> = inputs.iter().map(|s| s.is_some()).collect();
    for r in game.native_recipes_of_type(ENERGIZING) {
        let Some(list) = r.json["ingredients"].as_array() else {
            continue;
        };
        let ingredients: Vec<_> = list.iter().filter_map(|j| game.native_recipe_ingredient(j)).collect();
        if ingredients.is_empty() || ingredients.len() != list.len() {
            continue;
        }
        let fits = |i: usize, j: usize| inputs[i].is_some_and(|s| ingredients[j].test(s));
        if !rt::energizing_matches(&filled, ingredients.len(), fits) {
            continue;
        }
        let Some(result) = game.native_recipe_result(&r.json["result"]) else {
            continue;
        };
        let energy = r.json["energy"].as_i64().unwrap_or(1).max(1);
        return Some((rt::recipe_hash(&r.id), energy, result));
    }
    None
}

/// Whether the block at `p` takes energy coming in along `came` (the step
/// from the giver to it): a cell from any side (but not from another cell,
/// so two cells never pass energy back and forth), a rod from its base.
fn accepts(game: &Game<'_>, p: [i32; 3], came: [i32; 3], from_cell: bool) -> bool {
    match Kind::of(&game.native_block(p)) {
        Some(Kind::Cell(_)) => !from_cell,
        Some(Kind::Rod(_)) => {
            // The end rod's facing points away from the block it stands on.
            let facing = rt::facing_offset(&game.native_block_property(p, "facing").unwrap_or_default());
            came == facing
        }
        _ => false,
    }
}

/// Energy from the source at `pos` out to its acceptors, along cables.
fn give(game: &mut Game<'_>, pos: [i32; 3], kind: Kind, state: &mut rt::State) {
    let amount = state.energy.min(kind.transfer());
    if amount <= 0 {
        return;
    }
    let from_cell = matches!(kind, Kind::Cell(_));
    let targets = {
        let g: &Game<'_> = game;
        rt::acceptors_rated(pos, &mut |p| g.native_block(p), &mut |p, came| {
            accepts(g, p, came, from_cell)
        })
    };
    if targets.is_empty() {
        return;
    }
    let limits: Vec<i64> = targets.iter().map(|(_, l)| *l).collect();
    let mut rooms = Vec::with_capacity(targets.len());
    for (t, _) in &targets {
        let room = game
            .native_with_mod_block_entity(*t, |m| Kind::of(&m.id).map_or(0, |k| read(&m.nbt).room(k)))
            .unwrap_or(0);
        rooms.push(room);
    }
    let shares = rt::share_out(amount, &rooms, &limits);
    for ((t, _), &g) in targets.iter().zip(&shares) {
        if g <= 0 {
            continue;
        }
        let given = game.native_with_mod_block_entity(*t, |m| {
            let Some(k) = Kind::of(&m.id) else {
                return 0;
            };
            let mut s = read(&m.nbt);
            let g = g.min(s.room(k));
            s.energy += g;
            write(&mut m.nbt, &s);
            g
        });
        state.energy -= given.unwrap_or(0);
    }
}

/// A cell draws from a Mekanism energy cube whose front (its output face)
/// touches it, at most the cube's output per tick, 2 FE for every 5 J. Only
/// the cube's `energy` is read and changed, as Mekanism's own cables do.
fn draw_from_mekanism(game: &mut Game<'_>, pos: [i32; 3], kind: Kind, state: &mut rt::State) {
    for d in rt::DIRS {
        let room = state.room(kind);
        if room <= 0 {
            return;
        }
        let q = rt::offset(pos, d);
        let name = game.native_block(q);
        let Some(output) = rt::mekanism_cube_output(&name) else {
            continue;
        };
        let front = rt::facing_offset(&game.native_block_property(q, "facing").unwrap_or_default());
        if rt::offset(q, front) != pos {
            continue;
        }
        let gained = game
            .native_with_mod_block_entity(q, |m| {
                if m.id != name {
                    return 0;
                }
                let joules = get(&m.nbt, "energy");
                let (fe, taken) = rt::mekanism_draw(joules, output, room);
                if fe > 0 {
                    m.nbt.insert("energy".into(), Tag::Long(joules - taken));
                }
                fe
            })
            .unwrap_or(0);
        state.energy += gained.max(0);
    }
}

/// Where in a second of ticks a block looks for its orb (spread out).
fn stagger(pos: [i32; 3]) -> i64 {
    i64::from(
        pos[0]
            .wrapping_mul(31)
            .wrapping_add(pos[1].wrapping_mul(17))
            .wrapping_add(pos[2]),
    )
    .rem_euclid(20)
}

/// A rod: keeps the orb it filled while it is there and in range, looks
/// for the nearest one once a second when it has none, warms up and fills
/// it (`EnergizingRodTile.postTick`).
fn rod_tick(game: &mut Game<'_>, pos: [i32; 3], kind: Kind, state: &mut rt::State) {
    if state.linked && (!rt::in_range(pos, state.orb) || game.native_block(state.orb) != rt::ORB) {
        state.linked = false;
    }
    if !state.linked && game.world.game_time.rem_euclid(20) == stagger(pos) {
        let g: &Game<'_> = game;
        if let Some(orb) = rt::nearest_orb(pos, |p| g.native_block(p) == rt::ORB) {
            state.linked = true;
            state.orb = orb;
        }
    }
    if !state.linked {
        rt::rod_ready(kind, state, false);
        return;
    }
    let orb = state.orb;
    let active = game
        .native_with_mod_block_entity(orb, |m| read(&m.nbt).active)
        .unwrap_or(false);
    let give = rt::rod_ready(kind, state, active);
    if give > 0 {
        let taken = game
            .native_with_mod_block_entity(orb, |m| {
                let mut s = read(&m.nbt);
                let t = rt::orb_fill(&mut s, give);
                write(&mut m.nbt, &s);
                t
            })
            .unwrap_or(0);
        state.energy -= taken.clamp(0, state.energy);
    }
}

/// One server tick of a Powah block entity at `pos`.
pub(super) fn tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let Some(kind) = Kind::of(&entity.id) else {
        return;
    };
    let mut state = read(&entity.nbt);
    let mut made = 0;
    match kind {
        Kind::Orb => {
            if entity.items.len() < rt::ORB_SLOTS {
                entity.items.resize(rt::ORB_SLOTS, None);
            }
            let recipe = orb_recipe(game, &entity.items);
            let stacks = PowahStacks {
                items: world_items!(game),
            };
            rt::orb_tick(&mut state, &mut entity.items, recipe, &stacks);
            write(&mut entity.nbt, &state);
            entity.data = state.orb_data().to_vec();
            return;
        }
        Kind::Rod(_) => rod_tick(game, pos, kind, &mut state),
        Kind::Furnator(_) => {
            if entity.items.len() < 3 {
                entity.items.resize(3, None);
            }
            let items = world_items!(game);
            let providers = game.res.providers();
            let stacks = PowahStacks { items };
            made = rt::furnator_tick(
                kind,
                &mut state,
                &mut entity.items[rt::FUEL],
                |f| providers.burn_time(items, f, false),
                remainder,
                &stacks,
            );
        }
        Kind::Magmator(_) => {
            if entity.items.len() < 3 {
                entity.items.resize(3, None);
            }
            let stacks = PowahStacks {
                items: world_items!(game),
            };
            made = rt::magmator_tick(
                kind,
                &mut state,
                &mut entity.items,
                |s| &*s.item == LAVA_BUCKET,
                ItemStack::new(BUCKET, 1),
                &stacks,
            );
        }
        Kind::Cell(_) => draw_from_mekanism(game, pos, kind, &mut state),
    }
    if kind.is_source() {
        give(game, pos, kind, &mut state);
    }
    write(&mut entity.nbt, &state);
    entity.data = state.data(kind, made).to_vec();
    if kind.is_generator() {
        game.native_set_block_property(pos, "lit", if made > 0 { "true" } else { "false" });
    }
}

/// A right click on the orb (`EnergizingOrbBlock.useItemOn`): with a result
/// in it, the result goes to the player; else an item in hand goes in, one
/// at a time, to the first free ingredient slot. An empty hand opens its
/// screen. A lava bucket on a Magmator fills its tank and leaves the bucket.
pub(super) fn right_click(game: &mut Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if *canceled {
        return;
    }
    let Some(kind) = Kind::of(&game.native_block(pos)) else {
        return;
    };
    let Some((item, _, plain)) = game.native_held() else {
        return;
    };
    if !plain {
        return;
    }
    match kind {
        Kind::Orb => {
            *canceled = true;
            let out = game
                .native_with_mod_block_entity(pos, |m| m.items.get(rt::ORB_OUT).cloned().flatten())
                .flatten();
            if let Some(out) = out {
                let given = game.native_give_items(&out.item, out.count);
                if given > 0 {
                    game.native_with_mod_block_entity(pos, |m| {
                        if let Some(slot) = m.items.get_mut(rt::ORB_OUT) {
                            let left = slot.as_ref().map_or(0, |s| s.count.saturating_sub(given));
                            if left == 0 {
                                *slot = None;
                            } else if let Some(s) = slot.as_mut() {
                                s.count = left;
                            }
                        }
                    });
                }
                return;
            }
            let placed = game
                .native_with_mod_block_entity(pos, |m| {
                    if m.items.len() < rt::ORB_SLOTS {
                        m.items.resize(rt::ORB_SLOTS, None);
                    }
                    match (1..rt::ORB_SLOTS).find(|&i| m.items[i].is_none()) {
                        Some(i) => {
                            m.items[i] = Some(ItemStack::new(&item, 1));
                            true
                        }
                        None => false,
                    }
                })
                .unwrap_or(false);
            if placed {
                game.native_take_items(&item, 1);
            }
        }
        Kind::Magmator(_) if item == LAVA_BUCKET => {
            *canceled = true;
            let filled = game
                .native_with_mod_block_entity(pos, |m| {
                    let mut s = read(&m.nbt);
                    let ok = rt::magmator_fill(&mut s);
                    write(&mut m.nbt, &s);
                    ok
                })
                .unwrap_or(false);
            if filled && game.native_take_items(LAVA_BUCKET, 1) == 1 {
                game.native_give_items(BUCKET, 1);
            }
        }
        _ => {}
    }
}

/// The screens: the furnace menu's slots where a block has them, and the
/// orb's own menu.
pub(super) fn screen(block: &str, screen: &mut Option<NativeScreen>) {
    let Some(kind) = Kind::of(block) else {
        return;
    };
    let slots = match kind {
        Kind::Orb => vec![
            Some([124.0, 35.0]),
            Some([26.0, 26.0]),
            Some([44.0, 26.0]),
            Some([62.0, 26.0]),
            Some([26.0, 44.0]),
            Some([44.0, 44.0]),
            Some([62.0, 44.0]),
        ],
        Kind::Furnator(_) => vec![None, Some([56.0, 35.0]), None],
        Kind::Magmator(_) => vec![None, Some([56.0, 35.0]), Some([116.0, 35.0])],
        Kind::Cell(_) => vec![None, None, None],
        Kind::Rod(_) => return,
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
const ENERGY: u32 = 0xFF_E6_3C_3C;
const WORK: u32 = 0xFF_3C_96_E6;
const FIRE: u32 = 0xFF_F0_8C_28;
const LAVA: u32 = 0xFF_E0_5A_10;

/// Draws a block's screen: the energy bar on the right, the orb's progress,
/// the generators' fuel, and the numbers as text.
pub(super) fn screen_draw(game: &Game<'_>, block: &str, data: &[i32], batch: &mut HudBatch, origin: [f32; 2]) {
    let Some(kind) = Kind::of(block) else {
        return;
    };
    let d = |i: usize| data.get(i).copied().unwrap_or(0).max(0);
    let [ox, oy] = origin;
    if kind == Kind::Orb {
        let required = rt::join15(d(1), d(2));
        let stored = rt::join15(d(3), d(4));
        let (ax, ay, aw, ah) = (ox + 88.0, oy + 39.0, 28.0, 8.0);
        game.native_fill(batch, [ax, ay, aw, ah], BAR_BG);
        game.native_fill(batch, [ax, ay, aw * d(0).min(1000) as f32 / 1000.0, ah], WORK);
        let line = if required > 0 {
            format!("{} / {}", rt::fe(stored), rt::fe(required))
        } else {
            "Sem receita".to_owned()
        };
        game.native_text(batch, &line, [ox + 8.0, oy + 66.0], DARK, false);
        return;
    }
    let energy = i64::from(d(0)) * rt::energy_unit(kind);
    let cap = kind.capacity();
    let (bx, by, bw, bh) = (ox + 158.0, oy + 14.0, 10.0, 60.0);
    game.native_fill(batch, [bx, by, bw, bh], BAR_BG);
    let fill = (bh * (energy as f32 / cap.max(1) as f32)).clamp(0.0, bh);
    game.native_fill(batch, [bx + 1.0, by + bh - fill, bw - 2.0, fill], ENERGY);
    let text = format!("{} / {}", rt::fe(energy), rt::fe(cap));
    game.native_text(batch, &text, [ox + 8.0, oy + 62.0], DARK, false);
    match kind {
        Kind::Furnator(_) => {
            let (fx, fy, fw, fh) = (ox + 80.0, oy + 37.0, 50.0, 12.0);
            game.native_fill(batch, [fx, fy, fw, fh], BAR_BG);
            game.native_fill(batch, [fx, fy, fw * d(2).min(1000) as f32 / 1000.0, fh], FIRE);
            let line = format!("Gerando {} FE/t", d(3));
            game.native_text(batch, &line, [ox + 8.0, oy + 18.0], DARK, false);
        }
        Kind::Magmator(_) => {
            let tank = d(1).min(rt::MAGMATOR_TANK);
            let (ix, iy, iw, ih) = (ox + 8.0, oy + 14.0, 8.0, 44.0);
            game.native_fill(batch, [ix, iy, iw, ih], BAR_BG);
            let h = ih * tank as f32 / rt::MAGMATOR_TANK as f32;
            game.native_fill(batch, [ix + 1.0, iy + ih - h, iw - 2.0, h], LAVA);
            let line = format!("Lava: {}/{} mB", tank, rt::MAGMATOR_TANK);
            game.native_text(batch, &line, [ox + 80.0, oy + 18.0], DARK, false);
            let line = format!("Gerando {} FE/t", d(3));
            game.native_text(batch, &line, [ox + 80.0, oy + 28.0], DARK, false);
        }
        Kind::Cell(_) => {
            game.native_text(
                batch,
                "Entrada e saída por todos os lados",
                [ox + 8.0, oy + 22.0],
                DARK,
                false,
            );
            let line = format!("Até {}/t", rt::fe(kind.transfer()));
            game.native_text(batch, &line, [ox + 8.0, oy + 34.0], DARK, false);
        }
        Kind::Orb | Kind::Rod(_) => {}
    }
}

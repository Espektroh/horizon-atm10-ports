// Mekanism: MoreMachine for Horizon: the runtime (`mekmm_native`) over
// Horizon's item stacks, recipes and block entities, the energy drawn from
// the Mekanism port's cables, cubes and generators, and the screens.
// Added to the game as `crate::game::mekmm_glue`; the hook helpers call it.
// Every function returns at once for a block that isn't one of this mod's
// five machines.

use super::Game;
use super::mekmm_native as mm;
use super::native_hooks::NativeScreen;
use horizon_client::containers::{ModBlockEntity, ModContainer};
use horizon_client::hud::HudBatch;
use horizon_client::items::{ItemStack, Items};
use horizon_world::nbt::{Compound, JavaString, Tag};
use mm::MmKind;

/// The presser's own menu type (`data/mekmm/horizon/menu/presser.json`).
const MM_PRESSER_MENU: &str = "mekmm:presser";
/// The giver's saved keys this mod adds: the tick it was last drawn from and
/// how much was drawn then (so all these machines together take at most its
/// output per tick).
const MM_PULL_TICK: &str = "mekmm_pull_tick";
const MM_PULL_AMOUNT: &str = "mekmm_pull_amount";

struct MmItemStacks<'a> {
    items: &'a Items,
}

impl mm::MmStacks<ItemStack> for MmItemStacks<'_> {
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

fn mm_get(nbt: &Compound, key: &str) -> i64 {
    match nbt.get(&JavaString::from(key)) {
        Some(Tag::Long(v)) => *v,
        Some(Tag::Int(v)) => i64::from(*v),
        Some(Tag::Short(v)) => i64::from(*v),
        Some(Tag::Byte(v)) => i64::from(*v),
        _ => 0,
    }
}

fn mm_read(nbt: &Compound) -> mm::MmState {
    let up = |k: &str| mm_get(nbt, k).clamp(0, i64::from(mm::MM_MAX_UPGRADES)) as u8;
    mm::MmState {
        energy: mm_get(nbt, "energy").max(0),
        progress: mm_get(nbt, "progress").clamp(0, i64::from(i32::MAX)) as i32,
        speed_upgrades: up("speed_upgrades"),
        energy_upgrades: up("energy_upgrades"),
        chance: mm_get(nbt, "chance").clamp(0, 999) as i32,
    }
}

fn mm_write(nbt: &mut Compound, s: &mm::MmState) {
    nbt.insert("energy".into(), Tag::Long(s.energy));
    nbt.insert("progress".into(), Tag::Int(s.progress));
    nbt.insert("speed_upgrades".into(), Tag::Byte(s.speed_upgrades as i8));
    nbt.insert("energy_upgrades".into(), Tag::Byte(s.energy_upgrades as i8));
    nbt.insert("chance".into(), Tag::Int(s.chance));
}

/// Each machine gets a block entity of its own type.
pub(super) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none()
        && let Some(kind) = MmKind::of(block)
    {
        *entity = Some((kind.entity().to_owned(), Compound::new()));
    }
}

pub(super) fn ticks(block: &str, ticks: &mut bool) {
    if MmKind::of(block).is_some() {
        *ticks = true;
    }
}

/// Hoppers: the result out from below; the input from above (and the sides,
/// one-input machines); the stamper's mold from the sides; the presser's
/// second input from north and south, its third from west and east.
pub(super) fn container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() {
        return;
    }
    let Some(kind) = MmKind::of(block) else {
        return;
    };
    let furnace = Some("minecraft:furnace".to_owned());
    *container = Some(match kind {
        MmKind::Recycler | MmKind::Lathe | MmKind::RollingMill => ModContainer {
            slots: 3,
            faces: [vec![2], vec![0], vec![0], vec![0], vec![0], vec![0]],
            inputs: vec![0],
            outputs: vec![2],
            menu: furnace,
        },
        MmKind::Stamper => ModContainer {
            slots: 3,
            faces: [vec![2], vec![0], vec![1], vec![1], vec![1], vec![1]],
            inputs: vec![0, 1],
            outputs: vec![2],
            menu: furnace,
        },
        MmKind::Presser => ModContainer {
            slots: 4,
            faces: [vec![3], vec![0], vec![1], vec![1], vec![2], vec![2]],
            inputs: vec![0, 1, 2],
            outputs: vec![3],
            menu: Some(MM_PRESSER_MENU.to_owned()),
        },
    });
}

/// An ingredient field of a recipe, and its count field (1 when absent).
fn mm_part(
    game: &Game<'_>,
    json: &serde_json::Value,
    key: &str,
    count_key: &str,
    stack: Option<&ItemStack>,
) -> Option<u32> {
    let stack = stack?;
    let ingredient = game.native_recipe_ingredient(&json[key])?;
    if !ingredient.test(stack) {
        return None;
    }
    Some(json[count_key].as_u64().unwrap_or(1).clamp(1, 64) as u32)
}

/// The recipe of `kind` for what its input slots hold (`slots`).
fn mm_recipe(
    game: &Game<'_>,
    kind: MmKind,
    slots: &[Option<ItemStack>],
) -> Option<mm::MmRecipe<ItemStack>> {
    let at = |i: usize| slots.get(i).and_then(|s| s.as_ref());
    at(0)?;
    for r in game.native_recipes_of_type(kind.recipe_type()) {
        let json: &serde_json::Value = &r.json;
        let inputs = match kind {
            MmKind::Recycler | MmKind::Lathe | MmKind::RollingMill => {
                let Some(n) = mm_part(game, json, "input", "input_count", at(0)) else {
                    continue;
                };
                vec![(n, true)]
            }
            MmKind::Stamper => {
                let Some(n) = mm_part(game, json, "input", "input_count", at(0)) else {
                    continue;
                };
                let Some(m) = mm_part(game, json, "mold", "mold_count", at(1)) else {
                    continue;
                };
                vec![(n, true), (m, false)]
            }
            MmKind::Presser => {
                let Some(a) = mm_part(game, json, "primary_input", "primary_count", at(0)) else {
                    continue;
                };
                let Some(b) = mm_part(game, json, "secondary_input", "secondary_count", at(1))
                else {
                    continue;
                };
                let Some(c) = mm_part(game, json, "tertiary_input", "tertiary_count", at(2)) else {
                    continue;
                };
                vec![(a, true), (b, true), (c, true)]
            }
        };
        let Some(result) = game.native_recipe_result(&json["output"]) else {
            continue;
        };
        let chance = if kind == MmKind::Recycler {
            mm::mm_chance_permille(json["chance"].as_f64())
        } else {
            1000
        };
        return Some(mm::MmRecipe {
            inputs,
            result,
            chance,
        });
    }
    None
}

/// Whether some recipe of `kind` takes `stack` in the part `key` (for the
/// shift-click).
fn mm_takes(game: &Game<'_>, kind: MmKind, key: &str, stack: &ItemStack) -> bool {
    game.native_recipes_of_type(kind.recipe_type())
        .iter()
        .any(|r| {
            game.native_recipe_ingredient(&r.json[key])
                .is_some_and(|i| i.test(stack))
        })
}

/// Whether the block at `p` gives energy through its face `face`.
fn mm_gives(game: &Game<'_>, p: [i32; 3], face: [i32; 3]) -> bool {
    let Some(source) = mm::mm_source(&game.native_block(p)) else {
        return false;
    };
    let facing = game.native_block_property(p, "facing").unwrap_or_default();
    mm::mm_gives_through(source, &facing, face)
}

/// Draws energy into the machine at `pos` from the givers its cables reach.
fn mm_pull(game: &mut Game<'_>, pos: [i32; 3], state: &mut mm::MmState) {
    let room = state.room();
    if room <= 0 {
        return;
    }
    let givers = {
        let g: &Game<'_> = game;
        mm::mm_givers(pos, &mut |p| g.native_block(p), &mut |p, face| {
            mm_gives(g, p, face)
        })
    };
    if givers.is_empty() {
        return;
    }
    let now = game.world.game_time;
    let mut offers = Vec::with_capacity(givers.len());
    for (p, limit) in &givers {
        let rate = mm::mm_source(&game.native_block(*p)).map_or(0, |s| s.rate);
        let available = game
            .native_with_mod_block_entity(*p, |m| {
                let pulled = (mm_get(&m.nbt, MM_PULL_TICK), mm_get(&m.nbt, MM_PULL_AMOUNT));
                mm::mm_available(mm_get(&m.nbt, "energy"), rate, pulled, now)
            })
            .unwrap_or(0);
        offers.push((available, *limit));
    }
    let plan = mm::mm_pull_plan(room, &offers);
    for ((p, _), &want) in givers.iter().zip(&plan) {
        if want <= 0 {
            continue;
        }
        let got = game.native_with_mod_block_entity(*p, |m| {
            let energy = mm_get(&m.nbt, "energy").max(0);
            let take = want.min(energy);
            if take <= 0 {
                return 0;
            }
            let before = if mm_get(&m.nbt, MM_PULL_TICK) == now {
                mm_get(&m.nbt, MM_PULL_AMOUNT).max(0)
            } else {
                0
            };
            m.nbt.insert("energy".into(), Tag::Long(energy - take));
            m.nbt.insert(MM_PULL_TICK.into(), Tag::Long(now));
            m.nbt
                .insert(MM_PULL_AMOUNT.into(), Tag::Long(before + take));
            take
        });
        state.energy += got.unwrap_or(0);
    }
}

/// One server tick of a machine at `pos`.
pub(super) fn tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let Some(kind) = MmKind::of(&entity.id) else {
        return;
    };
    if entity.items.len() < kind.slots() {
        entity.items.resize(kind.slots(), None);
    }
    let mut state = mm_read(&entity.nbt);
    mm_pull(game, pos, &mut state);
    let recipe = mm_recipe(game, kind, &entity.items);
    let active = {
        let stacks = MmItemStacks {
            items: world_items!(game),
        };
        mm::mm_machine_tick(kind, &mut state, &mut entity.items, recipe, &stacks)
    };
    mm_write(&mut entity.nbt, &state);
    entity.data = state.data().to_vec();
    let lit = if active { "true" } else { "false" };
    if game.native_block_property(pos, "lit").as_deref() != Some(lit) {
        game.native_set_block_property(pos, "lit", lit);
    }
}

/// The screens: inputs in a column at x 64 (as the original), the result at
/// 116, 35.
pub(super) fn screen(block: &str, screen: &mut Option<NativeScreen>) {
    let Some(kind) = MmKind::of(block) else {
        return;
    };
    let slots = match kind {
        MmKind::Recycler | MmKind::Lathe | MmKind::RollingMill => {
            vec![Some([64.0, 35.0]), None, Some([116.0, 35.0])]
        }
        MmKind::Stamper => vec![Some([64.0, 17.0]), Some([64.0, 53.0]), Some([116.0, 35.0])],
        MmKind::Presser => vec![
            Some([64.0, 16.0]),
            Some([64.0, 35.0]),
            Some([64.0, 54.0]),
            Some([116.0, 35.0]),
        ],
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

const MM_DARK: u32 = 0x40_4040;
const MM_BAR_BG: u32 = 0xFF_2B_2B_2B;
const MM_ENERGY: u32 = 0xFF_3C_D2_3C;
const MM_WORK: u32 = 0xFF_3C_96_E6;

/// Draws a machine's screen: the energy bar on the right, the progress
/// between the inputs and the result, and the numbers on the left.
pub(super) fn screen_draw(
    game: &Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut HudBatch,
    origin: [f32; 2],
) {
    let Some(kind) = MmKind::of(block) else {
        return;
    };
    let d = |i: usize| data.get(i).copied().unwrap_or(0).max(0);
    let [ox, oy] = origin;
    let (speed, energy_up) = mm::mm_unpack_upgrades(data);
    let energy = i64::from(d(0)) * mm::mm_energy_unit(energy_up);
    let cap = mm::mm_capacity(energy_up);
    let (bx, by, bw, bh) = (ox + 158.0, oy + 14.0, 10.0, 60.0);
    game.native_fill(batch, [bx, by, bw, bh], MM_BAR_BG);
    let fill = (bh * (energy as f32 / cap as f32)).clamp(0.0, bh);
    game.native_fill(batch, [bx + 1.0, by + bh - fill, bw - 2.0, fill], MM_ENERGY);
    let (ax, ay, aw, ah) = (ox + 88.0, oy + 39.0, 22.0, 8.0);
    game.native_fill(batch, [ax, ay, aw, ah], MM_BAR_BG);
    game.native_fill(
        batch,
        [ax, ay, aw * d(1).min(1000) as f32 / 1000.0, ah],
        MM_WORK,
    );
    let x = ox + 8.0;
    let usage = format!("{} J/t", mm::mm_usage(speed, energy_up));
    game.native_text(batch, &usage, [x, oy + 18.0], MM_DARK, false);
    game.native_text(
        batch,
        &mm::mm_joules(energy),
        [x, oy + 30.0],
        MM_DARK,
        false,
    );
    let of = format!("/ {}", mm::mm_joules(cap));
    game.native_text(batch, &of, [x, oy + 40.0], MM_DARK, false);
    if speed > 0 || energy_up > 0 {
        let line = format!("V{speed} E{energy_up}");
        game.native_text(batch, &line, [x, oy + 52.0], MM_DARK, false);
    }
    if kind == MmKind::Recycler {
        let line = format!("Sorte {}/1000", d(3).min(999));
        game.native_text(batch, &line, [ox + 88.0, oy + 62.0], MM_DARK, false);
    }
}

/// A right click on a machine holding a speed or energy upgrade puts one in
/// (as the Mekanism port does), instead of opening its screen.
pub(super) fn right_click(game: &mut Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if *canceled || MmKind::of(&game.native_block(pos)).is_none() {
        return;
    }
    let Some((item, _, plain)) = game.native_held() else {
        return;
    };
    if !plain || (item != mm::MM_SPEED_UPGRADE && item != mm::MM_ENERGY_UPGRADE) {
        return;
    }
    *canceled = true;
    let added = game
        .native_with_mod_block_entity(pos, |m| {
            let mut s = mm_read(&m.nbt);
            let ok = s.upgrade(&item);
            mm_write(&mut m.nbt, &s);
            ok
        })
        .unwrap_or(false);
    if added {
        game.native_take_items(&item, 1);
    }
}

/// A machine broken gives its upgrades back to the player who broke it.
pub(super) fn destroyed(game: &mut Game<'_>, pos: [i32; 3]) {
    if MmKind::of(&game.native_block(pos)).is_none() {
        return;
    }
    let Some(s) = game.native_with_mod_block_entity(pos, |m| mm_read(&m.nbt)) else {
        return;
    };
    for (item, n) in [
        (mm::MM_SPEED_UPGRADE, s.speed_upgrades),
        (mm::MM_ENERGY_UPGRADE, s.energy_upgrades),
    ] {
        if n > 0 {
            game.native_give_items(item, u32::from(n));
        }
    }
}

/// Where a stack shift-clicked from the player's inventory goes in the
/// stamper (a mold to the mold slot) and the presser (each part to its
/// slot). Left to the game for the other machines, for the machine's own
/// slots and when no recipe takes it.
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
    let Some(kind) = MmKind::of(&entity.id) else {
        return;
    };
    let target = {
        let g: &Game<'_> = game;
        match kind {
            MmKind::Stamper => {
                if mm_takes(g, kind, "mold", stack) {
                    1
                } else if mm_takes(g, kind, "input", stack) {
                    0
                } else {
                    return;
                }
            }
            MmKind::Presser => {
                if mm_takes(g, kind, "primary_input", stack) {
                    0
                } else if mm_takes(g, kind, "secondary_input", stack) {
                    1
                } else if mm_takes(g, kind, "tertiary_input", stack) {
                    2
                } else {
                    return;
                }
            }
            _ => return,
        }
    };
    if target >= container_slots {
        return;
    }
    if entity.items.len() < kind.slots() {
        entity.items.resize(kind.slots(), None);
    }
    let max = world_items!(game)
        .get(&stack.item)
        .map_or(64, |i| i.max_stack);
    let moved = match entity.items[target].as_mut() {
        None => stack.count.min(max),
        Some(have) if have.same_item_same_components(stack) => {
            max.saturating_sub(have.count).min(stack.count)
        }
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

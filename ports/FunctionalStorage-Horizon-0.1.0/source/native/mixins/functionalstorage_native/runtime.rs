// Functional Storage for Horizon: the drawers' rules, free of Horizon types
// (std only), so `cargo test --offline` checks them outside the game.
//
// Functional Storage (c) Buuz135, Rid, MIT. Numbers and rules read from its
// source (github.com/Buuz135/FunctionalStorage, branch 1.21, 1.5.8):
// - FunctionalStorage.DrawerType: 1x1 = 32 stacks, 1x2 = 16, 2x2 = 8 per slot.
// - FunctionalStorageConfig: copper x8, gold x16, diamond x24, netherite x32;
//   StorageUpgradeItem: the iron "downgrade" sets the base to 1 stack.
// - SizeProvider.calculateAsFactor: base times the product of the factors.
// - BigInventoryHandler.getSlotLimit: floor(64 * multiplier * maxStack / 64).
// - ItemControllableDrawerTile: right click inserts the hand, a second right
//   click within 300 ms inserts every matching stack of the inventory; left
//   click takes 1 item, sneaking takes a whole stack; canChangeMultiplier.
// - ControllableDrawerTile.onSlotActivated: an upgrade in hand goes in the
//   first free upgrade slot, or replaces a weaker one when they are full.
// - DrawerBlock hit shapes: 1x2 slot 0 bottom, 1 top; 2x2 slot 0 bottom
//   right, 1 bottom left, 2 top right, 3 top left (seen from the front).

pub const ID: &str = "functionalstorage_native";
pub const NAME: &str = "Functional Storage (Horizon)";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str =
    "Gavetas 1x1, 1x2 e 2x2 de madeira, upgrades de armazenamento e de vazio, funis por todas as faces.";

pub const NS: &str = "functionalstorage";

/// The vanilla woods Functional Storage 1.5.8 has drawers of
/// (`DrawerWoodType`), in its order.
pub const WOODS: [&str; 10] = [
    "oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "crimson", "warped", "mangrove", "cherry",
];

/// Saved compound keys (format 1, see LEIA-ME "Formato dos dados").
pub const KEY_FORMAT: &str = "FsFormat";
pub const FORMAT: i32 = 1;
pub const KEY_AMOUNTS: &str = "Amounts";
pub const KEY_UPGRADES: &str = "Upgrades";
pub const KEY_VOID: &str = "Void";
/// A hash of what clients were last sent (a cache, not data).
pub const KEY_SHOWN: &str = "Shown";
/// The update tag's list of what each slot shows.
pub const KEY_VIEW: &str = "View";

/// Each drawer slot has three item slots in the block's container: the
/// stack hoppers take from, the slot hoppers fill, and one kept item that
/// remembers the stored item when more than a stack is stored.
pub const STRIDE: usize = 3;
pub const OUT: usize = 0;
pub const IN: usize = 1;
pub const KEEP: usize = 2;

/// Upgrade slots for storage upgrades (`DrawerTile.getStorageSlotAmount`).
pub const MAX_STORAGE_UPGRADES: usize = 4;
pub const VOID_UPGRADE: &str = "functionalstorage:void_upgrade";

/// Two right clicks closer than this (ticks) are a double click: 300 ms.
pub const DOUBLE_CLICK_TICKS: i64 = 6;
/// A held left click takes again after this many ticks
/// (`ItemControllableDrawerTile.removeTicks`).
pub const LEFT_CLICK_TICKS: i64 = 4;
/// `ItemControllableDrawerTile.onClicked`: a plain left click takes one
/// item, a sneaking one a whole stack.
pub const SNEAK_TAKES_STACK: bool = true;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    X1,
    X2,
    X4,
}

impl Size {
    pub fn slots(self) -> usize {
        match self {
            Size::X1 => 1,
            Size::X2 => 2,
            Size::X4 => 4,
        }
    }
    /// Stacks per slot without upgrades (`DrawerType.getSlotAmount`).
    pub fn base_stacks(self) -> u64 {
        match self {
            Size::X1 => 32,
            Size::X2 => 16,
            Size::X4 => 8,
        }
    }
    pub fn suffix(self) -> &'static str {
        match self {
            Size::X1 => "1",
            Size::X2 => "2",
            Size::X4 => "4",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Size::X1 => "1x1",
            Size::X2 => "1x2",
            Size::X4 => "2x2",
        }
    }
    pub fn container_slots(self) -> usize {
        self.slots() * STRIDE
    }
}

/// The drawer `block` (`functionalstorage:oak_2`) is, if it is one.
pub fn drawer_of(block: &str) -> Option<Size> {
    let rest = block.strip_prefix("functionalstorage:")?;
    let (wood, n) = rest.rsplit_once('_')?;
    if !WOODS.contains(&wood) {
        return None;
    }
    match n {
        "1" => Some(Size::X1),
        "2" => Some(Size::X2),
        "4" => Some(Size::X4),
        _ => None,
    }
}

/// Every drawer block id, wood by wood.
pub fn drawer_ids() -> Vec<String> {
    let mut out = Vec::new();
    for w in WOODS {
        for s in [Size::X1, Size::X2, Size::X4] {
            out.push(format!("{NS}:{w}_{}", s.suffix()));
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Upgrade {
    Iron,
    Copper,
    Gold,
    Diamond,
    Netherite,
}

impl Upgrade {
    pub const ALL: [Upgrade; 5] = [
        Upgrade::Iron,
        Upgrade::Copper,
        Upgrade::Gold,
        Upgrade::Diamond,
        Upgrade::Netherite,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Upgrade::Iron => "functionalstorage:iron_downgrade",
            Upgrade::Copper => "functionalstorage:copper_upgrade",
            Upgrade::Gold => "functionalstorage:gold_upgrade",
            Upgrade::Diamond => "functionalstorage:diamond_upgrade",
            Upgrade::Netherite => "functionalstorage:netherite_upgrade",
        }
    }
    /// `SizeProvider.ModifyFactor` (the iron one sets the base instead and
    /// keeps the factor: 1).
    pub fn factor(self) -> u64 {
        match self {
            Upgrade::Iron => 1,
            Upgrade::Copper => 8,
            Upgrade::Gold => 16,
            Upgrade::Diamond => 24,
            Upgrade::Netherite => 32,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Upgrade::Iron => "Ferro (1 pilha)",
            Upgrade::Copper => "Cobre x8",
            Upgrade::Gold => "Ouro x16",
            Upgrade::Diamond => "Diamante x24",
            Upgrade::Netherite => "Netherita x32",
        }
    }
}

pub fn upgrade_of(item: &str) -> Option<Upgrade> {
    Upgrade::ALL.into_iter().find(|u| u.id() == item)
}

/// The cap on a slot's count: Java's int.
pub const MAX_COUNT: u64 = i32::MAX as u64;

/// Stacks per slot: the base (1 with an iron downgrade) times every
/// upgrade's factor (`SizeProvider.calculateAsFactor`).
pub fn multiplier(size: Size, upgrades: &[Upgrade]) -> u64 {
    let base = if upgrades.contains(&Upgrade::Iron) {
        1
    } else {
        size.base_stacks()
    };
    upgrades
        .iter()
        .fold(base, |m, u| m.saturating_mul(u.factor()))
        .min(MAX_COUNT)
}

/// How many of an item stacking to `max_stack` a slot holds
/// (`BigInventoryHandler.getSlotLimit`).
pub fn capacity(multiplier: u64, max_stack: u32) -> u64 {
    multiplier.saturating_mul(u64::from(max_stack.max(1))).min(MAX_COUNT)
}

/// What a game stack is to these rules.
pub trait Stacks<S> {
    fn count(&self, s: &S) -> u32;
    /// Same item and components (`ItemStack.isSameItemSameComponents`).
    fn same(&self, a: &S, b: &S) -> bool;
    fn max_stack(&self, s: &S) -> u32;
    fn with_count(&self, s: &S, count: u32) -> S;
}

/// One drawer slot: the item it holds (its count means nothing) and how many.
#[derive(Clone, Debug, PartialEq)]
pub struct Slot<S> {
    pub proto: Option<S>,
    pub total: u64,
}

impl<S> Default for Slot<S> {
    fn default() -> Self {
        Slot { proto: None, total: 0 }
    }
}

/// A drawer's contents and upgrades, read from its block entity.
#[derive(Clone, Debug, PartialEq)]
pub struct Drawer<S> {
    pub size: Size,
    pub slots: Vec<Slot<S>>,
    pub upgrades: Vec<Upgrade>,
    pub void: bool,
    /// Stacks found in an input slot that are not the slot's item (none of
    /// these rules put one there): left where they are, for hoppers to take.
    pub strays: Vec<(usize, S)>,
}

fn count_of<S>(st: &impl Stacks<S>, s: &Option<S>) -> u64 {
    s.as_ref().map_or(0, |s| u64::from(st.count(s)))
}

impl<S: Clone> Drawer<S> {
    pub fn empty(size: Size) -> Self {
        Drawer {
            size,
            slots: (0..size.slots()).map(|_| Slot::default()).collect(),
            upgrades: Vec::new(),
            void: false,
            strays: Vec::new(),
        }
    }

    /// From the container's slots (`STRIDE` per drawer slot: out, in, kept)
    /// and the saved amounts beyond them. Missing slots or amounts read as
    /// empty, so an older or shorter save still loads.
    pub fn read(
        size: Size,
        items: &[Option<S>],
        amounts: &[i64],
        upgrades: &[Upgrade],
        void: bool,
        st: &impl Stacks<S>,
    ) -> Self {
        let mut d = Drawer::empty(size);
        d.upgrades = upgrades.iter().copied().take(MAX_STORAGE_UPGRADES).collect();
        d.void = void;
        let none: Option<S> = None;
        for i in 0..size.slots() {
            let at = |k: usize| items.get(i * STRIDE + k).unwrap_or(&none);
            let (out, inn, keep) = (at(OUT), at(IN), at(KEEP));
            let amount = amounts.get(i).copied().unwrap_or(0).max(0) as u64;
            let proto = [keep, out, inn]
                .into_iter()
                .flatten()
                .find(|s| st.count(s) > 0)
                .cloned();
            let mut total = amount + count_of(st, out) + count_of(st, keep);
            match (&proto, inn) {
                (Some(p), Some(s)) if st.count(s) > 0 => {
                    if st.same(p, s) {
                        total += u64::from(st.count(s));
                    } else {
                        d.strays.push((i, s.clone()));
                    }
                }
                _ => {}
            }
            // An amount saved with no item left to name it (a save edited by
            // hand): nothing to give back, so the slot is empty.
            if proto.is_none() {
                total = 0;
            }
            d.slots[i] = Slot { proto, total };
        }
        d
    }

    /// The container's slots and the saved amounts for this drawer: up to a
    /// stack in the out slot, one kept item when there is more, the rest
    /// counted in `amounts`. Strays stay in their input slot.
    pub fn layout(&self, st: &impl Stacks<S>) -> (Vec<Option<S>>, Vec<i64>) {
        let mut items: Vec<Option<S>> = vec![None; self.size.container_slots()];
        let mut amounts = vec![0i64; self.size.slots()];
        for (i, slot) in self.slots.iter().enumerate() {
            let Some(p) = slot.proto.as_ref().filter(|_| slot.total > 0) else {
                continue;
            };
            let max = u64::from(st.max_stack(p).max(1));
            let out = slot.total.min(max);
            let rest = slot.total - out;
            let keep = rest.min(1);
            items[i * STRIDE + OUT] = Some(st.with_count(p, out as u32));
            if keep > 0 {
                items[i * STRIDE + KEEP] = Some(st.with_count(p, 1));
            }
            amounts[i] = (rest - keep).min(i64::MAX as u64) as i64;
        }
        for (i, s) in &self.strays {
            if *i < self.slots.len() {
                items[i * STRIDE + IN] = Some(s.clone());
            }
        }
        (items, amounts)
    }

    pub fn multiplier(&self) -> u64 {
        multiplier(self.size, &self.upgrades)
    }

    /// Slot `i`'s capacity for `stack` (its own item when it has one).
    pub fn capacity_for(&self, i: usize, stack: &S, st: &impl Stacks<S>) -> u64 {
        let max = match self.slots.get(i).and_then(|s| s.proto.as_ref()) {
            Some(p) => st.max_stack(p),
            None => st.max_stack(stack),
        };
        capacity(self.multiplier(), max)
    }

    pub fn capacity(&self, i: usize, st: &impl Stacks<S>) -> u64 {
        match self.slots.get(i).and_then(|s| s.proto.as_ref()) {
            Some(p) => capacity(self.multiplier(), st.max_stack(p)),
            None => capacity(self.multiplier(), 64),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|s| s.total == 0) && self.strays.is_empty()
    }

    pub fn has_upgrades(&self) -> bool {
        !self.upgrades.is_empty() || self.void
    }

    /// Puts up to `count` of `stack` in slot `i` (`BigInventoryHandler.insertItem`):
    /// how many it takes from the giver. With a void upgrade it takes all
    /// of a matching item and the overflow is gone.
    pub fn insert(&mut self, i: usize, stack: &S, count: u64, st: &impl Stacks<S>) -> u64 {
        if count == 0 || i >= self.slots.len() {
            return 0;
        }
        let cap = self.capacity_for(i, stack, st);
        let void = self.void;
        let slot = &mut self.slots[i];
        match &slot.proto {
            Some(p) if !st.same(p, stack) => return 0,
            Some(_) => {}
            None => {
                if cap == 0 {
                    return 0;
                }
                slot.proto = Some(st.with_count(stack, 1));
                slot.total = 0;
            }
        }
        let room = cap.saturating_sub(slot.total);
        let n = count.min(room);
        slot.total += n;
        if slot.total == 0 {
            slot.proto = None;
        }
        if void { count } else { n }
    }

    /// Takes up to `count` from slot `i` (`extractItem`): the stack taken.
    pub fn extract(&mut self, i: usize, count: u64, st: &impl Stacks<S>) -> Option<(S, u64)> {
        let slot = self.slots.get_mut(i)?;
        let p = slot.proto.clone()?;
        let n = count.min(slot.total);
        if n == 0 {
            return None;
        }
        slot.total -= n;
        if slot.total == 0 {
            slot.proto = None;
        }
        Some((st.with_count(&p, 1), n))
    }

    /// Gives back `n` of `stack` that a player's full inventory refused,
    /// whatever the capacity: nothing taken out is lost. What could not go
    /// back (another item now in the slot: never, the same call took it).
    pub fn put_back(&mut self, i: usize, stack: &S, n: u64, st: &impl Stacks<S>) -> u64 {
        let Some(slot) = self.slots.get_mut(i) else {
            return n;
        };
        match &slot.proto {
            Some(p) if !st.same(p, stack) => return n,
            Some(_) => {}
            None => {
                slot.proto = Some(st.with_count(stack, 1));
                slot.total = 0;
            }
        }
        slot.total += n;
        0
    }

    /// Whether a hopper may put `stack` in slot `i`'s input: the slot holds
    /// that item and has room (or voids), or it is empty and no other slot
    /// holds the item.
    pub fn accepts(&self, i: usize, stack: &S, st: &impl Stacks<S>) -> bool {
        let Some(slot) = self.slots.get(i) else {
            return false;
        };
        if self.strays.iter().any(|(at, _)| *at == i) {
            return false;
        }
        match &slot.proto {
            Some(p) => st.same(p, stack) && (self.void || slot.total < self.capacity_for(i, stack, st)),
            None => {
                let elsewhere = self
                    .slots
                    .iter()
                    .any(|s| s.proto.as_ref().is_some_and(|p| st.same(p, stack)));
                !elsewhere && self.capacity_for(i, stack, st) > 0
            }
        }
    }

    /// The tick's upkeep: a void drawer drops what is over its capacity.
    /// Without one nothing is ever lost, even over capacity (an insert that
    /// came in whole through another mod). Returns how many were voided.
    pub fn settle(&mut self, st: &impl Stacks<S>) -> u64 {
        if !self.void {
            return 0;
        }
        let mut gone = 0;
        for i in 0..self.slots.len() {
            let cap = self.capacity(i, st);
            let slot = &mut self.slots[i];
            if slot.total > cap {
                gone += slot.total - cap;
                slot.total = cap;
            }
        }
        gone
    }

    /// Whether every slot fits in the capacity the drawer would have with
    /// `upgrades` (`canChangeMultiplier`).
    pub fn fits_with(&self, upgrades: &[Upgrade], st: &impl Stacks<S>) -> bool {
        let m = multiplier(self.size, upgrades);
        self.slots.iter().all(|s| match &s.proto {
            Some(p) => s.total <= capacity(m, st.max_stack(p)),
            None => true,
        })
    }

    /// An upgrade from the player's hand (`onSlotActivated`).
    pub fn install(&mut self, up: Upgrade, st: &impl Stacks<S>) -> Install {
        if self.upgrades.len() < MAX_STORAGE_UPGRADES {
            let mut next = self.upgrades.clone();
            next.push(up);
            if !self.fits_with(&next, st) {
                return Install::TooSmall;
            }
            self.upgrades = next;
            return Install::Added;
        }
        // Full: the first weaker one is swapped out and given back.
        for i in 0..self.upgrades.len() {
            if self.upgrades[i].factor() < up.factor() {
                let mut next = self.upgrades.clone();
                let old = next[i];
                next[i] = up;
                if !self.fits_with(&next, st) {
                    continue;
                }
                self.upgrades = next;
                return Install::Replaced(old);
            }
        }
        Install::Full
    }

    pub fn install_void(&mut self) -> bool {
        if self.void {
            return false;
        }
        self.void = true;
        true
    }

    /// Sneak + empty hand: takes out the last storage upgrade whose removal
    /// still fits the contents, then the void upgrade. The item id given back.
    pub fn remove_one(&mut self, st: &impl Stacks<S>) -> Result<&'static str, Removal> {
        if self.upgrades.is_empty() && !self.void {
            return Err(Removal::None);
        }
        for i in (0..self.upgrades.len()).rev() {
            let mut next = self.upgrades.clone();
            let up = next.remove(i);
            if self.fits_with(&next, st) {
                self.upgrades = next;
                return Ok(up.id());
            }
        }
        if self.void {
            self.void = false;
            return Ok(VOID_UPGRADE);
        }
        Err(Removal::TooFull)
    }

    /// A short hash of what clients show: items, counts, upgrades.
    pub fn shown_hash(&self, id_of: impl Fn(&S) -> String) -> i64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for b in bytes {
                h ^= u64::from(*b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        for s in &self.slots {
            if let Some(p) = &s.proto {
                eat(id_of(p).as_bytes());
            }
            eat(&s.total.to_le_bytes());
        }
        for u in &self.upgrades {
            eat(u.id().as_bytes());
        }
        eat(&[u8::from(self.void)]);
        h as i64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Install {
    Added,
    Replaced(Upgrade),
    /// Four upgrades, none weaker.
    Full,
    /// The contents would not fit (an iron downgrade on a full drawer).
    TooSmall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Removal {
    None,
    TooFull,
}

/// Which of a drawer's slots in the container slot `k` is, and its role.
pub fn container_slot(k: usize) -> (usize, usize) {
    (k / STRIDE, k % STRIDE)
}

/// The container's input slots (hoppers fill them) and output slots
/// (hoppers take from them: the out slots, and the inputs for strays).
pub fn container_inputs(size: Size) -> Vec<usize> {
    (0..size.slots()).map(|i| i * STRIDE + IN).collect()
}
pub fn container_outputs(size: Size) -> Vec<usize> {
    let mut v: Vec<usize> = (0..size.slots()).map(|i| i * STRIDE + OUT).collect();
    v.extend(container_inputs(size));
    v
}
/// What each face reaches: every out and in slot (Functional Storage gives
/// the same item handler on every side).
pub fn container_face(size: Size) -> Vec<usize> {
    let mut v = Vec::new();
    for i in 0..size.slots() {
        v.push(i * STRIDE + OUT);
        v.push(i * STRIDE + IN);
    }
    v
}

/// The step out of the block's front for its `facing` value.
pub fn facing_normal(facing: &str) -> Option<[i32; 3]> {
    match facing {
        "north" => Some([0, 0, -1]),
        "south" => Some([0, 0, 1]),
        "west" => Some([-1, 0, 0]),
        "east" => Some([1, 0, 0]),
        _ => None,
    }
}

/// Degrees about +y (right-handed) that turn the south-facing frame (front
/// at z = 1) to `facing`.
pub fn facing_yaw(facing: &str) -> f32 {
    match facing {
        "east" => 90.0,
        "north" => 180.0,
        "west" => 270.0,
        _ => 0.0,
    }
}

/// The view direction for Minecraft's yaw and pitch, in degrees.
pub fn look_dir(yaw: f32, pitch: f32) -> [f64; 3] {
    let (y, p) = (f64::from(yaw).to_radians(), f64::from(pitch).to_radians());
    [-y.sin() * p.cos(), -p.sin(), y.cos() * p.cos()]
}

pub const EYE_STANDING: f64 = 1.62;
pub const EYE_SNEAKING: f64 = 1.27;

/// Where the ray from `eye` along `dir` meets the front face of the block at
/// `pos`: (u, v), u from the viewer's left (0) to right (1), v from the
/// bottom (0) to the top (1). None when it never reaches it.
pub fn front_hit(facing: &str, pos: [i32; 3], eye: [f64; 3], dir: [f64; 3]) -> Option<(f64, f64)> {
    let n = facing_normal(facing)?;
    let axis = if n[0] != 0 { 0 } else { 2 };
    let plane = f64::from(pos[axis]) + if n[axis] > 0 { 1.0 } else { 0.0 };
    if dir[axis].abs() < 1e-9 {
        return None;
    }
    let t = (plane - eye[axis]) / dir[axis];
    if !t.is_finite() || t < 0.0 {
        return None;
    }
    let hit = [eye[0] + t * dir[0], eye[1] + t * dir[1], eye[2] + t * dir[2]];
    let local = [
        hit[0] - f64::from(pos[0]),
        hit[1] - f64::from(pos[1]),
        hit[2] - f64::from(pos[2]),
    ];
    let u = match facing {
        "north" => 1.0 - local[0],
        "south" => local[0],
        "west" => local[2],
        _ => 1.0 - local[2],
    };
    Some((u.clamp(0.0, 1.0), local[1].clamp(0.0, 1.0)))
}

/// The slot under (u, v) on a drawer's front.
pub fn slot_at(size: Size, u: f64, v: f64) -> usize {
    let top = v >= 0.5;
    let right = u >= 0.5;
    match size {
        Size::X1 => 0,
        Size::X2 => usize::from(top),
        Size::X4 => match (top, right) {
            (false, true) => 0,
            (false, false) => 1,
            (true, true) => 2,
            (true, false) => 3,
        },
    }
}

/// Where slot `i`'s item is drawn on the south-facing front (x from the
/// viewer's left... as the model's +x, y up) and its scale
/// (`DrawerRenderer.render1Slot/2Slot/4Slot`).
pub fn slot_center(size: Size, i: usize) -> ([f32; 2], f32) {
    match (size, i) {
        (Size::X1, _) => ([0.5, 0.5], 1.0),
        (Size::X2, 0) => ([0.5, 0.27], 0.5),
        (Size::X2, _) => ([0.5, 0.77], 0.5),
        (Size::X4, 0) => ([0.75, 0.27], 0.5),
        (Size::X4, 1) => ([0.25, 0.27], 0.5),
        (Size::X4, 2) => ([0.75, 0.77], 0.5),
        (Size::X4, _) => ([0.25, 0.77], 0.5),
    }
}

/// The drawer's front panel, in blocks from its back (the model's front
/// elements start half a pixel in).
pub const FRONT_Z: f32 = 1.0 - 0.5 / 16.0;

pub fn is_double_click(last: Option<i64>, now: i64) -> bool {
    matches!(last, Some(t) if now >= t && now - t < DOUBLE_CLICK_TICKS)
}

/// `NumberUtils.getFormatedBigNumber`: 999, 1.2K, 12.5K, 123K, 1.5M, 2.1B.
pub fn format_big(n: u64) -> String {
    fn tenths(n: u64, unit: u64) -> String {
        // DecimalFormat "####0.#" rounds half to even.
        let scaled = n * 10;
        let mut q = scaled / unit;
        let r = scaled % unit;
        if r * 2 > unit || (r * 2 == unit && !q.is_multiple_of(2)) {
            q += 1;
        }
        if q % 10 == 0 {
            format!("{}", q / 10)
        } else {
            format!("{}.{}", q / 10, q % 10)
        }
    }
    fn whole(n: u64, unit: u64) -> String {
        // Math.round: half up.
        format!("{}", (n + unit / 2) / unit)
    }
    if n >= 1_000_000_000 {
        format!("{}B", tenths(n, 1_000_000_000))
    } else if n >= 1_000_000 {
        if n > 100_000_000 {
            format!("{}M", whole(n, 1_000_000))
        } else {
            format!("{}M", tenths(n, 1_000_000))
        }
    } else if n >= 1000 {
        if n > 100_000 {
            format!("{}K", whole(n, 1000))
        } else {
            format!("{}K", tenths(n, 1000))
        }
    } else {
        n.to_string()
    }
}

/// The width of `text` in Minecraft's default font, in font pixels (the
/// characters a count uses: digits, '.', K, M, B).
pub fn text_width(text: &str) -> f32 {
    text.chars()
        .map(|c| match c {
            '.' | ',' | ':' | '!' | 'i' | '\'' => 2.0,
            'l' => 3.0,
            ' ' | 't' | 'I' => 4.0,
            'f' | 'k' => 5.0,
            _ => 6.0,
        })
        .sum::<f32>()
        - if text.is_empty() { 0.0 } else { 1.0 }
}

/// "1.2K / 2K" for the action bar.
pub fn amount_line(total: u64, cap: u64) -> String {
    format!("{} / {}", format_big(total), format_big(cap))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test stack: (item, count, max stack).
    #[derive(Clone, Debug, PartialEq)]
    struct T(&'static str, u32, u32);
    struct St;
    impl Stacks<T> for St {
        fn count(&self, s: &T) -> u32 {
            s.1
        }
        fn same(&self, a: &T, b: &T) -> bool {
            a.0 == b.0
        }
        fn max_stack(&self, s: &T) -> u32 {
            s.2
        }
        fn with_count(&self, s: &T, count: u32) -> T {
            T(s.0, count, s.2)
        }
    }
    const COBBLE: T = T("minecraft:cobblestone", 1, 64);
    const PEARL: T = T("minecraft:ender_pearl", 1, 16);
    const SWORD: T = T("minecraft:iron_sword", 1, 1);
    const DIRT: T = T("minecraft:dirt", 1, 64);

    fn total_in(items: &[Option<T>], amounts: &[i64]) -> u64 {
        items.iter().flatten().map(|s| u64::from(s.1)).sum::<u64>() + amounts.iter().map(|a| *a as u64).sum::<u64>()
    }

    #[test]
    fn ids_and_sizes() {
        assert_eq!(drawer_of("functionalstorage:oak_1"), Some(Size::X1));
        assert_eq!(drawer_of("functionalstorage:dark_oak_4"), Some(Size::X4));
        assert_eq!(drawer_of("functionalstorage:cherry_2"), Some(Size::X2));
        assert_eq!(drawer_of("functionalstorage:oak_3"), None);
        assert_eq!(drawer_of("functionalstorage:framed_1"), None);
        assert_eq!(drawer_of("minecraft:oak_planks"), None);
        assert_eq!(drawer_ids().len(), 30);
        assert!(drawer_ids().iter().all(|id| drawer_of(id).is_some()));
    }

    #[test]
    fn base_capacities_match_the_original() {
        // 1x1: 32 stacks, 1x2: 16, 2x2: 8 per slot.
        assert_eq!(capacity(multiplier(Size::X1, &[]), 64), 2048);
        assert_eq!(capacity(multiplier(Size::X2, &[]), 64), 1024);
        assert_eq!(capacity(multiplier(Size::X4, &[]), 64), 512);
        // 16-stacking and unstackable items: stacks, not items.
        assert_eq!(capacity(multiplier(Size::X1, &[]), 16), 512);
        assert_eq!(capacity(multiplier(Size::X1, &[]), 1), 32);
    }

    #[test]
    fn upgrades_multiply() {
        use Upgrade::*;
        assert_eq!(capacity(multiplier(Size::X1, &[Copper]), 64), 2048 * 8);
        assert_eq!(capacity(multiplier(Size::X1, &[Gold]), 64), 2048 * 16);
        assert_eq!(capacity(multiplier(Size::X1, &[Diamond]), 64), 2048 * 24);
        assert_eq!(capacity(multiplier(Size::X1, &[Netherite]), 64), 2048 * 32);
        // Factors multiply each other (calculateAsFactor).
        assert_eq!(multiplier(Size::X2, &[Copper, Gold]), 16 * 8 * 16);
        // Iron sets the base to one stack.
        assert_eq!(capacity(multiplier(Size::X1, &[Iron]), 64), 64);
        assert_eq!(capacity(multiplier(Size::X4, &[Iron, Copper]), 64), 512);
        // Four netherite on a 1x1 caps at Java's int.
        assert_eq!(capacity(multiplier(Size::X1, &[Netherite; 4]), 64), MAX_COUNT);
    }

    #[test]
    fn insert_and_extract_conserve_items() {
        let st = St;
        let mut d: Drawer<T> = Drawer::empty(Size::X1);
        assert_eq!(d.insert(0, &COBBLE, 64, &st), 64);
        assert_eq!(d.insert(0, &COBBLE, 3000, &st), 2048 - 64);
        assert_eq!(d.slots[0].total, 2048);
        assert_eq!(d.insert(0, &COBBLE, 1, &st), 0);
        // Another item doesn't go in a slot that holds one.
        assert_eq!(d.insert(0, &DIRT, 5, &st), 0);
        let (s, n) = d.extract(0, 64, &st).unwrap();
        assert_eq!((s.0, n), ("minecraft:cobblestone", 64));
        assert_eq!(d.slots[0].total, 2048 - 64);
        let (_, n) = d.extract(0, 5000, &st).unwrap();
        assert_eq!(n, 2048 - 64);
        // Empty again: it forgets the item (not locked).
        assert_eq!(d.slots[0], Slot::default());
        assert!(d.extract(0, 1, &st).is_none());
        assert_eq!(d.insert(0, &DIRT, 1, &st), 1);
    }

    #[test]
    fn layout_round_trips() {
        let st = St;
        for total in [0u64, 1, 63, 64, 65, 66, 1000, 1024] {
            let mut d: Drawer<T> = Drawer::empty(Size::X2);
            d.insert(1, &COBBLE, total, &st);
            d.insert(0, &PEARL, total.min(16 * 16), &st);
            let (items, amounts) = d.layout(&st);
            assert_eq!(items.len(), 6);
            assert_eq!(total_in(&items, &amounts), total + total.min(256));
            let back = Drawer::read(Size::X2, &items, &amounts, &[], false, &st);
            assert_eq!(back.slots[1].total, total);
            assert_eq!(back.slots[0].total, total.min(256));
            assert_eq!(back.slots[1].proto.is_some(), total > 0);
            // The out slot holds at most a stack; one is kept when there is more.
            if let Some(out) = &items[STRIDE + OUT] {
                assert!(out.1 <= 64);
            }
            assert_eq!(items[STRIDE + KEEP].is_some(), total > 64);
        }
    }

    #[test]
    fn a_refused_take_goes_back() {
        let st = St;
        let mut d: Drawer<T> = Drawer::empty(Size::X1);
        d.insert(0, &COBBLE, 2048, &st);
        let (s, n) = d.extract(0, 64, &st).unwrap();
        // The player had room for 10: 54 go back, even with the drawer full again.
        d.insert(0, &COBBLE, 64, &st);
        assert_eq!(d.put_back(0, &s, n - 10, &st), 0);
        assert_eq!(d.slots[0].total, 2048 + 54);
        let mut e: Drawer<T> = Drawer::empty(Size::X1);
        e.insert(0, &COBBLE, 1, &st);
        let (s, n) = e.extract(0, 1, &st).unwrap();
        assert_eq!(e.put_back(0, &s, n, &st), 0);
        assert_eq!(e.slots[0].total, 1);
        assert_eq!(e.put_back(0, &DIRT, 1, &st), 1);
    }

    /// Thousands of mixed operations (player inserts and takes, hoppers
    /// filling inputs and draining outputs, ticks through the saved layout,
    /// upgrades in and out): nothing appears or disappears without void.
    #[test]
    fn items_are_conserved_through_everything() {
        let st = St;
        let kinds = [COBBLE, PEARL, SWORD, DIRT];
        let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
        let mut next = |n: u64| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed % n
        };
        for size in [Size::X1, Size::X2, Size::X4] {
            let mut d: Drawer<T> = Drawer::empty(size);
            let mut expected: u64 = 0;
            for _ in 0..20_000 {
                let slot = next(size.slots() as u64) as usize;
                let kind = kinds[next(kinds.len() as u64) as usize].clone();
                match next(7) {
                    0 => {
                        let n = 1 + next(200);
                        expected += d.insert(slot, &kind, n, &st);
                    }
                    1 => {
                        let n = 1 + next(100);
                        if let Some((_, got)) = d.extract(slot, n, &st) {
                            expected -= got;
                        }
                    }
                    2 => {
                        // A hopper puts one item in an input, as the accept hook allows.
                        let (mut items, amounts) = d.layout(&st);
                        if d.accepts(slot, &kind, &st) {
                            let k = slot * STRIDE + IN;
                            match &mut items[k] {
                                Some(s) if s.0 == kind.0 && s.1 < s.2 => s.1 += 1,
                                Some(_) => continue,
                                None => items[k] = Some(T(kind.0, 1, kind.2)),
                            }
                            expected += 1;
                            d = Drawer::read(size, &items, &amounts, &d.upgrades.clone(), d.void, &st);
                        }
                    }
                    3 => {
                        // A hopper below takes one item from the first output with any.
                        let (mut it, am) = d.layout(&st);
                        for k in container_outputs(size) {
                            if let Some(s) = &mut it[k] {
                                s.1 -= 1;
                                if s.1 == 0 {
                                    it[k] = None;
                                }
                                expected -= 1;
                                break;
                            }
                        }
                        d = Drawer::read(size, &it, &am, &d.upgrades.clone(), d.void, &st);
                    }
                    4 => {
                        let up = Upgrade::ALL[next(5) as usize];
                        let _ = d.install(up, &st);
                    }
                    5 => {
                        let _ = d.remove_one(&st);
                    }
                    _ => {
                        // A tick: settle (no void here) and save.
                        assert_eq!(d.settle(&st), 0);
                        let (it, am) = d.layout(&st);
                        d = Drawer::read(size, &it, &am, &d.upgrades.clone(), d.void, &st);
                    }
                }
                let held: u64 = d.slots.iter().map(|s| s.total).sum();
                assert_eq!(held, expected, "{size:?}");
                let (it, am) = d.layout(&st);
                assert_eq!(total_in(&it, &am), expected);
            }
        }
    }

    #[test]
    fn hoppers_draining_the_out_slot_keep_the_item() {
        let st = St;
        let mut d: Drawer<T> = Drawer::empty(Size::X1);
        d.insert(0, &COBBLE, 200, &st);
        let (mut items, amounts) = d.layout(&st);
        // A pipe takes the whole out slot between two ticks.
        items[OUT] = None;
        let back = Drawer::read(Size::X1, &items, &amounts, &[], false, &st);
        assert_eq!(back.slots[0].total, 200 - 64);
        assert_eq!(back.slots[0].proto.as_ref().unwrap().0, "minecraft:cobblestone");
    }

    #[test]
    fn hopper_input_is_counted_and_absorbed() {
        let st = St;
        let mut d: Drawer<T> = Drawer::empty(Size::X1);
        d.insert(0, &COBBLE, 100, &st);
        let (mut items, amounts) = d.layout(&st);
        items[IN] = Some(T("minecraft:cobblestone", 5, 64));
        let back = Drawer::read(Size::X1, &items, &amounts, &[], false, &st);
        assert_eq!(back.slots[0].total, 105);
        let (items, amounts) = back.layout(&st);
        assert!(items[IN].is_none());
        assert_eq!(total_in(&items, &amounts), 105);
        // Into an empty drawer: the input names the item.
        let mut items = vec![None, Some(T("minecraft:dirt", 1, 64)), None];
        let back = Drawer::read(Size::X1, &items, &[0], &[], false, &st);
        assert_eq!(back.slots[0].total, 1);
        items = back.layout(&st).0;
        assert_eq!(items[OUT].as_ref().unwrap().0, "minecraft:dirt");
    }

    #[test]
    fn a_stray_stays_where_it_is() {
        let st = St;
        let items = vec![
            Some(T("minecraft:cobblestone", 10, 64)),
            Some(T("minecraft:dirt", 3, 64)),
            None,
        ];
        let d = Drawer::read(Size::X1, &items, &[0], &[], false, &st);
        assert_eq!(d.slots[0].total, 10);
        assert_eq!(d.strays.len(), 1);
        let (out, amounts) = d.layout(&st);
        assert_eq!(out[IN], Some(T("minecraft:dirt", 3, 64)));
        assert_eq!(total_in(&out, &amounts), 13);
        assert!(!d.accepts(0, &COBBLE, &st));
    }

    #[test]
    fn hopper_acceptance() {
        let st = St;
        let mut d: Drawer<T> = Drawer::empty(Size::X4);
        d.insert(1, &COBBLE, 512, &st);
        // Full slot: no.
        assert!(!d.accepts(1, &COBBLE, &st));
        // Not into an empty slot while another holds the item.
        assert!(!d.accepts(0, &COBBLE, &st));
        // Another item into an empty slot: yes; into slot 1: no.
        assert!(d.accepts(0, &DIRT, &st));
        assert!(!d.accepts(1, &DIRT, &st));
        d.extract(1, 1, &st);
        assert!(d.accepts(1, &COBBLE, &st));
        // Void: a full slot still takes its item, and the tick voids the excess.
        d.insert(1, &COBBLE, 1, &st);
        d.void = true;
        assert!(d.accepts(1, &COBBLE, &st));
        d.slots[1].total += 3;
        assert_eq!(d.settle(&st), 3);
        assert_eq!(d.slots[1].total, 512);
    }

    #[test]
    fn void_takes_everything_of_its_item_only() {
        let st = St;
        let mut d: Drawer<T> = Drawer::empty(Size::X1);
        assert!(d.install_void());
        assert!(!d.install_void());
        assert_eq!(d.insert(0, &COBBLE, 5000, &st), 5000);
        assert_eq!(d.slots[0].total, 2048);
        assert_eq!(d.insert(0, &DIRT, 10, &st), 0);
        // Without void nothing is lost over capacity.
        d.void = false;
        d.slots[0].total = 3000;
        assert_eq!(d.settle(&st), 0);
        assert_eq!(d.slots[0].total, 3000);
    }

    #[test]
    fn upgrade_install_and_removal() {
        use Upgrade::*;
        let st = St;
        let mut d: Drawer<T> = Drawer::empty(Size::X1);
        d.insert(0, &COBBLE, 100, &st);
        // An iron downgrade can't shrink a drawer below what it holds.
        assert_eq!(d.install(Iron, &st), Install::TooSmall);
        assert_eq!(d.install(Copper, &st), Install::Added);
        assert_eq!(d.capacity(0, &st), 2048 * 8);
        d.insert(0, &COBBLE, 5000, &st);
        // Removing copper would leave 5100 in a 2048 slot: refused.
        assert_eq!(d.remove_one(&st), Err(Removal::TooFull));
        for _ in 0..3 {
            assert_eq!(d.install(Copper, &st), Install::Added);
        }
        assert_eq!(d.install(Copper, &st), Install::Full);
        assert_eq!(d.install(Gold, &st), Install::Replaced(Copper));
        assert_eq!(d.upgrades, vec![Gold, Copper, Copper, Copper]);
        // Removal takes the last one that still fits.
        assert_eq!(d.remove_one(&st), Ok(Copper.id()));
        d.void = true;
        assert_eq!(d.remove_one(&st), Ok(Copper.id()));
        assert_eq!(d.remove_one(&st), Ok(Copper.id()));
        assert_eq!(d.remove_one(&st), Ok(VOID_UPGRADE));
        assert!(!d.void);
        assert_eq!(d.upgrades, vec![Gold]);
        let mut e: Drawer<T> = Drawer::empty(Size::X1);
        assert_eq!(e.remove_one(&st), Err(Removal::None));
        assert_eq!(e.install(Iron, &st), Install::Added);
        assert_eq!(e.capacity(0, &st), 64);
    }

    #[test]
    fn unstackables_count_in_stacks() {
        let st = St;
        let mut d: Drawer<T> = Drawer::empty(Size::X1);
        assert_eq!(d.insert(0, &SWORD, 40, &st), 32);
        let (items, amounts) = d.layout(&st);
        assert_eq!(items[OUT].as_ref().unwrap().1, 1);
        assert_eq!(items[KEEP].as_ref().unwrap().1, 1);
        assert_eq!(amounts[0], 30);
        assert_eq!(total_in(&items, &amounts), 32);
    }

    #[test]
    fn short_or_damaged_saves_load() {
        let st = St;
        let d = Drawer::read(Size::X4, &[], &[], &[], false, &st);
        assert!(d.is_empty());
        let d = Drawer::read(Size::X2, &[Some(T("minecraft:dirt", 2, 64))], &[-5, 7], &[], false, &st);
        assert_eq!(d.slots[0].total, 2);
        // An amount with no item to name it is dropped.
        assert_eq!(d.slots[1].total, 0);
    }

    #[test]
    fn front_hits_pick_the_slot() {
        // A drawer at the origin facing south; the player in front of it.
        let eye = [0.5, 1.62, 3.0];
        let toward = |x: f64, y: f64| {
            let d = [x - eye[0], y - eye[1], 1.0 - eye[2]];
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            [d[0] / l, d[1] / l, d[2] / l]
        };
        let (u, v) = front_hit("south", [0, 0, 0], eye, toward(0.8, 0.2)).unwrap();
        assert!((u - 0.8).abs() < 1e-9 && (v - 0.2).abs() < 1e-9);
        assert_eq!(slot_at(Size::X4, u, v), 0); // bottom right
        let (u, v) = front_hit("south", [0, 0, 0], eye, toward(0.2, 0.9)).unwrap();
        assert_eq!(slot_at(Size::X4, u, v), 3); // top left
        assert_eq!(slot_at(Size::X2, u, v), 1);
        assert_eq!(slot_at(Size::X1, u, v), 0);
        // Facing north, seen from the north: the viewer's right is west (low x).
        let eye_n = [0.5, 1.62, -3.0];
        let d: [f64; 3] = [0.1 - 0.5, 0.3 - 1.62, 3.0];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let (u, v) = front_hit("north", [0, 0, 0], eye_n, [d[0] / l, d[1] / l, d[2] / l]).unwrap();
        assert!(u > 0.5 && v < 0.5);
        assert_eq!(slot_at(Size::X4, u, v), 0);
        // Looking away: no hit.
        assert!(front_hit("south", [0, 0, 0], eye, [0.0, 0.0, 1.0]).is_none());
        // East and west.
        let (u, _) = front_hit("east", [0, 0, 0], [3.0, 0.5, 0.9], [-1.0, 0.0, 0.0]).unwrap();
        assert!((u - 0.1).abs() < 1e-9); // viewer looks west: south (+z) is their left
        let (u, _) = front_hit("west", [0, 0, 0], [-3.0, 0.5, 0.9], [1.0, 0.0, 0.0]).unwrap();
        assert!((u - 0.9).abs() < 1e-9);
    }

    #[test]
    fn look_direction_follows_minecraft() {
        let d = look_dir(0.0, 0.0);
        assert!((d[2] - 1.0).abs() < 1e-9); // yaw 0 looks south
        let d = look_dir(90.0, 0.0);
        assert!((d[0] + 1.0).abs() < 1e-9); // yaw 90 looks west
        let d = look_dir(0.0, 90.0);
        assert!((d[1] + 1.0).abs() < 1e-9); // pitch 90 looks down
    }

    #[test]
    fn double_clicks() {
        assert!(is_double_click(Some(100), 103));
        assert!(!is_double_click(Some(100), 106));
        assert!(!is_double_click(None, 3));
        assert!(!is_double_click(Some(100), 90));
    }

    #[test]
    fn big_numbers() {
        assert_eq!(format_big(999), "999");
        assert_eq!(format_big(1000), "1K");
        assert_eq!(format_big(1234), "1.2K");
        assert_eq!(format_big(1250), "1.2K");
        assert_eq!(format_big(1350), "1.4K");
        assert_eq!(format_big(2048), "2K");
        assert_eq!(format_big(16384), "16.4K");
        assert_eq!(format_big(131072), "131K");
        assert_eq!(format_big(1_500_000), "1.5M");
        assert_eq!(format_big(MAX_COUNT), "2.1B");
        assert_eq!(text_width("2K"), 11.0);
    }

    #[test]
    fn container_shape() {
        assert_eq!(container_inputs(Size::X2), vec![1, 4]);
        assert_eq!(container_outputs(Size::X2), vec![0, 3, 1, 4]);
        assert_eq!(container_face(Size::X1), vec![0, 1]);
        assert_eq!(container_slot(7), (2, 1));
        assert_eq!(Size::X4.container_slots(), 12);
    }

    #[test]
    fn shown_hash_changes_with_counts() {
        let st = St;
        let mut d: Drawer<T> = Drawer::empty(Size::X1);
        let id = |s: &T| s.0.to_string();
        let a = d.shown_hash(id);
        d.insert(0, &COBBLE, 1, &st);
        let b = d.shown_hash(id);
        d.insert(0, &COBBLE, 1, &st);
        assert!(a != b && b != d.shown_hash(id));
    }
}

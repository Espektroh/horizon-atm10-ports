// Pocket Storage for Horizon: the rules, with no Horizon types (only std).
// Added to the game as `crate::game::pocketstorage_rt`; tested on its own
// with `cargo test --offline` in `source/native/`.
//
// The numbers and rules come from Flanks255/PocketStorage 1.21.1 (MIT):
// `PSUTier` (slots, capacity), `PSUItemHandler.insertItemSlotless` /
// `extractItem` and `PocketStorageUnit.pickupEvent`.
//
// A unit (PSU) keeps one list of slots. Each slot holds one simple item (no
// components) and a count up to the tier's capacity. Here the list lives in
// the item itself, as a string in `minecraft:custom_data`:
// `minecraft:cobblestone*5000;;minecraft:dirt*3` (an empty slot is empty).

pub const ID: &str = "pocketstorage_native";
pub const NAME: &str = "Pocket Storage (Horizon)";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Unidades de armazenamento de bolso (PSU): clique direito abre; coleta automatica do que ja esta guardado. /psu buscar <texto> filtra a tela.";

/// The key in `minecraft:custom_data` holding the slots.
pub const DATA_KEY: &str = "pocketstorage_native:slots";
// The upgrade recipes' ingredient kind, `pocketstorage:empty_psu` (a PSU with
// nothing inside), is read by `ingredient.rs` / `ingredient_test.rs`, which
// live in the `crafting` module and spell it out.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tier {
    pub item: &'static str,
    pub name: &'static str,
    pub slots: usize,
    pub capacity: u32,
}

/// `PSUTier`: 8 x 0xFF, 16 x 0xFFF, 32 x 0xFFFF, 64 x 0xFFFFF.
pub const TIERS: [Tier; 4] = [
    Tier { item: "pocketstorage:psu_1", name: "PSU Tier 1", slots: 8, capacity: 0xFF },
    Tier { item: "pocketstorage:psu_2", name: "PSU Tier 2", slots: 16, capacity: 0xFFF },
    Tier { item: "pocketstorage:psu_3", name: "PSU Tier 3", slots: 32, capacity: 0xFFFF },
    Tier { item: "pocketstorage:psu_4", name: "PSU Tier 4", slots: 64, capacity: 0xFFFFF },
];

pub fn tier(item: &str) -> Option<&'static Tier> {
    TIERS.iter().find(|t| t.item == item)
}

pub fn is_psu(item: &str) -> bool {
    tier(item).is_some()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub item: String,
    pub count: u32,
}

pub type Slots = Vec<Option<Entry>>;

/// The slots a PSU of `tier` holds, read from its saved text (None: new).
/// Never drops a filled slot: a longer list keeps its extra filled slots.
pub fn decode(text: Option<&str>, tier: &Tier) -> Slots {
    let mut out: Slots = Vec::new();
    if let Some(text) = text
        && !text.is_empty()
    {
        for part in text.split(';') {
            out.push(parse_entry(part));
        }
    }
    if out.len() < tier.slots {
        out.resize(tier.slots, None);
    }
    while out.len() > tier.slots && out.last().is_some_and(Option::is_none) {
        out.pop();
    }
    out
}

fn parse_entry(part: &str) -> Option<Entry> {
    let (item, count) = part.rsplit_once('*')?;
    let count: u32 = count.trim().parse().ok()?;
    let item = item.trim();
    if count == 0 || item.is_empty() || !item.contains(':') {
        return None;
    }
    Some(Entry { item: item.to_owned(), count })
}

/// The saved text of `slots`; empty when nothing is inside.
pub fn encode(slots: &[Option<Entry>]) -> String {
    if slots.iter().all(Option::is_none) {
        return String::new();
    }
    let parts: Vec<String> = slots
        .iter()
        .map(|s| match s {
            Some(e) if e.count > 0 => format!("{}*{}", e.item, e.count),
            _ => String::new(),
        })
        .collect();
    let mut text = parts.join(";");
    while text.ends_with(';') {
        text.pop();
    }
    text
}

/// Whether some slot holds `item` (`PSUItemHandler.hasItem`).
pub fn has(slots: &[Option<Entry>], item: &str) -> bool {
    slots.iter().flatten().any(|e| e.item == item && e.count > 0)
}

/// Items a PSU never takes: other PSUs (`isItemValid`). Items with
/// components are refused by the glue, which can see them.
pub fn refused(item: &str) -> bool {
    is_psu(item) || item == "minecraft:air"
}

/// `insertItemSlotless`: `n` of `item` into the first slot holding it (up
/// to `capacity`), else (`allow_empty`) into the first empty slot. With
/// `allow_void` the part over the capacity is destroyed. The remainder.
pub fn insert(slots: &mut Slots, item: &str, n: u32, capacity: u32, allow_empty: bool, allow_void: bool) -> u32 {
    if n == 0 || refused(item) {
        return n;
    }
    if let Some(e) = slots.iter_mut().flatten().find(|e| e.item == item && e.count > 0) {
        let room = capacity.saturating_sub(e.count);
        let moved = n.min(room);
        e.count += moved;
        return if allow_void { 0 } else { n - moved };
    }
    if allow_empty && let Some(s) = slots.iter_mut().find(|s| s.as_ref().is_none_or(|e| e.count == 0)) {
        let moved = n.min(capacity);
        *s = Some(Entry { item: item.to_owned(), count: moved });
        return if allow_void { 0 } else { n - moved };
    }
    n
}

/// `extractItem`: up to `amount` (and up to one stack, `max_stack`) out of
/// slot `index`; the slot empties at zero. What came out.
pub fn extract(slots: &mut Slots, index: usize, amount: u32, max_stack: u32) -> Option<Entry> {
    let slot = slots.get_mut(index)?;
    let e = slot.as_mut()?;
    let n = amount.min(max_stack.max(1)).min(e.count);
    if n == 0 {
        return None;
    }
    let out = Entry { item: e.item.clone(), count: n };
    e.count -= n;
    if e.count == 0 {
        *slot = None;
    }
    Some(out)
}

/// `pickupEvent`: a picked-up stack of `item` that some slot already holds
/// goes in whole; the part over the capacity is destroyed ("voids excess").
/// Whether this PSU took it.
pub fn absorb(slots: &mut Slots, item: &str, n: u32, capacity: u32) -> bool {
    if n == 0 || refused(item) || !has(slots, item) {
        return false;
    }
    insert(slots, item, n, capacity, false, true);
    true
}

// ------------------------------------------------------------------ inventory

/// One stack of the player's inventory, as the rules see it: `simple` when
/// it has no components (the only kind a PSU takes or merges with).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvStack {
    pub item: String,
    pub count: u32,
    pub simple: bool,
}

/// Puts `n` of the simple `item` into `inv` as `Inventory.add` does (onto
/// simple stacks of it first, then empty slots, in slot order), the
/// `locked` slot left alone. What went in.
pub fn give(inv: &mut [Option<InvStack>], item: &str, n: u32, max: u32, locked: Option<usize>) -> u32 {
    let max = max.max(1);
    let mut left = n;
    for (i, s) in inv.iter_mut().enumerate() {
        if left == 0 {
            break;
        }
        if Some(i) == locked {
            continue;
        }
        if let Some(s) = s
            && s.simple
            && s.item == item
            && s.count < max
        {
            let k = left.min(max - s.count);
            s.count += k;
            left -= k;
        }
    }
    for (i, s) in inv.iter_mut().enumerate() {
        if left == 0 {
            break;
        }
        if Some(i) == locked || s.is_some() {
            continue;
        }
        let k = left.min(max);
        *s = Some(InvStack { item: item.to_owned(), count: k, simple: true });
        left -= k;
    }
    n - left
}

/// A click on an inventory slot of the open PSU: `amount` of that stack goes
/// in (onto its slot or into an empty one, never voided). What moved.
pub fn deposit(inv: &mut [Option<InvStack>], index: usize, amount: u32, slots: &mut Slots, capacity: u32, locked: Option<usize>) -> u32 {
    if Some(index) == locked {
        return 0;
    }
    let Some(Some(s)) = inv.get(index).cloned() else {
        return 0;
    };
    if !s.simple || refused(&s.item) {
        return 0;
    }
    let want = amount.min(s.count);
    let rest = insert(slots, &s.item, want, capacity, true, false);
    let moved = want - rest;
    if moved > 0 {
        let left = s.count - moved;
        inv[index] = (left > 0).then(|| InvStack { count: left, ..s });
    }
    moved
}

/// A click on a slot of the open PSU: `amount` (at most a stack) to the
/// inventory, only what fits leaves the PSU. What moved.
pub fn withdraw(slots: &mut Slots, index: usize, amount: u32, inv: &mut [Option<InvStack>], max: u32, locked: Option<usize>) -> u32 {
    let Some(Some(e)) = slots.get(index).cloned() else {
        return 0;
    };
    let want = amount.min(max.max(1)).min(e.count);
    let mut trial = inv.to_vec();
    let fits = give(&mut trial, &e.item, want, max, locked);
    if fits == 0 {
        return 0;
    }
    let got = extract(slots, index, fits, max).map_or(0, |x| x.count);
    let given = give(inv, &e.item, got, max, locked);
    debug_assert_eq!(given, got);
    got
}

// ---------------------------------------------------------------------- menu

/// How the chest menu of a tier is laid out: rows of the PSU's slots, then a
/// control row (previous page, search, info, next page).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub rows: u8,
    pub per_page: usize,
}

pub const MAX_CONTENT_ROWS: usize = 5;

pub fn layout(tier: &Tier) -> Layout {
    let content = tier.slots.div_ceil(9).clamp(1, MAX_CONTENT_ROWS);
    Layout { rows: (content + 1) as u8, per_page: content * 9 }
}

impl Layout {
    pub fn size(&self) -> usize {
        self.rows as usize * 9
    }
    pub fn control(&self) -> usize {
        self.size() - 9
    }
}

pub const C_PREV: usize = 0;
pub const C_SEARCH: usize = 4;
pub const C_INFO: usize = 6;
pub const C_NEXT: usize = 8;

/// The search text, as typed: lower case, spaces as `_`.
pub fn normalize(query: &str) -> String {
    query.trim().to_lowercase().split_whitespace().collect::<Vec<_>>().join("_")
}

/// Whether `item` (an id, `minecraft:cobblestone`) matches the normalized
/// `query`: a part of the id, or of its path with spaces as `_`.
pub fn matches(item: &str, query: &str) -> bool {
    query.is_empty() || item.to_lowercase().contains(query)
}

/// The PSU slots shown, in order: every slot without a search; the filled
/// slots that match with one.
pub fn visible(slots: &[Option<Entry>], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..slots.len()).collect();
    }
    slots
        .iter()
        .enumerate()
        .filter(|(_, s)| s.as_ref().is_some_and(|e| e.count > 0 && matches(&e.item, query)))
        .map(|(i, _)| i)
        .collect()
}

pub fn pages(l: &Layout, shown: usize) -> usize {
    shown.div_ceil(l.per_page).max(1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// A PSU slot, by its index in the PSU.
    Slot(usize),
    /// A cell of the content rows with nothing behind it.
    Blank,
    Prev,
    Next,
    Search,
    /// The player's inventory slot (0..36, `Inventory` numbering).
    Inventory(usize),
    Nothing,
}

/// `native_menu_inventory_slot`: the player's slots after the menu's own.
pub fn inventory_slot(size: usize, slot: usize) -> Option<usize> {
    let n = slot.checked_sub(size)?;
    match n {
        0..27 => Some(n + 9),
        27..36 => Some(n - 27),
        _ => None,
    }
}

pub fn target(l: &Layout, shown: &[usize], page: usize, slot: i32) -> Target {
    let Ok(slot) = usize::try_from(slot) else {
        return Target::Nothing;
    };
    let size = l.size();
    if slot >= size {
        return inventory_slot(size, slot).map_or(Target::Nothing, Target::Inventory);
    }
    if slot >= l.control() {
        return match slot - l.control() {
            C_PREV => Target::Prev,
            C_NEXT => Target::Next,
            C_SEARCH => Target::Search,
            _ => Target::Nothing,
        };
    }
    match shown.get(page * l.per_page + slot) {
        Some(&i) => Target::Slot(i),
        None => Target::Blank,
    }
}

/// What a click takes out of the PSU or puts in: left or right click one,
/// shift-click a whole stack. None: a click that does nothing here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Amount {
    One,
    Stack,
}

pub fn amount(button: i32, mode: i32) -> Option<Amount> {
    match (mode, button) {
        (0, 0) | (0, 1) => Some(Amount::One),
        (1, _) => Some(Amount::Stack),
        _ => None,
    }
}

/// What a click on an inventory slot puts in: the whole stack, except a
/// plain right click, one.
pub fn deposit_amount(button: i32, mode: i32) -> Option<u32> {
    match (mode, button) {
        (0, 1) => Some(1),
        (0, 0) | (1, _) => Some(u32::MAX),
        _ => None,
    }
}

/// The count a slot shows in the menu: the real count, kept to 1..=64.
pub fn shown_count(count: u32) -> u32 {
    count.clamp(1, 64)
}

pub fn stored_line(count: u32, capacity: u32) -> String {
    format!("Guardado: {} / {}", group(count), group(capacity))
}

pub fn info_line(tier: &Tier) -> String {
    format!("Guarda {} itens diferentes, ate {} de cada.", tier.slots, group(tier.capacity))
}

pub fn used_line(slots: &[Option<Entry>], tier: &Tier) -> String {
    let used = slots.iter().flatten().filter(|e| e.count > 0).count();
    format!("Tipos usados: {used} / {}", tier.slots)
}

/// `5000` as `5.000`.
pub fn group(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push('.');
        }
        out.push(c);
    }
    out
}

/// `/psu ...` (without the slash): Some(query) for `psu buscar <texto>`,
/// Some("") for `psu limpar`, None for anything else of ours.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Search(String),
    Clear,
    Help,
    NotOurs,
}

pub const COMMAND_SEARCH: &str = "psu buscar <texto:text>";
pub const COMMAND_CLEAR: &str = "psu limpar";

pub fn parse_command(line: &str) -> Command {
    let mut words = line.trim().splitn(3, ' ');
    if words.next() != Some("psu") {
        return Command::NotOurs;
    }
    match words.next() {
        Some("buscar") => {
            let q = normalize(words.next().unwrap_or(""));
            if q.is_empty() { Command::Clear } else { Command::Search(q) }
        }
        Some("limpar") => Command::Clear,
        _ => Command::Help,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(n: usize) -> &'static Tier {
        &TIERS[n - 1]
    }

    fn e(item: &str, count: u32) -> Option<Entry> {
        Some(Entry { item: item.into(), count })
    }

    fn inv(item: &str, count: u32) -> Option<InvStack> {
        Some(InvStack { item: item.into(), count, simple: true })
    }

    #[test]
    fn tiers_are_the_originals() {
        let got: Vec<(usize, u32)> = TIERS.iter().map(|t| (t.slots, t.capacity)).collect();
        assert_eq!(got, vec![(8, 255), (16, 4095), (32, 65535), (64, 1_048_575)]);
        assert!(is_psu("pocketstorage:psu_3"));
        assert!(!is_psu("minecraft:chest"));
    }

    #[test]
    fn encode_decode_round_trip() {
        let mut s = decode(None, t(1));
        assert_eq!(s.len(), 8);
        assert_eq!(encode(&s), "");
        s[0] = e("minecraft:cobblestone", 255);
        s[3] = e("minecraft:dirt", 3);
        let text = encode(&s);
        assert_eq!(text, "minecraft:cobblestone*255;;;minecraft:dirt*3");
        assert_eq!(decode(Some(&text), t(1)), s);
    }

    #[test]
    fn decode_is_tolerant_and_keeps_extra_filled_slots() {
        let s = decode(Some("bad;minecraft:a*0;minecraft:b*x;minecraft:c*7"), t(1));
        assert_eq!(s.len(), 8);
        assert_eq!(s[3], e("minecraft:c", 7));
        assert!(s[..3].iter().all(Option::is_none));
        // A tier-2 list read as tier 1 keeps its filled slots.
        let mut big = decode(None, t(2));
        big[12] = e("minecraft:stone", 9);
        let s = decode(Some(&encode(&big)), t(1));
        assert_eq!(s.len(), 13);
        assert_eq!(s[12], e("minecraft:stone", 9));
        // And read as tier 4 it just grows.
        assert_eq!(decode(Some(&encode(&big)), t(4)).len(), 64);
    }

    #[test]
    fn insert_matches_first_then_empty_and_caps() {
        let mut s = decode(None, t(1));
        assert_eq!(insert(&mut s, "minecraft:stone", 64, 255, true, false), 0);
        assert_eq!(insert(&mut s, "minecraft:dirt", 10, 255, true, false), 0);
        assert_eq!(s[0], e("minecraft:stone", 64));
        assert_eq!(s[1], e("minecraft:dirt", 10));
        // Over the capacity: the rest comes back...
        assert_eq!(insert(&mut s, "minecraft:stone", 250, 255, true, false), 59);
        assert_eq!(s[0], e("minecraft:stone", 255));
        // ...or is destroyed with allow_void.
        assert_eq!(insert(&mut s, "minecraft:stone", 50, 255, true, true), 0);
        assert_eq!(s[0], e("minecraft:stone", 255));
        // No empty slot allowed: refused whole.
        assert_eq!(insert(&mut s, "minecraft:sand", 5, 255, false, false), 5);
        // PSUs never go in.
        assert_eq!(insert(&mut s, "pocketstorage:psu_1", 1, 255, true, false), 1);
    }

    #[test]
    fn full_psu_refuses_new_kinds() {
        let mut s = decode(None, t(1));
        for k in 0..8 {
            assert_eq!(insert(&mut s, &format!("minecraft:i{k}"), 1, 255, true, false), 0);
        }
        assert_eq!(insert(&mut s, "minecraft:new", 3, 255, true, false), 3);
        assert_eq!(insert(&mut s, "minecraft:i7", 3, 255, true, false), 0);
    }

    #[test]
    fn tier1_holds_5000_cobblestone_only_as_255() {
        let mut s = decode(None, t(1));
        let mut rest = 0;
        for _ in 0..(5000 / 64) {
            rest += insert(&mut s, "minecraft:cobblestone", 64, t(1).capacity, true, false);
        }
        rest += insert(&mut s, "minecraft:cobblestone", 5000 % 64, t(1).capacity, true, false);
        assert_eq!(s[0], e("minecraft:cobblestone", 255));
        assert_eq!(rest, 5000 - 255);
        // Tier 3 (65.535) takes all 5000.
        let mut s3 = decode(None, t(3));
        assert_eq!(insert(&mut s3, "minecraft:cobblestone", 5000, t(3).capacity, true, false), 0);
        assert_eq!(decode(Some(&encode(&s3)), t(3))[0], e("minecraft:cobblestone", 5000));
    }

    #[test]
    fn extract_one_stack_and_empty() {
        let mut s = decode(None, t(1));
        s[2] = e("minecraft:stone", 100);
        assert_eq!(extract(&mut s, 2, 1, 64), e("minecraft:stone", 1));
        assert_eq!(extract(&mut s, 2, u32::MAX, 64).map(|x| x.count), Some(64));
        assert_eq!(s[2], e("minecraft:stone", 35));
        assert_eq!(extract(&mut s, 2, u32::MAX, 64).map(|x| x.count), Some(35));
        assert_eq!(s[2], None);
        assert_eq!(extract(&mut s, 2, 1, 64), None);
        assert_eq!(extract(&mut s, 99, 1, 64), None);
    }

    #[test]
    fn absorb_only_known_items_and_voids_excess() {
        let mut s = decode(None, t(1));
        assert!(!absorb(&mut s, "minecraft:stone", 10, 255));
        s[0] = e("minecraft:stone", 250);
        assert!(absorb(&mut s, "minecraft:stone", 10, 255));
        assert_eq!(s[0], e("minecraft:stone", 255));
        assert!(absorb(&mut s, "minecraft:stone", 64, 255));
        assert_eq!(s[0], e("minecraft:stone", 255));
        assert!(!absorb(&mut s, "minecraft:dirt", 1, 255));
    }

    #[test]
    fn give_merges_then_fills_and_skips_locked() {
        let mut v: Vec<Option<InvStack>> = vec![None; 36];
        v[0] = Some(InvStack { item: "pocketstorage:psu_1".into(), count: 1, simple: false });
        v[5] = inv("minecraft:stone", 60);
        v[6] = Some(InvStack { item: "minecraft:stone".into(), count: 1, simple: false });
        assert_eq!(give(&mut v, "minecraft:stone", 10, 64, Some(0)), 10);
        assert_eq!(v[5], inv("minecraft:stone", 64));
        assert_eq!(v[1], inv("minecraft:stone", 6));
        assert_eq!(v[6].as_ref().unwrap().count, 1);
        let mut full: Vec<Option<InvStack>> = vec![inv("minecraft:dirt", 64); 36];
        assert_eq!(give(&mut full, "minecraft:stone", 1, 64, None), 0);
    }

    #[test]
    fn deposit_and_withdraw() {
        let tier = t(1);
        let mut s = decode(None, tier);
        let mut v: Vec<Option<InvStack>> = vec![None; 36];
        v[0] = Some(InvStack { item: tier.item.into(), count: 1, simple: false });
        v[9] = inv("minecraft:cobblestone", 64);
        v[10] = Some(InvStack { item: "minecraft:diamond_sword".into(), count: 1, simple: false });
        // The PSU itself and items with components stay.
        assert_eq!(deposit(&mut v, 0, u32::MAX, &mut s, tier.capacity, Some(0)), 0);
        assert_eq!(deposit(&mut v, 10, u32::MAX, &mut s, tier.capacity, Some(0)), 0);
        // Right click: one.
        assert_eq!(deposit(&mut v, 9, 1, &mut s, tier.capacity, Some(0)), 1);
        assert_eq!(v[9], inv("minecraft:cobblestone", 63));
        assert_eq!(deposit(&mut v, 9, u32::MAX, &mut s, tier.capacity, Some(0)), 63);
        assert_eq!(v[9], None);
        assert_eq!(s[0], e("minecraft:cobblestone", 64));
        // Withdraw one, then a stack.
        assert_eq!(withdraw(&mut s, 0, 1, &mut v, 64, Some(0)), 1);
        assert_eq!(v[1], inv("minecraft:cobblestone", 1));
        assert_eq!(withdraw(&mut s, 0, u32::MAX, &mut v, 64, Some(0)), 63);
        assert_eq!(v[1], inv("minecraft:cobblestone", 64));
        assert_eq!(s[0], None);
    }

    #[test]
    fn deposit_never_voids() {
        let tier = t(1);
        let mut s = decode(None, tier);
        s[0] = e("minecraft:stone", 250);
        let mut v: Vec<Option<InvStack>> = vec![None; 36];
        v[3] = inv("minecraft:stone", 64);
        assert_eq!(deposit(&mut v, 3, u32::MAX, &mut s, tier.capacity, None), 5);
        assert_eq!(v[3], inv("minecraft:stone", 59));
    }

    #[test]
    fn withdraw_only_what_fits() {
        let mut s = decode(None, t(1));
        s[0] = e("minecraft:stone", 100);
        let mut v: Vec<Option<InvStack>> = vec![inv("minecraft:dirt", 64); 36];
        v[4] = inv("minecraft:stone", 60);
        assert_eq!(withdraw(&mut s, 0, u32::MAX, &mut v, 64, None), 4);
        assert_eq!(s[0], e("minecraft:stone", 96));
    }

    #[test]
    fn layouts_and_targets() {
        let rows: Vec<(u8, usize)> = TIERS.iter().map(|t| { let l = layout(t); (l.rows, l.per_page) }).collect();
        assert_eq!(rows, vec![(2, 9), (3, 18), (5, 36), (6, 45)]);
        let l = layout(t(4));
        let shown = visible(&decode(None, t(4)), "");
        assert_eq!(pages(&l, shown.len()), 2);
        assert_eq!(target(&l, &shown, 0, 0), Target::Slot(0));
        assert_eq!(target(&l, &shown, 1, 0), Target::Slot(45));
        assert_eq!(target(&l, &shown, 1, 18), Target::Slot(63));
        assert_eq!(target(&l, &shown, 1, 19), Target::Blank);
        assert_eq!(target(&l, &shown, 0, 45), Target::Prev);
        assert_eq!(target(&l, &shown, 0, 49), Target::Search);
        assert_eq!(target(&l, &shown, 0, 53), Target::Next);
        assert_eq!(target(&l, &shown, 0, 54), Target::Inventory(9));
        assert_eq!(target(&l, &shown, 0, 54 + 27), Target::Inventory(0));
        assert_eq!(target(&l, &shown, 0, 54 + 36), Target::Nothing);
        assert_eq!(target(&l, &shown, 0, -999), Target::Nothing);
        let l1 = layout(t(1));
        let shown1 = visible(&decode(None, t(1)), "");
        assert_eq!(target(&l1, &shown1, 0, 7), Target::Slot(7));
        assert_eq!(target(&l1, &shown1, 0, 8), Target::Blank);
        assert_eq!(target(&l1, &shown1, 0, 18), Target::Inventory(9));
    }

    #[test]
    fn search_filters_filled_slots() {
        let mut s = decode(None, t(2));
        s[1] = e("minecraft:cobblestone", 5);
        s[4] = e("minecraft:cobbled_deepslate", 5);
        s[7] = e("minecraft:dirt", 5);
        assert_eq!(visible(&s, &normalize("Cobble")), vec![1, 4]);
        assert_eq!(visible(&s, &normalize("cobbled deepslate")), vec![4]);
        assert_eq!(visible(&s, "nada").len(), 0);
        assert_eq!(visible(&s, "").len(), 16);
        let l = layout(t(2));
        let shown = visible(&s, "cobble");
        assert_eq!(target(&l, &shown, 0, 1), Target::Slot(4));
        assert_eq!(target(&l, &shown, 0, 2), Target::Blank);
    }

    #[test]
    fn clicks_and_text() {
        assert_eq!(amount(0, 0), Some(Amount::One));
        assert_eq!(amount(1, 0), Some(Amount::One));
        assert_eq!(amount(0, 1), Some(Amount::Stack));
        assert_eq!(amount(0, 2), None);
        assert_eq!(deposit_amount(1, 0), Some(1));
        assert_eq!(deposit_amount(0, 1), Some(u32::MAX));
        assert_eq!(deposit_amount(0, 4), None);
        assert_eq!(group(5000), "5.000");
        assert_eq!(group(1_048_575), "1.048.575");
        assert_eq!(group(255), "255");
        assert_eq!(stored_line(5000, 65535), "Guardado: 5.000 / 65.535");
        assert_eq!(shown_count(5000), 64);
        assert_eq!(shown_count(3), 3);
    }

    #[test]
    fn commands() {
        assert_eq!(parse_command("psu buscar  Cobble Stone "), Command::Search("cobble_stone".into()));
        assert_eq!(parse_command("psu buscar"), Command::Clear);
        assert_eq!(parse_command("psu limpar"), Command::Clear);
        assert_eq!(parse_command("psu"), Command::Help);
        assert_eq!(parse_command("give @s x"), Command::NotOurs);
        assert_eq!(parse_command("psux buscar a"), Command::NotOurs);
    }
}

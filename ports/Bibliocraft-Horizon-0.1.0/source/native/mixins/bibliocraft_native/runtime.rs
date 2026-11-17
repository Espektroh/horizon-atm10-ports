// Bibliocraft Legacy for Horizon: the furniture's rules, with no Horizon
// types (std only), so `cargo test` checks them outside the game. Added to
// the game as `crate::game::bibliocraft_rt`; `glue.rs` applies them to
// Horizon's block entities, stacks and players.
//
// Numbers and rules from Bibliocraft Legacy 1.6.6 (MinecraftschurliMods,
// MIT), branch version/1.21.1: BookcaseBlockEntity/BookcaseMenu,
// ShelfBlock.lookingAtSlot/ShelfMenu/ShelfBER, SwordPedestalBlock/BER,
// TableBlock.useItemOn/TableBER, BCFacingInteractibleBlock.useItemOn,
// ClientUtil.setupCenteredBER.

pub const ID: &str = "bibliocraft_native";
pub const NAME: &str = "Bibliocraft Legacy (Horizon)";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Estantes (16 livros), prateleiras (4 itens), mesas, assentos e o pedestal \
de espada do Bibliocraft Legacy. Precisa do mod de conteúdo bibliocraft.";

/// The 11 vanilla woods of the original (`BibliocraftApi` wood types).
pub const WOODS: [&str; 11] = [
    "oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "crimson", "warped", "mangrove", "bamboo", "cherry",
];
pub const PEDESTAL: &str = "bibliocraft:sword_pedestal";
pub const MENU_BOOKCASE: &str = "bibliocraft:bookcase";
pub const MENU_SHELF: &str = "bibliocraft:shelf";

/// What clients get of each slot (update tag): a list of `{id, components}`.
pub const KEY_VIEW: &str = "BcView";
/// A hash of what clients were last sent (saved; only a cache).
pub const KEY_SHOWN: &str = "BcShown";

/// The furniture with a block entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Bookcase,
    Shelf,
    Table,
    Pedestal,
}

impl Kind {
    /// The kind of block `block` (`bibliocraft:oak_bookcase`), if it is one.
    pub fn of(block: &str) -> Option<Kind> {
        if block == PEDESTAL {
            return Some(Kind::Pedestal);
        }
        let name = block.strip_prefix("bibliocraft:")?;
        for (suffix, kind) in [
            ("_bookcase", Kind::Bookcase),
            ("_shelf", Kind::Shelf),
            ("_table", Kind::Table),
        ] {
            if let Some(wood) = name.strip_suffix(suffix) {
                return WOODS.contains(&wood).then_some(kind);
            }
        }
        None
    }
    /// Container slots (`BookcaseBlockEntity` 16, `ShelfBlockEntity` 4,
    /// the pedestal 1; the table's item on top, its cloth slot not ported).
    pub fn slots(self) -> usize {
        match self {
            Kind::Bookcase => 16,
            Kind::Shelf => 4,
            Kind::Table | Kind::Pedestal => 1,
        }
    }
    /// The content mod's menu type the block opens, if it has a screen.
    pub fn menu(self) -> Option<&'static str> {
        match self {
            Kind::Bookcase => Some(MENU_BOOKCASE),
            Kind::Shelf => Some(MENU_SHELF),
            _ => None,
        }
    }
    /// Hoppers reach every slot from every face (the original's block
    /// entities are plain `Container`s); the table keeps its item to itself.
    pub fn hoppers(self) -> bool {
        !matches!(self, Kind::Table)
    }
}

/// `#bibliocraft:bookcase_books` = `#minecraft:bookshelf_books` (+ the
/// lectern's, already in it) for vanilla items. The same list is written in
/// `menu_slot_rules.rs`, which can't reach this module.
pub const BOOKS: [&str; 5] = [
    "minecraft:book",
    "minecraft:writable_book",
    "minecraft:written_book",
    "minecraft:enchanted_book",
    "minecraft:knowledge_book",
];

pub fn is_book(item: &str) -> bool {
    BOOKS.contains(&item)
}

/// A sword by its id, for when the item tags can't be read
/// (`#minecraft:swords` is asked first).
pub fn is_sword_id(item: &str) -> bool {
    item.ends_with("_sword")
}

/// May a hopper put an item in `slot` (`canPlaceItem` and the slot's stack
/// size: one book per bookcase slot, one sword on the pedestal)?
pub fn hopper_accepts(kind: Kind, slot: usize, slot_empty: bool, item_ok: bool) -> bool {
    if slot >= kind.slots() {
        return false;
    }
    match kind {
        Kind::Bookcase | Kind::Pedestal => slot_empty && item_ok,
        Kind::Shelf => true,
        Kind::Table => false,
    }
}

/// Where the screen puts each container slot (top left of the item, GUI
/// pixels): `BookcaseMenu.addSlots` (2 rows of 8) and `ShelfMenu.addSlots`.
pub fn screen_slots(kind: Kind) -> Option<Vec<[f32; 2]>> {
    match kind {
        Kind::Bookcase => Some(
            (0..16)
                .map(|n| {
                    let (i, j) = (n / 8, n % 8);
                    [17.0 + j as f32 * 18.0, 17.0 + i as f32 * 25.0]
                })
                .collect(),
        ),
        Kind::Shelf => Some(vec![[53.0, 15.0], [107.0, 15.0], [53.0, 53.0], [107.0, 53.0]]),
        _ => None,
    }
}

// ---- facing ----

pub fn facing_normal(facing: &str) -> Option<[i32; 3]> {
    match facing {
        "north" => Some([0, 0, -1]),
        "south" => Some([0, 0, 1]),
        "west" => Some([-1, 0, 0]),
        "east" => Some([1, 0, 0]),
        _ => None,
    }
}

/// `ClientUtil.setupCenteredBER`: the turn (degrees, counterclockwise from
/// above) that puts a south-facing drawing in front of `facing`.
pub fn ber_yaw(facing: &str) -> f32 {
    match facing {
        "east" => 90.0,
        "north" => 180.0,
        "west" => 270.0,
        _ => 0.0,
    }
}

/// The turn of a north-facing block model (the book models) as the
/// blockstate turns the bookcase (`y` 90 for east, clockwise): the same
/// counterclockwise.
pub fn model_yaw(facing: &str) -> f32 {
    (ber_yaw(facing) + 180.0) % 360.0
}

/// The view direction for Minecraft's yaw and pitch, in degrees.
pub fn look_dir(yaw: f32, pitch: f32) -> [f64; 3] {
    let (y, p) = (f64::from(yaw).to_radians(), f64::from(pitch).to_radians());
    [-y.sin() * p.cos(), -p.sin(), y.cos() * p.cos()]
}

pub const EYE_STANDING: f64 = 1.62;
pub const EYE_SNEAKING: f64 = 1.27;

/// Where the ray from `eye` along `dir` meets the plane `inset` blocks
/// behind the block's `facing` face (the shelf's front is half a block in):
/// (u, v), u from the viewer's left (0) to the right (1) as
/// `ShelfBlock.lookingAtSlot` measures it (along `facing.getClockWise()`),
/// v from the bottom (0) to the top (1). None when the ray never reaches it
/// or meets it outside the block.
pub fn front_hit(facing: &str, pos: [i32; 3], eye: [f64; 3], dir: [f64; 3], inset: f64) -> Option<(f64, f64)> {
    let n = facing_normal(facing)?;
    let axis = if n[0] != 0 { 0 } else { 2 };
    let plane = f64::from(pos[axis]) + if n[axis] > 0 { 1.0 - inset } else { inset };
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
    let v = local[1];
    const E: f64 = 1e-6;
    ((-E..=1.0 + E).contains(&u) && (-E..=1.0 + E).contains(&v)).then_some((u, v))
}

/// The shelf's front plane, half a block in from its facing face.
pub const SHELF_INSET: f64 = 0.5;

/// `ShelfBlock.lookingAtSlot`: 0 top left, 1 top right, 2 bottom left,
/// 3 bottom right (as the viewer sees them); none between them.
pub fn shelf_slot_at(u: f64, v: f64) -> Option<usize> {
    for i in 0..2 {
        for j in 0..2 {
            let min_x = j as f64 * 0.5625;
            let max_x = min_x + 0.4375;
            let max_y = 1.0 - i as f64 * 0.5;
            let min_y = max_y - 0.4375;
            if u >= min_x && u < max_x && v >= min_y && v < max_y {
                return Some(i * 2 + j);
            }
        }
    }
    None
}

/// `ShelfBER`: slot `n`'s item centre from the block's centre, in the
/// south-facing drawing (`setupCenteredBER` turns it to the facing).
pub fn shelf_offset(n: usize) -> [f32; 3] {
    let (i, j) = ((n / 2) as f32, (n % 2) as f32);
    [-0.28125 + j * 0.5625, 0.28125 - i * 0.5, -0.25]
}
pub const SHELF_SCALE: f32 = 0.4375;

/// The book models the bookcase draws for its filled slots (its
/// `BookcaseModel` adds book `i`'s quads when slot `i` holds something).
pub fn book_model(i: usize) -> String {
    format!("bibliocraft:block/template/bookcase/book_{i}")
}

pub fn standalone_models() -> Vec<String> {
    (0..16).map(book_model).collect()
}

// ---- clicks ----

/// What a right click on a shelf or the pedestal does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    /// Left to the game: the block's own use (the shelf's screen), or the
    /// item's (a block placed).
    Pass,
    /// The hand's whole stack and slot `n`'s trade places
    /// (`BCFacingInteractibleBlock.useItemOn`: `setItem(slot, stack)`,
    /// `setItemInHand(slot's stack)`).
    Swap(usize),
}

/// A shelf: from the front only, on a slot; sneaking with something in a
/// hand is the vanilla "secondary use" (the item acts).
pub fn shelf_click(sneaking: bool, hands_full: bool, front: bool, slot: Option<usize>) -> Click {
    if sneaking && hands_full {
        return Click::Pass;
    }
    match (front, slot) {
        (true, Some(n)) => Click::Swap(n),
        _ => Click::Pass,
    }
}

/// The sword pedestal: any face but the bottom (`canAccessFromDirection`),
/// with an empty hand or a sword (`canPlaceItem`: `#minecraft:swords`).
/// `held_sword` is None for an empty hand.
pub fn pedestal_click(sneaking: bool, hands_full: bool, bottom: bool, held_sword: Option<bool>) -> Click {
    if (sneaking && hands_full) || bottom {
        return Click::Pass;
    }
    match held_sword {
        None | Some(true) => Click::Swap(0),
        Some(false) => Click::Pass,
    }
}

/// What a right click on a table does (`TableBlock.useItemOn`, the item
/// slot; the cloth slot is not ported).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableClick {
    Pass,
    /// The click is used up with nothing changed (`ItemInteractionResult.FAIL`:
    /// the same item already lies there, or nothing in hand nor on top).
    Nothing,
    /// The item on top goes to the empty hand.
    Take,
    /// One of the hand's items goes on top; what lay there goes back to the
    /// hand if it is empty then, else to the inventory.
    Put,
}

pub fn table_click(
    sneaking: bool,
    hands_full: bool,
    bottom: bool,
    held: Option<&str>,
    on_top: Option<&str>,
) -> TableClick {
    if (sneaking && hands_full) || bottom {
        return TableClick::Pass;
    }
    match (held, on_top) {
        (None, None) => TableClick::Nothing,
        (None, Some(_)) => TableClick::Take,
        (Some(h), Some(t)) if h == t => TableClick::Nothing,
        (Some(_), _) => TableClick::Put,
    }
}

/// Swaps a slot and a hand (`Click::Swap`).
pub fn swap<T>(slot: &mut Option<T>, hand: &mut Option<T>) {
    std::mem::swap(slot, hand);
}

// ---- what clients were sent ----

/// FNV-1a over each slot's item, count and damage: when it changes, the
/// block entity is sent to clients again (`native_block_entity_sync`).
pub fn view_hash<'a>(slots: impl IntoIterator<Item = Option<(&'a str, u32, u32)>>) -> i64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |b: &[u8]| {
        for x in b {
            h ^= u64::from(*x);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for s in slots {
        match s {
            Some((id, count, damage)) => {
                eat(id.as_bytes());
                eat(&[0]);
                eat(&count.to_le_bytes());
                eat(&damage.to_le_bytes());
            }
            None => eat(&[1]),
        }
    }
    h as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_by_block() {
        assert_eq!(Kind::of("bibliocraft:oak_bookcase"), Some(Kind::Bookcase));
        assert_eq!(Kind::of("bibliocraft:dark_oak_shelf"), Some(Kind::Shelf));
        assert_eq!(Kind::of("bibliocraft:cherry_table"), Some(Kind::Table));
        assert_eq!(Kind::of("bibliocraft:sword_pedestal"), Some(Kind::Pedestal));
        assert_eq!(Kind::of("bibliocraft:red_oak_seat"), None);
        assert_eq!(Kind::of("bibliocraft:pale_oak_bookcase"), None);
        assert_eq!(Kind::of("minecraft:oak_bookcase"), None);
        assert_eq!(Kind::of("bibliocraft:oak_potion_shelf"), None);
        let all: usize = WOODS
            .iter()
            .flat_map(|w| ["bookcase", "shelf", "table"].map(|k| format!("bibliocraft:{w}_{k}")))
            .filter(|b| Kind::of(b).is_some())
            .count();
        assert_eq!(all, 33);
    }

    #[test]
    fn slots_and_menus() {
        assert_eq!(Kind::Bookcase.slots(), 16);
        assert_eq!(Kind::Shelf.slots(), 4);
        assert_eq!(Kind::Table.slots(), 1);
        assert_eq!(Kind::Pedestal.slots(), 1);
        assert_eq!(Kind::Bookcase.menu(), Some("bibliocraft:bookcase"));
        assert_eq!(Kind::Shelf.menu(), Some("bibliocraft:shelf"));
        assert_eq!(Kind::Table.menu(), None);
        assert_eq!(Kind::Pedestal.menu(), None);
        for k in [Kind::Bookcase, Kind::Shelf] {
            assert_eq!(screen_slots(k).unwrap().len(), k.slots());
        }
        let b = screen_slots(Kind::Bookcase).unwrap();
        assert_eq!(b[0], [17.0, 17.0]);
        assert_eq!(b[7], [143.0, 17.0]);
        assert_eq!(b[8], [17.0, 42.0]);
        assert_eq!(b[15], [143.0, 42.0]);
    }

    #[test]
    fn books_and_swords() {
        for b in BOOKS {
            assert!(is_book(b));
        }
        assert!(!is_book("minecraft:paper"));
        assert!(!is_book("minecraft:bookshelf"));
        assert!(is_sword_id("minecraft:diamond_sword"));
        assert!(is_sword_id("allthemodium:allthemodium_sword"));
        assert!(!is_sword_id("minecraft:diamond_axe"));
    }

    #[test]
    fn hoppers() {
        assert!(hopper_accepts(Kind::Bookcase, 3, true, true));
        assert!(!hopper_accepts(Kind::Bookcase, 3, false, true));
        assert!(!hopper_accepts(Kind::Bookcase, 3, true, false));
        assert!(!hopper_accepts(Kind::Bookcase, 16, true, true));
        assert!(hopper_accepts(Kind::Shelf, 0, false, false));
        assert!(!hopper_accepts(Kind::Shelf, 4, true, true));
        assert!(hopper_accepts(Kind::Pedestal, 0, true, true));
        assert!(!hopper_accepts(Kind::Pedestal, 0, false, true));
        assert!(!hopper_accepts(Kind::Table, 0, true, true));
    }

    #[test]
    fn yaws() {
        assert_eq!(ber_yaw("south"), 0.0);
        assert_eq!(ber_yaw("north"), 180.0);
        assert_eq!(model_yaw("north"), 0.0);
        assert_eq!(model_yaw("east"), 270.0);
        assert_eq!(model_yaw("south"), 180.0);
        assert_eq!(model_yaw("west"), 90.0);
    }

    /// Turning a north model by `model_yaw` counterclockwise (right hand
    /// about +Y) points its front (-z) where the block faces.
    #[test]
    fn model_front_points_to_facing() {
        for f in ["north", "east", "south", "west"] {
            let a = f64::from(model_yaw(f)).to_radians();
            let (x, z) = (0.0f64, -1.0f64);
            let rx = x * a.cos() + z * a.sin();
            let rz = -x * a.sin() + z * a.cos();
            let n = facing_normal(f).unwrap();
            assert!(
                (rx - f64::from(n[0])).abs() < 1e-9 && (rz - f64::from(n[2])).abs() < 1e-9,
                "{f}"
            );
        }
    }

    /// The same for the BER's south drawing: its +z goes to the facing.
    #[test]
    fn ber_front_points_to_facing() {
        for f in ["north", "east", "south", "west"] {
            let a = f64::from(ber_yaw(f)).to_radians();
            let (x, z) = (0.0f64, 1.0f64);
            let rx = x * a.cos() + z * a.sin();
            let rz = -x * a.sin() + z * a.cos();
            let n = facing_normal(f).unwrap();
            assert!(
                (rx - f64::from(n[0])).abs() < 1e-9 && (rz - f64::from(n[2])).abs() < 1e-9,
                "{f}"
            );
        }
    }

    #[test]
    fn look_direction() {
        let d = look_dir(0.0, 0.0);
        assert!((d[2] - 1.0).abs() < 1e-9);
        let d = look_dir(180.0, 0.0);
        assert!((d[2] + 1.0).abs() < 1e-9);
        let d = look_dir(0.0, 90.0);
        assert!((d[1] + 1.0).abs() < 1e-9);
    }

    #[test]
    fn shelf_slots() {
        assert_eq!(shelf_slot_at(0.2, 0.8), Some(0));
        assert_eq!(shelf_slot_at(0.8, 0.8), Some(1));
        assert_eq!(shelf_slot_at(0.2, 0.3), Some(2));
        assert_eq!(shelf_slot_at(0.8, 0.3), Some(3));
        assert_eq!(shelf_slot_at(0.5, 0.8), None);
        assert_eq!(shelf_slot_at(0.2, 0.52), None);
        assert_eq!(shelf_slot_at(0.2, 0.03), None);
    }

    /// A player standing south of a south-facing shelf, eyes at the shelf's
    /// upper half and a little to the west, looking straight north, aims at
    /// the top left slot; a little to the east, the top right one.
    #[test]
    fn aiming_at_a_shelf() {
        let pos = [10, 64, 20];
        let dir = look_dir(180.0, 0.0);
        let eye = [10.2, 64.8, 23.0];
        let (u, v) = front_hit("south", pos, eye, dir, SHELF_INSET).unwrap();
        assert!((u - 0.2).abs() < 1e-9 && (v - 0.8).abs() < 1e-9);
        assert_eq!(shelf_slot_at(u, v), Some(0));
        let eye = [10.8, 64.8, 23.0];
        let (u, v) = front_hit("south", pos, eye, dir, SHELF_INSET).unwrap();
        assert_eq!(shelf_slot_at(u, v), Some(1));
        // A north-facing shelf seen from the north: its viewer's left is east.
        let dir = look_dir(0.0, 0.0);
        let eye = [10.8, 64.3, 17.0];
        let (u, v) = front_hit("north", pos, eye, dir, SHELF_INSET).unwrap();
        assert!((u - 0.2).abs() < 1e-9 && (v - 0.3).abs() < 1e-9);
        assert_eq!(shelf_slot_at(u, v), Some(2));
        // East-facing, seen from the east (looking west, yaw 90).
        let dir = look_dir(90.0, 0.0);
        let eye = [13.0, 64.3, 20.8];
        let (u, _) = front_hit("east", pos, eye, dir, SHELF_INSET).unwrap();
        assert!((u - 0.2).abs() < 1e-9);
        // Looking away: no hit. Outside the block: no hit.
        assert!(front_hit("south", pos, [10.5, 64.5, 23.0], look_dir(0.0, 0.0), SHELF_INSET).is_none());
        assert!(front_hit("south", pos, [12.5, 64.5, 23.0], look_dir(180.0, 0.0), SHELF_INSET).is_none());
    }

    /// The shelf's drawn items sit over the slots the clicks reach: slot 0
    /// at the viewer's left and top.
    #[test]
    fn shelf_drawing_matches_clicks() {
        for n in 0..4 {
            let [x, y, z] = shelf_offset(n);
            // South drawing: the viewer looks north, left is -x.
            let u = f64::from(x) + 0.5;
            let v = f64::from(y) + 0.5;
            assert_eq!(shelf_slot_at(u, v), Some(n), "{n}");
            assert_eq!(z, -0.25);
        }
    }

    #[test]
    fn shelf_clicks() {
        assert_eq!(shelf_click(false, true, true, Some(2)), Click::Swap(2));
        assert_eq!(shelf_click(false, false, true, Some(0)), Click::Swap(0));
        assert_eq!(shelf_click(false, true, true, None), Click::Pass);
        assert_eq!(shelf_click(false, true, false, Some(1)), Click::Pass);
        assert_eq!(shelf_click(true, true, true, Some(1)), Click::Pass);
        assert_eq!(shelf_click(true, false, true, Some(1)), Click::Swap(1));
    }

    #[test]
    fn pedestal_clicks() {
        assert_eq!(pedestal_click(false, false, false, None), Click::Swap(0));
        assert_eq!(pedestal_click(false, true, false, Some(true)), Click::Swap(0));
        assert_eq!(pedestal_click(false, true, false, Some(false)), Click::Pass);
        assert_eq!(pedestal_click(false, false, true, None), Click::Pass);
        assert_eq!(pedestal_click(true, true, false, Some(true)), Click::Pass);
    }

    #[test]
    fn table_clicks() {
        let d = "minecraft:diamond";
        let a = "minecraft:apple";
        assert_eq!(table_click(false, true, false, Some(d), None), TableClick::Put);
        assert_eq!(table_click(false, true, false, Some(d), Some(a)), TableClick::Put);
        assert_eq!(table_click(false, true, false, Some(d), Some(d)), TableClick::Nothing);
        assert_eq!(table_click(false, false, false, None, Some(d)), TableClick::Take);
        assert_eq!(table_click(false, false, false, None, None), TableClick::Nothing);
        assert_eq!(table_click(false, true, true, Some(d), None), TableClick::Pass);
        assert_eq!(table_click(true, true, false, Some(d), None), TableClick::Pass);
    }

    #[test]
    fn swapping() {
        let mut slot = Some(1);
        let mut hand = Some(2);
        swap(&mut slot, &mut hand);
        assert_eq!((slot, hand), (Some(2), Some(1)));
        let mut slot: Option<i32> = None;
        let mut hand = Some(5);
        swap(&mut slot, &mut hand);
        assert_eq!((slot, hand), (Some(5), None));
    }

    #[test]
    fn hashes_follow_contents() {
        let empty = view_hash([None, None]);
        let one = view_hash([Some(("minecraft:book", 1, 0)), None]);
        let other = view_hash([None, Some(("minecraft:book", 1, 0))]);
        let two = view_hash([Some(("minecraft:book", 2, 0)), None]);
        assert_ne!(empty, one);
        assert_ne!(one, other);
        assert_ne!(one, two);
        assert_eq!(one, view_hash([Some(("minecraft:book", 1, 0)), None]));
    }

    #[test]
    fn book_models() {
        let m = standalone_models();
        assert_eq!(m.len(), 16);
        assert_eq!(m[0], "bibliocraft:block/template/bookcase/book_0");
        assert_eq!(m[15], "bibliocraft:block/template/bookcase/book_15");
    }
}

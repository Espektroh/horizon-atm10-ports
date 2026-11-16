//! Type-checks the bundle's glue and helpers against stand-ins of the Horizon types they use,
//! with the signatures copied from 66f3981 (see APIS.md). It only proves that the mod's own
//! code is consistent with those signatures; the real check is the launcher's build.
#![allow(dead_code, unused)]

use std::sync::Arc;

pub mod horizon_core {
    pub mod position {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
        pub struct BlockPos { pub x: i32, pub y: i32, pub z: i32 }
        // horizon-core/src/position.rs:13.
        impl BlockPos { pub const fn new(x: i32, y: i32, z: i32) -> Self { Self { x, y, z } } }
    }
}
pub mod horizon_world {
    // horizon-world/src/nbt.rs:6, 35, 42.
    pub mod nbt {
        #[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct JavaString(pub Vec<u16>);
        impl From<&str> for JavaString {
            fn from(value: &str) -> Self { Self(value.encode_utf16().collect()) }
        }
        impl JavaString {
            pub fn to_string_lossy(&self) -> String { String::from_utf16_lossy(&self.0) }
        }
        pub type Compound = std::collections::BTreeMap<JavaString, Tag>;
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum Tag {
            End, Byte(i8), Short(i16), Int(i32), Long(i64), Float(u32), Double(u64), ByteArray(Vec<u8>),
            String(JavaString), List(Vec<Tag>), Compound(Compound), IntArray(Vec<i32>), LongArray(Vec<i64>),
        }
    }
    pub mod registry {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct BlockStateId(pub u32);
        pub struct BlockRegistry;
        impl BlockRegistry {
            pub fn default_state(&self, name: &str) -> Option<BlockStateId> { let _ = name; None }
        }
    }
    pub mod level {
        pub mod flags {
            pub const UPDATE_CLIENTS: i32 = 2;
            pub const UPDATE_ALL: i32 = 3;
        }
    }
}
pub mod horizon_assets {
    // horizon-assets/src/item_model.rs:159.
    pub mod item_model {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum DisplayContext { None, Gui, Ground, Fixed }
    }
}
pub mod horizon_client {
    pub mod items {
        // horizon-items/src/items.rs:141 (ItemInfo: max_stack 144, block 146, remainder 174), 331, 994.
        use std::sync::Arc;
        use crate::horizon_world::registry::BlockStateId;
        #[derive(Clone, Debug, PartialEq)]
        pub struct StackData;
        #[derive(Clone, Debug, PartialEq)]
        pub struct ItemStack { pub item: Arc<str>, pub count: u32, pub damage: u32, pub extra: Option<Box<StackData>> }
        impl ItemStack {
            pub fn new(item: &str, count: u32) -> Self { Self { item: Arc::from(item), count, damage: 0, extra: None } }
        }
        pub struct ItemInfo { pub max_stack: u32, pub block: Option<BlockStateId>, pub remainder: Option<String> }
        pub struct Items;
        impl Items { pub fn get(&self, id: &str) -> Option<&ItemInfo> { let _ = id; None } }
    }
    pub mod item_data {
        // horizon-items/src/item_data.rs:715, 938 (client_lib.rs:176 re-exports it).
        use super::items::{ItemStack, StackData};
        use crate::horizon_world::nbt::Tag;
        pub fn encode_components(stack: &ItemStack) -> Option<Tag> { let _ = stack; None }
        pub fn decode_components(components: Option<&Tag>) -> (u32, Option<Box<StackData>>) { let _ = components; (0, None) }
    }
    pub mod containers {
        // horizon-client-render/src/chunk/block_entity.rs:44 (BlockEntity), 108/118 (ModBlockEntity
        // and its fields, Deref/DerefMut), 202 (ModContainer).
        use super::items::ItemStack;
        use crate::horizon_world::nbt::Compound;
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct ModBlockEntityFields { pub id: String, pub nbt: Compound, pub items: Vec<Option<ItemStack>>, pub data: Vec<i32> }
        pub struct ModBlockEntity { fields: ModBlockEntityFields }
        impl ModBlockEntity { pub fn new(fields: ModBlockEntityFields) -> Self { Self { fields } } }
        impl std::ops::Deref for ModBlockEntity { type Target = ModBlockEntityFields; fn deref(&self) -> &ModBlockEntityFields { &self.fields } }
        impl std::ops::DerefMut for ModBlockEntity { fn deref_mut(&mut self) -> &mut ModBlockEntityFields { &mut self.fields } }
        pub enum BlockEntity { Chest(Vec<Option<ItemStack>>), Mod(Box<ModBlockEntity>) }
        #[derive(Clone, Debug, Default, PartialEq, Eq)]
        pub struct ModContainer { pub slots: usize, pub faces: [Vec<usize>; 6], pub inputs: Vec<usize>, pub outputs: Vec<usize>, pub menu: Option<String> }
    }
    pub mod text {
        // horizon-client-core/src/text.rs:191, 239, 276 (client_lib.rs:159 re-exports it).
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Text;
        impl Text {
            pub fn literal(s: impl Into<String>) -> Self { let _ = s.into(); Text }
            pub fn translate(key: &str, args: &[Text]) -> Self { let _ = (key, args); Text }
            pub fn append(self, other: impl Into<Text>) -> Self { let _ = other.into(); self }
        }
    }
    pub mod mod_api {
        use super::items::ItemStack;
        use crate::horizon_world::registry::BlockStateId;
        pub struct NativeLootDrops<'a> {
            pub table: &'a str,
            pub dimension: &'a str,
            pub origin: [f64; 3],
            pub block: Option<([i32; 3], BlockStateId)>,
            pub entity: Option<(u32, &'a str)>,
            pub player: Option<u128>,
            pub tool: Option<&'a ItemStack>,
            pub explosion: Option<f32>,
            pub used: bool,
        }
        pub struct NativeMod { pub id: String, pub title: String, pub description: String, pub keys: Vec<String> }
        pub struct NativeServerPlayer { pub uuid: u128, pub game_mode: i32, pub position: [f64; 3] }
    }
    pub mod crafting {
        use super::items::ItemStack;
        // crafting.rs:64 (Ingredient), 210 (test), 1290 (CustomRecipe).
        pub struct Ingredient { pub order: Vec<String> }
        impl Ingredient { pub fn test(&self, stack: &ItemStack) -> bool { self.order.iter().any(|i| **i == *stack.item) } }
        pub struct CustomRecipe { pub id: String, pub json: std::sync::Arc<serde_json::Value> }
        pub struct Recipes;
        impl Recipes {
            pub fn remainders(&self, recipe: &str, grid: &[Option<ItemStack>], size: usize) -> Vec<Option<ItemStack>> {
                let _ = (recipe, grid);
                vec![None; size * size]
            }
        }
        // The injected HEAD: helper first, then the body.
        pub fn call_remainders(r: &Recipes, recipe: &str, grid: &[Option<ItemStack>], size: usize) -> Vec<Option<ItemStack>> {
            match mysticalagriculture_remainders(r, recipe, grid, size) {
                core::ops::ControlFlow::Break(v) => v,
                core::ops::ControlFlow::Continue(()) => r.remainders(recipe, grid, size),
            }
        }
        use crate::horizon_client;
        include!("../../../mixins/mysticalagriculture_native/remainders.rs");
    }
}
pub use horizon_client as horizon_client_lib;

pub mod game {
    // game.rs:9 (world_items!).
    macro_rules! world_items {
        ($s:ident) => {
            match &$s.world_tables {
                Some(t) => &t.items,
                None => $s.res.items(),
            }
        };
    }
    use crate::horizon_client::items::{ItemStack, Items};
    use crate::horizon_client::mod_api::NativeServerPlayer;
    use crate::horizon_client::containers::{BlockEntity, ModBlockEntity};
    use crate::horizon_client::crafting::{CustomRecipe, Ingredient};
    use crate::horizon_core::position::BlockPos;
    use crate::horizon_world::registry::{BlockRegistry, BlockStateId};
    use std::sync::Arc;

    pub struct StateFacts { pub name: Arc<str> }
    pub struct BlockFacts { pub states: Vec<StateFacts> }
    impl BlockFacts { pub fn get(&self, id: BlockStateId) -> &StateFacts { &self.states[id.0 as usize] } }
    // game.rs:2036 facts / items (private fns, reached from game's children).
    pub struct Resources { facts: BlockFacts, items: Items }
    impl Resources {
        fn facts(&self) -> &BlockFacts { &self.facts }
        fn items(&self) -> &Items { &self.items }
    }
    pub struct WorldTables { pub items: Items }
    // horizon-client-render/src/world.rs:84, 98, 394, 446; block_entities.rs:298.
    pub struct BlockEntities;
    impl BlockEntities { pub fn get(&self, pos: &(i32, i32, i32)) -> Option<&BlockEntity> { let _ = pos; None } }
    pub struct World { pub block_entities: BlockEntities, pub game_time: i64 }
    impl World {
        pub fn block(&self, x: i32, y: i32, z: i32) -> BlockStateId { let _ = (x, y, z); BlockStateId(0) }
        pub fn light(&self, x: i32, y: i32, z: i32) -> u8 { let _ = (x, y, z); 0 }
    }
    pub struct Assets { pub registry: &'static BlockRegistry }
    pub struct PackResources { assets: Arc<Assets> }
    // horizon-world/src/level/tick.rs:1912 (Behaviours::random_tick), ticks.rs:15 (Ticks).
    pub struct Behaviours;
    pub trait LevelAccess {}
    impl Behaviours {
        pub fn random_tick<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, pos: BlockPos) { let _ = (level, ticks, pos); }
    }
    pub struct Ticks;
    impl LevelAccess for GameLevel<'_> {}

    // game.rs:2385 res, 2420 pack, 2430 world, 2742 offhand, 2822 world_tables (private fields).
    pub struct Game<'a> {
        res: Arc<Resources>,
        world: World,
        pack: PackResources,
        offhand: Option<ItemStack>,
        world_tables: Option<WorldTables>,
        _p: std::marker::PhantomData<&'a ()>,
    }
    // game.rs:949-952.
    struct GameLevel<'w> { world: &'w mut World, facts: &'w BlockFacts }

    impl Game<'_> {
        pub(super) fn native_on(&self, mod_id: &str) -> bool { let _ = mod_id; true }
        pub(in crate::game) fn native_dimension(&self) -> &'static str { "minecraft:overworld" }
        pub(super) fn native_block(&self, pos: [i32; 3]) -> String { let _ = pos; String::new() }
        pub(super) fn native_acting_player(&self) -> Option<NativeServerPlayer> { None }
        pub(super) fn native_block_property(&self, pos: [i32; 3], key: &str) -> Option<String> { let _ = (pos, key); None }
        pub(super) fn native_set_block_property_with(&mut self, pos: [i32; 3], key: &str, value: &str, flags: i32) -> bool { let _ = (pos, key, value, flags); false }
        pub(super) fn native_take_items(&mut self, item: &str, count: u32) -> u32 { let _ = item; count }
        pub(super) fn native_give_items(&mut self, item: &str, count: u32) -> u32 { let _ = item; count }
        pub(super) fn native_set_block(&mut self, dimension: &str, pos: [i32; 3], state: BlockStateId, flags: i32) -> bool { let _ = (dimension, pos, state, flags); false }
        // native_hooks.rs:8603, 8826, 8856, 9375, 9395, 9425, 9484.
        pub(super) fn native_neighbor_signal(&mut self, dimension: &str, pos: [i32; 3]) -> i32 { let _ = (dimension, pos); 0 }
        pub(super) fn native_with_mod_block_entity<R>(&mut self, pos: [i32; 3], change: impl FnOnce(&mut ModBlockEntity) -> R) -> Option<R> { let _ = (pos, change); None }
        pub(super) fn native_block_entity_sync(&mut self, pos: [i32; 3]) { let _ = pos; }
        pub(super) fn native_recipes_of_type(&self, kind: &str) -> &[CustomRecipe] { let _ = kind; &[] }
        pub(super) fn native_recipe_ingredient(&self, json: &serde_json::Value) -> Option<Ingredient> { let _ = json; None }
        pub(super) fn native_recipe_result(&self, json: &serde_json::Value) -> Option<ItemStack> { let _ = json; None }
        pub(super) fn native_acting_id(&mut self) -> u128 { 0 }
        // native_server.rs:234, 707; native_host_bridge.rs:119.
        pub(super) fn native_action_bar(&mut self, uuid: u128, line: crate::horizon_client::text::Text) -> bool { let _ = (uuid, line); false }
        pub(super) fn native_give_stack(&mut self, uuid: u128, stack: ItemStack) -> Option<ItemStack> { let _ = uuid; Some(stack) }
        pub(super) fn native_sneaking(&self) -> bool { false }
        // game.rs:3715 (private).
        fn held_mut(&mut self) -> &mut Option<ItemStack> { &mut self.offhand }
        // game.rs:4589, 5321 (private).
        fn with_level<R>(&mut self, f: impl FnOnce(&Behaviours, &mut GameLevel<'_>, &mut Ticks) -> R) -> R {
            let facts = BlockFacts { states: Vec::new() };
            let mut level = GameLevel { world: &mut self.world, facts: &facts };
            f(&Behaviours, &mut level, &mut Ticks)
        }
        fn process_level_events(&mut self) -> Result<(), String> { Ok(()) }
    }

    // native_block_entity_draw.rs:47 (fields), 169-196 (pose stack), 722 (native_draw_item).
    pub mod native_block_entity_draw {
        pub struct BlockEntityDraw { pub partial: f32, pub game_time: i64, pub light: u32, block: [i32; 3] }
        impl BlockEntityDraw {
            pub fn push(&mut self) {}
            pub fn pop(&mut self) {}
            pub fn translate(&mut self, by: [f32; 3]) { let _ = by; }
            pub fn rotate_about(&mut self, axis: [f32; 3], degrees: f32, pivot: [f32; 3]) { let _ = (axis, degrees, pivot); }
            pub fn scale(&mut self, by: [f32; 3]) { let _ = by; }
        }
        impl super::Game<'_> {
            pub(super) fn native_draw_item(
                &self,
                draw: &mut BlockEntityDraw,
                stack: &crate::horizon_client::items::ItemStack,
                display: crate::horizon_assets::item_model::DisplayContext,
                owner_yaw: Option<f32>,
                seed: i32,
            ) -> bool { let _ = (draw, stack, display, owner_yaw, seed); false }
        }
    }
    // game.rs:437.
    pub use native_block_entity_draw::BlockEntityDraw;

    pub mod native_hooks {
        use super::{Game, GameLevel};
        use crate::horizon_client::items::ItemStack;
        pub(crate) use crate::horizon_client::mod_api::{NativeLootDrops, NativeMod};
        use crate::horizon_core::position::BlockPos;
        use crate::horizon_world::registry::BlockStateId;
        use crate::{horizon_client, horizon_core, horizon_world};

        impl Game<'_> {
            pub(super) fn native_loot_drops(&mut self, context: &NativeLootDrops<'_>, drops: &mut Vec<ItemStack>) {
                mysticalagriculture_loot_drops(&mut *self, context, &mut *drops);
            }
            pub(super) fn native_use_item_on(&mut self, stack: &ItemStack, pos: [i32; 3], face: [i32; 3], claimed: &mut bool) {
                let _ = face;
                mysticalagriculture_use_item_on(&mut *self, stack, pos, &mut *claimed);
            }
            pub(super) fn native_mods(&mut self, mods: &mut Vec<NativeMod>) {
                mysticalagriculture_listed(&mut *mods);
            }
            // native_hooks.rs:3065, 3076, 3115, 3202, 3221, 3270, 6892.
            pub(super) fn native_block_entity(&mut self, block: &str, entity: &mut Option<(String, crate::horizon_world::nbt::Compound)>) {
                mysticalagriculture_block_entity(block, &mut *entity);
            }
            pub(super) fn native_block_entity_ticks(&self, block: &str, ticks: &mut bool) {
                mysticalagriculture_ticks(block, &mut *ticks);
            }
            pub(super) fn native_block_entity_container(&self, block: &str, container: &mut Option<crate::horizon_client::containers::ModContainer>) {
                mysticalagriculture_container(block, &mut *container);
            }
            pub(super) fn native_block_entity_tick(&mut self, pos: [i32; 3], entity: &mut crate::horizon_client::containers::ModBlockEntity) {
                mysticalagriculture_tick(&mut *self, pos, &mut *entity);
            }
            pub(super) fn native_block_entity_update_tag(pos: [i32; 3], entity: &crate::horizon_client::containers::ModBlockEntity, tag: &mut Option<crate::horizon_world::nbt::Compound>) {
                let _ = pos;
                mysticalagriculture_update_tag(&*entity, &mut *tag);
            }
            pub(super) fn native_block_entity_submit(&self, pos: [i32; 3], block: &str, state: BlockStateId, entity: &crate::horizon_client::containers::ModBlockEntity, draw: &mut super::BlockEntityDraw) {
                let _ = state;
                mysticalagriculture_submit(&*self, pos, block, &*entity, &mut *draw);
            }
            pub(super) fn native_right_click_block(&mut self, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
                let _ = face;
                mysticalagriculture_right_click(&mut *self, pos, &mut *canceled);
            }
        }
        pub struct Ticks;
        impl GameLevel<'_> {
            pub(super) fn native_random_tick(&mut self, ticks: &mut Ticks, dimension: u8, pos: BlockPos, state: BlockStateId, handled: &mut bool) {
                let _ = (ticks, dimension);
                mysticalagriculture_random_tick(&*self, pos, state, &mut *handled);
            }
            pub(super) fn native_is_valid_bonemeal_target(&self, dimension: u8, pos: BlockPos, state: BlockStateId, valid: &mut Option<bool>) {
                let _ = (dimension, pos);
                mysticalagriculture_bone_meal(self, state, &mut *valid);
            }
        }
        include!("../../../mixins/mysticalagriculture_native/loot_drops.rs");
        include!("../../../mixins/mysticalagriculture_native/use_item_on.rs");
        include!("../../../mixins/mysticalagriculture_native/random_tick.rs");
        include!("../../../mixins/mysticalagriculture_native/bone_meal.rs");
        include!("../../../mixins/mysticalagriculture_native/list.rs");
        include!("../../../mixins/mysticalagriculture_native/block_entity.rs");
        include!("../../../mixins/mysticalagriculture_native/ticks.rs");
        include!("../../../mixins/mysticalagriculture_native/container.rs");
        include!("../../../mixins/mysticalagriculture_native/tick.rs");
        include!("../../../mixins/mysticalagriculture_native/update_tag.rs");
        include!("../../../mixins/mysticalagriculture_native/submit.rs");
        include!("../../../mixins/mysticalagriculture_native/right_click.rs");
    }
    #[test]
    fn infusion_blocks_get_entities_containers_and_views() {
        use crate::horizon_client::containers::{ModBlockEntity, ModBlockEntityFields, ModContainer};
        use crate::horizon_client::items::ItemStack;
        use crate::horizon_world::nbt::{JavaString, Tag};
        use crate::game::mysticalagriculture_glue as g;
        let altar = "mysticalagriculture:infusion_altar";
        let pedestal = "mysticalagriculture:infusion_pedestal";
        let mut e = None;
        g::block_entity(altar, &mut e);
        assert_eq!(e.as_ref().map(|(id, _)| id.as_str()), Some(altar));
        let mut e = None;
        g::block_entity("minecraft:stone", &mut e);
        assert!(e.is_none());
        let (mut t1, mut t2) = (false, false);
        g::ticks(altar, &mut t1);
        g::ticks(pedestal, &mut t2);
        assert!(t1 && !t2);
        let mut c: Option<ModContainer> = None;
        g::container(altar, &mut c);
        let c = c.unwrap();
        assert_eq!((c.slots, c.outputs.clone(), c.inputs.len()), (2, vec![1], 0));
        assert!(c.faces.iter().all(|f| f == &vec![1]));
        let mut c: Option<ModContainer> = None;
        g::container(pedestal, &mut c);
        assert_eq!(c.unwrap().slots, 1);
        // The update tag names each slot's item ("" for an empty one).
        let m = ModBlockEntity::new(ModBlockEntityFields {
            id: altar.into(),
            items: vec![Some(ItemStack::new("mysticalagriculture:prosperity_seed_base", 1))],
            ..Default::default()
        });
        let mut tag = None;
        g::update_tag(&m, &mut tag);
        let tag = tag.unwrap();
        let Some(Tag::List(view)) = tag.get(&JavaString::from("mysticalagriculture_view")) else { panic!() };
        assert_eq!(view.len(), 2);
        let id = |t: &Tag| match t {
            Tag::Compound(c) => match c.get(&JavaString::from("id")) {
                Some(Tag::String(s)) => s.to_string_lossy(),
                _ => panic!(),
            },
            _ => panic!(),
        };
        assert_eq!(id(&view[0]), "mysticalagriculture:prosperity_seed_base");
        assert_eq!(id(&view[1]), "");
    }
    pub mod mysticalagriculture_rt {
        include!("../../../mixins/mysticalagriculture_native/runtime.rs");
    }
    pub mod mysticalagriculture_glue {
        use crate::{horizon_assets, horizon_client, horizon_core, horizon_world};
        include!("../../../mixins/mysticalagriculture_native/glue.rs");
    }
}

pub mod mod_content {
    pub struct Block { pub id: String }
    struct NewBlock<'a> { decl: &'a Block }
    struct Rename { from: String, to: String }
    enum J { Null, Bool(bool), Str(String), Arr(Vec<J>), Obj(Vec<(String, J)>) }
    impl J {
        fn get_mut(&mut self, key: &str) -> Option<&mut J> {
            match self { J::Obj(o) => o.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v), _ => None }
        }
        fn set(&mut self, key: &str, value: J) {
            if let Some(v) = self.get_mut(key) { *v = value; } else if let J::Obj(o) = self { o.push((key.to_owned(), value)); }
        }
        fn at(&mut self, path: &str) -> Option<&mut J> {
            path.split('/').filter(|p| !p.is_empty()).try_fold(self, |j, key| j.get_mut(key))
        }
        fn as_str(&self) -> Option<&str> { match self { J::Str(s) => Some(s), _ => None } }
        fn rename(&mut self, r: &Rename, ids: bool) {
            mysticalagriculture_seed_crop(&mut *self, r);
            let _ = ids;
        }
    }
    struct Plan;
    impl Plan {
        fn override_gameplay(&self, b: &NewBlock, combo: &[(String, String)], row: &mut J) {
            mysticalagriculture_crop_seed(b, &mut *row);
            let _ = combo;
        }
    }
    include!("../../../mixins/mysticalagriculture_native/crop_seed.rs");
    include!("../../../mixins/mysticalagriculture_native/seed_crop.rs");

    #[test]
    fn seeds_point_at_their_crop() {
        let mut row = J::Obj(vec![("fields".into(), J::Obj(vec![("block".into(), J::Str("minecraft:wheat".into()))]))]);
        let r = Rename { from: "minecraft:wheat_seeds".into(), to: "mysticalagriculture:iron_seeds".into() };
        row.rename(&r, false);
        assert_eq!(row.at("fields/block").and_then(|b| b.as_str().map(str::to_owned)).as_deref(), Some("mysticalagriculture:iron_crop"));
        let mut other = J::Obj(vec![("fields".into(), J::Obj(vec![("block".into(), J::Str("minecraft:wheat".into()))]))]);
        let r = Rename { from: "minecraft:wheat_seeds".into(), to: "othermod:seeds".into() };
        other.rename(&r, false);
        assert_eq!(other.at("fields/block").and_then(|b| b.as_str().map(str::to_owned)).as_deref(), Some("minecraft:wheat"));
    }

    #[test]
    fn crop_rows_name_their_seeds() {
        let decl = Block { id: "mysticalagriculture:nether_star_crop".into() };
        let mut row = J::Obj(vec![("item".into(), J::Str("minecraft:air".into()))]);
        Plan.override_gameplay(&NewBlock { decl: &decl }, &[], &mut row);
        assert_eq!(row.at("item").and_then(|b| b.as_str().map(str::to_owned)).as_deref(), Some("mysticalagriculture:nether_star_seeds"));
    }
}

#[test]
fn crystal_wears_in_the_grid() {
    use horizon_client::crafting::{call_remainders, Recipes};
    use horizon_client::items::ItemStack;
    let mut crystal = ItemStack::new("mysticalagriculture:infusion_crystal", 1);
    crystal.damage = 5;
    let grid = vec![Some(crystal.clone()), Some(ItemStack::new("mysticalagriculture:inferium_essence", 1)), None, None];
    let out = call_remainders(&Recipes, "x", &grid, 2);
    assert_eq!(out[0].as_ref().map(|s| s.damage), Some(6));
    assert!(out[1].is_none());
    crystal.damage = 999;
    let out = call_remainders(&Recipes, "x", &[Some(crystal), None, None, None], 2);
    assert!(out[0].is_none());
}


//! Type-checks the bundle's glue and helpers against stand-ins of the Horizon types they use,
//! with the signatures copied from 66f3981 (see APIS.md). It only proves that the mod's own
//! code is consistent with those signatures; the real check is the launcher's build.
#![allow(dead_code, unused)]

use std::sync::Arc;

pub mod horizon_core {
    pub mod position {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
        pub struct BlockPos { pub x: i32, pub y: i32, pub z: i32 }
    }
}
pub mod horizon_world {
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
pub mod horizon_client {
    pub mod items {
        use std::sync::Arc;
        #[derive(Clone, Debug, PartialEq)]
        pub struct ItemStack { pub item: Arc<str>, pub count: u32, pub damage: u32, pub extra: Option<Box<u8>> }
        impl ItemStack {
            pub fn new(item: &str, count: u32) -> Self { Self { item: Arc::from(item), count, damage: 0, extra: None } }
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
        pub struct NativeServerPlayer { pub uuid: u128, pub game_mode: i32 }
    }
    pub mod crafting {
        use super::items::ItemStack;
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
    use crate::horizon_client::items::ItemStack;
    use crate::horizon_client::mod_api::NativeServerPlayer;
    use crate::horizon_world::registry::{BlockRegistry, BlockStateId};
    use std::sync::Arc;

    pub struct StateFacts { pub name: Arc<str> }
    pub struct BlockFacts { pub states: Vec<StateFacts> }
    impl BlockFacts { pub fn get(&self, id: BlockStateId) -> &StateFacts { &self.states[id.0 as usize] } }
    pub struct Resources { facts: BlockFacts }
    impl Resources { pub fn facts(&self) -> &BlockFacts { &self.facts } }
    pub struct World;
    impl World { pub fn block(&self, x: i32, y: i32, z: i32) -> BlockStateId { let _ = (x, y, z); BlockStateId(0) } }
    pub struct Assets { pub registry: &'static BlockRegistry }
    pub struct PackResources { assets: Arc<Assets> }

    pub struct Game<'a> { res: Arc<Resources>, world: World, pack: PackResources, _p: std::marker::PhantomData<&'a ()> }
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
    }

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
    }
    pub mod mysticalagriculture_rt {
        include!("../../../mixins/mysticalagriculture_native/runtime.rs");
    }
    pub mod mysticalagriculture_glue {
        use crate::{horizon_client, horizon_core, horizon_world};
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

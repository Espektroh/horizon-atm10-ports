#![allow(dead_code, unused)]
// Stand-ins for the Horizon 66f3981 items powah glue.rs uses, with the
// shapes read from the Horizon source. Only a type check of glue.rs.




pub mod game {
    use horizon_client::items::{ItemStack, Items};
    use horizon_world::registry::BlockStateId;
    macro_rules! world_items {
        ($s:ident) => { match &$s.world_tables { Some(t) => &t.items, None => $s.res.items() } };
    }
    macro_rules! world_recipes {
        ($s:ident) => { match &$s.world_tables { Some(t) => &t.recipes, None => $s.res.recipes() } };
    }
    pub struct Cooked { pub result: ItemStack }
    pub struct Recipes;
    impl Recipes { pub fn cook(&self, _kind: &str, _input: &ItemStack) -> Option<&Cooked> { None } }
    pub struct Tables { pub items: Items, pub recipes: Recipes }
    pub struct Facts;
    pub struct Fact { pub name: String, pub full_collision: bool }
    impl Facts { pub fn get(&self, _s: BlockStateId) -> &Fact { unimplemented!() } }
    pub struct Providers;
    impl Providers { pub fn burn_time(&self, _items: &Items, _s: &ItemStack, _x: bool) -> i32 { 0 } }
    pub struct Res;
    impl Res {
        pub fn items(&self) -> &Items { unimplemented!() }
        pub fn recipes(&self) -> &Recipes { unimplemented!() }
        pub fn facts(&self) -> &Facts { unimplemented!() }
        pub fn providers(&self) -> &Providers { unimplemented!() }
    }
    pub struct World { pub day_time: i64, pub game_time: i64 }
    impl World { pub fn block(&self, _x: i32, _y: i32, _z: i32) -> BlockStateId { BlockStateId(0) } }
    pub enum Dim { Overworld, Nether, End }
    pub struct Ingredient;
    impl Ingredient { pub fn test(&self, _s: &ItemStack) -> bool { false } }
    pub struct CustomRecipe { pub id: String, pub json: std::sync::Arc<serde_json::Value> }
    pub struct Game<'a> { pub world_tables: Option<Tables>, pub res: Res, pub world: World, pub dim: Dim, _p: std::marker::PhantomData<&'a ()> }
    impl Game<'_> {
        pub(super) fn native_block(&self, _p: [i32; 3]) -> String { String::new() }
        pub(super) fn native_block_property(&self, _p: [i32; 3], _k: &str) -> Option<String> { None }
        pub(super) fn native_set_block_property(&mut self, _p: [i32; 3], _k: &str, _v: &str) -> bool { false }
        pub(super) fn native_with_mod_block_entity<R>(&mut self, _p: [i32; 3], _f: impl FnOnce(&mut horizon_client::containers::ModBlockEntity) -> R) -> Option<R> { None }
        pub(super) fn native_recipes_of_type(&self, _k: &str) -> &[CustomRecipe] { &[] }
        pub(super) fn native_recipe_ingredient(&self, _j: &serde_json::Value) -> Option<Ingredient> { None }
        pub(super) fn native_recipe_result(&self, _j: &serde_json::Value) -> Option<ItemStack> { None }
        pub(super) fn native_held(&self) -> Option<(String, u32, bool)> { None }
        pub(super) fn native_take_items(&mut self, _i: &str, _c: u32) -> u32 { 0 }
        pub(super) fn native_give_items(&mut self, _i: &str, _c: u32) -> u32 { 0 }
        pub(super) fn native_fill(&self, _b: &mut horizon_client::hud::HudBatch, _r: [f32; 4], _c: u32) {}
        pub(super) fn native_text(&self, _b: &mut horizon_client::hud::HudBatch, _t: &str, _at: [f32; 2], _c: u32, _shadow: bool) {}
    }
    pub mod helpers {
        #![allow(dead_code)]
        include!("../../../mixins/powah_native/block_entity.rs");
        include!("../../../mixins/powah_native/ticks.rs");
        include!("../../../mixins/powah_native/container.rs");
        include!("../../../mixins/powah_native/tick.rs");
        include!("../../../mixins/powah_native/screen.rs");
        include!("../../../mixins/powah_native/screen_draw.rs");
        include!("../../../mixins/powah_native/list.rs");
        include!("../../../mixins/powah_native/right_click.rs");
    }
    pub struct GameLevel<'a> { pub facts: &'a Facts, pub registry: &'a horizon_world::registry::Registry }
    pub mod native_hooks {
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct NativeMod { pub id: String, pub title: String, pub description: String, pub keys: Vec<[String; 3]> }
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct NativeScreen { pub width: f32, pub height: f32, pub slots: Vec<Option<[f32; 2]>>, pub inventory_y: f32, pub inventory_x: f32, pub own_labels: bool, pub own_background: bool }
    }
    #[path = "../../../../mixins/powah_native/runtime.rs"]
    pub mod powah_native;
    #[path = "../../../../mixins/powah_native/glue.rs"]
    pub mod powah_glue;
}

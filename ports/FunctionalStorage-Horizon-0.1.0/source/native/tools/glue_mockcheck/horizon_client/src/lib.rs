#![allow(dead_code, unused)]
// Stand-ins with the shapes of Horizon 66f3981. Only a type check; APIS.md has the
// file:line of each real item.
pub mod items {
    // horizon-items/src/items.rs:142 (ItemInfo), 331 (ItemStack), 994 (Items::get);
    // item_data.rs:259 (new), 348 (custom_name), 408 (same_item_same_components).
    use horizon_world::registry::BlockStateId;
    use std::sync::Arc;
    #[derive(Clone, Debug, PartialEq)]
    pub struct StackData;
    #[derive(Clone, Debug, PartialEq)]
    pub struct ItemStack {
        pub item: Arc<str>,
        pub count: u32,
        pub damage: u32,
        pub extra: Option<Box<StackData>>,
    }
    impl ItemStack {
        pub fn new(item: &str, count: u32) -> Self {
            ItemStack { item: Arc::from(item), count, damage: 0, extra: None }
        }
        pub fn same_item_same_components(&self, other: &Self) -> bool {
            self.item == other.item
        }
        pub fn custom_name(&self) -> Option<String> {
            None
        }
    }
    pub struct ItemInfo {
        pub max_stack: u32,
        pub block: Option<BlockStateId>,
    }
    pub struct Items;
    impl Items {
        pub fn get(&self, _id: &str) -> Option<&ItemInfo> {
            None
        }
    }
}
pub mod item_data {
    // horizon-items/src/item_data.rs:715, 938 (re-exported by client_lib.rs:176).
    use crate::items::{ItemStack, StackData};
    use horizon_world::nbt::Tag;
    pub fn encode_components(_stack: &ItemStack) -> Option<Tag> {
        None
    }
    pub fn decode_components(_components: Option<&Tag>) -> (u32, Option<Box<StackData>>) {
        (0, None)
    }
}
pub mod containers {
    // horizon-client-render/src/chunk/block_entity.rs:91, 108, 118, 202.
    use crate::items::ItemStack;
    use horizon_world::nbt::Compound;
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct ModBlockEntityFields {
        pub id: String,
        pub nbt: Compound,
        pub items: Vec<Option<ItemStack>>,
        pub data: Vec<i32>,
    }
    pub struct ModBlockEntity {
        fields: ModBlockEntityFields,
    }
    impl std::ops::Deref for ModBlockEntity {
        type Target = ModBlockEntityFields;
        fn deref(&self) -> &Self::Target {
            &self.fields
        }
    }
    impl std::ops::DerefMut for ModBlockEntity {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.fields
        }
    }
    pub enum BlockEntity {
        Chest,
        Mod(Box<ModBlockEntity>),
    }
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct ModContainer {
        pub slots: usize,
        pub faces: [Vec<usize>; 6],
        pub inputs: Vec<usize>,
        pub outputs: Vec<usize>,
        pub menu: Option<String>,
    }
}
pub mod block_entities {
    // horizon-client-render/src/block_entities.rs:31, 298.
    pub type Pos = (i32, i32, i32);
    pub struct BlockEntities;
    impl BlockEntities {
        pub fn get(&self, _pos: &Pos) -> Option<&crate::containers::BlockEntity> {
            None
        }
    }
}
pub mod world {
    // horizon-client-render/src/world.rs:84, 98, 446.
    pub struct World {
        pub block_entities: crate::block_entities::BlockEntities,
        pub game_time: i64,
    }
    impl World {
        pub fn light(&self, _x: i32, _y: i32, _z: i32) -> u8 {
            0
        }
    }
}
pub mod text {
    // horizon-client-core/src/text.rs:167, 191, 239, 276.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct Text;
    impl Text {
        pub fn literal(_s: impl Into<String>) -> Self {
            Text
        }
        pub fn translate(_key: &str, _args: &[Text]) -> Self {
            Text
        }
        pub fn append(self, _other: impl Into<Text>) -> Self {
            self
        }
    }
}
pub mod mod_api {
    // horizon-client/src/mod_api.rs:373.
    pub struct NativeServerPlayer {
        pub uuid: u128,
        pub name: String,
        pub permission: u8,
        pub dimension: String,
        pub position: [f64; 3],
        pub game_mode: i32,
        pub health: f32,
        pub yaw: f32,
        pub pitch: f32,
        pub entity_id: i32,
        pub address: String,
    }
    pub struct NativeMod {
        pub id: String,
        pub title: String,
        pub description: String,
        pub keys: Vec<[String; 3]>,
    }
}

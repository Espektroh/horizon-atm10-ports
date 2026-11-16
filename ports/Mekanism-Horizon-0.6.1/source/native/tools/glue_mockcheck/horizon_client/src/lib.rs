#![allow(dead_code, unused)]

    pub mod items {
        use std::sync::Arc;
        #[derive(Clone, Debug, PartialEq)]
        pub struct ItemStack { pub item: Arc<str>, pub count: u32, pub damage: u32 }
        impl ItemStack {
            pub fn new(item: impl Into<Arc<str>>, count: u32) -> Self { ItemStack { item: item.into(), count, damage: 0 } }
            pub fn same_item_same_components(&self, other: &Self) -> bool { self.item == other.item }
        }
        pub struct ItemInfo { pub max_stack: u32 }
        pub struct Items;
        impl Items { pub fn get(&self, _id: &str) -> Option<&ItemInfo> { None } }
    }
    pub mod containers {
        use crate::items::ItemStack;
        use horizon_world::nbt::Compound;
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct ModBlockEntityFields { pub id: String, pub nbt: Compound, pub items: Vec<Option<ItemStack>>, pub data: Vec<i32> }
        pub struct ModBlockEntity { fields: ModBlockEntityFields }
        impl std::ops::Deref for ModBlockEntity { type Target = ModBlockEntityFields; fn deref(&self) -> &Self::Target { &self.fields } }
        impl std::ops::DerefMut for ModBlockEntity { fn deref_mut(&mut self) -> &mut Self::Target { &mut self.fields } }
        #[derive(Clone, Debug, Default, PartialEq, Eq)]
        pub struct ModContainer { pub slots: usize, pub faces: [Vec<usize>; 6], pub inputs: Vec<usize>, pub outputs: Vec<usize>, pub menu: Option<String> }
    }
    pub mod hud { pub struct HudBatch; }

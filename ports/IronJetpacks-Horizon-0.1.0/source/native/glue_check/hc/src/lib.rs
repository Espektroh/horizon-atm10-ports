//! Stub of the exact signatures of horizon_client (lib) used by the glue (66f3981).
pub mod items {
    use horizon_world::nbt::Tag;
    use std::sync::Arc;
    #[derive(Clone, Debug, Default)]
    pub struct Wire(Option<Arc<()>>);
    impl PartialEq for Wire { fn eq(&self, _: &Self) -> bool { true } }
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct StackData {
        pub custom_name: Option<Tag>,
        pub container: Vec<(u8, ItemStack)>,
        pub other: Vec<(String, Tag)>,
        pub wire: Wire,
    }
    impl StackData {
        pub fn other(&self, id: &str) -> Option<&Tag> { self.other.iter().find(|(k, _)| k == id).map(|(_, v)| v) }
        pub fn set_other(&mut self, id: &str, value: Option<Tag>) { self.other.retain(|(k, _)| k != id); if let Some(v) = value { self.other.push((id.to_owned(), v)); } }
        pub fn is_empty(&self) -> bool { *self == Self::default() }
    }
    #[derive(Clone, Debug, PartialEq)]
    pub struct ItemStack { pub item: Arc<str>, pub count: u32, pub damage: u32, pub extra: Option<Box<StackData>> }
    impl ItemStack {
        pub fn new(item: &str, count: u32) -> Self { Self { item: Arc::from(item), count, damage: 0, extra: None } }
        pub fn data(&self) -> Option<&StackData> { self.extra.as_deref() }
        pub fn data_mut(&mut self) -> &mut StackData { self.extra.get_or_insert_with(Box::default) }
        pub fn tidy(&mut self) { if self.extra.as_ref().is_some_and(|d| d.is_empty()) { self.extra = None; } }
    }
}
pub mod text {
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct Text(pub String);
    impl Text {
        pub fn literal(s: impl Into<String>) -> Self { Self(s.into()) }
        pub fn legacy(s: &str) -> Self { Self(s.to_owned()) }
    }
}
pub mod first_person {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Hand { Main, Off }
}
pub mod hud {
    #[derive(Default)]
    pub struct HudBatch { pub fills: Vec<([f32; 4], u32)>, pub texts: Vec<(String, [f32; 2], u32)> }
}
pub mod extras {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Corner { TopLeft, TopRight, BottomLeft, BottomRight }
}
pub mod extras_hud {
    use crate::extras::Corner;
    pub struct Corners { pub width: f32, pub height: f32 }
    impl Corners {
        pub fn place(&mut self, corner: Corner, w: f32, h: f32) -> (f32, f32) {
            let x = match corner { Corner::TopLeft | Corner::BottomLeft => 2.0, _ => self.width - 2.0 - w };
            let y = match corner { Corner::TopLeft | Corner::TopRight => 2.0, _ => self.height - 2.0 - h };
            (x, y)
        }
    }
}
pub mod containers {
    use horizon_world::nbt::Compound;
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct ModBlockEntityFields { pub id: String, pub nbt: Compound, pub items: Vec<Option<crate::items::ItemStack>>, pub data: Vec<i32> }
    pub struct ModBlockEntity { fields: ModBlockEntityFields }
    impl ModBlockEntity { pub fn new(fields: ModBlockEntityFields) -> Self { Self { fields } } }
    impl std::ops::Deref for ModBlockEntity { type Target = ModBlockEntityFields; fn deref(&self) -> &ModBlockEntityFields { &self.fields } }
    impl std::ops::DerefMut for ModBlockEntity { fn deref_mut(&mut self) -> &mut ModBlockEntityFields { &mut self.fields } }
}
pub mod geometry {
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Vec3 { pub x: f64, pub y: f64, pub z: f64 }
}
pub mod motion {
    #[derive(Clone, Debug, Default)]
    pub struct PlayerFluids { pub was_touching_water: bool }
    #[derive(Clone, Debug, Default)]
    pub struct PlayerMotion {
        pub velocity: crate::geometry::Vec3,
        pub ground: horizon_world::ground_support::GroundState,
        pub fluids: PlayerFluids,
        pub fall_distance: f64,
        pub fall_flying: bool,
        pub spectator: bool,
        pub sprinting: bool,
        pub yaw: f32,
    }
}
pub mod lan {
    pub mod guest {
        pub struct Guest { pub conn: u32, pub uuid: u128, pub above_ground_ticks: i32, pub placed: bool }
    }
    pub mod hub {
        pub struct Hub { pub guests: Vec<super::guest::Guest> }
    }
}
pub mod mod_api {
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct NativeMod { pub id: String, pub title: String, pub description: String, pub keys: Vec<[String; 3]> }
    #[derive(Clone)] pub struct NativeServerPlayer { pub uuid: u128, pub name: String, pub permission: u8, pub dimension: String, pub position: [f64; 3], pub game_mode: i32, pub health: f32, pub yaw: f32, pub pitch: f32, pub entity_id: i32, pub address: String }
}

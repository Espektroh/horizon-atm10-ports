#![allow(dead_code, unused)]
// crates/horizon-assets/src/item_model.rs:159
pub mod item_model {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum DisplayContext {
        None,
        ThirdPersonLeftHand,
        ThirdPersonRightHand,
        FirstPersonLeftHand,
        FirstPersonRightHand,
        Head,
        Gui,
        Ground,
        Fixed,
        OnShelf,
    }
}

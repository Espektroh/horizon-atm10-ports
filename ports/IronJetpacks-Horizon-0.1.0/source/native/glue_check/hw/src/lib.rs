//! Stub of the exact signatures of horizon_world used by the glue (66f3981).
pub mod nbt {
    use std::collections::BTreeMap;
    #[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct JavaString(pub Vec<u16>);
    impl From<&str> for JavaString { fn from(v: &str) -> Self { Self(v.encode_utf16().collect()) } }
    impl JavaString { pub fn to_string_lossy(&self) -> String { String::from_utf16_lossy(&self.0) } }
    pub type Compound = BTreeMap<JavaString, Tag>;
    #[derive(Clone, Debug, PartialEq)]
    pub enum Tag { End, Byte(i8), Short(i16), Int(i32), Long(i64), Float(u32), Double(u64), ByteArray(Vec<u8>), String(JavaString), List(Vec<Tag>), Compound(Compound), IntArray(Vec<i32>), LongArray(Vec<i64>) }
    impl Tag {
        pub fn get(&self, key: &str) -> Option<&Tag> { match self { Self::Compound(m) => m.get(&JavaString::from(key)), _ => None } }
    }
}
pub mod ground_support {
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct GroundState { pub on_ground: bool, pub horizontal_collision: bool, pub on_ground_no_blocks: bool }
}

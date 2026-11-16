#![allow(dead_code, unused)]
// Stand-ins with the shapes of Horizon 66f3981 (crates/horizon-world/src/nbt.rs:6,35,42;
// registry.rs:16). Only a type check.
pub mod nbt {
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct JavaString(pub Vec<u16>);
    impl From<&str> for JavaString {
        fn from(s: &str) -> Self {
            JavaString(s.encode_utf16().collect())
        }
    }
    impl JavaString {
        pub fn to_string_lossy(&self) -> String {
            String::from_utf16_lossy(&self.0)
        }
    }
    pub type Compound = std::collections::BTreeMap<JavaString, Tag>;
    #[derive(Clone, Debug, PartialEq)]
    pub enum Tag {
        End,
        Byte(i8),
        Short(i16),
        Int(i32),
        Long(i64),
        Float(u32),
        Double(u64),
        ByteArray(Vec<u8>),
        String(JavaString),
        List(Vec<Tag>),
        Compound(Compound),
        IntArray(Vec<i32>),
        LongArray(Vec<i64>),
    }
}
pub mod registry {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BlockStateId(pub u32);
}

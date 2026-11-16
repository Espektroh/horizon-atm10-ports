#![allow(dead_code, unused)]

    pub mod nbt {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct JavaString(pub Vec<u16>);
        impl From<&str> for JavaString { fn from(s: &str) -> Self { JavaString(s.encode_utf16().collect()) } }
        pub type Compound = std::collections::BTreeMap<JavaString, Tag>;
        #[derive(Clone, Debug, PartialEq)]
        pub enum Tag { Byte(i8), Short(i16), Int(i32), Long(i64), Float(f32), Double(f64), String(JavaString) }
    }
    pub mod registry {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct BlockStateId(pub u32);
        pub struct Registry;
        impl Registry { pub fn default_state(&self, _name: &str) -> Option<BlockStateId> { None } }
    }

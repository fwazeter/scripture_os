use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// Newtype for Work Registry (e.g., 786 = Quran)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkID(pub i32);

/// Newtype for Macro Level (e.g., Surah or Book)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroID(pub i32);

/// Newtype for Namespace (e.g., 0x02 = Root Anchor, 205 = Sahih)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamespaceID(pub i16);

/// The Sub bitmask representing state (e.g., Logical Anchor, RTL/LTR)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubMask(pub u16);

impl SubMask {
    pub const LOGICAL_ANCHOR: u16 = 0x0001;
    pub const RTL: u16 = 0x0002;
    pub const STRUCTURAL_MARKER: u16 = 0x0004;
}

/// The 5-part Universal Coordinate for Fractal Semantic Indexing v4.0
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    pub work: WorkID,
    pub macro_level: MacroID,
    pub namespace: NamespaceID,
    pub sub_mask: SubMask,
    pub lex_key: SmallVec<[u8; 16]>,
}

/// Represents the actual text row and its semantic FSI address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptureAtom {
    pub coordinate: Coordinate,
    pub text_content: String,
}
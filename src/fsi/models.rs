//! # FSI Domain Models (The "Stable Bottom")
//!
//! ### Architectural Design Decision: Fractional Scripture Index (FSI)
//! This module defines the absolute "DNA" of Scripture OS. By wrapping primitive types
//! into Rust NewTypes, we ensure compile-time safety across all engines and repository
//! implementations.

use serde::{Deserialize, Serialize};
use std::fmt;

/// ## `WorkID`
/// **Type:** `i32` (PostgreSQL `INTEGER`)
///
/// ### Architectural Design Decision: Type Safety over Primitives
/// Wraps the macro structural division to prevent accidental swapping with NamespaceIDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkID(pub i32);

/// ## `MacroID`
/// **Type:** `i32`
///
/// ### Architectural Design Decision: Structural Division
/// Represents a macro structural division (e.g., Chapter 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MacroID(pub i32);

/// ## `NamespaceID`
/// **Type:** `i32`
///
/// ### Architectural Design Decision: Taxonomy Identification
/// The taxonomy identifier for the language/translation block (e.g., 1000 for Arabic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamespaceID(pub i32);

/// ## `LexiconID`
/// **Type:** `i64` (PostgreSQL `BIGINT`)
///
/// ### Architectural Design Decision: Universal Dictionary Pointer
/// Points to the exact lexical entry. Uses `i64` to accommodate billions of unique words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LexiconID(pub i64);

/// ## `SubMask`
/// **Type:** `i16` (PostgreSQL `SMALLINT`)
///
/// ### Architectural Design Decision: Structural Metadata
/// Defines the structural role of the text (e.g., 1 for Anchor/Skeleton, 0 for Translation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubMask(pub i16);

/// ## `LexKey`
/// **Type:** `String`
///
/// ### Architectural Design Decision: Deep Fractional Nesting
/// The horizontal position of the word (e.g., "00001", "00001.a"). Kept as a string
/// to easily support infinitely deep insertions without re-indexing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LexKey(pub String);

/// ## `Coordinate`
///
/// ### Architectural Design Decision: 3D Mapping
/// Groups the "Where" into a cohesive struct. Represents exactly where an atom lives
/// in the multi-dimensional scroll.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Coordinate {
    pub work_id: WorkID,
    pub macro_id: MacroID,
    pub lex_key: LexKey,
}

impl Coordinate {
    /// ## `to_path_string`
    /// **Parameters:** `&self`
    ///
    /// ### Architectural Design Decision: Path Generation
    /// Helper to cleanly format the coordinate as a string (e.g., "786.1.00001.a")
    /// for UI routing or API responses.
    pub fn to_path_string(&self) -> String {
        format!("{}.{}.{}", self.work_id.0, self.macro_id.0, self.lex_key.0)
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_path_string())
    }
}

/// ## `ScriptureAtom`
///
/// ### Architectural Design Decision: Atomic Content Binding
/// Maps directly to a row in the data layer. Represents a single linguistic unit bound
/// to its coordinate and cryptographic signature.
///
/// **AI Prompt Hint:** Do not modify the `merkle_hash` type without updating the
/// `fsi_crypto` module mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptureAtom {
    pub coordinate: Coordinate,
    pub namespace_id: NamespaceID,
    pub lexicon_id: LexiconID,
    pub sub_mask: SubMask,
    pub merkle_hash: Vec<u8>,
}

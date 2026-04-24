//! # Lexicon Domain Models (The "Typesetter's Drawer")
//!
//! ### Architectural Design Decision: Domain Encapsulation
//! This module defines the atomic molds for language strings and their universal
//! conceptual mappings across Scripture OS.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ## `LexiconID`
/// **Type:** `i64` (PostgreSQL `BIGINT`)
///
/// ### Architectural Design Decision: Type Safety over Primitives
/// Wraps the 64-bit integer to prevent accidental swapping with a `WorkID` or `NamespaceID`.
/// This is the precise pointer to the physical "sound" mold in the drawer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LexiconID(pub i64);

/// ## `ConceptID`
/// **Type:** `String` (ISO 639-1)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LanguageCode(pub String);

/// ## `LexiconEntry`
///
/// ### Architectural Design Decision: Sound and Morphological Segregation
/// Represents exactly one row in the `lexicon_entries` table. `morphology` is deferred
/// to a JSONB blob so that Arabic root-stems and Greek lemma-logic do not force schema changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconEntry {
    pub id: LexiconID,
    pub lang: LanguageCode,
    pub body: String,
    pub concept_id: Option<ConceptID>,
    pub morphology: Option<serde_json::Value>,
}

/// ## `RegistryConcept`
///
/// ### Architectural Design Decision: The Idea Map
/// Represents the universal registry. Separates the human-readable label from external
/// academic thesaurus references (like Strong's Concordance).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConcept {
    pub concept_id: ConceptID,
    pub primary_label: String,
    pub thesaurus_refs: Option<serde_json::Value>,
}

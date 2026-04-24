//! # Lexicon Domain (The "Typesetter's Drawer")
//!
//! ### Architectural Design Decision: Domain Encapsulation
//! Everything required to manage the universal dictionary of strings is isolated
//! here. External modules (like FSI or Content) interact strictly through the
//! exported Engine and Models, entirely oblivious to the underlying database logic.
pub mod engine;
pub mod models;
pub mod repository;

// Re-export the crucial public interfaces
pub use engine::CoreLexiconEngine;
pub use models::{ConceptID, LanguageCode, LexiconID};

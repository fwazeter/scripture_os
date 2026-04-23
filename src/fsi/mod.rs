//! # FSI Domain (The "Stable Bottom")
//!
//! ### Architectural Design Decision: Domain Encapsulation
//! This module manages the core identity of the system. It is the only module
//! allowed to define the structure of a ScriptureAtom.
pub mod lex_key;
pub mod models;

// Re-export core types for easier access from other domains
pub use models::*;

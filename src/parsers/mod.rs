//! # Parsers Domain
//!
//! ### Architectural Design Decision: Strategy Pattern for Ingestion
//! This module defines the `ScriptureParser` trait. By separating the
//! *format* //! (e.g., CSV, JSON, Uthmani Text) from the *ingestion logic*, we keep the
//! Ingestion Engine purely focused on database orchestration and cryptographic hashing.
pub mod quran;

use crate::fsi::models::{Coordinate, NamespaceID};
use crate::utils::errors::ScriptureError;

/// A DTO representing raw text extracted from a source file before it is
/// mapped to a dictionary/lexicon ID and hashed.
pub struct ParsedEntry {
    pub coordinate: Coordinate,
    pub text: String,
    pub namespace_id: NamespaceID,
}

/// ## `ScriptureParser`
///
/// ### Architectural Design Decision: Pluggable Formats
/// Any new scriptural format (like USFM or OSIS) simply needs to implement
/// this trait to be natively ingested by Scripture OS.
pub trait ScriptureParser: Send + Sync {
    /// Translates raw string data into a universal vector of parsed entries.
    fn parse(&self, raw_content: &str) -> Result<Vec<ParsedEntry>, ScriptureError>;
}

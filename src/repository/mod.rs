//! # Repository Domain (The "Data Access Layer")
//!
//! ### Architectural Design Decision: Contract-First Development
//! This module defines the `ScriptureRepository` trait. By defining this
//! contract before implementation, we decouple our business logic (Engines)
//! from the physical storage (Postgres).
pub mod lexicon_repo;
pub mod mock;
pub mod postgres;

use crate::fsi::models::{Coordinate, LexiconID, ScriptureAtom};
use crate::utils::errors::ScriptureError;
use async_trait::async_trait;

/// ## `ScriptureRepository`
///
/// ### Architectural Design Decision: Trait-Based Persistence
/// All data operations must go through this trait. This allows us to
/// switch between Postgres and Mock repositories without touching
/// engine logic.
#[async_trait]
pub trait ScriptureRepository: Send + Sync {
    /// Fetches a single atom by its 3D FSI coordinate.
    async fn get_atom_by_coordinate(
        &self,
        coord: Coordinate,
    ) -> Result<ScriptureAtom, ScriptureError>;

    /// Looks up an FSI coordinate based on a human-readable alias (e.g., "quran.1.1").
    async fn resolve_alias(&self, path_string: &str) -> Result<Coordinate, ScriptureError>;

    /// Fetches the immediately following logical atom in the sequence
    async fn get_next_atom(&self, current: Coordinate) -> Result<ScriptureAtom, ScriptureError>;

    /// Inserts raw text into the universal dictionary and returns its unique pointer.
    async fn insert_lexicon_entry(&self, text: &str) -> Result<LexiconID, ScriptureError>;

    /// Batch inserts a slice of ScriptureAtoms into the FSI structural spine.
    async fn insert_atoms(&self, atoms: &[ScriptureAtom]) -> Result<(), ScriptureError>;

    /// Fetches the raw text string from the dictionary using its ID.
    async fn get_lexicon_text(&self, lexicon_id: LexiconID) -> Result<String, ScriptureError>;
}

/// A thread-safe type alias for injecting the repository into engines.
pub type SharedScriptureRepository = std::sync::Arc<dyn ScriptureRepository + Send + Sync>;

//! # Repository Domain (The "Data Access Layer")
//!
//! ### Architectural Design Decision: Contract-First Development
//! This module defines the `ScriptureRepository` trait. By defining this
//! contract before implementation, we decouple our business logic (Engines)
//! from the physical storage (Postgres).
pub mod mock;
pub mod postgres;

use crate::fsi::models::{ScriptureAtom, WorkID};
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
    /// Fetches a single atom by its absolute FSI coordinate.
    async fn get_atom_by_coordinate(
        &self,
        cood: Coordinate,
    ) -> Result<ScriptureAtom, ScriptureError>;

    /// Fetches a range of atoms (e.g. a chapter) for a specific work and namespace.
    async fn get_atom_range(
        &self,
        work: crate::fsi::models::WorkID,
        ns: crate::fsi::models::NamespaceID,
    ) -> Result<Vec<ScriptureAtom>, ScriptureError>;
}

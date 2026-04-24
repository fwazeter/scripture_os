//! # Traversal Engine (The "Guide")
//!
//! ### Architectural Design Decision: Relational Navigation
//! This engine provides the logic to move sequentially through the text,
//! entirely decoupled from the actual content assembly.

use crate::fsi::models::{Coordinate, ScriptureAtom};
use crate::repository::ScriptureRepository;
use crate::utils::errors::ScriptureError;
use std::sync::Arc;

/// ## `CoreTraversalEngine`
///
/// ### Architectural Design Decision: Dependency Injection (DI)
pub struct CoreTraversalEngine {
    repo: Arc<dyn ScriptureRepository + Send + Sync>,
}

impl CoreTraversalEngine {
    pub fn new(repo: Arc<dyn ScriptureRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    /// ## `get_next_atom`
    /// **Parameters:** `current_coord: Coordinate`
    ///
    /// ### Architectural Design Decision: Sequential Movement
    /// Identifies the absolute next logical atom in the sequence, crossing
    /// chapter boundaries if necessary.
    pub async fn get_next_atom(
        &self,
        current_coord: Coordinate,
    ) -> Result<ScriptureAtom, ScriptureError> {
        self.repo.get_next_atom(current_coord).await
    }
}

// ==========================================
// DUAL-TRACK VERIFICATION
// ==========================================
#[cfg(test)]
mod tests {}

#[cfg(test)]
mod mock_tests {}

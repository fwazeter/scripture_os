//! # Content Engine (The "Assembler")
//!
//! ### Architectural Design Decision: Decoupling Storage from Assembly
//! This module is responsible for assembling `ScriptureAtom` records into readable structures.
//! It depends purely on the `ScriptureRepository` trait to allow for database mocking.

use crate::fsi::models::{Coordinate, ScriptureAtom};
use crate::repository::ScriptureRepository;
use crate::utils::errors::ScriptureError;
use std::sync::Arc;

/// ## `CoreContentEngine`
///
/// ### Architectural Design Decision: Dependency Injection (DI)
/// This struct encapsulates a thread-safe `Arc` to the repository, allowing
/// for modular state management and cross-thread safety in Axum.
pub struct CoreContentEngine {
    repo: Arc<dyn ScriptureRepository + Send + Sync>,
}

impl CoreContentEngine {
    pub fn new(repo: Arc<dyn ScriptureRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    /// ## `fetch_atom`
    /// **Parameters:** `coordinate: Coordinate` (The exact FSI address).
    ///
    /// ### Architectural Design Decision: Atomic Retrieval
    /// Fetches a single atom. This acts as the foundational method for building
    /// larger text spans.
    ///
    /// **AI Prompt Hint:** Always handle the `ScriptureError::NotFound` explicitly here.
    pub async fn fetch_atom(
        &self,
        coordinate: Coordinate,
    ) -> Result<ScriptureAtom, ScriptureError> {
        self.repo.get_atom_by_coordinate(coordinate).await
    }
}

// ==========================================
// DUAL-TRACK VERIFICATION
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;
    // Concrete integration tests interacting with the real PostgreSQL implementation
    // via test_utils::setup_db() would go here.
}

#[cfg(test)]
mod mock_tests {
    use super::*;
    // Isolated Mock Tests testing business logic in total isolation
    // by injecting a `struct MockRepository` would go here.
}

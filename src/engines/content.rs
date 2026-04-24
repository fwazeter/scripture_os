//! # Content Engine (The "Assembler")
//!
//! ### Architectural Design Decision: Decoupling Storage from Assembly
//! This module is responsible for assembling `ScriptureAtom` records and their
//! associated dictionary texts into readable structures.

use crate::fsi::models::{Coordinate, ScriptureAtom};
use crate::repository::SharedScriptureRepository;
use crate::utils::errors::ScriptureError;
use serde::Serialize;

/// A Presentation DTO sent to the API Gateway
#[derive(Serialize)]
pub struct ReadableVerse {
    pub coordinate: String,
    pub text: String,
}

/// ## `CoreContentEngine`
///
/// ### Architectural Design Decision: Dependency Injection (DI)
/// This struct encapsulates a thread-safe `Arc` to the repository, allowing
/// for modular state management and cross-thread safety in Axum.
pub struct CoreContentEngine {
    repo: SharedScriptureRepository,
}

impl CoreContentEngine {
    pub fn new(repo: SharedScriptureRepository) -> Self {
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

    /// ## `fetch_readable_verse`
    /// **Parameters:** `coordinate: Coordinate`
    ///
    /// ### Architectural Design Decision: Assembly
    /// Fetches the atom from the scroll, then immediately looks up its dictionary text,
    /// combining them into a single presentation-ready struct.
    pub async fn fetch_readable_verse(
        &self,
        coordinate: Coordinate,
    ) -> Result<ReadableVerse, ScriptureError> {
        // 1. Get the exact FSI DNA
        let atom = self.repo.get_atom_by_coordinate(coordinate.clone()).await?;

        // 2. Resolve the Lexicon pointer to actual text.
        let text = self.repo.get_lexicon_text(atom.lexicon_id).await?;

        // 3. Assemble for the user
        Ok(ReadableVerse {
            coordinate: coordinate.to_path_string(),
            text,
        })
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

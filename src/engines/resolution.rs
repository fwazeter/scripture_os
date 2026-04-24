//! # Resolution Engine (The "Router")
//!
//! ### Architectural Design Decision: Human to Machine Translation
//! This engine acts as the boundary between human-readable shorthands and
//! the strict mathematical boundaries of the FSI domain.

use crate::fsi::models::Coordinate;
use crate::repository::ScriptureRepository;
use crate::utils::errors::ScriptureError;
use std::sync::Arc;

/// ## `CoreResolutionEngine`
///
/// ### Architectural Design Decision: Dependency Injection (DI)
/// Encapsulates the repository to perform alias lookups safely
pub struct CoreResolutionEngine {
    repo: Arc<dyn ScriptureRepository + Send + Sync>,
}

impl CoreResolutionEngine {
    pub fn new(repo: Arc<dyn ScriptureRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    /// ## `resolve_path`
    /// **Parameters:** `input_string: &str`
    ///
    /// ### Architectural Design Decision: Path Resolution
    /// Translates a string path (e.g., "quran.1.1") into an absolute `Coordinate`.
    ///
    /// **AI Prompt Hint:** Future iterations should include Regex parsing here to
    /// handle conversational inputs like "John 3:16".
    pub async fn resolve_path(&self, input_string: &str) -> Result<Coordinate, ScriptureError> {
        // In a full implementation, parsing / regex logic goes here before hitting the repo.
        let sanitized_input = input_string.trim().to_lowercase();
        self.repo.resolve_alias(&sanitized_input).await
    }
}

// ==========================================
// DUAL-TRACK VERIFICATION
// ==========================================
#[cfg(test)]
mod tests {}

#[cfg(test)]
mod mock_tests {}

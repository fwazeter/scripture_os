//! # Ingestion Engine (The "Seeder")
//!
//! ### Architectural Design Decision: Decoupled Orchestration
//! This engine takes ANY format (via `ScriptureParser`) and ANY database
//! (via `SharedScriptureRepository`), binds them together with `blake3` cryptography,
//! and safely persists them.

use crate::fsi::models::{ScriptureAtom, SubMask};
use crate::parsers::ScriptureParser;
use crate::repository::SharedScriptureRepository;
use crate::utils::errors::ScriptureError;
use std::sync::Arc;

/// ## `CoreIngestionEngine`
///
/// ### Architectural Design Decision: Dependency Injection
pub struct CoreIngestionEngine {
    repo: SharedScriptureRepository,
}

impl CoreIngestionEngine {
    pub fn new(repo: SharedScriptureRepository) -> Self {
        Self { repo }
    }

    /// ## `ingest_file`
    /// **Parameters:** `raw_content: &str`, `parser: Arc<dyn ScriptureParser>`
    ///
    /// ### Architectural Design Decision: Dynamic Ingestion
    /// By passing the parser as a dynamic trait, this single engine method can ingest
    /// Quran, Bible, or Commentary files without changing a line of internal logic.
    pub async fn ingest_file(
        &self,
        raw_content: &str,
        parser: Arc<dyn ScriptureParser + Send + Sync>,
    ) -> Result<(), ScriptureError> {
        // 1. Parse format specific string into our generic DTOs
        let entries = parser.parse(raw_content)?;
        let mut atoms_to_insert = Vec::new();

        for entry in entries {
            // 2. Generate the Cryptographic Merkle Hash (BLAKE3) of the text
            let hash = blake3::hash(entry.text.as_bytes());
            let merkle_hash = hash.as_bytes().to_vec();

            // 3. Save the actual string to the Universal Dictionary
            let lexicon_id = self.repo.insert_lexicon_entry(&entry.text).await?;

            // 4. Construct the pure FSI Atom
            let atom = ScriptureAtom {
                coordinate: entry.coordinate,
                namespace_id: entry.namespace_id,
                lexicon_id,
                sub_mask: SubMask(0), // Defaulting to 0 (Translation/Standard Text)
                merkle_hash,
            };

            atoms_to_insert.push(atom);
        }

        // 5. Batch persist the FSI spine addresses
        self.repo.insert_atoms(&atoms_to_insert).await?;

        Ok(())
    }
}

// ==========================================
// DUAL-TRACK VERIFICATION
// ==========================================
#[cfg(test)]
mod tests {}

#[cfg(test)]
mod mock_tests {}

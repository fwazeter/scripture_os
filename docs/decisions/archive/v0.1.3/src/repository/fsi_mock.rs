use async_trait::async_trait;
use anyhow::Result;
use parking_lot:: RwLock;
use std::sync::Arc;

use crate::fsi::models::{ScriptureAtom, MacroID, NamespaceID, WorkID};
use crate::repository::fsi_repo::FsiRepository;

/// # Mock FSI Repository (Track B)
///
/// ### Architectural Design Decision: In-Memory Testing
/// Implements the `FsiRepository` contract using an in-memory `RwLock<Vec<ScriptureAtom>>`.
/// This allows us to test the Business Logic/Engines (Track B) in total isolation without
/// spinning up a PostgreSQL database.
///
/// **AI Prompt Hint:** When writing Engine mock tests, inject this struct populated with
/// hand-crafted `ScriptureAtom` data to simulate various FSI sequence scenarios.
pub struct MockFsiRepository {
    // Thread-safe internal state utilizing parking_lot as mandated by repo standards
    store:Arc<RwLock<Vec<ScriptureAtom>>>,
}

impl MockFsiRepository {
    // Creates an empty mock repository
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Helper to preload the repository with exact test state
    pub fn with_data(atoms: Vec<ScriptureAtom>) -> Self {
        Self {
            store: Arc::new(RwLock::new(atoms)),
        }
    }
}

#[async_trait]
impl FsiRepository for MockFsiRepository {
    async fn get_sequence(
        &self,
        work: WorkID,
        macro_level: MacroID,
        namespace: NamespaceID,
        start_lex: Option<&[u8]>,
        end_lex: Option<&[u8]>,
    ) -> Result<Vec<ScriptureAtom>> {
        let lock = self.store.read();

        // 1. Explicitly filter core IDs first
        let mut filtered: Vec<ScriptureAtom> = lock.iter()
            .filter(|a| {
                a.coordinate.work == work
                    && a.coordinate.macro_level == macro_level
                    && a.coordinate.namespace == namespace
            })
            .cloned()
            .collect();

        // 2. Safely retain based on bounds (if provided)
        if let Some(start) = start_lex {
            filtered.retain(|a| a.coordinate.lex_key.as_slice() >= start);
        }
        if let Some(end) = end_lex {
            filtered.retain(|a| a.coordinate.lex_key.as_slice() <= end);
        }

        // 3. Sort ascending by LexKey
        filtered.sort_by(|a, b| a.coordinate.lex_key.cmp(&b.coordinate.lex_key));

        Ok(filtered)
    }

    async fn insert_atoms(&self, atoms: Vec<ScriptureAtom>) -> Result<()> {
        let mut lock = self.store.write();

        /*for atom in atoms {
            // Basic check to loosely mimic the SQL UNIQUE constraint on the coordinate
            // This caused ~3 min test in mocks.
            let exists = lock.iter().any(|a| a.coordinate == atom.coordinate);
            if !exists {
                lock.push(atom);
            }
        }*/
        lock.extend(atoms);

        Ok(())
    }
}
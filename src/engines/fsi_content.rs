use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::fsi::models::{MacroID, NamespaceID, SubMask, WorkID};
use crate::repository::fsi_repo::FsiRepository;
use crate::engines::FsiContentEngine;

pub struct CoreContentEngine {
    repo: Arc<dyn FsiRepository + Send + Sync>,
}

impl CoreContentEngine {
    pub fn new(repo: Arc<dyn FsiRepository + Send + Sync>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl FsiContentEngine for CoreContentEngine {
    /// ## `assemble_macro`
    /// **Parameters:** `work: WorkID`, `macro_level: MacroID`, `namespace: NamespaceID`.
    ///
    /// ### Architectural Design Decision: Lexical Assembly
    /// FSI v4.0 stores words as isolated atoms. This function queries the continuous
    /// mathematical sequence of atoms for a specific container (like a Surah) and assembles them.
    /// It dynamically formats the output based on the `SubMask`
    /// (e.g., separating Ayahs with structural markers).
    ///
    /// **AI Prompt Hint:** Currently ojins words with spaces. Future iterations supporting
    /// languages like Thai (which do not use spaces) will need logic to detect the
    /// `WorkID` and alter the joining strategy accordingly.
    async fn assemble_macro(
        &self,
        work: WorkID,
        macro_level: MacroID,
        namespace: NamespaceID
    ) -> Result<String> {
        // 1. Fetch the raw mathematical sequence of words from the abstracte drepo
        let atoms = self.repo.get_sequence(work, macro_level, namespace, None, None).await?;

        let mut result = String::new();

        // 2. Assemble the atoms
        for atom in atoms {
            if (atom.coordinate.sub_mask.0 & SubMask::STRUCTURAL_MARKER) != 0 {
               result.push_str(&format!("\n[{}] ", atom.text_content));
            } else {
                result.push_str(&atom.text_content);
                result.push(' ');
            }
        }

        Ok(result.trim_end().to_string())
    }
}

// ==========================================
// Dual-Track Verification: Track B (Mocks)
// ==========================================
#[cfg(test)]
mod mock_tests {
    use super::*;
    use crate::fsi::models::{Coordinate, ScriptureAtom};
    use crate::fsi::lex_key::generate_sequential;
    use crate::repository::fsi_mock::MockFsiRepository;

    #[tokio::test]
    async fn test_assemble_macro_mock() {
        // 1. Arrange: Build fake FSI atoms
        let w_id = WorkID(786);
        let m_id = MacroID(1);
        let n_id = NamespaceID(0x02);

        let atom1 = ScriptureAtom {
            coordinate: Coordinate {
                work: w_id, macro_level: m_id, namespace: n_id,
                sub_mask: SubMask(SubMask::LOGICAL_ANCHOR),
                lex_key: generate_sequential(0),
            },
            text_content: "بِسْمِ".to_string(),
        };

        let atom2 = ScriptureAtom {
            coordinate: Coordinate {
                work: w_id, macro_level: m_id, namespace: n_id,
                sub_mask: SubMask(SubMask::LOGICAL_ANCHOR),
                lex_key: generate_sequential(1),
            },
            text_content: "ٱللَّهِ".to_string(),
        };

        // Inject the mock data into the Mock Repository
        let mock_repo = Arc::new(MockFsiRepository::with_data(vec![atom1, atom2]));

        // Inject the Mock Repository into the CoreContentEngine
        let engine = CoreContentEngine::new(mock_repo);

        // 2. Act
        let result = engine.assemble_macro(w_id, m_id, n_id).await.unwrap();

        // 3. Assert
        assert_eq!(result, "بِسْمِ ٱللَّهِ");
    }
}
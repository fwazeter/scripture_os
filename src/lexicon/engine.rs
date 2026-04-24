//! # Lexicon Engine (The "Gated Vault")
//!
//! ### Architectural Design Decision: Anti-Corruption Layer
//! External domains cannot query the lexicon tables directly. This engine enforces
//! idempotency, tokenization rules, and secure ID resolution.
use crate::lexicon::repository::SharedLexiconRepository;
use crate::lexicon::{LanguageCode, LexiconID};
use crate::utils::errors::ScriptureError;

// Assume WorkID is defined in core models
use crate::fsi::models::WorkID;

/// ## `CoreLexiconEngine`
///
/// ### Architectural Design Decision: Dependency Injection (DI)
/// Encapsulates a thread-safe `Arc` to the repository.
pub struct CoreLexiconEngine {
    repository: SharedLexiconRepository,
}

impl CoreLexiconEngine {
    pub fn new(repository: SharedLexiconRepository) -> Self {
        Self { repository }
    }

    /// ## `register_text`
    /// **Parameters:** `work_id: WorkID`, `lang: LanguageCode`, `text_block: &str`
    ///
    /// ### Architectural Design Decision: Pipeline Orchestration
    /// Acts as the ingestion gateway. Splits large blocks of scripture into tokens,
    /// feeds them sequentially into the idempotent lexicon, and returns the sequential
    /// molds (LexiconIDs) required by the H-FSI scroll.
    ///
    /// **AI Prompt Hint:** Tokenization logic is heavily language-dependent. In the future,
    /// inject a `dyn TokenizerStrategy` rather than hardcoding string splitting here.
    pub async fn register_text(
        &self,
        _work_id: WorkID,
        lang: LanguageCode,
        text_block: &str,
    ) -> Result<Vec<LexiconID>, ScriptureError> {
        // Step 1: Tokenization (Naively split by whitespace for V1)
        // Future enhancement: use language-specific Tokenizer injected via trait
        let tokens: Vec<&str> = text_block.split_whitespace().collect();
        let mut lexicon_ids = Vec::with_capacity(tokens.len());

        // Step 2 & 3: Lexicon Check & Binding
        for token in tokens {
            let id = self.insert_lexicon_entry(token, &lang).await?;
            lexicon_ids.push(id);
        }

        Ok(lexicon_ids)
    }

    /// ## `insert_lexicon_entry`
    /// **Parameters:** `text: &str`, `lang: &LanguageCode`
    ///
    /// ### Architectural Design Decision: Idempotent Insertion
    /// Enforces the uniqueness of the "Typesetter's Mold". Wraps the repository call
    /// which must implement the `INSERT ... ON CONFLICT DO NOTHING` CTE pattern.
    pub async fn insert_lexicon_entry(
        &self,
        text: &str,
        lang: &LanguageCode,
    ) -> Result<LexiconID, ScriptureError> {
        self.repository.insert_lexicon_entry(text, lang).await
    }

    /// ## `get_lexicon_text`
    /// **Parameters:** `id: LexiconID`
    ///
    /// ### Architectural Design Decision: Atomic Resolution
    /// Used heavily by the Assembly/Content Engine to turn FSI pointers back into
    /// human-readable presentations.
    pub async fn get_lexicon_text(&self, id: LexiconID) -> Result<String, ScriptureError> {
        self.repository.get_lexicon_text(id).await
    }
}

// ==========================================
// DUAL-TRACK VERIFICATION
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;
    // Concrete integration tests interacting with the real PostgreSQL implementation
    // utilizing CTE ON CONFLICT logic via test_utils::setup_db() go here.
}

#[cfg(test)]
mod mock_tests {
    use super::*;
    // Isolated Mock Tests verifying the tokenization pipeline and idempotent
    // orchestrations using a `struct MockLexiconRepository` go here.
}

use crate::lexicon::{LanguageCode, LexiconID};
use crate::utils::errors::ScriptureError;
use async_trait::async_trait;

/// ## `LexiconRepository`
///
/// ### Architectural Design Decision: Storage Abstraction
/// Defines the data access contract for the Typesetter's Drawer. This ensures the engine
///can seamlessly swap between PostgreSQL (production) and Mock Hashes (testing).
#[async_trait]
pub trait LexiconRepository: Send + Sync {
    /// Inserts a text entry idempotently (INSERT ... ON CONFLICT DO NOTHING)
    async fn insert_lexicon_entry(
        &self,
        text: &str,
        lang: &LanguageCode,
    ) -> Result<LexiconID, ScriptureError>;

    /// Resolves a drawer ID back into a legible string
    async fn get_lexicon_text(&self, id: LexiconID) -> Result<String, ScriptureError>;
}

// Type alias for Dependency Injection
pub type SharedLexiconRepository = std::sync::Arc<dyn LexiconRepository>;

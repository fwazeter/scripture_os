//! # Content Engine (The "Assembler")
//!
//! This module acts as the orchestrator for text retrieval. It is designed
//! to be completely agnostic of the database implementation by relying
//! on the `ScriptureRepository` trait.
//!
//! ### Architectural Design Decision: Stand-off Markup Bridge
//! Scripture OS utilizes a Stand-off Markup architecture where text is stored
//! sequentially and is ignorant of hierarchical addresses.
//! The Content Engine bridges this gap by mapping `ltree` paths to sequence boundaries.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::models::ScriptureContent;
use crate::repository::ScriptureRepository;
use super::ContentEngine;

/// # Core Content Engine
///
/// This is the primary implementation of the `ContentEngine` trait.
///
/// ### Architectural Design Decison: Dependency Injection (DI)
/// Instead of requiring a repository reference to be passed into every standalone function,
/// this struct encapsulates the dependency. It holds a thread-safe, reference-counted
/// pointer (`Arc`) to any type that implements `ScriptureRepository`.
///
/// This ensures the engine can be injected into Axum `AppState` and safely shared
/// across multiple concurrent web requests.
pub struct CoreContentEngine {
    repo: Arc<dyn ScriptureRepository + Send + Sync>,
}

impl CoreContentEngine {
    /// Bootstraps the engine by injecting the required data layer repository
    pub fn new(repo: Arc<dyn ScriptureRepository + Send + Sync>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ContentEngine for CoreContentEngine {
    /// ## `fetch_text`
    /// **Parameters:** /// * `path: &str` (The canonical `ltree` address to fetch, e.g., "bible.nt.john.1.1").
    ///
    /// ### Architectural Design Decision: Sequence-to-Address Assembly
    /// This function solves the problem of retrieving content for a structural node that
    /// does not physically contain text. It translates a hierarchical "Address" into a
    /// contiguous "Range" of text segments.
    ///
    /// ### Design Decision: Two-Step Resolution Process
    /// 1. **Boundary Resolution:** The engine requests the start and end sequence indices
    ///    for the provided path from the injected repository (`self.repo`).
    /// 2. **Content Aggregation:** It then fetches every text segment falling within
    ///    that sequence range across all available editions.
    ///
    /// ### Technical Context: Polymorphic Retrieval
    /// Because this logic is range-based, the exact same function handles fetching a
    /// single verse, an entire chapter, or a whole book simply by providing an `ltree`
    /// path of varying depth.
    ///
    /// **AI Prompt Hint:** When modifying retrieval logic for "Parallel Reading" or
    /// "Side-by-Side" views, ensure results are ordered by `absolute_index ASC` to
    /// maintain reading flow, then by `is_source DESC` to prioritize original
    /// language manuscripts.
    async fn fetch_text(&self, path: &str) -> Result<Vec<ScriptureContent>> {
        // Delegates the specific range-finding and text-fetching logic to the repository.
        repo.fetch_text(path).await
    }
}


#[cfg(test)]
mod tests {
    use crate::repository::postgres::PostgresRepository;
    use super::*;

    #[tokio::test]
    async fn test_fetch_range_psalm_title() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        // 1. Initialize the concrete repository and wrap it in an Arc for DI
        let repo = Arc::new(PostgresRepository::new(&pool));

        // 2. Inject the repository into the Engine
        let engine = CoreContentEngine::new(repo);

        // 3. Call the trait method on the engine
        // Bible groups both title segments into one node (indices 1000 and 1001)
        let results = engine.fetch_text("bible.ot.psalms.51.title").await.unwrap();

        // Should return 6 rows: 2 indices (1000, 1001) * 3 Bible translations (KJV, NIV, LXX)
        assert_eq!(results.len(), 6);
    }
}
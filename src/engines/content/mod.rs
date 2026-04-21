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

use crate::models::{ScriptureContent, Comparison};
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
        self.repo.fetch_text(path).await
    }

    /// ## `get_comparison`
    /// ### Architectural Design Decision: Node-Centric Grouping
    /// Leverages the respoistory's `absolute_index` sorting to sequentially group
    /// flat `ScriptureContent` rows into structured `Comparison` blocks. This
    /// guarantees that translations of the exact same semantic unit stay locked together.
    async fn get_comparison(&self, path: &str) -> Result<Vec<Comparison>> {
        let contents = self.repo.fetch_text(path).await?;

        if contents.is_empty() {
            return Ok(vec![]);
        }

        let mut comparisons: Vec<Comparison> = Vec::new();

        // Group contiguous content rows that share the same node_id
        for content in contents {
            if let Some(comp) = comparisons.last_mut().filter(|c| c.node_id == content.node_id) {
                // We are already building a comparison for this node, add to it
                comp.contents.push(content);
            } else {
                // This is a new node, start a new Comparison block
                comparisons.push(Comparison {
                    node_id: content.node_id,
                    path: content.path.clone(),
                    contents: vec![content],
                });
            }
        }

        Ok(comparisons)
    }
}


// --- Track A: Concrete Integration Tests ---
#[cfg(test)]
mod tests {
    use crate::repository::postgres::PostgresRepository;
    use super::*;

    #[tokio::test]
    async fn test_fetch_range_psalm_title() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        let repo = Arc::new(PostgresRepository::new(pool));
        let engine = CoreContentEngine::new(repo);

        let results = engine.fetch_text("bible.ot.psalms.51.title").await.unwrap();
        assert_eq!(results.len(), 6);
    }

    // NEW: Integration test for get_comparison
    #[tokio::test]
    async fn test_get_comparison_groups_translations() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        let repo = Arc::new(PostgresRepository::new(pool));
        let engine = CoreContentEngine::new(repo);

        // Fetch John 17:3
        let comparisons = engine.get_comparison("bible.nt.john.17.3").await.unwrap();

        // There should be exactly 1 node (John 17:3)
        assert_eq!(comparisons.len(), 1);

        // That single node should contain 2 translations (KJV and SBLGNT)
        assert_eq!(comparisons[0].contents.len(), 2);
        assert_eq!(comparisons[0].path, "bible.nt.john.17.3");
    }
}

// --- Track B: Isolated Mock Tests ---
// NEW: Testing the business logic without hitting a real database
#[cfg(test)]
mod mock_tests {
    use super::*;
    use crate::models::{HierarchyNode, Adjacency};
    use uuid::Uuid;

    struct MockRepository;

    #[async_trait]
    impl ScriptureRepository for MockRepository {
        async fn fetch_text(&self, path: &str) -> Result<Vec<ScriptureContent>> {
            let node_id = Uuid::new_v4();
            // Return two fake rows for the same node to simulate translations
            Ok(vec![
                ScriptureContent {
                    node_id,
                    path: path.to_string(),
                    body_text: "Mock English".to_string(),
                    edition_name: "Mock_EN".to_string(),
                    language_code: "en".to_string(),
                    absolute_index: 100,
                    translation_metadata: None,
                },
                ScriptureContent {
                    node_id,
                    path: path.to_string(),
                    body_text: "Mock Greek".to_string(),
                    edition_name: "Mock_GR".to_string(),
                    language_code: "grc".to_string(),
                    absolute_index: 100,
                    translation_metadata: None,
                }
            ])
        }

        // Stub other required trait methods returning empty/default
        async fn get_hierarchy(&self, _p: &str) -> Result<Vec<HierarchyNode>> { Ok(vec![]) }
        async fn get_adjacent_nodes(&self, _id: Uuid) -> Result<Adjacency> {
            Ok(Adjacency { previous: None, next: None })
        }
        async fn resolve_address(&self, _w: &str, _a: &str) -> Result<Option<String>> { Ok(None) }

        async fn search(
            &self,
            _query: &str,
            _scope: Option<&str>,
            _limit: i64,
            _offset: i64
        ) -> Result<crate::models::Pagination<crate::models::SearchMatch>> {
            unimplemented!("Search is not tested in this mock")
        }
    }

    #[tokio::test]
    async fn test_comparison_grouping_logic() {
        let repo = Arc::new(MockRepository);
        let engine = CoreContentEngine::new(repo);

        let comparisons = engine.get_comparison("mock.path").await.unwrap();

        // The engine should group the 2 rows from the mock repo into 1 comparison block
        assert_eq!(comparisons.len(), 1);
        assert_eq!(comparisons[0].contents.len(), 2);
    }
}

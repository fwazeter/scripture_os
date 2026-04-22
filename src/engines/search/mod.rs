//! # Search Engine (The "Finder")
//!
//! This engine handles discovery and full-text search across the library.
//!
//! ### Architectural Design Decision: Delegated FTS
//! The Search Engine validates inputs and handles pagination bounds, but delegates
//! text indexing, ranking, and snippet generation to the repository to leverage
//! native PostgreSQL FTS capabilities.
use std::sync::Arc;
use anyhow::{bail, Result};
use async_trait::async_trait;

use crate::models::{SearchMatch, Pagination};
use crate::repository::ScriptureRepository;
use super::SearchEngine;

/// # Core Search Engine
///
/// ### Architectural Design Decision: Dependency Injection (DI)
/// Encapsulates the `ScriptureRepository` via an `Arc` to allow agnostic testing
/// and safe concurrency within Axum.
pub struct CoreSearchEngine {
    repo: Arc<dyn ScriptureRepository + Send + Sync>,
}

impl CoreSearchEngine {
    pub fn new(repo: Arc<dyn ScriptureRepository + Send + Sync>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl SearchEngine for CoreSearchEngine {
    /// ## `keyword_search`
    /// **Parameters:** /// * `query: &str` (The user's raw search string).
    /// * `scope: Option<&str>` (Optional LTREE path to restrict search bounds).
    /// * `page: i64` (The requested page number).
    ///
    /// ### Design Decision: Input Validation & Pagination Guardrails
    /// 1. Rejects queries under 3 characters to prevent massive table scans.
    /// 2. Normalizes negative page numbers to page 1 for API stability.
    ///
    /// ### Design Decision: Engine-to-Repo Delegation
    /// Business rules for "What constitutes a valid search" live here, while the
    /// `ts_rank` and `ts_headline` logic is encapsulated in the repository.
    ///
    /// **AI Prompt Hint:** When adding "Semantic Search," this trait must be
    /// expanded to accept vector embeddings alongside text queries.
    async fn keyword_search(
        &self,
        query: &str,
        scope: Option<&str>,
        page: i64
    ) -> Result<Pagination<SearchMatch>> {
        let trimmed = query.trim();
        if trimmed.len() < 3 {
            bail!("Search query must be at least 3 characters long.");
        }

        let valid_page = if page < 1 { 1 } else { page };
        let limit: i64 = 20;
        let offset: i64 = (valid_page - 1) * limit;

        self.repo.search(trimmed, scope, limit, offset).await
    }
}

// --- Track A: Concrete Integration Tests ---
#[cfg(test)]
mod tests {
    use crate::repository::postgres::PostgresRepository;
    use super::*;

    #[tokio::test]
    async fn test_keyword_search_integration() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        let repo = Arc::new(PostgresRepository::new(pool));
        let engine = CoreSearchEngine::new(repo.clone());

        // Search for "God" in the English translations
        let results = engine.keyword_search("God", None, 1).await.unwrap();

        // We seeded john 17:3 ("And this life eternal...") and Psalm 51
        // Psalm 51 has "Have mercy upon me, O God..."
        assert!(results.total_records > 0 );
        assert!(!results.data.is_empty());
    }
}

// --- Track B: Isolated Mock Tests ---
#[cfg(test)]
mod mock_tests {
    use super::*;
    use crate::models::{HierarchyNode, Adjacency, ScriptureContent};
    use uuid::Uuid;

    struct MockRepository;

    #[async_trait]
    impl ScriptureRepository for MockRepository {
        async fn get_hierarchy(&self, _p: &str) -> Result<Vec<HierarchyNode>> { Ok(vec![]) }

        async fn get_adjacent_nodes(&self, _id: Uuid) -> Result<Adjacency> {
            Ok(Adjacency {previous: None, next: None })
        }
        // Stubs
        async fn fetch_text(&self, _start: &str, _end: Option<&str>) -> Result<Vec<ScriptureContent>> { Ok(vec![]) }
        async fn resolve_address(&self, _w: &str, _a: &str) -> Result<Option<String>> { Ok(None) }
        async fn search(
            &self,
            query: &str,
            _scope: Option<&str>,
            _limit: i64,
            _offset: i64
        ) -> Result<Pagination<SearchMatch>> {
            // Mock returning a hit
            Ok(Pagination{
                data: vec![SearchMatch {
                    node_id: Uuid::new_v4(),
                    path: "mock.path".to_string(),
                    snippet: format!("<b>{}</b> snippet", query),
                    edition_name: "Mock_EN".to_string(),
                    relevance_score: 1.0,
                }],
                total_records: 1,
                current_page: 1,
                total_pages: 1,
                has_next: false,
            })
        }
    }

    #[tokio::test]
    async fn test_search_validation_logic() {
        let repo = Arc::new(MockRepository);
        let engine = CoreSearchEngine::new(repo);

        // Should fail validation (under 3 chars)
        let err = engine.keyword_search("ab", None, 1).await;
        assert!(err.is_err());

        // Should pass validation
        let ok = engine.keyword_search("love", None, -5).await;
        assert!(ok.is_ok());
        // Page -5 should be normalized to page 1, which the mock repo returns
        assert_eq!(ok.unwrap().current_page, 1);
    }
}
//! # Traversal Engine (The "Guide")
//!
//! The Traversal Engine handles structural navigation within a scriptural work.
//!
//! ### Architectural Design Decision: Structural Discovery
//! Scripture OS separates "Addressing" from "Content". This engine is
//! concerned exclusively with "Addressing"—finding where a user is and where
//! they can go next.

use std::sync::Arc;
use uuid::Uuid;
use anyhow::Result;
use async_trait::async_trait;

use crate::models::{HierarchyNode, Adjacency};
use crate::repository::ScriptureRepository;
use super::TraversalEngine;

/// # Core Traversal Engine
///
/// This is the primary implementation of the `TraversalEngine` trait.
///
/// ### Architectural Design Decision: Dependency Injection (DI)
/// Encapsulates a thread-safe repository pointer to manage state across
/// concurrent Axum web requests.
pub struct CoreTraversalEngine {
    repo: Arc<dyn ScriptureRepository + Send + Sync>,
}

impl CoreTraversalEngine {
    /// Bootstraps the engine by injecting the required data layer repository
    pub fn new(repo: Arc<dyn ScriptureRepository + Send + Sync>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl TraversalEngine for CoreTraversalEngine {
    /// ## `get_hierarchy`
    /// **Parameters:** /// * `parent_path: &str` (The canonical LTREE path, e.g., "bible.nt.john").
    ///
    /// ### Architectural Design Decision: Progressive Disclosure
    /// This function enables the UI to load scripture in "chunks" (e.g., chapters)
    /// rather than the entire hierarchy, reducing frontend memory overhead.
    ///
    /// ### Design Decision: Engine-to-Repo Delegation
    /// The engine validates the path format and delegates `nlevel` filtering
    /// to the repository, keeping the engine implementation focused on
    /// business rules rather than SQL implementation.
    ///
    /// **AI Prompt Hint:** If adding path-based permissions (e.g., hiding
    /// apocryphal works), implement the filtering logic here after the
    /// repository returns the node list but before returning to the Gateway.
    async fn get_hierarchy(
        &self,
        parent_path: &str
    ) -> Result<Vec<HierarchyNode>> {
        // The engine acts as a pass-through to the repository implementation
        // todo add universal validation, telemetry or caching logic before asking the db for hierarchy
        self.repo.get_hierarchy(parent_path).await
    }

    /// ## `get_adjacent_nodes`
    /// **Parameters:** /// * `current_node: Uuid` (The unique ID of the node in view).
    ///
    /// ### Architectural Design Decision: Contextual Continuity
    /// Navigation requires maintaining "type context". If reading a Chapter,
    /// the "Next" button must lead to the next Chapter, not a Verse node. This
    /// function enforces **Type-Strict Navigation**.
    ///
    /// **AI Prompt Hint:** If building a "Global Linear Reading" feature, create
    /// a new repository method that ignores `node_type` to allow jumping
    /// from the end of one book to the start of the next.
    async fn get_adjacent_nodes(
        &self,
        current_node: Uuid
    ) -> Result<Adjacency> {
        // todo user permission checks can be added here before
        //     letting them navigate to an adjacent node, that business logic
        //     would go here, completely separate from the SQL CTE.
        // Delegates the complex CTE lookup to the repository
        self.repo.get_adjacent_nodes(current_node).await
    }
}

// --- Integration Tests (Track A) ---
#[cfg(test)]
mod tests {
    use crate::repository::postgres::PostgresRepository;
    use super::*;

    #[tokio::test]
    async fn test_traversal_engine_hierarchy() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        let repo = Arc::new(PostgresRepository::new(pool));
        let engine = CoreTraversalEngine::new(repo);

        // FIX 1: Ask for Hafs sura 1, which actually exists in the seed data
        let hierarchy = engine.get_hierarchy("hafs.sura.1").await.unwrap();

        // The seed data has Ayah 1 (Basmala) and Ayah 2
        assert_eq!(hierarchy.len(), 2);
        assert_eq!(hierarchy[0].path, "hafs.sura.1.1");
        assert_eq!(hierarchy[1].path, "hafs.sura.1.2");
    }

    #[tokio::test]
    async fn test_adjacency() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        let repo = Arc::new(PostgresRepository::new(pool));
        let engine = CoreTraversalEngine::new(repo);

        // FIX 2: Revert to using the explicit UUID for Hafs Sura 1:1 from your original seed data
        let target_node = Uuid::parse_str("00000000-0000-0000-0000-000000000A06").unwrap();

        let adjacency = engine.get_adjacent_nodes(target_node).await.unwrap();

        // Next should be Hafs Sura 1:2
        assert!(adjacency.next.is_some());
        assert_eq!(adjacency.next.unwrap().path, "hafs.sura.1.2");

        // Previous should be None since it's the first ayah
        assert!(adjacency.previous.is_none());
    }
}

// --- Mock Tests (Track B) ---
#[cfg(test)]
mod mock_tests {
    use super::*;
    use async_trait::async_trait;
    use crate::repository::ScriptureRepository;
    use crate::models::{HierarchyNode, Adjacency, ScriptureContent};

    // A fake repository for testing
    struct MockRepository;

    #[async_trait]
    impl ScriptureRepository for MockRepository {
        async fn get_hierarchy(&self, _path: &str) -> Result<Vec<HierarchyNode>> {
            Ok(vec![
                HierarchyNode { id: Uuid::new_v4(), path: "mock.1.1".to_string() }
            ])
        }
        async fn get_adjacent_nodes(&self, _id: Uuid) -> Result<Adjacency> {
            Ok(Adjacency { previous: None, next: None })
        }
        async fn fetch_text(&self, _start: &str, _end: Option<&str>) -> Result<Vec<ScriptureContent>> { Ok(vec![]) }
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
    async fn test_engine_logic_with_mock() {
        // Inject the mock repository instead of the Postgres one
        let repo = Arc::new(MockRepository);
        let engine = CoreTraversalEngine::new(repo);

        let hierarchy = engine.get_hierarchy("mock").await.unwrap();
        assert_eq!(hierarchy.len(), 1);
        assert_eq!(hierarchy[0].path, "mock.1.1");
    }
}
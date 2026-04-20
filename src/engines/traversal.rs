//! # Traversal Engine (The "Guide")
//!
//! The Traversal Engine handles structural navigation within a scriptural work.
//! It acts as the orchestration layer between the API handlers and the data repository,
//! ensuring that users can discover the hierarchy (Chapters in a Book) or move
//! linearly (Next/Previous Chapter).
//!
//! ### Architectural Design Decision: Structural Discovery
//! Scripture OS separates "Addressing" from "Content". The Traversal engine is
//! concerned exclusively with "Addressing"—finding where a user is and where
//! they can go next.

use uuid::Uuid;
use anyhow::Result;
use crate::models::{HierarchyNode, Adjacency};
use crate::repository::ScriptureRepository;

/// ## `get_hierarchy`
/// **Parameters:** /// * `repo: &dyn ScriptureRepository` (The data provider instance).
/// * `parent_path: &str` (The canonical LTREE path, e.g., "bible.nt.john").
///
/// ### Architectural Design Decision: Progressive Disclosure
/// This function enables the UI to load scripture in "chunks" (e.g., a list of
/// chapters) rather than downloading the entire hierarchy at once. This
/// significantly reduces frontend memory overhead.
///
/// ### Design Decision: Engine-to-Repo Delegation
/// The engine validates the path format (todo) and then delegates the specific
/// `nlevel` filtering logic to the repository. This keeps the engine
/// implementation simple and focused on business rules.
///
/// ### Technical Context: Trait-Based Dispatch
/// By taking `&dyn ScriptureRepository`, this function remains agnostic of
/// whether it is talking to PostgreSQL, a local SQLite cache, or a Mock
/// object in a unit test.
///
/// **AI Prompt Hint:** If adding path-based permissions or "feature flags"
/// (e.g., hiding certain apocryphal books), implement that filtering logic
/// here after receiving the nodes from the repository.
pub async fn get_hierarchy(
    repo: &dyn ScriptureRepository,
    parent_path: &str
) -> Result<Vec<HierarchyNode>> {
    // The engine acts as a pass-through to the repository implementation
    // todo add universal validation, telemetry or caching logic before asking the db for hierarchy
    repo.get_hierarchy(parent_path).await
}

/// ## `get_adjacent`
/// **Parameters:** /// * `repo: &dyn ScriptureRepository` (The data provider instance).
/// * `current_node_id: Uuid` (The unique ID of the node currently in view).
///
/// ### Architectural Design Decision: Contextual Continuity
/// Scripture navigation requires maintaining the "type context". If a user is
/// reading a Chapter, the "Next" button should take them to the next Chapter,
/// not the first verse of the current chapter. This function ensures
/// **Type-Strict Navigation**.
///
/// ### Design Decision: Identity-Based Adjacency
/// We use the `Uuid` rather than the `path` string for lookup because IDs
/// are immutable, whereas paths might change if a hierarchy is restructured.
/// The repository uses this ID to anchor the "Previous" and "Next" search.
///
/// ### Technical Context: Option Handling
/// The return type `Adjacency` contains two `Option<HierarchyNode>` fields.
/// This naturally handles the "Start of Book" and "End of Book" edge cases
/// where one or both neighbors might not exist.
///
/// **AI Prompt Hint:** If you are building a "Reading Plan" feature, you
/// may need to create a new version of this function that ignores `node_type`
/// to allow jumping across different types of nodes (e.g., from the end
/// of a Testament to the start of a Gospel).
pub async fn get_adjacent(
    repo: &dyn ScriptureRepository,
    current_node: Uuid
) -> Result<Adjacency> {
    // todo user permission checks can be added here before
    //     letting them navigate to an adjacent node, that business logic
    //      would go here, completely separate from the SQL CTE.
    // Delegates the complex CTE lookup to the repository
    repo.get_adjacent_nodes(current_node).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::postgres::PostgresRepository;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_get_hierarchy_hafs() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        // instantiate the repo
        let repo = PostgresRepository::new(pool);

        // Ask for children of Hafs sura 1
        let children = get_hierarchy(&repo, "hafs.sura.1").await.unwrap();

        // The seed data has Ayah 1 (Basmala) and Ayah 2
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].path, "hafs.sura.1.1");
        assert_eq!(children[1].path, "hafs.sura.1.2");
    }

    #[tokio::test]
    async fn test_get_adjacent_nodes_hafs() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;
        let repo = PostgresRepository::new(pool);

        // Target: Hafs Sura 1:1 (ID: ...0A06)
        let target_node = Uuid::parse_str("00000000-0000-0000-0000-000000000A06").unwrap();
        let adjacency = get_adjacent(&repo, target_node).await.unwrap();

        // Next should be Hafs Sura 1:2
        assert!(adjacency.next.is_some());
        assert_eq!(adjacency.next.unwrap().path, "hafs.sura.1.2");

        // Previous should be None since it's the first ayah
        assert!(adjacency.previous.is_none());
    }
}

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
        async fn fetch_text(&self, _path: &str) -> Result<Vec<ScriptureContent>> { Ok(vec![]) }
        async fn resolve_address(&self, _w: &str, _a: &str) -> Result<Option<String>> { Ok(None) }
    }

    #[tokio::test]
    async fn test_engine_logic_with_mock() {
        let mock_repo = MockRepository;

        // This test runs in microseconds because it doesn't touch the DB!
        let results = get_hierarchy(&mock_repo, "any.path").await.unwrap();

        assert_eq!(results[0].path, "mock.1.1");
    }
}
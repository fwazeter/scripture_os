//! # Traversal Engine (The "Guide")
//!
//! The Traversal Engine handles structural navigation within a scriptural work.
//! Because Scripture OS uses PostgreSQL's `ltree` extension, hierarchical relationships
//! (parents, children, siblings) are native to the database layer.
//!
//! This module provides functions for traversing down the tree (finding children)
//! and traversing laterally across the tree (finding adjacent siblings).

use uuid::Uuid;
use anyhow::Result;
use crate::models::{HierarchyNode, Adjacency};
use crate::repository::ScriptureRepository;

/// Retrieves the direct children of a given hierarchical node.
///
/// For example, if passed a Book path (`"bible.nt.john"`), it returns all the Chapters.
/// If passed a Chapter path (`"bible.nt.john.17"`), it returns all the Verses.
///
/// # Design Decisions
/// * **`ltree` Operators:** This utilizes two specific `ltree` operators.
///   The `<@` operator ensures we only fetch descendants of the `parent_path`.
///   The `nlevel()` function ensures we only fetch *direct* children (depth + 1),
///   preventing the database from accidentally returning thousands of verses when
///   we only wanted a list of chapters.
/// * **Ordering:** Results are strictly ordered by `start_index ASC` to ensure
///   chapters and verses are returned in sequential reading order.
pub async fn get_hierarchy(
    repo: &dyn ScriptureRepository,
    parent_path: &str
) -> Result<Vec<HierarchyNode>> {
    // todo add universal validation, telemetry or caching logic before asking the db for hierarchy
    repo.get_hierarchy(parent_path).await
}

/// Determines the immediately preceding and immediately following structural nodes.
///
/// This is primarily used for frontend UI components (e.g., "Next Chapter" or "Previous Verse" buttons).
///
/// # SQL Architecture: The CTE (Common Table Expression)
/// This function uses a highly optimized `WITH` clause to perform three lookups in a single query:
/// 1. `current_node`: Fetches the metadata (type, and boundary indices) of the target ID.
/// 2. `prev_node`: Finds the one node in the same work, of the same type (e.g., 'verse'),
///    whose `end_index` immediately precedes the current node's `start_index`.
/// 3. `next_node`: Finds the one node whose `start_index` immediately follows the current `end_index`.
///
/// # Edge Case Handling
/// Because the adjacency checks evaluate the sequential `start_index` and `end_index` rather
/// than relying on textual path manipulation, it flawlessly handles transitions across
/// hierarchical boundaries (e.g., stepping from the last verse of Chapter 1 directly into
/// the first verse of Chapter 2).
pub async fn get_adjacent_nodes(
    repo: &dyn ScriptureRepository,
    current_node: Uuid
) -> Result<Adjacency> {
    // todo user permission checks can be added here before
    //     letting them navigate to an adjacent node, that business logic
    //      would go here, completely separate from the SQL CTE.

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
        let adjacency = get_adjacent_nodes(&repo, target_node).await.unwrap();

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
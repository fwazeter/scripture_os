//! # Traversal Engine (The "Guide")
//!
//! The Traversal Engine handles structural navigation within a scriptural work.
//! Because Scripture OS uses PostgreSQL's `ltree` extension, hierarchical relationships
//! (parents, children, siblings) are native to the database layer.
//!
//! This module provides functions for traversing down the tree (finding children)
//! and traversing laterally across the tree (finding adjacent siblings).

use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Result;
use crate::models::{HierarchyNode, Adjacency};

// Private helper struct for SQLx to map our complex adjacency query
#[derive(sqlx::FromRow)]
struct AdjacencyRow {
    prev_id: Option<Uuid>,
    prev_path: Option<String>,
    next_id: Option<Uuid>,
    next_path: Option<String>,
}

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
pub async fn get_hierarchy(pool: &PgPool, parent_path: &str) -> Result<Vec<HierarchyNode>> {
    let nodes = sqlx::query_as::<_, HierarchyNode>(
        r#"
            SELECT id, path::text as path
            FROM nodes
            WHERE path <@ $1::text::ltree
                AND nlevel(path) = nlevel($1::text::ltree) + 1
            ORDER BY start_index ASC
            "#
    )
    .bind(parent_path).fetch_all(pool)
    .await?;

    Ok(nodes)
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
pub async fn get_adjacent_nodes(pool: &PgPool, current_node_id: Uuid) -> Result<Adjacency> {
    // A single optimized query using a CTE (WITH clause) to fetch previous/next nodes
    let row = sqlx::query_as::<_, AdjacencyRow>(
        r#"
            WITH current_node AS (
                SELECT work_id, node_type, start_index, end_index FROM nodes WHERE id = $1
            ),
            prev_node AS (
                SELECT id, path::text as path FROM nodes
                WHERE work_id = (SELECT work_id FROM current_node)
                    AND node_type = (SELECT node_type FROM current_node)
                    AND end_index < (SELECT start_index FROM current_node)
                ORDER BY end_index DESC LIMIT 1
            ),
            next_node AS (
                SELECT id, path::text as path FROM nodes
                WHERE work_id = (SELECT work_id FROM current_node)
                    AND node_type = (SELECT node_type FROM current_node)
                    AND start_index > (SELECT end_index FROM current_node)
                ORDER BY start_index ASC LIMIT 1
            )
            SELECT
                (SELECT id FROM prev_node) as prev_id,
                (SELECT path FROM prev_node) as prev_path,
                (SELECT id FROM next_node) as next_id,
                (SELECT path FROM next_node) as next_path
            "#
    )
        .bind(current_node_id).fetch_one(pool)
        .await?;

    Ok(Adjacency {
        previous: row.prev_id.zip(row.prev_path).map(|(id, path)| HierarchyNode { id, path }),
        next: row.next_id.zip(row.next_path).map(|(id, path)| HierarchyNode { id, path }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_get_hierarchy_hafs() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        // Ask for children of Hafs sura 1
        let children = get_hierarchy(&pool, "hafs.sura.1").await.unwrap();

        // The seed data has Ayah 1 (Basmala) and Ayah 2
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].path, "hafs.sura.1.1");
        assert_eq!(children[1].path, "hafs.sura.1.2");
    }

    #[tokio::test]
    async fn test_get_adjacent_nodes_hafs() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        // Target: Hafs Sura 1:1 (ID: ...0A06)
        let target_node = Uuid::parse_str("00000000-0000-0000-0000-000000000A06").unwrap();
        let adjacency = get_adjacent_nodes(&pool, target_node).await.unwrap();

        // Next should be Hafs Sura 1:2
        assert!(adjacency.next.is_some());
        assert_eq!(adjacency.next.unwrap().path, "hafs.sura.1.2");

        // Previous should be None since it's the first ayah
        assert!(adjacency.previous.is_none());
    }
}
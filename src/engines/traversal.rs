use sqlx::PgPool;
use uuid::Uuid;
use serde::Serialize;
use anyhow::Result;

#[derive(sqlx::FromRow, Serialize, Debug, PartialEq)]
pub struct HierarchyNode {
    pub id: Uuid,
    pub path: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Adjacency {
    pub previous: Option<HierarchyNode>,
    pub next: Option<HierarchyNode>,
}

// Helper struct for SQLx to map our complex adjacency query
#[derive(sqlx::FromRow)]
struct AdjacencyRow {
    prev_id: Option<Uuid>,
    prev_path: Option<String>,
    next_id: Option<Uuid>,
    next_path: Option<String>,
}

/// Fetches the direct children of a given ltree path (e.g. all chapters in a book).
pub async fn get_hierarchy(pool: &PgPool, parent_path: &str) -> Result<Vec<HierarchyNode>> {
    let nodes = sqlx::query_as::<_, HierarchyNode>(
        r#"
            SELECT id, path::text as path
            FROM nodes
            WHERE path <@ $1::ltree
                AND nlevel(path) = nlevel($1::ltree) + 1
            ORDER BY sort_order ASC
            "#
    )
    .bind(parent_path).fetch_all(pool)
    .await?;

    Ok(nodes)
}

/// Finds the immediately preceding and following nodes of the same type within the work.
pub async fn get_adjacent_nodes(pool: &PgPool, current_node_id: Uuid) -> Result<Adjacency> {
    // A single optimized query using a CTE (WITH clause) to fetch previous/next nodes
    let row = sqlx::query_as::<_, AdjacencyRow>(
        r#"
            WITH current_node AS (
                SELECT work_id, node_type, sort_order FROM nodes WHERE id = $1
            ),
            prev_node AS (
                SELECT id, path::text as path FROM nodes
                WHERE work_id = (SELECT work_id FROM current_node)
                    AND node_type = (SELECT node_type FROM current_node)
                    AND sort_order < (SELECT sort_order FROM current_node)
                ORDER BY sort_order DESC LIMIT 1
            ),
            next_node AS (
                SELECT id, path::text as path FROM nodes
                WHERE work_id = (SELECT work_id FROM current_node)
                    AND node_type = (SELECT node_type FROM current_node)
                    AND sort_order > (SELECT sort_order FROM current_node)
                ORDER BY sort_order ASC LIMIT 1
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

    // Map the flat SQL row into our nested Rust Adjacency struct
    let previous = match (row.prev_id, row.prev_path) {
        (Some(id), Some(path)) => Some(HierarchyNode { id, path }),
        _ => None,
    };

    let next = match (row.next_id, row.next_path) {
        (Some(id), Some(path)) => Some(HierarchyNode { id, path }),
        _ => None,
    };

    Ok(Adjacency { previous, next })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_get_hierarchy() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        // 2. Ask for children of the Book of John ("bible_test.nt.john")
        // Should return Chapter 17 based on seed data, but not verses.
        let children = get_hierarchy(&pool, "bible_test.nt.john").await.unwrap();

        assert_eq!(children.len(), 1);
        assert_eq!(children[0].path, "bible_test.nt.john.17");
    }

    #[tokio::test]
    async fn test_get_adjacent_nodes() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        // Target: John 17:3 (ID: ....0102)
        // Previous should be 17:2, next 17:4
        let target_node = Uuid::parse_str("00000000-0000-0000-0000-000000000102").unwrap();

        let adjacency = get_adjacent_nodes(&pool, target_node).await.unwrap();

        assert!(adjacency.previous.is_some());
        assert_eq!(adjacency.previous.unwrap().path, "bible_test.nt.john.17.2");

        assert!(adjacency.next.is_some());
        assert_eq!(adjacency.next.unwrap().path, "bible_test.nt.john.17.4");
    }
}
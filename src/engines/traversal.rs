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

/// Fetches the direct children of a given ltree path (e.g. all chapters in a book).
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

/// Finds the immediately preceding and following nodes of the same type within the work.
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
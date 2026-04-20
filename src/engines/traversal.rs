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
    use sqlx::postgres::PgPoolOptions;
    use dotenvy::dotenv;
    use std::env;

    async fn setup_db() -> PgPool {
        dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgPoolOptions::new().connect(&db_url)
            .await.unwrap()
    }

    // Helper function to seed data for tests
    async fn seed_traversal_data(pool: &PgPool) {
        let work_id = Uuid::parse_str("77777777-7777-7777-7777-777777777770").unwrap();

        //  Setup isolated test data
        sqlx::query("INSERT INTO works (id, title, slug) VALUES ($1, 'Traversal Test', 'trav_test') ON CONFLICT DO NOTHING")
            .bind(work_id).execute(pool)
            .await.unwrap();

        // Insert Parent (Book)
        sqlx::query("INSERT INTO nodes (id, work_id, path, node_type, sort_order) VALUES ('77777777-7777-7777-7777-777777777771', $1, 'trav_test.book1', 'book', 1.0) ON CONFLICT DO NOTHING")
            .bind(work_id).execute(pool)
            .await.unwrap();

        // Insert Children (Chapters)
        sqlx::query("INSERT INTO nodes (id, work_id, path, node_type, sort_order) VALUES ('77777777-7777-7777-7777-777777777772', $1, 'trav_test.book1.1', 'chapter', 1.1) ON CONFLICT DO NOTHING")
            .bind(work_id).execute(pool)
            .await.unwrap();
        sqlx::query("INSERT INTO nodes (id, work_id, path, node_type, sort_order) VALUES ('77777777-7777-7777-7777-777777777773', $1, 'trav_test.book1.2', 'chapter', 1.2) ON CONFLICT DO NOTHING")
            .bind(work_id).execute(pool)
            .await.unwrap();

        // Insert GrandChildren (verse - should NOT be returned by hierarchy)
        sqlx::query("INSERT INTO nodes (id, work_id, path, node_type, sort_order) VALUES ('77777777-7777-7777-7777-777777777774', $1, 'trav_test.book1.1.1', 'verse', 1.11) ON CONFLICT DO NOTHING")
            .bind(work_id).execute(pool)
            .await.unwrap();
    }
    #[tokio::test]
    async fn test_get_hierarchy() {
        let pool = setup_db().await;

        // 1. Setup isolated test data using our new helper
        seed_traversal_data(&pool).await;

        // 2. Execute
        let children = get_hierarchy(&pool, "trav_test.book1").await.unwrap();

        // 3. Assert (Should only return the two chapters, not the verse)
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].path, "trav_test.book1.1");
        assert_eq!(children[1].path, "trav_test.book1.2");
    }

    #[tokio::test]
    async fn test_get_adjacent_nodes() {
        let pool = setup_db().await;

        // 1. Setup isolated test data using our new helper
        seed_traversal_data(&pool).await;

        // We will test adjacency on Chapter 1 (ID ...7772).
        // It should have NO previous and Chapter 2 (ID ...7773) as next.
        let target_node = Uuid::parse_str("77777777-7777-7777-7777-777777777772").unwrap();

        let adjacency = get_adjacent_nodes(&pool, target_node).await.unwrap();

        assert!(adjacency.previous.is_none());
        assert!(adjacency.next.is_some());
        assert_eq!(adjacency.next.unwrap().path, "trav_test.book1.2");
    }
}
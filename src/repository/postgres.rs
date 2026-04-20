use sqlx::PgPool;
use async_trait::async_trait;
use uuid::Uuid;
use anyhow::Result;

use crate::models::{HierarchyNode, Adjacency, ScriptureContent};
use super::ScriptureRepository;

// Private helper struct for SQLx to map our complex adjacency query TO REMOVE
#[derive(sqlx::FromRow)]
struct AdjacencyRow {
    prev_id: Option<Uuid>,
    prev_path: Option<String>,
    next_id: Option<Uuid>,
    next_path: Option<String>,
}

pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ScriptureRepository for PostgresRepository {
    async fn get_hierarchy(&self, parent_path: &str) -> Result<Vec<HierarchyNode>> {
        // Uses the ltree operators `<@` and `nlevel()` to fetch direct descendants.
        let nodes = sqlx::query_as::<_, HierarchyNode>(
            r#"
                SELECT id, path::text as path
                FROM nodes
                WHERE path <@ $1::text::ltree
                    AND nlevel(path) = nlevel($1::text::ltree) + 1
                ORDER BY start_index ASC
                "#
        )
            .bind(&parent_path)
            .fetch_all(&self.pool)
            .await?;

        Ok(nodes)
    }

    async fn get_adjacent_nodes(&self, current_node_id: Uuid) -> Result<Adjacency> {
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
        .bind(current_node_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Adjacency {
            previous: row.prev_id.zip(row.prev_path).map(|(id, path)| HierarchyNode { id, path }),
            next: row.next_id.zip(row.next_path).map(|(id, path)| HierarchyNode { id, path }),
        })
    }

    async fn fetch_text(&self, path: &str) -> Result<Vec<ScriptureContent>> {
        // 1. Fetch the range bounds for the requested path
        let bounds = sqlx::query!(
            r#"
            SELECT start_index, end_index FROM nodes WHERE path = $1::text::ltree
            "#,
            path
        ).fetch_one(&self.pool).await?;

        // 2. Fetch all texts that fall between those bounds
        let contents = sqlx::query_as::<_, ScriptureContent>(
            r#"
                SELECT
                    t.body_text,
                    e.name as edition_name,
                    e.language_code,
                    t.absolute_index
                FROM texts t
                JOIN editions e ON t.edition_id = e.id
                WHERE t.absolute_index BETWEEN $1 and $2
                    AND e.work_id = (SELECT work_id FROM nodes WHERE path = $3::text::ltree)
                ORDER BY t.absolute_index ASC, e.is_source DESC
                "#
        )
            .bind(bounds.start_index)
            .bind(bounds.end_index)
            .bind(path)
            .fetch_all(&self.pool)
            .await?;

        Ok(contents)
    }

    async fn resolve_address(&self, work_slug: &str, alias: &str) -> Result<Option<String>> {
        // Extracts the alias lookup out of the resolution engine
        let record = sqlx::query!(
            r#"
            SELECT n.path::text as base_path
            FROM node_aliases na
            JOIN nodes n ON na.node_id = n.id
            JOIN works w ON n.work_id = w.id
            WHERE na.alias ILIKE $1 AND w.slug = $2
            LIMIT 1
            "#,
            alias,
            work_slug
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(record.and_then(|r| r.base_path))
    }
}
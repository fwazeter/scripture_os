//! # PostgreSQL Repository Implementation
//!
//! This module acts as the "Physical Layer" of Scripture OS. It translates
//! domain-specific requests (like "Give me the next chapter") into optimized
//! SQL queries utilizing PostgreSQL-specific extensions.
//!
//! ### Architectural Design Decision: Stand-off Markup
//! Scripture OS does not store text inside the hierarchy nodes. Instead, it uses
//! a "Spine" (`nodes`) and "Content" (`texts`) model.
//! 1. `nodes` (Spine): Provides the address/path using `ltree`.
//! 2. `texts` (Content): Stores the actual strings, indexed by an `absolute_index`.
//!
//! This separation allows us to represent multiple translations (English, Greek,
//! Hebrew) for the exact same structural node without duplicating the hierarchy.

use sqlx::PgPool;
use async_trait::async_trait;
use uuid::Uuid;
use anyhow::Result;

use crate::models::{
    HierarchyNode,
    Adjacency,
    ScriptureContent,
    SearchMatch,
    Pagination
};
use super::ScriptureRepository;

pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    /// Creates a new repository instance.
    ///
    /// **Expects:** A pre-configured `PgPool` with the `ltree` and `pgcrypto`
    /// extensions already enabled in the target database.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Private helper struct for SQLx to map our complex adjacency query
#[derive(sqlx::FromRow)]
struct AdjacencyRow {
    prev_id: Option<Uuid>,
    prev_path: Option<String>,
    next_id: Option<Uuid>,
    next_path: Option<String>,
}

#[async_trait]
impl ScriptureRepository for PostgresRepository {
    /// ## `get_hierarchy`
    /// **Parameters:** `parent_path: &str` (The canonical LTREE path of the parent node).
    ///
    /// ### Architectural Design Decision: Depth-Limited Exploration
    /// Prevents "Data Flooding" by ensuring that a request for a high-level node (e.g., a "Work")
    /// doesn't accidentally return every verse within it.
    ///
    /// ### Design Decision: Structural Integrity
    /// Uses the `nlevel()` PostgreSQL function to calculate direct lineage.
    /// 1. Identifies the depth of the `parent_path`.
    /// 2. Filters for nodes exactly one level deeper (`nlevel + 1`).
    ///
    /// ### SQL Quirk: Double Casting for LTREE
    /// Because `sqlx` does not have a native `ltree` type, we must cast the input string
    /// to `text` first, and then to `ltree` (`$1::text::ltree`) to ensure the GIST
    /// index is utilized correctly by the query optimizer.
    ///
    /// **AI Prompt Hint:** If implementing "Deep Retrieval" or "Recursive Menus,"
    /// remove the `nlevel` constraint while keeping the `<@` (descendant) operator.
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

    /// ## `get_adjacent_nodes`
    /// **Parameters:** `current_node_id: Uuid` (The unique identifier of the node the user is currently viewing).
    ///
    /// ### Architectural Design Decision: Context-Aware Navigation
    /// A major challenge in scripture navigation is "Type Drift"—for example, clicking 'Next' on a
    /// Chapter node and accidentally receiving a Verse node.
    ///
    /// This function enforces **Type-Strict Adjacency**:
    /// 1. It identifies the `node_type` (Chapter, Verse, Book) of the current ID.
    /// 2. It restricts the search to only siblings of that exact same type within the same `work_id`.
    ///
    /// ### Design Decision: Boundary-Based Proximity
    /// Instead of relying on sequential IDs (which can be fragmented) or `ltree` path string manipulation
    /// (which is computationally expensive for adjacency), we use the **Universal Sequence Index**.
    /// * **Previous:** The node of the same type whose `end_index` is the highest value still strictly less than our `start_index`.
    /// * **Next:** The node of the same type whose `start_index` is the lowest value still strictly greater than our `end_index`.
    ///
    /// ### SQL Quirk: The CTE "Single Trip" Strategy
    /// We use a Common Table Expression (CTE) to perform three logical lookups in one database round-trip:
    /// 1. `current_node`: Establishes the anchor point metadata (work, type, and indices).
    /// 2. `prev_node`: Scans "backwards" from the anchor.
    /// 3. `next_node`: Scans "forwards" from the anchor.
    ///
    /// **AI Prompt Hint:** If implementing a "Wrap-around" feature (e.g., going from the last chapter of
    /// Revelation back to Genesis 1), the `prev_node` and `next_node` CTEs would need to be updated
    /// to handle `NULL` results by performing a `MIN`/`MAX` fallback.
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

        // Mapping Logic: We use .zip() to ensure that a HierarchyNode is only
        // created if BOTH the ID and the Path are present in the SQL result.
        Ok(Adjacency {
            previous: row.prev_id.zip(row.prev_path).map(|(id, path)| HierarchyNode { id, path }),
            next: row.next_id.zip(row.next_path).map(|(id, path)| HierarchyNode { id, path }),
        })
    }

    /// ## `fetch_text`
    /// **Parameters:** `path: &str` (The canonical address, e.g., "bible.nt.jn.3.16").
    ///
    /// ### Architectural Design Decision: Sequence-to-Address Mapping
    /// This function implements the core bridge between the hierarchical "Spine" and the linguistic "Content".
    /// In the Stand-off Markup model, a node (like a Chapter) is a pointer to a range of absolute indices.
    ///
    /// ### Design Decision: Two-Step Resolution
    /// 1. **Boundary Resolution:** Fetches the ID, `work_id`, `start_index` and `end_index` from the `nodes` table.
    /// 2. **Content Aggregation:** Queries the `texts` table for all rows falling between those bounds.
    ///
    /// ### SQL Quirk: Context Injection
    /// Because the `texts` table does not inherently know its hierarchical path (Stand-off markup),
    /// we dynamically inject `$1` (node_id) and `$2` (path) directly into the SELECT projection.
    /// We also inject `NULL::jsonb` to fulfill the `translation_metadata` Option field.
    async fn fetch_text(&self, path: &str) -> Result<Vec<ScriptureContent>> {
        // 1. Fetch the range bounds, ID, and work_id for the requested path
        let target_node = sqlx::query!(
            r#"
            SELECT id, work_id, start_index, end_index FROM nodes WHERE path = $1::text::ltree
            "#,
            path
        ).fetch_one(&self.pool).await?;

        // 2. Fetch texts within those bounds, and inject the node context into the response
        let contents = sqlx::query_as::<_, ScriptureContent>(
            r#"
                SELECT
                    $1::uuid as node_id,
                    $2::text as path,
                    t.body_text,
                    e.name as edition_name,
                    e.language_code,
                    t.absolute_index,
                    NULL::jsonb as translation_metadata
                FROM texts t
                JOIN editions e ON t.edition_id = e.id
                WHERE t.absolute_index BETWEEN $3 and $4
                    AND e.work_id = $5::uuid
                ORDER BY t.absolute_index ASC, e.is_source DESC
                "#
        )
            .bind(target_node.id)
            .bind(path)
            .bind(target_node.start_index)
            .bind(target_node.end_index)
            .bind(target_node.work_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(contents)
    }

    /// ## `resolve_address`
    /// **Parameters:** /// * `work_slug: &str` (The unique slug for the scriptural corpus, e.g., "bible").
    /// * `alias: &str` (The human shorthand provided by the user, e.g., "Jn").
    ///
    /// ### Architectural Design Decision: Human-to-Machine Translation
    /// Decouples the user's interface from the internal database structure. This allows
    /// "Jn" to map to the canonical path `bible.nt.john` without hardcoding strings in Rust.
    ///
    /// ### Design Decision: Case-Insensitive Mapping
    /// We use `ILIKE` to handle user variance (e.g., "jn" vs "JN"). The result is limited to 1
    /// to ensure deterministic resolution even if overlapping aliases exist in the DB.
    ///
    /// ### SQL Quirk: Multi-Table Decoupling
    /// This query joins `node_aliases`, `nodes`, and `works` to ensure the alias resolution
    /// is scoped correctly to the specific tradition being queried.
    ///
    /// **AI Prompt Hint:** To support multi-language book names (e.g., "Génesis" in Spanish),
    /// simply add the localized string as a new entry in the `node_aliases` table.
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

    /// ## `search`
    /// ### SQL Quirk: Postgres FTS (Full-Text Search)
    /// 1. `to_tsvector`: Converts the body text into searchable tokens.
    /// 2. `websearch_to_tsquery`: Converts user input ("holy spirit") into a boolean query.
    /// 3. `ts_rank`: Calculates the `relevance_score` to sort the best matches first.
    /// 4. `ts_headline`: Generates a short snippet wrapping the matched words in `<b>` tags.
    ///
    /// ### Design Decision: Scoped Search
    /// If `path_scope` is provided, we use the `ltree` descendant oeprator `<@` to only
    /// search within a specific book or testament.
    async fn search(
        &self,
        query: &str,
        path_scope: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Pagination<SearchMatch>> {
        // Step 1: Count total records for pagination
        // In a massive production DB, this would need to be cache, but for now we'l do a live count.
        // todo caching for big production dbs
        let count_query = r#"
            SELECT COUNT(*)
            FROM texts t
            JOIN editions e ON t.edition_id = e.id
            JOIN nodes n ON e.work_id = n.work_id AND t.absolute_index BETWEEN n.start_index AND n.end_index
            WHERE to_tsvector('english', t.body_text) @@ websearch_to_tsquery('english', $1)
            AND ($2::text IS NULL OR n.path <@ $2::text::ltree)
            "#;

        let total_records: i64 = sqlx::query_scalar(count_query)
            .bind(query)
            .bind(path_scope)
            .fetch_one(&self.pool)
            .await?;

        // Step 2: Fetch the actual ranked and snippeted results
        let search_query = r#"
            WITH search_results AS (
                SELECT
                    n.id as node_id,
                    n.path::text as path,
                    e.name as edition_name,
                    ts_headline('english', t.body_text, websearch_to_tsquery('english', $1), 'StartSel=<b>, StopSel=</b>') as snippet,
                    ts_rank(to_tsvector('english', t.body_text), websearch_to_tsquery('english', $1)) as relevance_score
                FROM texts t
                JOIN editions e ON t.edition_id = e.id
                JOIN nodes n ON e.work_id = n.work_id AND t.absolute_index BETWEEN n.start_index AND n.end_index
                WHERE to_tsvector('english', t.body_text) @@ websearch_to_tsquery('english', $1)
                AND ($2::text IS NULL OR n.path <@ $2::text::ltree)
            )
            SELECT * FROM search_results
            ORDER BY relevance_score DESC
            LIMIT $3 OFFSET $4
            "#;

        let matches = sqlx::query_as::<_, SearchMatch>(search_query)
            .bind(query)
            .bind(path_scope)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        // Calculate pagination metadata
        let total_pages = (total_records as f64 / limit as f64).ceil() as i64;
        let current_page = ( offset / limit ) +1;

        Ok(Pagination{
            data: matches,
            total_records,
            current_page,
            total_pages,
            has_next: current_page < total_pages,
        })
    }
}
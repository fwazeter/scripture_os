use sqlx::PgPool;
use crate::models::ScriptureContent;

/// Retrieves text and edition metadata for a given ltree path.
/// Matches the `fetchText` specification in dev plan.
pub async fn fetch_text(pool: &PgPool, path: &str) -> Result<Vec<ScriptureContent>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT t.body_text, e.name as edition_name, e.language_code
        FROM texts t
        JOIN nodes n ON t.node_id = n.id
        JOIN editions e ON t.edition_id = e.id
        WHERE n.path <@ $1::ltree
        ORDER BY t.body_text ASC
        "#
    )
    .bind(path).fetch_all(pool)
    .await
}
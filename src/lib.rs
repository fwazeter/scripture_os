pub mod models;
use models:: ScriptureContent;
use sqlx:: PgPool;

// Shared logic for testing, main file, etc.
pub async fn get_verses_by_path(pool: &PgPool, path: &str) -> Result<Vec<ScriptureContent>, sqlx::Error> {
    sqlx:: query_as(
        r#"
                SELECT t.body_text, e.name as edition_name, e.language_code
                FROM texts t
                JOIN nodes n ON t.node_id = n.id
                JOIN editions e ON t.edition_id = e.id
                WHERE n.path <@ $1::ltree
                ORDER BY n.sort_order ASC
            "#
    )
        .bind(path)
        .fetch_all(pool)
        .await
}
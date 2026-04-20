use sqlx::PgPool;
use serde::Serialize;
use anyhow::Result;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_multiple_editions() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        // Fetch John 17:3. Seed data has both KJV & Greek Translations.
        let results = fetch_text(&pool, "bible_test.nt.john.17.3").await.unwrap();

        assert_eq!(results.len(), 2);
    }
}
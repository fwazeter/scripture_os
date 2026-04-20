use sqlx::PgPool;
use anyhow::Result;
use crate::models::ScriptureContent;

/// Retrieves text and edition metadata for a given ltree path.
/// Matches the `fetchText` specification in dev plan.
pub async fn fetch_text(pool: &PgPool, path: &str) -> Result<Vec<ScriptureContent>> {
    // 1. Fetch the range bounds for the requested path
    let bounds = sqlx::query!(
        r#"
        SELECT start_index, end_index FROM nodes WHERE path = $1::text::ltree
        "#,
        path
    )
        .fetch_one(pool)
        .await?;

    // 2. Fetch all texts taht fall between those bounds for the correct Work
    let contents = sqlx::query_as::<_, ScriptureContent>(
        r#"
            SELECT
                t.body_text,
                e.name as edition_name,
                e.language_code,
                t.absolute_index
            FROM texts t
            JOIN editions e ON t.edition_id = e.id
            WHERE t.absolute_index BETWEEN $1 AND $2
                AND e.work_id = (SELECT work_id FROM nodes WHERE path = $3::ltree)
            ORDER BY t.absolute_index ASC, e.is_source DESC
            "#
    )
        .bind(bounds.start_index)
        .bind(bounds.end_index)
        .bind(path)
        .fetch_all(pool)
        .await?;

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_range_psalm_title() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        // Bible groups both title segments into one node (indices 1000 and 1001)
        let results = fetch_text(&pool, "bible.ot.psalms.51.title").await.unwrap();

        // Should return 6 rows: 2 indices (1000, 1001) * 3 Bible translations (KJV, NIV, LXX)
        assert_eq!(results.len(), 6);
    }
}
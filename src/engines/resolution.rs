use sqlx::PgPool;
use regex::Regex;
use anyhow::{Result, Context};

/// Resolves a human-readable query like "John 3:16" into an ltree path using DB aliases.
pub async fn parse_address(pool: &PgPool, work_slug: &str, input: &str) -> Result<String> {
    // 1. Extract book, chapter and verse using regex
    // Matches "Jn 17:3", "John 17:3", "1 John 17:3", etc.
    let re = Regex::new(r"^(?P<book>(\d\s)?[A-Za-z]+)\s+(?P<chapter>\d+):(?P<verse>\d+)$")
        .context("Failed to compile regex")?;

    let caps = re.captures(input).context("Invalid address format. Expected format: 'Book Chapter:Verse'")?;

    let alias_input = caps.name("book").unwrap().as_str();
    let chapter = caps.name("chapter").unwrap().as_str();
    let verse = caps.name("verse").unwrap().as_str();

    // 2. Query the DB to resolve the alias to the canonical ltree path
    // We join `nodes` and `node_aliases` to find the base path for the book.
    let record = sqlx::query!(
        r#"
        SELECT n.path::text as base_path
        FROM node_aliases na
        JOIN nodes n ON na.node_id = n.id
        JOIN works w ON n.work_id = w.id
        WHERE na.alias ILIKE $1 AND w.slug = $2
        LIMIT 1
        "#,
        alias_input,
        work_slug,
    )
        .fetch_optional(pool)
        .await?;

    // 3. Assemble final ltree path
    if let Some(row) = record {
        // e.g. base_path = "bible.nt.john", chapter = "3", verse = "16"
        let path = row.base_path.unwrap_or_default();
        Ok(format!("{}.{}.{}", path, chapter, verse))
    } else {
        anyhow::bail!("Book alias '{}' not found in work '{}", alias_input, work_slug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_address_basic_routing() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

       // Seed data maps "Jn" -> "bible_test.nt.john"
        let ltree_path = parse_address(&pool, "bible_test", "Jn 17:3").await.unwrap();

        // 5. Assert
        assert_eq!(ltree_path, "bible_test.nt.john.17.3");
    }
}
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
    use sqlx::postgres::PgPoolOptions;
    use dotenvy::dotenv;
    use std::env;
    use uuid::Uuid;

    async fn setup_db() -> PgPool {
        dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgPoolOptions::new().connect(&db_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_parse_address_basic_routing() {
        let pool = setup_db().await;

       // Use fixed UUIDS and unique test slugs so does not collide with integration tests
        let work_id = Uuid::parse_str("99999999-9999-9999-9999-999999999991").unwrap();
        let node_id = Uuid::parse_str("99999999-9999-9999-9999-999999999992").unwrap();

        // 1. Insert Work
        sqlx::query(
            r#"
                INSERT INTO works (id, title, slug)
                VALUES ($1, 'Bible Unit Test', 'bible_test')
                ON CONFLICT (id) DO NOTHING
                "#
        )
            .bind(work_id).execute(&pool)
            .await.unwrap();

        // 2. Insert Node (using ON CONFLICT DO UPDATE to ensure our fixed node_id is the one saved)
        sqlx::query(
            r#"
                INSERT INTO nodes (id, work_id, path, node_type, sort_order)
                VALUES ($1, $2, 'bible_test.nt.john', 'book', 1.0)
                ON CONFLICT (path) DO UPDATE SET id = $1
                "#
        )
        .bind(node_id).bind(work_id).execute(&pool)
        .await.unwrap();

        // 3. Insert Alias
        sqlx::query(
            r#"
                INSERT INTO node_aliases (node_id, alias, is_canonical)
                VALUES ($1, 'JnTest', true)
                ON CONFLICT DO NOTHING
                "#
        )
            .bind(node_id).execute(&pool)
            .await.unwrap();

        // 4. Execute Target function
        let ltree_path = parse_address(&pool, "bible_test", "JnTest 3:16")
            .await.unwrap();

        // 5. Assert
        assert_eq!(ltree_path, "bible_test.nt.john.3.16");
    }
}
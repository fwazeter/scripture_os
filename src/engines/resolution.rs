//! # Resolution Engine (The "Router")
//!
//! The Resolution Engine is responsible for translating messy, human-readable input strings
//! (e.g., `"Jn 17:3"`) into strict, canonical database addresses (e.g., `"bible.nt.john.17.3"`).
//!
//! It relies heavily on the `node_aliases` table to map arbitrary abbreviations, alternate
//! spellings, and differing language inputs to their permanent `ltree` base paths.

use sqlx::PgPool;
use regex::Regex;
use anyhow::{Result, Context};

/// Parses a human shorthand string and resolves it to a canonical `ltree` path.
///
/// # Logic Flow
/// 1. **Regex Extraction:** The function uses a regular expression to parse the input into three
///    distinct capture groups: `book` (the alias), `chapter`, and `verse`.
/// 2. **Alias Lookup:** It queries the `node_aliases` table using a case-insensitive match (`ILIKE`)
///    to find the base `ltree` path for the extracted book alias.
/// 3. **Path Assembly:** It concatenates the base path, chapter, and verse into a valid `ltree` string.
///
/// # Design Decisions
/// * **Why Regex?** Standardizing human input is notoriously difficult (e.g., "1 John", "I Jn", "1Jn").
///   By extracting just the alphanumeric prefix and resolving it against a dedicated alias table,
///   we offload the complexity of alternative naming from the Rust codebase directly into the database schema.
/// * **Error Handling:** This function utilizes the `anyhow` crate to provide rich, chained error context.
///   If the regex fails or the alias is missing, it returns a precise string detailing exactly what went wrong.
///
/// # Arguments
/// * `pool` - A shared reference to the Postgres connection pool.
/// * `work_slug` - The specific work to search within (e.g., `"bible"` or `"quran_hafs"`).
/// * `input` - The raw string provided by the user/client (e.g., `"Jn 17:3"`).
pub async fn parse_address(pool: &PgPool, work_slug: &str, input: &str) -> Result<String> {
    // 1. Extract book, chapter and verse using regex
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
        let ltree_path = parse_address(&pool, "bible", "Jn 17:3").await.unwrap();

        // 5. Assert
        assert_eq!(ltree_path, "bible.nt.john.17.3");
    }
}
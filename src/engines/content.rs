//! # Content Engine (The "Assembler")
//!
//! This module is responsible for bridging the gap between the Structural Spine (`nodes`)
//! and the Universal Sequence (`texts`). Because Scripture OS utilizes a Stand-off Markup
//! architecture, texts are stored sequentially and are completely ignorant of their
//! hierarchical addresses (like Book, Chapter, or Verse).
//!
//! The Content Engine acts as the assembler, taking an `ltree` address, finding its
//! start and end sequence boundaries, and returning all the text that falls within that range.

use anyhow::Result;
use crate::models::ScriptureContent;
use crate::repository::ScriptureRepository;

/// Retrieves the physical text and translation metadata for a given structural address.
///
/// This function executes a two-step resolution process to accommodate the Stand-off Markup model:
/// 1. **Boundary Resolution:** It looks up the requested `ltree` path in the `nodes` table
///    to find the `start_index` and `end_index`. This defines the contiguous block of text requested.
/// 2. **Content Aggregation:** It queries the `texts` table for all rows whose `absolute_index`
///    falls between those bounds, joining against the `editions` table to attach translation data.
///
/// # Design Decisions & SQL Quirks
/// * **The `::text::ltree` Cast:** Notice the `$1::text::ltree` casting in the queries.
///   The `sqlx` macro enforces strict compile-time type checking. Because `path` is a custom
///   PostgreSQL `ltree` type, `sqlx` normally expects a specialized `PgLTree` Rust struct.
///   By casting the Rust `&str` to `text` and then to `ltree` within Postgres, we bypass
///   this strictness and keep our Rust function signatures clean.
/// * **Ordering:** Results are ordered by `absolute_index ASC` to ensure the text reads chronologically,
///   and secondarily by `is_source DESC` so original language manuscripts appear before translations
///   if multiple editions are returned.
///
/// # Arguments
/// * `pool` - A shared reference to the Postgres connection pool.
/// * `path` - The canonical `ltree` path to fetch (e.g., `"bible.nt.john.17.3"`).
pub async fn fetch_text(repo: &dyn ScriptureRepository, path: &str) -> Result<Vec<ScriptureContent>> {
    repo.fetch_text(path).await
}

#[cfg(test)]
mod tests {
    use crate::repository::postgres::PostgresRepository;
    use super::*;

    #[tokio::test]
    async fn test_fetch_range_psalm_title() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;
        // Wrap pool in repo for engine
        let repo = PostgresRepository::new(pool);

        // Bible groups both title segments into one node (indices 1000 and 1001)
        let results = fetch_text(&repo, "bible.ot.psalms.51.title").await.unwrap();

        // Should return 6 rows: 2 indices (1000, 1001) * 3 Bible translations (KJV, NIV, LXX)
        assert_eq!(results.len(), 6);
    }
}
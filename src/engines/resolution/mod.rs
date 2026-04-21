//! # Resolution Engine (The "Router")
//!
//! The Resolution Engine is responsible for translating messy, human-readable input strings
//! (e.g., "Jn 17:3") into strict, canonical database addresses (e.g., "bible.nt.john.17.3").
//!
//! ### Architectural Design Decision: Address Abstraction
//! Scripture OS separates the "Human Interface" from the "Canonical Spine". This engine
//! ensures that the internal database schema (LTREE) remains decoupled from the
//! specific abbreviations or languages used by a client.

use anyhow::{Context, Result};
use async_trait::async_trait;
use regex::Regex;
use std::sync::Arc;

use crate::repository::ScriptureRepository;
use super::ResolutionEngine;

/// # Core Resolution Engine
///
/// This is the primary implementation of the `ResolutionEngine` trait.
///
/// ### Architectural Design Decision: Dependency Injection (DI)
/// By encapsulating the `ScriptureRepository` inside this struct via an `Arc`,
/// the engine manages its own data access. This design paves the way for Phase 3
/// (The "Versification Mapper"), allowing us to inject additional mapping utilities
/// into this struct in the future without changing the public trait contract.
pub struct CoreResolutionEngine{
    repo: Arc<dyn ScriptureRepository + Send + Sync>,
}

impl CoreResolutionEngine {
    /// Bootstraps the engine by injecting the required data layer repository.
    pub fn new (repo: Arc<dyn ScriptureRepository + Send + Sync>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ResolutionEngine for CoreResolutionEngine {
    /// ## `parse_address`
    /// **Parameters:** /// * `work_slug: &str` (The identifier for the corpus, e.g., "bible").
    /// * `input: &str` (The raw shorthand string from the user, e.g., "Jn 17:3").
    ///
    /// ### Architectural Design Decision: Normalization through Aliasing
    /// Standardizing human input is notoriously difficult (e.g., "1 John" vs "I Jn"). Instead
    /// of hardcoding every variant in Rust, we extract the core components and resolve
    /// them against a dedicated `node_aliases` via the injected repository.
    ///
    /// ### Design Decision: Regex-Based Decomposition
    /// The function executes a three-step resolution flow:
    /// 1. **Extraction:** Uses a regular expression to split the input into `book`, `chapter`, and `verse` groups.
    /// 2. **Alias Resolution:** Delegates the `book` string to the repository to find the canonical base path.
    /// 3. **Canonical Assembly:** Recombines the base path with the validated numeric components into a final LTREE string.
    ///
    /// ### Technical Context: Case-Insensitive Matching
    /// While the regex ensures the *structure* of the input is correct, the repository
    /// handles the *semantics* of the alias (e.g., mapping "jn" and "JN" to the same node)
    /// via case-insensitive database lookups.
    ///
    /// **AI Prompt Hint:** If you need to support more complex addressing (like "John 3:16-17"
    /// or "Gen 1:1, 5"), the Regex here must be updated to handle non-numeric characters
    /// in the `verse` capture group.
    async fn parse_address(&self, work_slug: &str, input: &str) -> Result<String> {
        // 1. Extract book, chapter and verse using regex
        // The pattern allows for leading numbers (e.g., "1 John") followed by a space and colon-separated digits.
        // todo universalize the formatting so its not hard coded to chapter: verse
        let re = Regex::new(r"^(?P<book>(\d\s)?[A-Za-z]+)\s+(?P<chapter>\d+):(?P<verse>\d+)$")
            .context("Failed to compile regex")?;

        let caps = re.captures(input).context("Invalid address format. Expected format: 'Book Chapter:Verse'")?;

        let alias_input = caps.name("book").unwrap().as_str();
        let chapter = caps.name("chapter").unwrap().as_str();
        let verse = caps.name("verse").unwrap().as_str();

        // 2. Resolve the book alias toa  canonical LTREE base path using the injected repo
        let base_path = self.repo.resolve_address(work_slug, alias_input).await?;

        // 3. Assemble the final address string
        if let Some(path) = base_path {
            Ok(format!("{}.{}.{}", path, chapter, verse))
        } else {
            anyhow::bail!("Book alias '{}' not found", alias_input)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::postgres::PostgresRepository;
    use super::*;

    #[tokio::test]
    async fn test_parse_address_basic_routing() {
        let pool = crate::test_utils::setup_db().await;
        crate::test_utils::seed_universal_data(&pool).await;

        // 1. Initialize the concrete repository and wrap it in an Arc for DI
        let repo = Arc::new(PostgresRepository::new(pool));

        // 2. Inject the repository into the Engine
        let engine = CoreResolutionEngine::new(repo);

        // 3. Call the trait method on the engine (Notice we no longer pass `&repo` here)
        // Seed data maps "Jn" -> "bible_test.nt.john"
        let ltree_path = engine.parse_address("bible", "Jn 17:3").await.unwrap();

        // 4. Assert
        assert_eq!(ltree_path, "bible.nt.john.17.3");
    }
}
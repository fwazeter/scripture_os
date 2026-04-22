//! # Resolution Engine (The "Router")
//!
//! The Resolution Engine is responsible for translating messy, human-readable input strings
//! (e.g., "Jn 17:3", "Quran 3:1") into strict, canonical database addresses (e.g., "bible.nt.john.17.3").
//!
//! ### Architectural Design Decision: Universal Coordinate Parsing
//! The engine is completely ignorant of "Chapters" or "Verses". It splits input into an
//! `Alias` ("1 John") and `Coordinates` ("3:16-18"). This allows it to parse any
//! scriptural stricture (like Rig Veda's 1.1.1) natively.

use anyhow::{Context, Result};
use async_trait::async_trait;
use regex::Regex;
use std::sync::Arc;

use crate::models::ResolvedAddress;
use crate::repository::ScriptureRepository;
use super::ResolutionEngine;

// --- Reusable Parsing Functions ---

/// Extracts the human name (alias) and the traversal numbers (coords)
fn extract_alias_and_coords(input: &str) -> Result<(String, String)> {
    // Matches an optional leading number, the book name, and then dumps all remaining
    // numeric/punctuation data into a single 'coords' bucket.
    let re = Regex::new(
        r"^(?P<alias>(?:\d\s+)?[a-zA-Z]+(?:\s+[a-zA-Z]+)*)\s*(?P<coords>[\d\:\.\-a-zA-Z]*)$")
        .context("Failed to compile regex")?;

    let caps = re.captures(input).context("Invalid address format.")?;
    let alias = caps.name("alias").unwrap().as_str().to_string().trim().to_string();
    let coords = caps.name("coords").unwrap().as_str().trim().to_string();

    Ok((alias, coords))
}

/// Builds the start and end paths, inheriting context for shorthand ranges (e.g., 3:16-18)
fn build_coordinate_paths(base_path: &str, coords: &str) -> ResolvedAddress {
    if coords.is_empty() {
        return ResolvedAddress {
            start_path: base_path.to_string(),
            end_path: None,
        }
    }

    // Normalize any colons or custom delimiters into ltree dot notation
    let normalized = coords.replace(":", ".");
    let parts: Vec<&str> = normalized.split("-").collect();

    let start_path = format!("{}.{}", base_path, parts[0]);

    if parts.len() == 1 {
        return ResolvedAddress { start_path, end_path: None };
    }

    // Handle the Range (End Path)
    let start_segments: Vec<&str> = parts[0].split('.').collect();
    let end_segments: Vec<&str> = parts[1].split('.').collect();

    // Smart Context Inheriting: If user typed "17:2-3", end_segments is just ["3"].
    // We inherit the "17" from the start_segments.
    let end_path = if end_segments.len() < start_segments.len() {
        let mut full_end = start_segments[0..(start_segments.len() - end_segments.len())].to_vec();
        full_end.extend(end_segments);
        format!("{}.{}", base_path, full_end.join("."))
    } else {
        // Cross-chapter range (e.g., "3:1 - 4:55")
        format!("{}.{}", base_path, parts[1])
    };

    ResolvedAddress {
        start_path,
        end_path: Some(end_path),
    }
}

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
    async fn parse_address(&self, work_slug: &str, input: &str) -> Result<ResolvedAddress> {
        let (alias, coords) = extract_alias_and_coords(input)?;

        let base_path = self.repo.resolve_address(work_slug, &alias).await?
            .context(format!("Book alias '{}' not found", alias))?;

        Ok(build_coordinate_paths(&base_path, &coords))
    }
}

// --- Track A: Concrete Integration Tests ---
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

        let resolved = engine.parse_address("bible", "Jn 17:3").await.unwrap();
        assert_eq!(resolved.start_path, "bible.nt.john.17.3");
        assert!(resolved.end_path.is_none());
    }
}

// --- Track B: Isolated Mock Tests ---
#[cfg(test)]
mod mock_tests {
    use super::*;
    use crate::models::{HierarchyNode, Adjacency, ScriptureContent, SearchMatch, Pagination};
    use uuid::Uuid;

    struct MockRepository;

    #[async_trait]
    impl ScriptureRepository for MockRepository {
        async fn resolve_address(&self, work_slug: &str, alias: &str) -> Result<Option<String>> {
            if work_slug == "mock_work" && alias == "MockAlias" {
                Ok(Some("mock.canonical.path".to_string()))
            } else {
                Ok(None)
            }
        }
        // Stubs
        async fn get_hierarchy(&self, _p: &str) -> Result<Vec<HierarchyNode>> { Ok(vec![]) }
        async fn get_adjacent_nodes(&self, _id: Uuid) -> Result<Adjacency> { Ok(Adjacency { previous: None, next: None }) }
        async fn fetch_text(&self, _p: &str) -> Result<Vec<ScriptureContent>> { Ok(vec![]) }
        async fn search(&self, _q: &str, _s: Option<&str>, _l: i64, _o: i64) -> Result<Pagination<SearchMatch>> { unimplemented!() }
    }

    #[tokio::test]
    async fn test_universal_coordinate_parsing() {
        let repo = Arc::new(MockRepository);
        let engine = CoreResolutionEngine::new(repo);

        // 1. Standard Single Node
        let res = engine.parse_address("mock_work", "MockAlias 3:16").await.unwrap();
        assert_eq!(res.start_path, "mock.canonical.path.3.16");

        // 2. Alpha-numeric Node (e.g., 16a)
        let res_alpha = engine.parse_address("mock_work", "MockAlias 3:16a").await.unwrap();
        assert_eq!(res_alpha.start_path, "mock.canonical.path.3.16a");

        // 3. Short Range (Inherits Chapter)
        let res_short = engine.parse_address("mock_work", "MockAlias 3:16-18").await.unwrap();
        assert_eq!(res_short.start_path, "mock.canonical.path.3.16");
        assert_eq!(res_short.end_path.unwrap(), "mock.canonical.path.3.18");

        // 4. Cross-Chapter Range (Quran 3:1 - 4:55)
        let res_cross = engine.parse_address("mock_work", "MockAlias 3:1-4:55").await.unwrap();
        assert_eq!(res_cross.start_path, "mock.canonical.path.3.1");
        assert_eq!(res_cross.end_path.unwrap(), "mock.canonical.path.4.55");

        // 5. Whole Book (No coords)
        let res_book = engine.parse_address("mock_work", "MockAlias").await.unwrap();
        assert_eq!(res_book.start_path, "mock.canonical.path");
    }
}

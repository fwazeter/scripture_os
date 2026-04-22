//! # Repository Abstraction (The "Contract")
//!
//! This module defines the `ScriptureRepository` trait, which serves as the
//! primary data access layer for Scripture OS.
//!
//! ### Architectural Design Decision: Engine-Storage Decoupling
//! By defining a trait, we separate the business logic of the "Engines"
//! from physical storage details. This ensures that the Service Layer
//! remains pure, testable, and agnostic of specific database syntax.

pub mod postgres;
pub use self::postgres::PostgresRepository;

use anyhow::Result;
use uuid::Uuid;
use crate::models::{
    HierarchyNode,
    Adjacency,
    ScriptureContent,
    SearchMatch,
    Pagination
};
use async_trait::async_trait;

/// The core trait for all data operations in Scripture OS.
///
/// All methods are asynchronous and return `anyhow::Result` to provide
/// flexible error propagation across the system.
#[async_trait]
pub trait ScriptureRepository: Send + Sync {
    /// ## `get_hierarchy`
    /// **Parameters:** `parent_path: &str` (The canonical hierarchical path).
    ///
    /// ### Architectural Design Decision: Breadth-First Navigation
    /// This function is the primary tool for structural discovery, designed
    /// to return immediate children to populate UI menus without loading
    /// the entire tree.
    ///
    /// ### Technical Context: Depth Restriction
    /// Implementations should only return direct children (nlevel + 1) to prevent
    /// accidental recursive data floods during navigation.
    ///
    /// **AI Prompt Hint:** If creating a new implementation (e.g., for SQLite),
    /// ensure the query filters specifically for the next level in the hierarchy string.
    async fn get_hierarchy(&self, parent_path: &str) -> Result<Vec<HierarchyNode>>;

    /// ## `get_adjacent_nodes`
    /// **Parameters:** `current_node_id: Uuid` (The unique ID of the anchor node).
    ///
    /// ### Architectural Design Decision: Linear Flow Preservation
    /// Supports the linear consumption of scripture, allowing "Next" and
    /// "Previous" logic while respecting the user's structural context.
    ///
    /// ### Design Decision: Type-Strict Siblings
    /// The contract requires that "adjacent" nodes must be of the same `node_type`
    /// (e.g., the next Chapter, not the first verse of the next Chapter).
    ///
    /// **AI Prompt Hint:** Use `Option::zip` when mapping results to ensure that
    /// adjacency data is only returned if both the ID and Path are valid.
    async fn get_adjacent_nodes(&self, current_node_id: Uuid) -> Result<Adjacency>;

    /// ## `fetch_text`
    /// **Parameters:** /// * `start_path: &str` (The canonical address to begin retrieval).
    /// * `end_path: Option<&str>` (An optional canonical address to end retrieval).
    ///
    /// ### Architectural Design Decision: Content Assembly
    /// Bridges the Structural Spine and the sequential text. It must resolve the
    /// sequence indices of the provided path(s) and return all linguistic content
    /// falling within those boundaries.
    ///
    /// ### Technical Context: Stand-off Markup
    /// Because text is stored in a separate sequence from the hierarchy, the
    /// implementer must resolve the path's index boundaries before fetching text.
    ///
    /// **AI Prompt Hint:** Ensure that the result vector is ordered by `is_source DESC`
    /// so that original languages (Greek/Hebrew) appear first in the array.
    async fn fetch_text(&self, start_path: &str, end_path: Option<&str>) -> Result<Vec<ScriptureContent>>;

    /// ## `resolve_address`
    /// **Parameters:** /// * `work_slug: &str` (The scope of the work, e.g., "bible").
    /// * `alias: &str` (The human-readable shorthand, e.g., "Jn").
    ///
    /// ### Architectural Design Decision: Alias Abstraction
    /// Decouples user input from internal database paths, allowing the system
    /// to support multiple languages and abbreviations without code changes.
    ///
    /// ### Design Decision: Case-Insensitivity
    /// The contract requires `alias` matches to be case-insensitive to accommodate
    /// varied user input (e.g., "gen" vs "Gen").
    ///
    /// **AI Prompt Hint:** If the alias is not found, return `Ok(None)` rather
    /// than an error, as a missing alias is a valid logical outcome.
    async fn resolve_address(&self, work_slug: &str, alias: &str) -> Result<Option<String>>;

    /// ## `search`
    /// **Parameters:** /// * `query: &str` (The raw user search string).
    /// * `path_scope: Option<&str>` (Optional LTREE path to restrict the search).
    /// * `limit: i64`, `offset: i64` (Pagination controls).
    ///
    /// ### Architectural Design Decision: Delegated FTS
    /// Full-Text Search (FTS) is highly database specific. This method allows
    /// the repository to utilize native engine optimizations (like Postgres `ts_vector`)
    /// while keeping the Service Layer ignorant of the indexing strategy.
    ///
    /// **AI Prompt Hint:** When implementing for Postgres, utilize `websearch_to_tsquery`
    /// to support Google-style search operators (e.g., "word1 -word2").
    async fn search(
        &self,
        query: &str,
        path_scope: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Pagination<SearchMatch>>;
}
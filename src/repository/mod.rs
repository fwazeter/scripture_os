//! # Repository Abstraction (The "Contract")
//!
//! This module defines the `ScriptureRepository` trait, which serves as the primary
//! data access layer for Scripture OS.
//!
//! ### Architectural Design Decision: Engine-Storage Decoupling
//! By defining a trait, we separate the business logic of the "Engines" from the
//! physical storage details. This ensures that the Resolution, Content,
//! and Traversal engines remain pure and testable.
//!
//! ### Design Decision: Async Trait Pattern
//! Because native Rust traits do not yet fully support `async fn` in a way that allows
//! for dynamic dispatch (`dyn Trait`), we utilize the `async_trait` macro.
//! This allows our Engines to remain agnostic of the concrete repository type.
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
    /// **Parameters:** `parent_path: &str` (The canonical hierarchical path of the parent node).
    ///
    /// ### Architectural Design Decision: Breadth-First Navigation
    /// This function is the primary tool for structural discovery. It is designed
    /// to return immediate children to populate UI menus without loading the entire tree.
    ///
    /// ### Design Decision: Contractual Ordering
    /// Implementers **must** return nodes in their natural reading order (typically
    /// determined by `sort_order` or `start_index`).
    ///
    /// ### Technical Context: Depth Restriction
    /// Implementations should only return direct children (depth + 1) to prevent
    /// accidental recursive data floods.
    ///
    /// **AI Prompt Hint:** If you are creating a new implementation (e.g., for SQLite),
    /// ensure the query filters specifically for the next level in the hierarchy string.
    async fn get_hierarchy(&self, parent_path: &str) -> Result<Vec<HierarchyNode>>;

    /// ## `get_adjacent_nodes`
    /// **Parameters:** `current_node_id: Uuid` (The unique ID of the anchor node).
    ///
    /// ### Architectural Design Decision: Linear Flow Preservation
    /// Scripture is consumed both hierarchically and linearly. This function
    /// supports the linear flow, allowing "Next" and "Previous" logic.
    ///
    /// ### Design Decision: Type-Strict Siblings
    /// The contract requires that "adjacent" nodes must be of the same `node_type`
    /// (e.g., the next Chapter, not the first verse of the next Chapter).
    ///
    /// ### Technical Context: Boundary Handling
    /// Implementers must handle "Edge of the World" scenarios. If a node
    /// is the first in a work, `previous` should return `None`.
    ///
    /// **AI Prompt Hint:** Use `Option::zip` when mapping results to ensure that
    /// adjacency data is only returned if both the ID and Path are valid.
    async fn get_adjacent_nodes(&self, current_node_id:Uuid) -> Result<Adjacency>;

    /// ## `fetch_text`
    /// **Parameters:** `path: &str` (The canonical address to retrieve, e.g., "bible.nt.john.1").
    ///
    /// ### Architectural Design Decision: Content Assembly
    /// This function bridges the Structural Spine and the actual text.
    /// It must support fetching single verses or entire chapters based on the path depth.
    ///
    /// ### Design Decision: Multi-Edition Support
    /// The repository is expected to return *all* available translations or
    /// manuscripts for the given path, allowing the Content Engine to format
    /// side-by-side views.
    ///
    /// ### Technical Context: Stand-off Markup
    /// Because text is stored in a separate sequence from the hierarchy, the
    /// implementer must resolve the path's index boundaries before fetching text.
    ///
    /// **AI Prompt Hint:** Ensure that the result vector is ordered by `is_source DESC`
    /// so that original languages (Greek/Hebrew) appear first in the array.
    async fn fetch_text(&self, path: &str) -> Result<Vec<ScriptureContent>>;

    /// ## `resolve_address`
    /// **Parameters:** /// * `work_slug: &str` (The scope of the work, e.g., "bible").
    /// * `alias: &str` (The human-readable shorthand, e.g., "Jn").
    ///
    /// ### Architectural Design Decision: Alias Abstraction
    /// Decouples user input from internal database paths. This allows
    /// the system to support multiple languages and abbreviations without changing the core code.
    ///
    /// ### Design Decision: Case-Insensitivity
    /// The contract requires `alias` matches to be case-insensitive to accommodate
    /// varied user input (e.g., "gen" vs "Gen").
    ///
    /// ### Technical Context: Scoped Resolution
    /// Resolution must be scoped to the `work_slug` to prevent name collisions
    /// between different scriptural traditions.
    ///
    /// **AI Prompt Hint:** If the alias is not found, return `Ok(None)` rather
    /// than an error, as a missing alias is a valid logical outcome.
    async fn resolve_address(&self, work_slug: &str, alias: &str) -> Result<Option<String>>;

    /// ## `search`
    /// **Parameters** /// * `query: &str` (The raw user search string).
    /// * `path_scope: Option<&str>` (An optional LTREE path to restrict the search, e.g., "bible.nt").
    /// * `limit: i64`, `offset: i64` (For pagination)
    ///
    /// ### Architectural Design Decision: Delegated FTS
    /// Full-Text Search (FTS) is highly database specific. By placing this in the repository,
    /// we allow Postgres to use its native `ts_vector` and `ts_headline` functions, keeping
    /// the Service Layer ignorant of the indexing strategy.
    async fn search(
        &self,
        query: &str,
        path_scope: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Pagination<SearchMatch>>;
}
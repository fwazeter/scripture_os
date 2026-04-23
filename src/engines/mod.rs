use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use crate::fsi::models::{MacroID, NamespaceID, WorkID};
use crate::models::{
    HierarchyNode,
    Adjacency,
    ResolvedAddress,
    ScriptureContent,
    Comparison,
    SearchMatch,
    Pagination
};

// Export the submodules (legacy)
pub mod content;
pub mod resolution;
pub mod traversal;
pub mod search;

// -- New FSI v4.0 Modules --
pub mod fsi_content; // temp. name to avoid colliding with legacy content.

// -- Service Layer Contracts ---

/// # The Content Engine Contract
///
/// ### Architectural Design Decision: Decoupling Assembly from Storage
/// This trait defines how the system requests assembled text. The Engine knows nothing
/// about SQL or physical tables; it only knows how to ask the Repository for FSI
/// sequences and stitch them together into a final product.
#[async_trait]
pub trait FsiContentEngine: Send + Sync {
    async fn assemble_macro(
        &self,
        work: WorkID,
        macro_level: MacroID,
        namespace: NamespaceID,
    ) -> Result<String>;
}


/// Text assembly engine trait - legacy
#[async_trait]
pub trait ContentEngine: Send + Sync {
    /// Retrieves text segments for a given canonical ltree path or range.
    async fn fetch_text(&self, start_path: &str, end_path: Option<&str>) -> Result<Vec<ScriptureContent>>;

    /// Groups translations by their shared canonical node for side-by-side viewing.
    async fn get_comparison(&self, start_path: &str, end_path: Option<&str>) -> Result<Vec<Comparison>>;

    // Future methods to implement:
    // async get_verse_bundle(&self, path: &str) -> Result<...>;
    // async fn get_parallel(&self, path: &str, editions: &[&str]) -> Result<...>;
}

/// Address normalization engine trait
#[async_trait]
pub trait ResolutionEngine: Send + Sync {
    /// Parses a human-readable shorthand (e.g., "Jn 17:3) into an LTREE path.
    async fn parse_address(&self, work_slug: &str, input: &str) -> Result<ResolvedAddress>;

    // Future methods to implement:
    // async fn validate_path(&self, path: &str) -> Result<bool>;
}

#[async_trait]
pub trait TraversalEngine: Send + Sync {
    async fn get_hierarchy(&self, parent_path: &str) -> Result<Vec<HierarchyNode>>;
    async fn get_adjacent_nodes(&self, node_id: Uuid) -> Result<Adjacency>;
}

/// Discovery and search engine trait
#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// Performs a keyword search, optionally scoped to a specific hierarchy path.
    async fn keyword_search(&self, query: &str, scope: Option<&str>, page: i64) -> Result<Pagination<SearchMatch>>;
}
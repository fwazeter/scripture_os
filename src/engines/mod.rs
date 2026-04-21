use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::models::{HierarchyNode, Adjacency, ScriptureContent, Comparison};

// Export the submodules
pub mod content;
pub mod resolution;
pub mod traversal;

// -- Service Layer Contracts ---

/// Text assembly engine trait
#[async_trait]
pub trait ContentEngine: Send + Sync {
    /// Retrieves text segments for a given canonical ltree path.
    async fn fetch_text(&self, path: &str) -> Result<Vec<ScriptureContent>>;

    /// Groups translations by their shared canonical node for side-by-side viewing.
    async fn get_comparison(&self, path: &str) -> Result<Vec<Comparison>>;

    // Future methods to implement:
    // async get_verse_bundle(&self, path: &str) -> Result<...>;
    // async fn get_parallel(&self, path: &str, editions: &[&str]) -> Result<...>;
}

/// Address normalization engine trait
#[async_trait]
pub trait ResolutionEngine: Send + Sync {
    /// Parses a human-readable shorthand (e.g., "Jn 17:3) into an LTREE path.
    async fn parse_address(&self, work_slug: &str, input: &str) -> Result<String>;

    // Future methods to implement:
    // async fn validate_path(&self, path: &str) -> Result<bool>;
}

#[async_trait]
pub trait TraversalEngine: Send + Sync {
    async fn get_hierarchy(&self, parent_path: &str) -> Result<Vec<HierarchyNode>>;
    async fn get_adjacent_nodes(&self, node_id: Uuid) -> Result<Adjacency>;
}
pub mod postgres;
pub use self::postgres::PostgresRepository;

use anyhow::Result;
use uuid::Uuid;
use crate::models::{HierarchyNode, Adjacency, ScriptureContent};
use async_trait::async_trait;

// We are using async_trait because standard Rust traits don't fully support async functions
#[async_trait]
pub trait ScriptureRepository: Send + Sync {
    // Traversal Needs
    async fn get_hierarchy(&self, parent_path: &str) -> Result<Vec<HierarchyNode>>;
    async fn get_adjacent_nodes(&self, current_node_id:Uuid) -> Result<Adjacency>;

    // Content Needs
    async fn fetch_text(&self, path: &str) -> Result<Vec<ScriptureContent>>;

    // Resolution Needs
    async fn resolve_address(&self, work_slug: &str, alias: &str) -> Result<Option<String>>;
}
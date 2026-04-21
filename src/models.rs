//! # Scripture OS Data Models
//!
//! This module defines the central "Data Contracts" for Scripture OS.
//! These structs act as the bridge between the PostgreSQL/LTREE Data Layer
//! and the Gateway Layer (Axum API)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- 1. Core Content Models ---

/// ## Scripture Content
/// Represents a single united of translated text.
///
/// ### Architectural Design Decision: Contextual Enrichment
/// Added `node_id`, `path`, and `translation_metadata` to ensure that every text
/// snippet knows exactly where it belogns in the structural hierarchy. This prevents
/// "orphaned text" and is critical for AI context feeding.
#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, PartialEq)]
pub struct ScriptureContent {
    pub node_id: Uuid,
    pub path: String,
    pub body_text: String,
    pub edition_name: String,
    pub language_code: String,
    pub absolute_index: i32,
    // Using JSON to allow flexible, tradition-specific metadata (e.g., strong's numbers, footnotes)
    pub translation_metadata: Option<serde_json::Value>,
}

/// ## Comparison
/// A specialized container for side-by-side reading.
///
/// ### Architectural Design Decision: Node-Centric Comparison
/// Instead of a flat list of texts, a Comparison groups multiple translations
/// (e.g., KJV and SBLGNT) under their shared canonical node
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Comparison {
    pub node_id: Uuid,
    pub path: String,
    pub contents: Vec<ScriptureContent>,
}

// --- 2. Structural & Traversal Models ---

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, PartialEq)]
pub struct ScriptureNode {
    pub id: Uuid,
    pub path: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, PartialEq)]
pub struct HierarchyNode {
    pub id: Uuid,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Adjacency {
    pub previous: Option<HierarchyNode>,
    pub next: Option<HierarchyNode>,
}

// --- 3. Discovery Models ---

/// ## Search Match
/// Represents a ranked result from the Search Engine.
///
/// ### Architectural Design Decision: Relevance Ranking
/// For AI retrieval (RAG) and Full-Text Search (FTS), returning the `relevance_score`
/// allows the frontend or LLM to filter results by confidence.
#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, PartialEq)]
pub struct SearchMatch {
    pub node_id: Uuid,
    pub path: String,
    pub snippet: String, // typically the highlighted text snippet from Postgres ts_headline
    pub edition_name: String,
    pub relevance_score: f32,
}


/// ## Pagination Wrapper
/// A generic container to standardize list responses across the Axum API
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Pagination<T> {
    pub data: Vec<T>,
    pub total_records: i64,
    pub current_page: i64,
    pub total_pages: i64,
    pub has_next: bool,
}
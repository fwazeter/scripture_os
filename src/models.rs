use serde::Serialize;
use uuid::Uuid;

#[derive(sqlx::FromRow, Serialize, Debug, PartialEq)]
pub struct ScriptureContent {
    pub body_text: String,
    pub edition_name: String,
    pub language_code: String,
    pub absolute_index: i32,
}

#[allow(dead_code)]
pub struct ScriptureNode {
    pub id: Uuid,
    pub path: String,
}

#[derive(sqlx::FromRow, Serialize, Debug, PartialEq)]
pub struct HierarchyNode {
    pub id: Uuid,
    pub path: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Adjacency {
    pub previous: Option<HierarchyNode>,
    pub next: Option<HierarchyNode>,
}
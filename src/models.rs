use serde::Serialize;
use uuid::Uuid;

#[derive(sqlx::FromRow, Serialize)]
pub struct ScriptureContent {
    pub body_text: String,
    pub edition_name: String,
    pub language_code: String,
}

#[allow(dead_code)]
pub struct ScriptureNode {
    pub id: Uuid,
    pub path: String,
}
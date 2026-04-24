//! # Read Controller (v1)
//!
//! ### Architectural Design Decision: Thin Controllers
//! All Controllers should contain absolutely zero business logic. Their sole responsibility
//! is translating HTTP data (Paths, Queries, JSON) into FSI Domain Models, calling the
//! Engine, and translating the Result back into HTTP Status Codes
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::api::AppState;
use crate::engines::content::ReadableVerse;
use crate::fsi::models::{Coordinate, LexKey, MacroID, WorkID};

/// ## `read_verse_handler`
/// **Parameters:** `state: AppState`, `path: Path` parameters.
///
/// ### Architectural Design Decision: HTTP to Domain Translation.
/// Converts the raw string/integer path parameters from the URL directly into
/// our pure FSI `Coordinate` NewTypes before passing them to the Content Engine.
pub async fn read_verse_handler(
    State(state): State<AppState>,
    Path((work_id, macro_id, lex_key)): Path<(i32, i32, String)>,
) -> Result<Json<ReadableVerse>, (StatusCode, String)> {
    // 1. Translate HTTP input to FSI Domain Object
    let coordinate = Coordinate {
        work_id: WorkID(work_id),
        macro_id: MacroID(macro_id),
        lex_key: LexKey(lex_key),
    };

    // 2. Delegate to the Engine
    match state.content_engine.fetch_readable_verse(coordinate).await {
        Ok(verse) => Ok(Json(verse)), // 3a. Translate success to JSON 200 OK
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            format!("Failed to retrieve FSI coordinate: {}", e), // 3b. Translate error to 404
        )),
    }
}

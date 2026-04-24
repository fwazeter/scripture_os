//! # API Version 1
//!
//! ### Architectural Design Decision: Versioned Endpoint Encapsulation
//! Groups all v1 routes together. If a v2 is ever required, it will get its own
//! parallel folder and router, preventing breaking changes.
pub mod read;

use crate::api::AppState;
use axum::Router;
use axum::routing::get;

/// ## `build_router`
///
/// ### Techncial Context: State Propagation
/// Notice the return type is `Router<AppState>`. This tells the Rust compiler that
/// this specific nested router expects the parent router to provide it with `AppState`.
pub fn build_router() -> Router<AppState> {
    Router::new().route(
        "/read/{work_id}/{macro_id}/{lex_key}",
        get(read::read_verse_handler),
    )
}

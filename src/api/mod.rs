//! # API Gateway Domain
//!
//! ### Architectural Design Decision: Delivery Mechanism Layer
//! This module decouples HTTP presentation logic from the core business engines.
//! It encapsulates the Axum web framework, ensuring that the inner domains remain
//! completely unaware of HTTP requests, statuses, or JSON formatting.
pub mod v1;

use crate::engines::content::CoreContentEngine;
use axum::Router;
use std::sync::Arc;

/// ## `AppState`
///
/// ### Architectural Design Decision: Immutable State Container
/// Holds the thread-safe `Arc` references to our Engines. This container is injected
/// into the Axum router and safely shared across hundreds of concurrent HTTP requests.
#[derive(Clone)]
pub struct AppState {
    pub content_engine: Arc<CoreContentEngine>,
}

/// ## `build_router`
///
/// ### Architectural Design Decision: Modular Routing
/// Constructs the top-level API router. By nesting versioned routers (e.g., `/v1`),
/// we ensure future API versions (`/v2`) can be introduced without breaking existing clients.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Nest all v1 routers under the /api/v1 path
        .nest("/api/v1", v1::build_router())
        .with_state(state)
}

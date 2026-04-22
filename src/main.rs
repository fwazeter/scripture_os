use axum::{
    extract::{Path, State, Query},
    routing::get,
    Json, Router,
};
use sqlx::postgres::PgPoolOptions;
use std:: sync::Arc;
use serde::Deserialize;

// Import traits and concrete engines
use scripture_os::engines::{
    ContentEngine, ResolutionEngine, TraversalEngine, SearchEngine,
    content::CoreContentEngine,
    resolution::CoreResolutionEngine,
    traversal::CoreTraversalEngine,
    search::CoreSearchEngine,
};
use scripture_os::repository::postgres::PostgresRepository;

// The central application state utilizing Dependency Injection (DI)
#[derive(Clone)]
struct AppState {
    content: Arc<dyn ContentEngine>,
    resolution: Arc<dyn ResolutionEngine>,
    traversal: Arc<dyn TraversalEngine>,
    search: Arc<dyn SearchEngine>,
}

// Struct for handling query parameters
#[derive(Deserialize)]
struct SearchParams {
    q: String,
    scope: Option<String>,
    page: Option<i64>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    // Create connection pool for web server to share
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // 1. Initialize concrete Data Layer
    let repo = Arc::new(PostgresRepository::new(pool));

    // 2. Initialize the Service Layer Engines and inject the repository
    let app_state = AppState {
        content: Arc::new(CoreContentEngine::new(repo.clone())),
        resolution: Arc::new(CoreResolutionEngine::new(repo.clone())),
        traversal: Arc::new(CoreTraversalEngine::new(repo.clone())),
        search: Arc::new(CoreSearchEngine::new(repo.clone())),
    };

    // 3. Build Gateway Layer (Axum API)
    // Define routes
    let app = Router::new()
        .route("/api/v1/content/{*path}", get(get_content))
        .route("/api/v1/compare/{*path}", get(get_comparison))
        .route("/api/v1/hierarchy/{*path}", get(get_hierarchy))
        .route("/api/v1/resolve/{work_slug}/{address}", get(resolve_address))
        .route("/api/v1/search", get(search_keyword))
        .with_state(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// --- Route Handlers ---
async fn get_content(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Json<serde_json::Value> {
    match state.content.fetch_text(&path).await {
        Ok(text) => Json(serde_json::json!({ "data": text })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn get_hierarchy(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Json<serde_json::Value> {
    match state.traversal.get_hierarchy(&path).await {
        Ok(nodes) => Json(serde_json::json!({ "data": nodes })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn resolve_address(
    State(state): State<AppState>,
    Path((work_slug, address)): Path<(String, String)>,
) -> Json<serde_json::Value> {
    match state.resolution.parse_address(&work_slug, &address).await {
        Ok(resolved) => Json(serde_json::json!({ "data": resolved })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn get_comparison(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Json<serde_json::Value> {
    match state.content.get_comparison(&path).await {
        Ok(comparisons) => Json(serde_json::json!({ "data": comparisons })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// route handler for search --
async fn search_keyword(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Json<serde_json::Value> {
    let page = params.page.unwrap_or(1);

    // as_deref() converts Option<String> to Option<&str>
    match state.search.keyword_search(&params.q, params.scope.as_deref(), page).await {
        Ok(results) => Json(serde_json::json!({ "data": results })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}
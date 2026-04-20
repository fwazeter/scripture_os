use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
    http::StatusCode
};
use serde::Deserialize;
use scripture_os::engines::{resolution, content, traversal};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use dotenvy:: dotenv;
use serde_json::{Value, json};

// Struct to parse the `?q=...` from the URL
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    // Create connection pool for web server to share
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Define routes
    let app = Router::new()
        .route("/v1/read/{work_slug}", get(handle_read_scripture))
        .route("/v1/structure/{path}", get(handle_get_structure))
        .with_state(pool);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Scripture OS listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Core Gateway handler
async fn handle_read_scripture(
    Path(work_slug): Path<String>,
    Query(search): Query<SearchQuery>,
    State(pool): State<sqlx::PgPool>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {

    // Engine 1: Resolution (Convert "Jn 17:3" -> "bible_int.nt.john.17.3")
    let ltree_path = match resolution::parse_address(&pool, &work_slug, &search.q).await {
        Ok(path) => path,
        Err(e) => {
            let error_response = json!({ "error": format!("Could not resolve address: {}", e) });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Engine 2: Content (Fetch actual text using ltree path)
    let verses = match content::fetch_text(&pool, &ltree_path).await {
        Ok(v) => v,
        Err(e) => {
            let error_response = json!({ "error": format!("Database error: {}", e) });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Return successful JSON payload
    Ok(Json(json!({
        "query": search.q,
        "resolved_path": ltree_path,
        "content": verses
    })))
}

async fn handle_get_structure(
    Path(path): Path<String>,
    State(pool): State<sqlx::PgPool>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {

    // 1. Get direct children of the path (e.g., chapters inside a book)
    let children = match traversal::get_hierarchy(&pool, &path).await {
        Ok(nodes) => nodes,
        Err(e) => {
            let error_response = json!({ "error": format!("Failed to fetch hierarchy: {}", e) });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // Note we coulde also look up UUID for the `path` and pass it into
    // `get_adjacent_nodes` here if we want to return the previous/next buttons
    // in the same API call. For now, we will return the children structure
    Ok(Json(json!({
        "path": path,
        "children": children
    })))
}
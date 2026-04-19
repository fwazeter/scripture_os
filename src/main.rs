use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use scripture_os::models::ScriptureContent;
use scripture_os::engines::content::fetch_text;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use dotenvy:: dotenv;

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
        .route("/verses/{path}", get(handle_get_verses))
        .with_state(pool);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Scripture OS listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Handler logic
async fn handle_get_verses(
    Path(path): Path<String>,
    State(pool): State<sqlx::postgres::PgPool>,
) -> Json<Vec<ScriptureContent>> {
    let verses = fetch_text(&pool, &path)
        .await
        .unwrap_or_else(|_| vec![]);
    Json(verses)
}
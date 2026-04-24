//! # Scripture OS Server (The "Gateway")
//!
//! ### Architectural Design Decision: Pure Composition Root
//! This binary is entirely devoid of HTTP or Business logic. It exists solely to
//! bootstrap the environment, initialize database connections, wire the Dependency
//! Injection (DI) containers, and start the TCP listener.
//!
//! cargo run --bin scripture_os

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;

use scripture_os::api::{AppState, build_router};
use scripture_os::engines::content::CoreContentEngine;
use scripture_os::repository::postgres::ScripturePostgresRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load Environment
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // 2. Initialize Database Provider
    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // 3. Dependency Injection Wiring
    let postgres_repo = ScripturePostgresRepository::new(pool);
    let repo_arc = Arc::new(postgres_repo); // Abstracted to SharedScriptureRepo

    // Inject Repository into Engine
    let content_engine = Arc::new(CoreContentEngine::new(repo_arc));

    // 4. Construct Application State
    let state = AppState { content_engine };

    // 5. Build Router (Delegated to the API Domain)
    let app = build_router(state);

    // 6. Start Server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Scripture OS Gateway running on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

//! # Scripture OS Seeder CLI (The "Data Loader")
//!
//! ### Architectural Design Decision: Composition Root
//! This binary serves as the composition root for the ingestion pipeline. It reads
//! environment variables, instantiates concrete data providers (PostgreSQL) and strategies
//! (QuranPipeParser), brinds them to their abstract traits, and injects into the Engine.
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fs;
use std::sync::Arc;

use scripture_os::engines::ingestion::CoreIngestionEngine;
use scripture_os::fsi::models::{NamespaceID, WorkID};
use scripture_os::parsers::quran::QuranPipeParser;
use scripture_os::repository::postgres::ScripturePostgresRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Parse Command Line Arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: cargo run --bin seed -- <FILE_PATH> <WORK_ID> <NAMESPACE_ID>");
        eprintln!("Example: cargo run --bin seed -- data/quran-uthmani.txt 786 1000");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let work_id = WorkID(args[2].parse::<i32>().expect("WorkID must be an integer"));
    let namespace_id = NamespaceID(
        args[3]
            .parse::<i32>()
            .expect("NamespaceID must be an integer"),
    );

    println!("Starting Ingestion Pipeline for File: {}", file_path);

    // 2. Read the Raw File
    let raw_content = fs::read_to_string(file_path)?;

    // 3. Setup Database Connection
    // Note: Assuming dotenv is loaded or DATABASE_URL is explicitly set in the environment.
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in the environment");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // ==========================================
    // DEPENDENCY INJECTION (DI) WIRING
    // ==========================================

    // 4. Instantiate the Repository (The Persistence Strategy)
    let postgres_repo = ScripturePostgresRepository::new(pool);
    let repo_arc = Arc::new(postgres_repo); // Casts to SharedScriptureRepoistory

    // 5. Instantiate the Parser (the Format Strategy)
    // By Passing the IDs here, the parser applies them uniformly to all parsed FSI coordinates
    let quran_parser = QuranPipeParser {
        work_id,
        namespace_id,
    };

    let parser_arc = Arc::new(quran_parser); // Casts to Arc<dyn ScriptureParser>

    // 6. Instantiate the Orchestrator
    let ingestion_engine = CoreIngestionEngine::new(repo_arc);

    // ==========================================
    // EXECUTE
    // ==========================================

    println!("Executing Core Ingestion Engine...");

    match ingestion_engine.ingest_file(&raw_content, parser_arc).await {
        Ok(_) => println!("✅ Successfully ingested {} into Scripture OS!", file_path),
        Err(e) => eprintln!("❌ Ingestion Failed: {}", e),
    }

    Ok(())
}

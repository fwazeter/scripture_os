use anyhow::Result;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

use scripture_os::fsi::models::{NamespaceID, SubMask, WorkID};
use scripture_os::fsi::seeder::{ingest_csv_format, ingest_tanzil_format};
use scripture_os::repository::fsi_postgres::PostgresRepository;

/// # Scripture OS Database Seeder Utility
///
/// ### Architectural Design Decision: CLI Tooling
/// By isolating the database seeding logic into a separate binary (`src/bin/seed.rs`),
/// we ensure the main application API remains lightweight and decoupled from file parsing logic.
/// This utility can be executed via `cargo run --bin seed`
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    // 1. Establish db connection
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres@localhost:5432/scripture_os".to_string());

    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let repo = PostgresRepository::new(pool);

    // 2. Simple CLI Routing
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --bin seed <command>");
        println!("Commands: seed_all");
        return Ok(());
    }

    match args[1].as_str() {
        "seed_all" => {
            println!("Seeding Work 786 (Quran) Track 1000: Uthmani Root...");
            ingest_tanzil_format(
                &repo,
                "data/quran-uthmani.txt",
                WorkID(786),
                NamespaceID(1000),
                SubMask(SubMask::LOGICAL_ANCHOR | SubMask::RTL),
            )
            .await?;

            println!("Seeding Track 1001: Arabic Simple Clean...");
            ingest_tanzil_format(
                &repo,
                "data/quran-simple-clean.txt",
                WorkID(786),
                NamespaceID(1001),
                SubMask(SubMask::LOGICAL_ANCHOR | SubMask::RTL),
            )
            .await?;

            // --- 10000 BLOCK: ENGLISH TRANSLATIONS ---
            println!("Seeding Work 786 (Quran) Track 10019: Khalifa Translation...");
            ingest_csv_format(
                &repo,
                "data/verse_nodes.csv",
                WorkID(786),
                NamespaceID(10019),
                SubMask(0x0000),
            )
            .await?;

            println!("Seeding Track 10020: Sahih International...");
            ingest_tanzil_format(
                &repo,
                "data/en.sahih.txt",
                WorkID(786),
                NamespaceID(10020),
                SubMask(0x0000),
            )
            .await?;

            println!("Seeding Track 10021: Yusuf Ali...");
            ingest_tanzil_format(
                &repo,
                "data/en.yusufali.txt",
                WorkID(786),
                NamespaceID(10021),
                SubMask(0x0000),
            )
            .await?;

            println!("Taxonomy Seeding Complete!");
        }

        _ => {
            println!("Unknown command: {}", args[1]);
        }
    }

    Ok(())
}

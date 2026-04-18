use scripture_os::get_verses_by_path; // Import from lib
use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;

// Seeding logic should be included here.

#[tokio::test]
async fn test_chapter_retrieval() {
    // explicitly load .env file for test binary
    dotenv().ok();

    // get DB url
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env for integration tests");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await.unwrap();

    // 1. (Optional) Run your seeding logic here if the DB is empty
    //    ... insert code ...

    // Run actual test
    let results = get_verses_by_path(&pool, "bible.nt.john.17").await.unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].edition_name, "KJV");
}
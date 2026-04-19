use scripture_os::get_verses_by_path; // Import from lib
use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;
use uuid::Uuid;
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

// Parallel Versions Test
#[tokio::test]
async fn test_parallel_versions() {
    dotenv().ok();
    let pool = PgPoolOptions::new().connect(&env::var("DATABASE_URL").unwrap()).await.unwrap();

    // 1. Add a 2nd edition
    let greek_edition_id = Uuid::new_v4();
    sqlx::query(r#"
                    INSERT INTO editions (id, work_id, name, language_code, is_source)
                    VALUES ($1, (SELECT id FROM works LIMIT 1), 'SBLGNT', 'grc', true)
                    ON CONFLICT DO NOTHING"# )
        .bind(greek_edition_id)
        .execute(&pool).await.unwrap();

    // 2. Link Greek text to the SAME John 17:3 node
    sqlx::query(r#"
                        INSERT INTO texts (node_id, edition_id, body_text)
                        VALUES ((SELECT id FROM nodes WHERE path = 'bible.nt.john.17.3'),
                                (SELECT id FROM editions WHERE name = 'SBLGNT'),
                                'αὕτη δέ ἐστιν ἡ αἰώνιος ζωή, ἵνα γινώσκωσιν σὲ τὸν μόνον ἀληθινὸν θεὸν καὶ ὃν ἀπέστειλας Ἰησοῦν Χριστόν.')
                                ON CONFLICT DO NOTHING"# )
        .execute(&pool).await.unwrap();

    // 3. Fetch and verify
    let results = get_verses_by_path(&pool, "bible.nt.john.17.3").await.unwrap();

    // Check that we got 2 results for the same path
    assert!(results.len() >= 2, "Should have retrieved both KJV and Greek Versions");
}
use scripture_os::engines::content::fetch_text;
use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;
use uuid::Uuid;

async fn setup_db() -> sqlx::PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env for integration tests");

    PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap()
}

// Reusable seeding function for our tests
async fn seed_base_data(pool: &sqlx::PgPool) {
    // 1. Tradition
    sqlx::query(r#"
        INSERT INTO traditions (id, name)
        VALUES ('11111111-1111-1111-1111-111111111111', 'Abrahamic')
        ON CONFLICT DO NOTHING
    "#).execute(pool).await.unwrap();

    // 2. Work
    sqlx::query(r#"
        INSERT INTO works (id, tradition_id, slug, title)
        VALUES ('22222222-2222-2222-2222-222222222222', '11111111-1111-1111-1111-111111111111', 'bible', 'The Holy Bible')
        ON CONFLICT DO NOTHING
    "#).execute(pool).await.unwrap();

    // 3. Edition
    sqlx::query(r#"
        INSERT INTO editions (id, work_id, name, language_code, is_source)
        VALUES ('33333333-3333-3333-3333-333333333333', '22222222-2222-2222-2222-222222222222', 'KJV', 'en', false)
        ON CONFLICT DO NOTHING
    "#).execute(pool).await.unwrap();

    // 4. Node
    sqlx::query(r#"
        INSERT INTO nodes (id, work_id, path, node_type, sort_order)
        VALUES ('44444444-4444-4444-4444-444444444444', '22222222-2222-2222-2222-222222222222', 'bible.nt.john.17.3', 'verse', 1.0)
        ON CONFLICT DO NOTHING
    "#).execute(pool).await.unwrap();

    // 5. Text
    sqlx::query(r#"
        INSERT INTO texts (id, node_id, edition_id, body_text)
        VALUES ('55555555-5555-5555-5555-555555555555', '44444444-4444-4444-4444-444444444444', '33333333-3333-3333-3333-333333333333', 'And this is life eternal, that they might know thee the only true God...')
        ON CONFLICT DO NOTHING
    "#).execute(pool).await.unwrap();
}

#[tokio::test]
async fn test_chapter_retrieval() {
    let pool = setup_db().await;

    // 1. Run seeding logic
    seed_base_data(&pool).await;

    // 2. Run actual test
    let results = fetch_text(&pool, "bible.nt.john.17").await.unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].edition_name, "KJV");
}

// Parallel Versions Test
#[tokio::test]
async fn test_parallel_versions() {
    let pool = setup_db().await;

    // 1. Make sure base data exists
    seed_base_data(&pool).await;

    // 2. Add a 2nd edition
    let greek_edition_id = Uuid::parse_str("88888888-8888-8888-8888-888888888886").unwrap();

    sqlx::query(r#"
        INSERT INTO editions (id, work_id, name, language_code, is_source)
        VALUES ($1, '88888888-8888-8888-8888-888888888882', 'SBLGNT', 'grc', true)
        ON CONFLICT DO NOTHING"# )
        .bind(greek_edition_id)
        .execute(&pool).await.unwrap();

    // 3. Link Greek text to the SAME John 17:3 node
    sqlx::query(r#"
        INSERT INTO texts (node_id, edition_id, body_text)
        VALUES ('88888888-8888-8888-8888-888888888884',
                (SELECT id FROM editions WHERE name = 'SBLGNT' LIMIT 1),
                'αὕτη δέ ἐστιν ἡ αἰώνιος ζωή, ἵνα γινώσκωσιν σὲ τὸν μόνον ἀληθινὸν θεὸν καὶ ὃν ἀπέστειλας Ἰησοῦν Χριστόν.')
        ON CONFLICT DO NOTHING"# )
        .execute(&pool).await.unwrap();

    // 4. Fetch and verify
    let results = fetch_text(&pool, "bible.nt.john.17.3").await.unwrap();

    // Check that we got 2 results for the same path
    assert!(results.len() >= 2, "Should have retrieved both KJV and Greek Versions");
}
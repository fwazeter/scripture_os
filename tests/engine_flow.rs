use std::sync::Arc;
use scripture_os::repository::postgres::PostgresRepository;
use scripture_os::engines::{ContentEngine, ResolutionEngine};
use scripture_os::engines::content::CoreContentEngine;
use scripture_os::engines::resolution::CoreResolutionEngine;
use scripture_os::test_utils;

#[tokio::test]
async fn test_full_resolution_to_content_pipeline() {
    let pool = test_utils::setup_db().await;
    test_utils::seed_universal_data(&pool).await;

    // 1. Setup dependency injection (DI)
    let repo = Arc::new(PostgresRepository::new(pool));
    let resolution_engine = CoreResolutionEngine::new(repo.clone());
    let content_engine = CoreContentEngine::new(repo.clone());

    // 2. Step 1: Resolve Shorthand Address (User enters: "Jn 17:3")
    let path = resolution_engine.parse_address("bible", "Jn 17:3").await.unwrap();
    assert_eq!(path, "bible.nt.john.17.3");

    // 3. Step 2: Fetch Content using the resolved path
    let texts = content_engine.fetch_text(&path).await.unwrap();

    // Verify we got the correct absolute index and translations (KJV & SBLGNT)
    assert!(!texts.is_empty(), "Should retrieve text");
    assert_eq!(texts[0].absolute_index, 4000);
    assert_eq!(texts.len(), 2);
}
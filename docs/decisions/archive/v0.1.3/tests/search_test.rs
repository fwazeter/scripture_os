use std::sync::Arc;
use scripture_os::repository::postgres::PostgresRepository;
use scripture_os::engines::SearchEngine;
use scripture_os::engines::search::CoreSearchEngine;
use scripture_os::test_utils;

#[tokio::test]
async fn test_global_search_flow() {
    let pool = test_utils::setup_db().await;
    test_utils::seed_universal_data(&pool).await;

    let repo = Arc::new(PostgresRepository::new(pool));
    let engine = CoreSearchEngine::new(repo);

    // Test 1: Broad search across all texts
    // "mercy" exists in the seeded Psalm 51 (KJV & NIV)
    let results = engine.keyword_search("mercy", None, 1).await.unwrap();

    assert!(results.total_records > 0, "Should find records containing 'mercy'");
    assert!(!results.data.is_empty(), "Data vector should not be empty");

    let snippet = &results.data[0].snippet;
    assert!(
        snippet.contains("<b>"),
        "Snippet should contain HTML highlighting tags. Actual snippet returned by Postgres: {}",
        snippet
    );

    // Test 2: Scoped search
    // "life" exists in John 17:3. We scope it strictly to the New Testament.
    let scoped_results =engine.keyword_search("life", Some("bible.nt"), 1).await.unwrap();

    assert!(scoped_results.total_records > 0, "Should find 'life' in the New Testament");
    assert!(scoped_results.data[0].path.starts_with("bible.nt"), "Path must respect scope");
}
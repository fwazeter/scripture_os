pub mod common;

use scripture_os::engines::content::fetch_text;

#[tokio::test]
async fn test_chapter_retrieval() {
    let pool = common::setup_db().await;
    common::seed_universal_data(&pool).await;

    // Fetch all of chapter 17
    let results = fetch_text(&pool, "bible_test.nt.john.17").await.unwrap();

    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_parallel_versions() {
    let pool = common::setup_db().await;
    common::seed_universal_data(&pool).await;

    // The seed data already has both KJV and SBLGNT natively inserted
    let results = fetch_text(&pool, "bible_test.nt.john.17.3").await.unwrap();

    assert!(results.len() >= 2, "Should have retrieved both KJV and Greek Versions");
}

#[tokio::test]
async fn test_quran_retrieval() {
    let pool = common::setup_db().await;
    common::seed_universal_data(&pool).await;

    let results = fetch_text(&pool, "quran_test.110.1").await.unwrap();
    assert_eq!(results[0].edition_name, "Clear_Quran");
}
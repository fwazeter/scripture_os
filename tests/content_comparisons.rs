use std::sync::Arc;
use scripture_os::repository::postgres::PostgresRepository;
use scripture_os::engines::ContentEngine;
use scripture_os::engines::content::CoreContentEngine;
use scripture_os::test_utils;

#[tokio::test]
async fn test_theological_vs_authorial_overlapping_hierarchies() {
    let pool = test_utils::setup_db().await;
    test_utils::seed_universal_data(&pool).await;

    let repo = Arc::new(PostgresRepository::new(pool));
    let engine = CoreContentEngine::new(repo);

    // Fetch Rigveda via Mandala system and Ashtaka system
    let mandala = engine.fetch_text("rigveda.mandala.1.sukta.1.mantra.1").await.unwrap();
    let ashtaka = engine.fetch_text("rigveda.ashtaka.1.adhyaya.1.varga.1.mantra.1").await.unwrap();

    // Architectural Design Decision: Sequence-to-Address Assembly
    // Different address paths must resolve to the exact same text index.
    assert_eq!(mandala[0].absolute_index, 3000);
    assert_eq!(mandala, ashtaka);
}

#[tokio::test]
async fn test_cross_tradition_psalm_numbering_shifts() {
    let pool = test_utils::setup_db().await;
    test_utils::seed_universal_data(&pool).await;

    let repo = Arc::new(PostgresRepository::new(pool));
    let engine = CoreContentEngine::new(repo);

    // Bible (Christian) treats title as unnumbered metadata
    let bible_title = engine.fetch_text("bible.ot.psalms.51.title").await.unwrap();
    assert_eq!(bible_title.len(), 6); // 2 indices * 3 translations

    // Tanakh (Jewish) counts the first title line as Verse 1
    let tanakh_v1 = engine.fetch_text("tanakh.ketuvim.psalms.51.1").await.unwrap();
    assert_eq!(tanakh_v1[0].absolute_index, 1000);
}
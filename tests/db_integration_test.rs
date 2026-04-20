use scripture_os::engines::{content, resolution, traversal};
use scripture_os::test_utils;
use uuid::Uuid;
use scripture_os::repository::postgres::PostgresRepository;

#[tokio::test]
async fn test_end_to_end_routing_and_fetching() {
    let pool = test_utils::setup_db().await;
    test_utils::seed_universal_data(&pool).await;

    let repo = PostgresRepository::new(pool);

    // 1. Resolution Engine: User searches for "Jn 17:3" in the "bible" work
    let resolved_path = resolution::parse_address(&repo, "bible", "Jn 17:3").await.unwrap();
    assert_eq!(resolved_path, "bible.nt.john.17.3");

    // 2. Content Engine: Fetch the texts for the resolved path
    let texts = content::fetch_text(&repo, &resolved_path).await.unwrap();

    // 3. Verify we got the correct absolute index and translations
    assert!(!texts.is_empty(), "Should retrieve text");
    assert_eq!(texts[0].absolute_index, 4000);

    // Seed data has 2 translations for this verse (KJV and SBLGNT)
    assert_eq!(texts.len(), 2);
    let has_english = texts.iter().any(|t| t.language_code == "en");
    let has_greek = texts.iter().any(|t| t.language_code == "grc");
    assert!(has_english && has_greek, "Should fetch both KJV and Greek parallel versions");
}

#[tokio::test]
async fn test_bible_vs_tanakh_psalm_shift() {
    let pool = test_utils::setup_db().await;
    test_utils::seed_universal_data(&pool).await;

    let repo = PostgresRepository::new(pool);

    // Christian bible groups title into a single unnumbered block (indices 1000 & 1001)
    let bible_title = content::fetch_text(&repo, "bible.ot.psalms.51.title").await.unwrap();
    // 2 textual indices * 3 translations (KJV, NIV, LXX) = 6 rows
    assert_eq!(bible_title.len(), 6);

    // Hebrew Tanakh counts first title line as Verse 1 (Index 1000)
    let tanakh_v1 = content::fetch_text(&repo, "tanakh.ketuvim.psalms.51.1").await.unwrap();
    assert_eq!(tanakh_v1.len(), 1); // only 1 hebrew translation
    assert_eq!(tanakh_v1[0].absolute_index, 1000);

    // Christian Bible Verse 1 doesn't start until Index 1002!
    let bible_v1 = content::fetch_text(&repo, "bible.ot.psalms.51.1").await.unwrap();
    assert_eq!(bible_v1[0].absolute_index, 1002);
}

#[tokio::test]
async fn test_quran_hafs_vs_warsh_shift() {
    let pool = test_utils::setup_db().await;
    test_utils::seed_universal_data(&pool).await;

    let repo = PostgresRepository::new(pool);

    // Hafs 1:1 explicitly includes the Basmala (Index 2000)
    let hafs_v1 = content::fetch_text(&repo, "hafs.sura.1.1").await.unwrap();
    assert_eq!(hafs_v1[0].absolute_index, 2000);
    assert!(hafs_v1.iter().any(|t| t.body_text.contains("بِسْمِ اللَّهِ")), "Hafs 1:1 should be the Basmala");

    // Warsh 1:1 considers the Basmla an unnumbered title and starts at "Alhamdulilah" (Index 2001)
    let warsh_v1 = content::fetch_text(&repo, "warsh.sura.1.1").await.unwrap();
    assert_eq!(warsh_v1[0].absolute_index, 2001);
    assert!(warsh_v1.iter().any(|t| t.body_text.contains("الْحَمْدُ لِلَّهِ")), "Warsh 1:1 should be Alhamdulillah");
}

#[tokio::test]
async fn test_rigveda_overlapping_hierarchies() {
    let pool = test_utils::setup_db().await;
    test_utils::seed_universal_data(&pool).await;

    let repo = PostgresRepository::new(pool);

    // Fetch via the Theological / Author System (Mandala)
    let mandala_texts = content::fetch_text(&repo, "rigveda.mandala.1.sukta.1.mantra.1").await.unwrap();

    // Fetch via the Oral Memorization System (Ashtaka)
    let ashtaka_texts = content::fetch_text(&repo, "rigveda.ashtaka.1.adhyaya.1.varga.1.mantra.1").await.unwrap();

    // Because they both act as arbitrary pointers to the same linear sequence (Index 3000) ...
    // The results must be absolutely identical
    assert_eq!(mandala_texts[0].absolute_index, 3000);
    assert_eq!(ashtaka_texts[0].absolute_index, 3000);
    assert_eq!(mandala_texts.len(), 2, "Should retrieve Sanskrit and English translation");
    assert_eq!(mandala_texts, ashtaka_texts);
}

#[tokio::test]
async fn test_traversal_adjacency_across_shifts() {
    let pool = test_utils::setup_db().await;
    test_utils::seed_universal_data(&pool).await;

    let repo = PostgresRepository::new(pool);

    // Target: Hafs Sura 1:1 (ID: ...0A06)
    let hafs_1_1_id = Uuid::parse_str("00000000-0000-0000-0000-000000000A06").unwrap();

    // Verify the Traversal engine can successfully find the next node
    // by comparing start_index and end_index ranges
    let adjacency = traversal::get_adjacent_nodes(&repo, hafs_1_1_id).await.unwrap();

    assert!(adjacency.previous.is_none(), "Ayah 1 has no previous node");
    assert!(adjacency.next.is_some(), "Ayah 1 should have a next node");
    assert_eq!(adjacency.next.as_ref().unwrap().path, "hafs.sura.1.2");
}
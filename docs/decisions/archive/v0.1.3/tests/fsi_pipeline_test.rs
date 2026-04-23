use std::sync::Arc;
use anyhow::Result;

use scripture_os::fsi::models::{MacroID, NamespaceID, WorkID};
use scripture_os::fsi::seeder::{ingest_uthmani_quran, ingest_khalifa_csv};
use scripture_os::repository::fsi_mock::MockFsiRepository;
use scripture_os::engines::FsiContentEngine;
use scripture_os::engines::fsi_content::CoreContentEngine;

/// How to run:
/// cargo test --test fsi_pipeline_test -- --nocapture
#[tokio::test]
async fn test_fsi_end_to_end_pipeline() -> Result<()> {
    // 1. Arrange: Create our thread-safe Mock Repo
    let mock_repo = Arc::new(MockFsiRepository::new());

    // 2. Act: Run the Seeder on the real Quran text
    // This will generate thousands of Base-62 LexKeys and populate the mock repo.
    println!("Ingesting quran-uthmani.txt into FSI coordinates..");
    ingest_uthmani_quran(&*mock_repo, "tests/quran-uthmani.txt").await?;

    // 3. Act: Initialize the Assembler Engine with our populated repo
    let engine = CoreContentEngine::new(mock_repo);

    // Ask the engine to assemble Work 786 (Quran), Macro 1 (Surah 1), Namespace 0x02 (Arabic Root)
    println!("Assembling MacroID(1)...");
    let result = engine.assemble_macro(WorkID(786), MacroID(1), NamespaceID(0x02)).await?;

    // 4. Assert: Verify the engine perfectly reconstructed the Surah with structural markers
    // Updated to match the exact Unicode representation (without the Tatweel/Kashida)
    let expected_fatihah = "\n[AYAH:1] بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ \n[AYAH:2] ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ \n[AYAH:3] ٱلرَّحْمَٰنِ ٱلرَّحِيمِ \n[AYAH:4] مَٰلِكِ يَوْمِ ٱلدِّينِ \n[AYAH:5] إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ \n[AYAH:6] ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ \n[AYAH:7] صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ";

    assert_eq!(result, expected_fatihah);
    println!("Success! The FSI pipeline perfectly reconstructed Surah Al-Fatihah from word-atoms.");

    Ok(())
}

#[tokio::test]
async fn test_fsi_dual_track_pipeline() -> Result<()> {
    let mock_repo = Arc::new(MockFsiRepository::new());

    println!("Ingesting Uthmani Arabic...");
    ingest_uthmani_quran(&*mock_repo, "tests/quran-uthmani.txt").await?;

    println!("Ingesting Khalifa Translation...");
    ingest_khalifa_csv(&*mock_repo, "tests/verse_nodes.csv").await?;

    let engine = CoreContentEngine::new(mock_repo);

    let arabic_result = engine.assemble_macro(WorkID(786), MacroID(1), NamespaceID(0x02)).await?;
    println!("\nArabic (Track 0x02): \n{}", arabic_result);

    let english_result = engine.assemble_macro(WorkID(786), MacroID(1), NamespaceID(19)).await?;
    println!("\nEnglish (Track 19): \n{}", english_result);

    let expected_english = "\n[AYAH:1] In the name of GOD, Most Gracious, Most Merciful. \n[AYAH:2] Praise be to GOD, Lord of the universe. \n[AYAH:3] Most Gracious, Most Merciful. \n[AYAH:4] Master of the Day of Judgment. \n[AYAH:5] You alone we worship; You alone we ask for help. \n[AYAH:6] Guide us in the right path; \n[AYAH:7] the path of those whom You blessed; not of those who have deserved wrath, nor of the strayers.";

    assert_eq!(english_result, expected_english);

    Ok(())
}
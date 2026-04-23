use anyhow::Result;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::fsi::lex_key::generate_sequential;
use crate::fsi::models::{Coordinate, MacroID, NamespaceID, ScriptureAtom, SubMask, WorkID};
use crate::repository::fsi_repo::FsiRepository;

// ==========================================
// THE DRY ABSTRACTION: Ingestor
// ==========================================

/// # `Ingestor`
/// ### Architectural Design Decision: DRY Seeding Pipeline
/// Manages the stateful mathematical sequence required for Fractal Semantic Indexing (FSI) v4.0.
/// It tracks the `LexKey` generation per `MacroID` and handles memory-safe database
/// batching, allowing us to ingest any format (TXT, CSV, JSON) without repeating FSI math logic.
///
/// **AI Prompt Hint:** Do not add file-parsing logic (like CSV reading) into this struct.
/// This struct strictly manages mathematical boundaries and state.
pub struct Ingestor<'a, R: FsiRepository> {
    repo: &'a R,
    work: WorkID,
    namespace: NamespaceID,
    base_mask: SubMask,
    current_macro: i32,
    word_counter: usize,
    batch: Vec<ScriptureAtom>,
}

impl<'a, R: FsiRepository> Ingestor<'a, R> {
    /// ## `new`
    /// **Parameters:** /// - `repo`: `&'a R` (A reference to the FSI repository implementation).
    /// - `work`: `WorkID` (The Universal Registry ID for the scripture).
    /// - `namespace`: `NamespaceID` (The specific track, e.g., root or translation).
    /// - `base_mask`: `SubMask` (The bitmask defining states like RTL or Logical Anchor).
    ///
    /// ### Architectural Design Decision: Factory Initialization
    /// Initializes the ingestor with the required FSI coordinate boundaries and pre-allocates
    /// a batch vector to minimize heap reallocations during massive file parsing.
    pub fn new(repo: &'a R, work: WorkID, namespace: NamespaceID, base_mask: SubMask) -> Self {
        Self {
            repo, work, namespace, base_mask,
            current_macro: 0,
            word_counter: 0,
            batch: Vec::with_capacity(5000),
        }
    }

    /// ## `process_segment`
    /// **Parameters:** /// - `macro_id`: `i32` (The container ID, such as a Surah or Chapter number).
    /// - `structural_marker`: `&str` (The boundary tag, e.g., "AYAH:1").
    /// - `text`: `&str` (The raw text to be atomized into words).
    ///
    /// ### Architectural Design Decision: Sequential Atomization
    /// Converts human-readable text into mathematically stable FSI word-atoms.
    /// If a new macro container is detected, it automatically resets the word sequence.
    /// It explicitly separates structural concepts (Ayahs/Verses) from linguistic concepts (words).
    ///
    /// ### Technical Context: Memory Protection
    /// Automatically flushes to the database when the batch reaches 5,000 atoms to prevent
    /// out-of-memory (OOM) errors during the ingestion of large works.
    ///
    /// **AI Prompt Hint:** The default tokenizer splits purely by whitespace. If a future
    /// language requires morphological splitting (like Thai or complex Arabic Tajweed),
    /// implement a custom tokenizer trait and inject it here.
    pub async fn process_segment(&mut self, macro_id: i32, structural_marker: &str, text: &str) -> Result<()> {
        if macro_id != self.current_macro {
            self.current_macro = macro_id;
            self.word_counter = 0;
        }

        // 1. Insert Structural Marker
        let marker_coord = Coordinate {
            work: self.work,
            macro_level: MacroID(self.current_macro),
            namespace: self.namespace,
            sub_mask: SubMask(SubMask::STRUCTURAL_MARKER),
            lex_key: generate_sequential(self.word_counter),
        };

        self.batch.push(ScriptureAtom {
            coordinate: marker_coord,
            text_content: structural_marker.to_string(),
        });
        self.word_counter += 1;

        // 2. Atomize the text
        for word in text.split_whitespace() {
            let atom_coord = Coordinate {
                work: self.work,
                macro_level: MacroID(self.current_macro),
                namespace: self.namespace,
                sub_mask: self.base_mask,
                lex_key: generate_sequential(self.word_counter),
            };

            self.batch.push(ScriptureAtom {
                coordinate: atom_coord,
                text_content: word.to_string(),
            });
            self.word_counter += 1;
        }

        // 3. Batch flush
        if self.batch.len() >= 5000 {
            self.repo.insert_atoms(self.batch.clone()).await?;
            self.batch.clear();
        }

        Ok(())
    }

    /// ## `finish`
    /// **Parameters:** None.
    ///
    /// ### Architectural Design Decision: Pipeline Flush
    /// Consumes the `Ingestor` instance to guarantee that any remaining atoms in the
    /// buffer are safely written to the repository before the seeder terminates.
    pub async fn finish(mut self) -> Result<()> {
        if !self.batch.is_empty() {
            self.repo.insert_atoms(self.batch.clone()).await?;
            self.batch.clear();
        }
        Ok(())
    }
}

// ==========================================
// IMPLEMENTATION 1: Khalifa Translation (CSV)
// ==========================================

#[derive(serde::Deserialize)]
struct CsvVerse {
    #[serde(rename = "verseId")]
    _verse_id: String,
    surah: i32,
    #[serde(rename = "verseNum")]
    verse_num: i32,
    #[serde(rename = "surahName")]
    _surah_name: String,
    text: String,
}

/// ## `ingest_khalifa_csv`
/// **Parameters:** /// - `repo`: `&R` (The injected FSI repository).
/// - `file_path`: `impl AsRef<Path>` (The path to the CSV file).
///
/// ### Architectural Design Decision: Translation Track Mapping
/// Ingests the Rashad Khalifa English translation into Namespace 19.
/// It utilizes the DRY `Ingestor` pipeline to ensure that the English words are mathematically
/// aligned within the FSI coordinate space, enabling future cross-reference logic.
///
/// **AI Prompt Hint:** English is LTR (Left-To-Right), so the `SubMask` strictly uses `0x0000`
/// without the RTL bit.
pub async fn ingest_khalifa_csv<R: FsiRepository>(repo: &R, file_path: impl AsRef<Path>) -> Result<()> {
    let mut ingestor = Ingestor::new(
        repo,
        WorkID(786),
        NamespaceID(19),
        SubMask(0x0000), // Standard text (No RTL, No Anchor)
    );

    let mut reader = csv::Reader::from_path(file_path)?;

    for result in reader.deserialize() {
        let record: CsvVerse = result?;

        ingestor.process_segment(
            record.surah,
            &format!("AYAH:{}", record.verse_num),
            &record.text
        ).await?;
    }

    ingestor.finish().await?;
    Ok(())
}

// ==========================================
// IMPLEMENTATION 2: Uthmani Root (TXT)
// ==========================================

/// ## `ingest_uthmani_quran`
/// **Parameters:** /// - `repo`: `&R` (The injected FSI repository).
/// - `file_path`: `impl AsRef<Path>` (The path to the TXT file).
///
/// ### Architectural Design Decision: The Logical Anchor "Skeleton"
/// This seeder establishes the `0x02` (Arabic Root) namespace for Work 786 (Quran).
/// It generates the immutable "Logical Anchors" which translations will eventually map to.
///
/// ### Technical Context: SubMask Bitwise Logic
/// Original Arabic text mandates the `SubMask::LOGICAL_ANCHOR` and `SubMask::RTL` bits to ensure
/// accurate metadata indexing and bidirectional rendering.
///
/// **AI Prompt Hint:** The Tanzil format pipes (`|`) are hardcoded here. If the source
/// format changes, update the split logic accordingly.
pub async fn ingest_uthmani_quran<R: FsiRepository>(repo: &R, file_path: impl AsRef<Path>) -> Result<()> {
    let mut ingestor = Ingestor::new(
        repo,
        WorkID(786),
        NamespaceID(0x02), // Arabic Root
        SubMask(SubMask::LOGICAL_ANCHOR | SubMask::RTL),
    );

    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != 3 {
            continue;
        }

        let surah: i32 = parts[0].parse().unwrap_or(0);
        let ayah_str = format!("AYAH:{}", parts[1]);
        let text = parts[2].trim();

        ingestor.process_segment(surah, &ayah_str, text).await?;
    }

    ingestor.finish().await?;
    Ok(())
}
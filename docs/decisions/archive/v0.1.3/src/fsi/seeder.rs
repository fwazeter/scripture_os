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

// ==========================================
// IMPLEMENTATION 1: Standard Translation (CSV)
// ==========================================

#[derive(serde::Deserialize)]
struct StandardTranslationCsv {
    // Looks for a column named 'chapter' first. If missing, tries the 'surah' alias.
    #[serde(alias = "surah")]
    chapter: i32,

    // Looks for 'verse_num'. If missing, tries 'verse' or 'verseNum'.
    #[serde(alias = "verseNum", alias = "verse")]
    verse_num: i32,

    text: String,
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
            repo,
            work,
            namespace,
            base_mask,
            current_macro: 0,
            word_counter: 0,
            batch: Vec::with_capacity(2000),
        }
    }

    /// Internal helper to guarantee the batch never exceeds the safe threshold,
    /// regardless of how massive a single input string is.
    async fn check_flush(&mut self) -> Result<()> {
        if self.batch.len() >= 2000 {
            self.repo.insert_atoms(self.batch.clone()).await?;
            self.batch.clear();
        }
        Ok(())
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
    pub async fn process_segment(
        &mut self,
        macro_id: i32,
        structural_marker: &str,
        text: &str,
    ) -> Result<()> {
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

            // Flush inside the loop in case a parsing error feeds excessive words.
            self.check_flush().await?;
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

/// ## `ingest_khalifa_csv`
/// **Parameters:** /// - `repo`: `&R` (The injected FSI repository).
/// - `file_path`: `impl AsRef<Path>` (The path to the CSV file).
/// - `work_id`: `WorkID`
/// - `namespace_id`: `NamespaceID`
/// - `base_mask`: `SubMask`
///
/// ### Architectural Design Decision: Format-Based Ingestion
/// Decouples the file parsing logic from specific scriptures. This function can now
/// ingest ANY translation that follows the standard 5-column CSV layout, routing it
/// to the dynamically injected FSI coordinates.
///
/// **AI Prompt Hint:** English is LTR (Left-To-Right), so the `SubMask` strictly uses `0x0000`
/// without the RTL bit.
pub async fn ingest_csv_format<R: FsiRepository>(
    repo: &R,
    file_path: impl AsRef<Path>,
    work_id: WorkID,
    namespace_id: NamespaceID,
    base_mask: SubMask,
) -> Result<()> {
    let mut ingestor = Ingestor::new(repo, work_id, namespace_id, base_mask);

    let mut reader = csv::Reader::from_path(file_path)?;

    for result in reader.deserialize() {
        let record: StandardTranslationCsv = result?;

        ingestor
            .process_segment(
                record.chapter,
                &format!("VERSE:{}", record.verse_num),
                &record.text,
            )
            .await?;
    }

    ingestor.finish().await?;
    Ok(())
}

// ==========================================
// IMPLEMENTATION 2: Tanzil Root (TXT)
// ==========================================

/// ## `ingest_tanzil_format`
/// /// **Parameters:** /// - `repo`: `&R` (The injected FSI repository).
/// - `file_path`: `impl AsRef<Path>` (The path to the TXT file).
/// - `work_id`: `WorkID`
/// - `namespace_id`: `NamespaceID`
/// - `base_mask`: `SubMask`
/// ### Architectural Design Decision: Format-Based Ingestion
/// Parses any raw scripture text that utilizes the standard Tanzil pipe-delimited format
/// (`Surah|Ayah|Text`). It no longer hardcodes the Arabic root, making it reusable for
/// other original-language texts formatted by the Tanzil project.
/// **AI Prompt Hint:** The Tanzil format pipes (`|`) are hardcoded here. If the source
/// format changes, update the split logic accordingly.
pub async fn ingest_tanzil_format<R: FsiRepository>(
    repo: &R,
    file_path: impl AsRef<Path>,
    work_id: WorkID,
    namespace_id: NamespaceID,
    base_mask: SubMask,
) -> Result<()> {
    let mut ingestor = Ingestor::new(repo, work_id, namespace_id, base_mask);

    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != 3 {
            continue;
        }

        let surah: i32 = parts[0].parse().unwrap_or(0);
        let ayah_str = format!("VERSE:{}", parts[1]);
        let text = parts[2].trim();

        ingestor.process_segment(surah, &ayah_str, text).await?;
    }

    ingestor.finish().await?;
    Ok(())
}

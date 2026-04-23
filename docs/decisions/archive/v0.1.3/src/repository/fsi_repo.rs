use async_trait::async_trait;
use anyhow::Result; // Note: Swap this out for custom ScriptureError later

use crate::fsi::models::{ScriptureAtom, WorkID, MacroID, NamespaceID};

/// # The FSI Repository Contract
///
/// ### Architectural Design Decision: Repository Abstraction
/// Scripture OS decouples the Service Layer (Engines) from Data Layer (Postgres) using
/// this trait. Engines no longer know about SQL syntax or database pools. They only
/// know how to ask for FSI coordinates.
///
/// This allows us to use a `MockRepository` for Track B engine testing, and enables
/// future database swaps (e.g., SQLite for mobile) without breaking business logic.
#[async_trait]
pub trait FsiRepository {

    /// ## `get_sequence`
    /// **Parameters:** /// - `work`: The specific registry ID (e.g., Quran 786).
    /// - `macro_level`: The container ID (e.g., Surah 1).
    /// - `namespace`: The target track (e.g., 0x02 for Arabic, 019 for RK, 205 for Sahih).
    /// - `start_lex`: / `end_lex`: Optional boundaries for slicing the text.
    ///
    /// ### Architectural Design Decision: Sequence Retrieval
    /// In FSI v4.0, text is physically partitioned and sorted lexicographically by `LexKey`.
    /// This function retrieves a continuous mathematical sequence of atoms (words)
    /// for a specific namespace, forming the basis for engine assembly.
    ///
    /// **AI Prompt Hint:** Implementations of this method MUST ensure results are ordered
    /// strictly ascending by `lex_key` to prevent assembler jumbling.
    async fn get_sequence(
        &self,
        work: WorkID,
        macro_level: MacroID,
        namespace: NamespaceID,
        start_lex: Option<&[u8]>,
        end_lex: Option<&[u8]>,
    ) -> Result<Vec<ScriptureAtom>>;

    /// ## `insert_atom`
    /// **Parameters:** `atoms: Vec<ScriptureAtom>`
    ///
    /// ### Architectural Design Decision: Bulk Ingestion
    /// Used by the Atomizer seeders (like the Uthmani ingestor) and future WASM
    /// AI plugins to write massive sequences of data to the Universal Coordinate system.
    async fn insert_atoms(&self, atoms: Vec<ScriptureAtom>) -> Result<()>;

    // Future Expansion:
    // async get_atom(&self, coord: &Coordinate) -> Result<Option<ScriptureAtom>>;
    // async get_cross_references(&self, coord: &Coordinate) -> Result<Vec<Coordinate>>;
}


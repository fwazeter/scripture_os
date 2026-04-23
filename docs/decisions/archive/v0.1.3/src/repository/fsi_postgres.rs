use anyhow::Result;
use async_trait::async_trait;
use smallvec::SmallVec;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::fsi::models::{Coordinate, MacroID, NamespaceID, ScriptureAtom, SubMask, WorkID};
use crate::repository::fsi_repo::FsiRepository;

// Internal struct to map SQL rows before converting to the domain model
#[derive(sqlx::FromRow)]
struct TextRow {
    work_id: i32,
    macro_id: i32,
    namespace_id: i16,
    sub_mask: i16,
    lex_key: String,
    text_content: String,
}

/// # Postgres FSI Repository (Track A)
///
/// ### Architectural Design Decision: Concrete Persistence
/// Implements the `FsiRepository` contract for PostgreSQL. It translates our abstract
/// mathematical domain models (`Coordinate`, `ScriptureAtom`) into raw
/// SQL operations to persist the "Big Scroll" permanently on disk.
///
/// **AI Prompt Hint:** Never leak `sqlx` specific types (like `PgPool`) outside of this struct.
/// The Service Layer must remain completely oblivious to the database technology being used.
pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    /// Initializes a new repository with an injected db connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FsiRepository for PostgresRepository {
    /// ## `get_sequence`
    /// **Parameters:** `work`, `macro_level`,, `namespace`, `start_lex`, `end_lex`.
    ///
    /// ### Architectural Design Decision: Sequence Retrieval & Translation
    /// Fetches a continuous mathematical sequence of FSI coordinates from disk.
    /// Because `lex_key` is indexed via a B-Tree (`idx_fsi_sequence`), this query runs
    /// at extreme speeds, pulling sequential words exactly as they were laid down.
    ///
    /// ### Technical Context: LexKey String Conversion
    /// The database stores `lex_key` as a `VARCHAR` but our Rust domain model keeps it as
    /// a `SmallVec<[u8: 16]>` on the stack for memory safety. This function handles
    /// the safe encoding/decoding between the database adn the stack.
    async fn get_sequence(
        &self,
        work: WorkID,
        macro_level: MacroID,
        namespace: NamespaceID,
        start_lex: Option<&[u8]>,
        end_lex: Option<&[u8]>,
    ) -> Result<Vec<ScriptureAtom>> {
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT work_id, macro_id, namespace_id, sub_mask, lex_key, text_content\
            FROM fsi_texts WHERE ",
        );

        builder
            .push("work_id = ")
            .push_bind(work.0)
            .push(" AND macro_id = ")
            .push_bind(macro_level.0)
            .push(" AND namespace_id = ")
            .push_bind(namespace.0);

        if let Some(start) = start_lex {
            let start_str = String::from_utf8_lossy(start).to_string();
            builder.push(" AND lex_key >= ").push_bind(start_str);
        }

        if let Some(end) = end_lex {
            let end_str = String::from_utf8_lossy(end).to_string();
            builder.push(" AND lex_key <= ").push_bind(end_str);
        }

        // The Engine requires strict ascending order by LexKey
        builder.push(" ORDER BY lex_key ASC");

        let rows: Vec<TextRow> = builder.build_query_as().fetch_all(&self.pool).await?;

        // Translate SQL rows back into the mathematical Domain Model
        let atoms = rows
            .into_iter()
            .map(|row| ScriptureAtom {
                coordinate: Coordinate {
                    work: WorkID(row.work_id),
                    macro_level: MacroID(row.macro_id),
                    namespace: NamespaceID(row.namespace_id),
                    sub_mask: SubMask(row.sub_mask as u16),
                    lex_key: SmallVec::from_slice(row.lex_key.as_bytes()),
                },
                text_content: row.text_content,
            })
            .collect();

        Ok(atoms)
    }

    /// ## `insert_atoms`
    /// **Parameters:** `atoms: Vec<ScriptureAtom>`
    ///
    /// ### Architectural Design Decision: High-Volume Batch Ingestion
    /// The `Ingestor` breaks down massive files (like the Quran) and sends them here
    /// in batches of 5,000. To prevent the database from choking on 5,000 individual
    /// `INSERT` statements, we use `QueryBuilder::push_values` to construct a single,
    /// massive query string, executing the batch in one network round-trip.
    ///
    /// **AI Prompt Hint:** If we hit Postgres's 65,535 parameter limit in the future,
    /// the `Ingestor` batch size in `src/fsi/seeder.rs` must be lowered.
    async fn insert_atoms(&self, atoms: Vec<ScriptureAtom>) -> Result<()> {
        if atoms.is_empty() {
            return Ok(());
        }

        let mut builder = QueryBuilder::<Postgres>::new(
            "INSERT INTO fsi_texts (work_id, macro_id, namespace_id, sub_mask, lex_key, text_content) ",
        );

        builder.push_values(atoms, |mut b, atom| {
            // Safely convert the stack-allocated SmallVec to a standard String for SQL
            let lex_str = String::from_utf8_lossy(&atom.coordinate.lex_key).to_string();

            b.push_bind(atom.coordinate.work.0)
                .push_bind(atom.coordinate.macro_level.0)
                .push_bind(atom.coordinate.namespace.0)
                .push_bind(atom.coordinate.sub_mask.0 as i16) // Postgres doesn't have u16, map to smallint
                .push_bind(lex_str)
                .push_bind(atom.text_content);
        });

        // The composit UNIQUE constraint on the Coordinate prevents collision insertion
        builder
            .push(" ON CONFLICT (work_id, macro_id, lex_key, namespace_id, sub_mask) DO NOTHING");

        builder.build().execute(&self.pool).await?;

        Ok(())
    }
}

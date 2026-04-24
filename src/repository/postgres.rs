//! # PostgreSQL Repository Implementation (The "Data Adapter")
//!
//! ### Architectural Design Decision: Anti-Corruption Layer
//! This module translates raw SQL rows into pure FSI domain models.
//! The engines never see `sqlx` errors or primitives; they only see `ScriptureAtom`
//! or `ScriptureError`.

use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder, Row};

use crate::fsi::models::{
    Coordinate, LexKey, LexiconID, MacroID, NamespaceID, ScriptureAtom, SubMask, WorkID,
};
use crate::repository::ScriptureRepository;
use crate::utils::errors::ScriptureError;

/// ## `ScripturePostgresRepository`
///
/// ### Architectural Design Decision: Concrete Database Implementation
/// The concrete implementation of the data layer using `sqlx`. It manages the connection pool
/// and isolates all SQL-specific logic from the rest of the application.
pub struct ScripturePostgresRepository {
    pool: PgPool,
}

impl ScripturePostgresRepository {
    /// ## `new`
    /// **Parameters:** `pool: PgPool` (The configured PostgreSQL connection pool).
    ///
    /// ### Architectural Design Decision: Dependency Injection
    /// Accepts a pre-configured database pool to allow the application layer to manage
    /// connection configurations, limits, and environmental variables.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// ## `fetch_atom_with_query`
    /// **Parameters:** `query: &str` (The raw SQL query string), `coordinate: &Coordinate` (The mathematical FSI address).
    ///
    /// ### Architectural Design Decision: DRY Query Execution
    /// Helper method to DRY up fetching atoms, eliminating duplicated code.
    /// Both exact and sequential fetches bind the exact same FSI coordinate parameters.
    ///
    /// ### Technical Context: SQLx Parameter Binding
    /// Extracts the primitive values (`.0`) from the NewType structs to bind them properly
    /// to the PostgreSQL driver parameters.
    ///
    /// **AI Prompt Hint:** Ensure any new `SELECT` query utilizing this helper exactly matches
    /// the expected column output shape of the `AtomRow` DTO.
    async fn fetch_atom_with_query(
        &self,
        query: &str,
        coordinate: &Coordinate,
    ) -> Result<ScriptureAtom, ScriptureError> {
        let row: AtomRow = sqlx::query_as(query)
            .bind(coordinate.work_id.0)
            .bind(coordinate.macro_id.0)
            .bind(&coordinate.lex_key.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(ScriptureError::DatabaseError)?
            .ok_or(ScriptureError::NotFound)?;

        Ok(row.into())
    }
}

// ==========================================
// DATA TRANSFER OBJECTS (DTOs) & MAPPINGS
// ==========================================

#[derive(sqlx::FromRow)]
struct AtomRow {
    work_id: i32,
    macro_id: i32,
    lex_key: String,
    namespace_id: i32,
    lexicon_id: i64,
    sub_mask: i16,
    merkle_hash: Vec<u8>,
}

impl From<AtomRow> for ScriptureAtom {
    fn from(row: AtomRow) -> Self {
        ScriptureAtom {
            coordinate: Coordinate {
                work_id: WorkID(row.work_id),
                macro_id: MacroID(row.macro_id),
                lex_key: LexKey(row.lex_key),
            },
            namespace_id: NamespaceID(row.namespace_id),
            lexicon_id: LexiconID(row.lexicon_id),
            sub_mask: SubMask(row.sub_mask),
            merkle_hash: row.merkle_hash,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AliasRow {
    work_id: i32,
    macro_id: i32,
    lex_key: String,
}

impl From<AliasRow> for Coordinate {
    fn from(row: AliasRow) -> Self {
        Coordinate {
            work_id: WorkID(row.work_id),
            macro_id: MacroID(row.macro_id),
            lex_key: LexKey(row.lex_key),
        }
    }
}

// ==========================================
// REPOSITORY IMPLEMENTATION
// ==========================================

#[async_trait]
impl ScriptureRepository for ScripturePostgresRepository {
    /// ## `get_atom_by_coordinate`
    /// **Parameters:** `coord: Coordinate` (The exact FSI coordinate to locate).
    ///
    /// ### Architectural Design Decision: Absolute Point Retrieval
    /// Fetches a single atom by its definitive 3D mathematical coordinate. This acts as the
    /// foundational data retrieval method for the Content Engine.
    async fn get_atom_by_coordinate(
        &self,
        coordinate: Coordinate,
    ) -> Result<ScriptureAtom, ScriptureError> {
        let query = r#"
            SELECT work_id, macro_id, lex_key, namespace_id, lexicon_id, sub_mask, merkle_hash
            FROM fsi_scroll
            WHERE work_id = $1 AND macro_id = $2 AND lex_key = $3
        "#;
        self.fetch_atom_with_query(query, &coordinate).await
    }

    /// ## `resolve_alias`
    /// **Parameters:** `path_string: &str` (The human-readable string, e.g., "quran.1.1").
    ///
    /// ### Architectural Design Decision: Path Resolution
    /// Bridges the gap between human input and mathematical coordinates. Converts a conversational
    /// alias into a strict FSI `Coordinate` using the `fsi_aliases` table.
    async fn resolve_alias(&self, path_string: &str) -> Result<Coordinate, ScriptureError> {
        let query = r#"
            SELECT work_id, macro_id, lex_key
            FROM fsi_aliases
            WHERE alias = $1
        "#;

        let row: AliasRow = sqlx::query_as(query)
            .bind(path_string)
            .fetch_optional(&self.pool)
            .await
            .map_err(ScriptureError::DatabaseError)?
            .ok_or(ScriptureError::NotFound)?;

        Ok(row.into())
    }

    /// ## `get_next_atom`
    /// **Parameters:** `current: Coordinate` (The point of origin for the traversal).
    ///
    /// ### Architectural Design Decision: Database-Level Traversal
    /// Offloads sequential traversal logic to PostgreSQL. It finds the next atom by looking
    /// for a strictly greater FSI coordinate within the same scriptural work.
    ///
    /// ### Technical Context: Lexicographical Sorting
    /// Uses an `ORDER BY` clause coupled with a `LIMIT 1` to efficiently identify the immediately
    /// succeeding verse, even if it crosses macro (chapter) boundaries.
    async fn get_next_atom(&self, current: Coordinate) -> Result<ScriptureAtom, ScriptureError> {
        let query = r#"
            SELECT work_id, macro_id, lex_key, namespace_id, lexicon_id, sub_mask, merkle_hash
            FROM fsi_scroll
            WHERE work_id = $1
              AND (macro_id > $2 OR (macro_id = $2 AND lex_key > $3))
            ORDER BY macro_id ASC, lex_key ASC
            LIMIT 1
        "#;
        self.fetch_atom_with_query(query, &current).await
    }

    /// ## `insert_lexicon_entry`
    /// **Parameters:** `text: &str` (The raw text snippet to index).
    ///
    /// ### Architectural Design Decision: Idempotency & De-duplication
    /// Tries to insert the raw text into the lexicon. If the exact text already exists,
    /// it catches the conflict and returns the existing pointer, minimizing dictionary bloat.
    ///
    /// ### Technical Context: CTE (Common Table Expression)
    /// Utilizes a `WITH` query to handle the insert and fallback to a `SELECT` union on conflict
    /// without throwing a unique constraint database error.
    async fn insert_lexicon_entry(&self, text: &str) -> Result<LexiconID, ScriptureError> {
        let query = r#"
            WITH new_row AS (
                INSERT INTO fsi_lexicon (body_text)
                VALUES ($1)
                ON CONFLICT (body_text) DO NOTHING
                RETURNING id
            )
            SELECT id FROM new_row
            UNION
            SELECT id FROM fsi_lexicon WHERE body_text = $1;
        "#;

        let row = sqlx::query(query)
            .bind(text)
            .fetch_one(&self.pool)
            .await
            .map_err(ScriptureError::DatabaseError)?;

        let id: i64 = row.try_get("id").map_err(ScriptureError::DatabaseError)?;
        Ok(LexiconID(id))
    }

    /// ## `insert_atoms`
    /// **Parameters:** `atoms: &[ScriptureAtom]` (A slice of fully mapped scriptural atoms).
    ///
    /// ### Architectural Design Decision: High-Performance Batch Insertion
    /// Using `sqlx::QueryBuilder` minimizes database round-trips. It batches
    /// all atoms generated by the Ingestion Engine into a single massive query.
    ///
    /// ### Technical Context: Conflict Resolution (UPSERT)
    /// Employs `ON CONFLICT DO UPDATE` so that re-running the ingestion pipeline safely updates
    /// existing paths (e.g., fixing a typo) rather than crashing the pipeline.
    ///
    /// **AI Prompt Hint:** If the `ScriptureAtom` schema is updated, you must remember to update
    /// the parameter bindings and the `EXCLUDED` update rules in this builder.
    async fn insert_atoms(&self, atoms: &[ScriptureAtom]) -> Result<(), ScriptureError> {
        if atoms.is_empty() {
            return Ok(());
        }

        let mut query_builder = QueryBuilder::new(
            "INSERT INTO fsi_scroll (work_id, macro_id, lex_key, namespace_id, lexicon_id, sub_mask, merkle_hash) ",
        );

        query_builder.push_values(atoms, |mut b, atom| {
            b.push_bind(atom.coordinate.work_id.0)
                .push_bind(atom.coordinate.macro_id.0)
                .push_bind(atom.coordinate.lex_key.0.clone())
                .push_bind(atom.namespace_id.0)
                .push_bind(atom.lexicon_id.0)
                .push_bind(atom.sub_mask.0)
                .push_bind(atom.merkle_hash.clone());
        });

        query_builder.push(
            " ON CONFLICT (work_id, macro_id, lex_key, namespace_id)
              DO UPDATE SET
                  lexicon_id = EXCLUDED.lexicon_id,
                  merkle_hash = EXCLUDED.merkle_hash,
                  sub_mask = EXCLUDED.sub_mask",
        );

        let query = query_builder.build();
        query
            .execute(&self.pool)
            .await
            .map_err(ScriptureError::DatabaseError)?;

        Ok(())
    }

    /// ## `get_lexicon_text`
    ///
    /// ### Architectural Design Decision: Dictionary Resolution
    /// Resolves the dictionary pointer into human-readable text.
    async fn get_lexicon_text(&self, lexicon_id: LexiconID) -> Result<String, ScriptureError> {
        let query = "SELECT body_text FROM fsi_lexicon WHERE id = $1";

        let row = sqlx::query(query)
            .bind(lexicon_id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(ScriptureError::DatabaseError)?
            .ok_or(ScriptureError::NotFound)?;

        let text: String = row
            .try_get("body_text")
            .map_err(ScriptureError::DatabaseError)?;
        Ok(text)
    }
}

//! # PostgreSQL Repository Implementation
//!
//! ### Architectural Design Decision: Anti-Corruption Layer
//! This module translates raw SQL rows into pure FSI domain models.
//! The engines never see `sqlx` errors or primitives; they only see `ScriptureAtom`
//! or `ScriptureError`

use crate::fsi::models::{
    Coordinate, LexKey, LexiconID, MacroID, NamespaceID, ScriptureAtom, SubMask, WorkID,
};
use crate::repository::ScriptureRepository;
use crate::utils::errors::ScriptureError;
use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder, Row};

/// ## `ScripturePostgresRepo`
/// The concrete implementation of the data layer using `sqlx`.
pub struct ScripturePostgresRepo {
    pool: PgPool,
}

/// A private Data Transfer Object (DTO) used solely to catch flat SQL rows.
/// This prevents our pure FSI models from needing `sqlx::FromRow`.
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

#[async_trait]
impl ScriptureRepository for ScripturePostgresRepo {
    /// ## `resolve_alias`
    ///
    /// ### Architectural Design Decision: Alias Translation
    /// Resolves human-readable shorthands (e.g., "quran.1.1") into pure FSI coordinates
    /// by querying a dedicated alias mapping table.
    async fn resolve_alias(&self, path_string: &str) -> Result<Coordinate, ScriptureError> {
        let query = r#"
            SELECT work_id, macro_id, lex_key
            FROM fsi_aliases
            WHERE alias = $1
        "#;

        let row = sqlx::query(query)
            .bind(path_string)
            .fetch_optional(&self.pool)
            .await
            .map_err(ScriptureError::DatabaseError)?
            .ok_or(ScriptureError::NotFound)?;

        // Manually map the raw rows into pure NewTypes
        let work_id: i32 = row
            .try_get("work_id")
            .map_err(ScriptureError::DatabaseError)?;
        let macro_id: i32 = row
            .try_get("macro_id")
            .map_err(ScriptureError::DatabaseError)?;
        let lex_key: String = row
            .try_get("lex_key")
            .map_err(ScriptureError::DatabaseError)?;

        Ok(Coordinate {
            work_id: WorkID(work_id),
            macro_id: MacroID(macro_id),
            lex_key: LexKey(lex_key),
        })
    }

    /// ## `get_next_atom`
    ///
    /// ### Architectural Design Decision: Database-Level Traversal
    /// Offloads the sequential traversal logic to PostgreSQL. It finds the next atom
    /// by looking for a strictly greater FSI coordinate within the same work.
    async fn get_next_atom(&self, current: Coordinate) -> Result<ScriptureAtom, ScriptureError> {
        let query = r#"
            SELECT work_id, macro_id, lex_key, namespace_id, lexicon_id, sub_mask, merkle_hash
            FROM fsi_scroll
            WHERE work_id = $1
              AND (macro_id > $2 OR (macro_id = $2 AND lex_key > $3))
            ORDER BY macro_id ASC, lex_key ASC
            LIMIT 1
        "#;

        let row: AtomRow = sqlx::query_as(query)
            .bind(current.work_id.0)
            .bind(current.macro_id.0)
            .bind(&current.lex_key.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(ScriptureError::DatabaseError)?
            .ok_or(ScriptureError::NotFound)?;

        Ok(ScriptureAtom {
            coordinate: Coordinate {
                work_id: WorkID(row.work_id),
                macro_id: MacroID(row.macro_id),
                lex_key: LexKey(row.lex_key),
            },
            namespace_id: NamespaceID(row.namespace_id),
            lexicon_id: LexiconID(row.lexicon_id),
            sub_mask: SubMask(row.sub_mask),
            merkle_hash: row.merkle_hash,
        })
    }
    /// ## `get_atom_by_coordinate`
    /// **Parameters:** `coordinate: Coordinate`
    ///
    /// ### Architectural Design Decision: Explicit Model Mapping
    /// We query the flat `fsi_scroll` table, catch it with `AtomRow`, and manually
    /// construct the nested `ScriptureAtom` to preserve domain purity.
    async fn get_atom_by_coordinate(
        &self,
        coordinate: Coordinate,
    ) -> Result<ScriptureAtom, ScriptureError> {
        let query = r#"
        SELECT work_id, macro_id, lex_key, namespace_id, lexicon_id, sub_mask, merkle_hash
        FROM fsi_scroll
        WHERE work_id = $1 AND macro_id = $2 AND lex_key = $3
        "#;

        let row: AtomRow = sqlx::query_as(query)
            // Unwrap the NewTypes back into their primitives for the SQL driver
            // The `.0` accesses the inner primitive `i32` of our NewType, keeping SQL driver happy
            // and Rust implementation safe.
            .bind(coordinate.work_id.0)
            .bind(coordinate.macro_id.0)
            .bind(&coordinate.lex_key.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(ScriptureError::DatabaseError)?
            .ok_or(ScriptureError::NotFound)?;

        // Manually assemble the pure domain model
        Ok(ScriptureAtom {
            coordinate: Coordinate {
                work_id: WorkID(row.work_id),
                macro_id: MacroID(row.macro_id),
                lex_key: LexKey(row.lex_key),
            },
            namespace_id: NamespaceID(row.namespace_id),
            lexicon_id: LexiconID(row.lexicon_id),
            sub_mask: SubMask(row.sub_mask),
            merkle_hash: row.merkle_hash,
        })
    }

    // --- [ INGESTION METHODS ] ---

    /// ## `insert_lexicon_entry`
    ///
    /// ### Architectural Design Decision: Idempotency & De-duplication
    /// Tries to insert the raw text into the lexicon. If the exact text already exists,
    /// it catches the conflict and returns the existing `LexiconID`.
    async fn insert_lexicon_entry(&self, text: &str) -> Result<LexiconID, ScriptureError> {
        let query = r#"
        WITH new_row AS (
        INSERT INTO fsi_lexicon (body_text)
        VALUES ($1)
        ON CONFLICT (body_text) DO NOHTING
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
    ///
    /// ### Architectural Design Decision: High-Performacne Batch Insertion
    /// Using `sqlx::QueryBuilder` minimizes database round-trips. It batches
    /// all atoms generated by the Ingestion Engine into a single massive query.
    async fn insert_atoms(&self, atoms: &[ScriptureAtom]) -> Result<(), ScriptureError> {
        if atoms.is_empty() {
            return Ok(());
        }

        let mut query_builder = QueryBuilder::new(
            "INSERT INTO fsi_scroll (work_id, macro_id, lex_key, namespace_id, lexicon_id, sub_mask, merkle_hash) ",
        );

        // Batches the array of Atoms directly into the SQL statement parameters
        query_builder.push_values(atoms, |mut b, atom| {
            b.push_bind(atom.coordinate.work_id.0)
                .push_bind(atom.coordinate.macro_id.0)
                .push_bind(atom.coordinate.lex_key.0.clone())
                .push_bind(atom.namespace_id.0)
                .push_bind(atom.lexicon_id.0)
                .push_bind(atom.sub_mask.0)
                .push_bind(atom.merkle_hash.clone());
        });

        // Add conflict resolution so re-running ingestion updates existing paths
        query_builder.push(
            "ON CONFLICT (work_id, macro_id, lex_key, namespace_id)\
            DO UPDATE SET\
            lexicon_id = EXCLUDED.lexicon_id, \
            merkle_hash = EXCLUDED.merkle_hash,\
            sub_mask = EXCLUDED.sub_mask",
        );

        let query = query_builder.build();
        query
            .execute(&self.pool)
            .await
            .map_err(ScriptureError::DatabaseError)?;

        Ok(())
    }
}

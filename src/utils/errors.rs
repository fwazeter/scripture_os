use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScriptureError {
    #[error("Coordinate not found in FSI")]
    NotFound,
    #[error("Database connection or query failure: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Parsing error: {0}")]
    ParseError(String),
    #[error("Cryptographic mismatch in Merkle Tree")]
    IntegrityError,
}

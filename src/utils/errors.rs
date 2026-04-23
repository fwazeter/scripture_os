use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScriptureError {
    #[error("Coordinate not found in FSI")]
    NotFound,
    #[error("Database connection failure")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Cryptographic mismatch in Merkle Tree")]
    IntegrityError,
}

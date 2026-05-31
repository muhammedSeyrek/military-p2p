use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Not found")]
    NotFound,

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Protocol error: {0}")]
    Protocol(#[from] mp_protocol::ProtocolError),
}

pub type Result<T> = std::result::Result<T, StorageError>;

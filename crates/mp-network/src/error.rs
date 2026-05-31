use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Certificate error: {0}")]
    Certificate(String),

    #[error("TLS error: {0}")]
    Tls(String),

    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Server bind error: {0}")]
    Bind(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Storage error: {0}")]
    Storage(#[from] mp_storage::StorageError),

    #[error("Protocol error: {0}")]
    Protocol(#[from] mp_protocol::ProtocolError),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, NetworkError>;

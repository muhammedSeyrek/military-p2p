use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("RAID error: {0}")]
    Raid(String),

    #[error("Invalid part index: {got}, expected 0..{total}")]
    InvalidPartIndex { got: usize, total: usize },

    #[error("Missing parts: {missing:?} (need indices 0..{total})")]
    MissingParts { missing: Vec<usize>, total: usize },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid rank: {0}")]
    InvalidRank(String),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;

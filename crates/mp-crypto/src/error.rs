use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("AES encryption failed: {0}")]
    AesEncrypt(String),

    #[error("AES decryption failed: {0}")]
    AesDecrypt(String),

    #[error("RSA error: {0}")]
    Rsa(#[from] rsa::Error),

    #[error("PKCS8 error: {0}")]
    Pkcs8(String),

    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Invalid key length: expected {expected}, got {got}")]
    InvalidKeyLength { expected: usize, got: usize },

    #[error("Merkle tree error: {0}")]
    Merkle(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;

//! Cryptographic primitives for the Military P2P system.
//!
//! - `aes`: AES-256-GCM authenticated encryption
//! - `rsa_keys`: RSA-2048 OAEP-SHA256 key wrapping
//! - `hash`: SHA-256 hashing
//! - `merkle`: Merkle tree for tamper detection
//! - `error`: shared error type for the crate

pub mod aes;
pub mod error;
pub mod hash;
pub mod merkle;
pub mod rsa_keys;

pub use error::CryptoError;

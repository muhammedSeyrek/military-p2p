//! AES-256-GCM authenticated encryption.
//!
//! GCM mode provides both confidentiality and integrity via an auth tag,
//! so tampering is detected at the cipher layer in addition to the Merkle layer.

use crate::error::{CryptoError, Result};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

/// A 256-bit AES key.
#[derive(Clone)]
pub struct AesKey(pub [u8; KEY_SIZE]);

impl AesKey {
    /// Generate a fresh random key from the OS RNG.
    pub fn random() -> Self {
        let mut key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        Self(key)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != KEY_SIZE {
            return Err(CryptoError::InvalidKeyLength {
                expected: KEY_SIZE,
                got: bytes.len(),
            });
        }
        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(bytes);
        Ok(Self(key))
    }

    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.0
    }
}

/// AES-GCM ciphertext bundled with its nonce.
///
/// The nonce must travel with the ciphertext for decryption.
#[derive(Clone, Debug)]
pub struct Ciphertext {
    pub nonce: [u8; NONCE_SIZE],
    pub data: Vec<u8>,
}

impl Ciphertext {
    /// Serialize as `nonce || data`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(NONCE_SIZE + self.data.len());
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&self.data);
        out
    }

    /// Parse a `nonce || data` byte sequence.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < NONCE_SIZE {
            return Err(CryptoError::AesDecrypt(
                "ciphertext too short for nonce".into(),
            ));
        }
        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&bytes[..NONCE_SIZE]);
        Ok(Self {
            nonce,
            data: bytes[NONCE_SIZE..].to_vec(),
        })
    }
}

/// Encrypt plaintext with a fresh random nonce.
pub fn encrypt(key: &AesKey, plaintext: &[u8]) -> Result<Ciphertext> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_bytes()));

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let data = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::AesEncrypt(e.to_string()))?;

    Ok(Ciphertext {
        nonce: nonce_bytes,
        data,
    })
}

/// Decrypt a ciphertext. Returns an error if the auth tag fails to verify.
pub fn decrypt(key: &AesKey, ct: &Ciphertext) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_bytes()));
    let nonce = Nonce::from_slice(&ct.nonce);

    cipher
        .decrypt(nonce, ct.data.as_ref())
        .map_err(|e| CryptoError::AesDecrypt(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = AesKey::random();
        let msg = b"Operation Sand Campaign: advance at 03:00.";
        let ct = encrypt(&key, msg).unwrap();
        let pt = decrypt(&key, &ct).unwrap();
        assert_eq!(pt, msg);
    }

    #[test]
    fn tamper_detected() {
        let key = AesKey::random();
        let ct = encrypt(&key, b"order").unwrap();
        let mut tampered = ct.clone();
        tampered.data[0] ^= 0x01;
        assert!(decrypt(&key, &tampered).is_err());
    }

    #[test]
    fn wrong_key_fails() {
        let k1 = AesKey::random();
        let k2 = AesKey::random();
        let ct = encrypt(&k1, b"order").unwrap();
        assert!(decrypt(&k2, &ct).is_err());
    }

    #[test]
    fn serialize_roundtrip() {
        let key = AesKey::random();
        let ct = encrypt(&key, b"test").unwrap();
        let bytes = ct.to_bytes();
        let ct2 = Ciphertext::from_bytes(&bytes).unwrap();
        let pt = decrypt(&key, &ct2).unwrap();
        assert_eq!(pt, b"test");
    }
}

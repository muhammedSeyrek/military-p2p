//! RSA-2048 key generation and OAEP-SHA256 wrapping.
//!
//! Used to encrypt the per-operation AES key + nonce for each recipient.
//! OAEP is chosen over PKCS1 v1.5 to avoid Bleichenbacher attacks.

use crate::error::{CryptoError, Result};
use rand::rngs::OsRng;
use rsa::{
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    Oaep, RsaPrivateKey, RsaPublicKey,
};
use sha2::Sha256;

pub const RSA_BITS: usize = 2048;

pub struct KeyPair {
    pub private: RsaPrivateKey,
    pub public: RsaPublicKey,
}

impl KeyPair {
    pub fn generate() -> Result<Self> {
        let mut rng = OsRng;
        let private = RsaPrivateKey::new(&mut rng, RSA_BITS).map_err(CryptoError::Rsa)?;
        let public = RsaPublicKey::from(&private);
        Ok(Self { private, public })
    }

    pub fn private_to_pem(&self) -> Result<String> {
        self.private
            .to_pkcs8_pem(LineEnding::LF)
            .map(|s| s.to_string())
            .map_err(|e| CryptoError::Pkcs8(e.to_string()))
    }

    pub fn public_to_pem(&self) -> Result<String> {
        self.public
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| CryptoError::Pkcs8(e.to_string()))
    }
}

pub fn private_from_pem(pem: &str) -> Result<RsaPrivateKey> {
    RsaPrivateKey::from_pkcs8_pem(pem).map_err(|e| CryptoError::Pkcs8(e.to_string()))
}

pub fn public_from_pem(pem: &str) -> Result<RsaPublicKey> {
    RsaPublicKey::from_public_key_pem(pem).map_err(|e| CryptoError::Pkcs8(e.to_string()))
}

pub fn encrypt_with_public(pubkey: &RsaPublicKey, data: &[u8]) -> Result<Vec<u8>> {
    let mut rng = OsRng;
    let padding = Oaep::new::<Sha256>();
    pubkey
        .encrypt(&mut rng, padding, data)
        .map_err(CryptoError::Rsa)
}

pub fn decrypt_with_private(privkey: &RsaPrivateKey, ct: &[u8]) -> Result<Vec<u8>> {
    let padding = Oaep::new::<Sha256>();
    privkey.decrypt(padding, ct).map_err(CryptoError::Rsa)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aes::AesKey;

    #[test]
    fn keypair_pem_roundtrip() {
        let kp = KeyPair::generate().unwrap();
        let priv_pem = kp.private_to_pem().unwrap();
        let pub_pem = kp.public_to_pem().unwrap();

        let _ = private_from_pem(&priv_pem).unwrap();
        let _ = public_from_pem(&pub_pem).unwrap();
    }

    #[test]
    fn encrypt_aes_key_with_rsa() {
        let kp = KeyPair::generate().unwrap();
        let aes_key = AesKey::random();

        let encrypted = encrypt_with_public(&kp.public, aes_key.as_bytes()).unwrap();
        let decrypted = decrypt_with_private(&kp.private, &encrypted).unwrap();

        assert_eq!(decrypted, aes_key.as_bytes());
    }
}

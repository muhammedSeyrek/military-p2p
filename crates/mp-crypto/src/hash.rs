//! SHA-256 helpers.

use sha2::{Digest, Sha256};

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Concatenate two 32-byte hashes and SHA-256 the result.
/// Used to build Merkle tree internal nodes.
pub fn sha256_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

pub fn to_hex(hash: &[u8; 32]) -> String {
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_hash_matches_known_value() {
        let h = sha256(b"");
        assert_eq!(
            to_hex(&h),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn deterministic() {
        assert_eq!(sha256(b"hello"), sha256(b"hello"));
    }

    #[test]
    fn different_inputs_different_outputs() {
        assert_ne!(sha256(b"order-1"), sha256(b"order-2"));
    }
}

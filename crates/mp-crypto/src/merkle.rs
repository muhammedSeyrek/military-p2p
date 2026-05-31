//! Merkle tree for tamper detection over a list of byte chunks.
//!
//! Each leaf is `SHA-256(chunk)`. Internal nodes are `SHA-256(left || right)`.
//! Odd nodes are paired with themselves (instead of duplicating a leaf).

use crate::error::{CryptoError, Result};
use crate::hash::{sha256, sha256_pair};

#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// `levels[0]` = leaf hashes, `levels[N-1]` = `[root]`
    levels: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    /// Build a tree from the given chunks (one leaf per chunk).
    pub fn from_chunks(chunks: &[Vec<u8>]) -> Result<Self> {
        if chunks.is_empty() {
            return Err(CryptoError::Merkle("no chunks provided".into()));
        }

        let leaves: Vec<[u8; 32]> = chunks.iter().map(|c| sha256(c)).collect();
        let mut levels = vec![leaves];

        while levels.last().unwrap().len() > 1 {
            let current = levels.last().unwrap();
            let mut next = Vec::with_capacity((current.len() + 1) / 2);

            for pair in current.chunks(2) {
                let combined = if pair.len() == 2 {
                    sha256_pair(&pair[0], &pair[1])
                } else {
                    // Odd node out — pair with itself.
                    sha256_pair(&pair[0], &pair[0])
                };
                next.push(combined);
            }
            levels.push(next);
        }

        Ok(Self { levels })
    }

    pub fn root(&self) -> [u8; 32] {
        *self.levels.last().unwrap().first().unwrap()
    }

    pub fn leaf_count(&self) -> usize {
        self.levels[0].len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_chunk_root_equals_leaf_hash() {
        let chunks = vec![b"only".to_vec()];
        let tree = MerkleTree::from_chunks(&chunks).unwrap();
        assert_eq!(tree.root(), sha256(b"only"));
    }

    #[test]
    fn four_chunks_three_levels() {
        let chunks: Vec<Vec<u8>> = (0..4u8).map(|i| vec![i]).collect();
        let tree = MerkleTree::from_chunks(&chunks).unwrap();
        assert_eq!(tree.levels.len(), 3);
    }

    #[test]
    fn tampering_one_byte_changes_root() {
        let original: Vec<Vec<u8>> = vec![
            b"A0".to_vec(),
            b"B0".to_vec(),
            b"C0".to_vec(),
            b"D0".to_vec(),
        ];
        let mut tampered = original.clone();
        tampered[2][0] ^= 0x01;

        let r1 = MerkleTree::from_chunks(&original).unwrap().root();
        let r2 = MerkleTree::from_chunks(&tampered).unwrap().root();
        assert_ne!(r1, r2);
    }

    #[test]
    fn odd_number_of_chunks() {
        let chunks: Vec<Vec<u8>> = vec![b"A".to_vec(), b"B".to_vec(), b"C".to_vec()];
        let tree = MerkleTree::from_chunks(&chunks).unwrap();
        assert_eq!(tree.leaf_count(), 3);
    }
}

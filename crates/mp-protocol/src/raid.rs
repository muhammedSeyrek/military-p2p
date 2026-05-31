//! RAID-0 style split/join over byte arrays.
//!
//! `split` divides the input into `num_parts` near-equal chunks. The last
//! chunk absorbs the remainder, so `join` reassembles the exact original.
//! No redundancy — losing any single part means data is unrecoverable.

use crate::error::{ProtocolError, Result};

/// Split `data` into `num_parts` chunks. The final chunk includes any
/// leftover bytes when the length is not evenly divisible.
pub fn split(data: &[u8], num_parts: usize) -> Result<Vec<Vec<u8>>> {
    if num_parts == 0 {
        return Err(ProtocolError::Raid("num_parts must be > 0".into()));
    }
    if data.is_empty() {
        return Err(ProtocolError::Raid("cannot split empty data".into()));
    }
    if data.len() < num_parts {
        return Err(ProtocolError::Raid(format!(
            "data length {} < num_parts {}",
            data.len(),
            num_parts
        )));
    }

    let base_size = data.len() / num_parts;
    let mut parts = Vec::with_capacity(num_parts);

    for i in 0..num_parts {
        let start = i * base_size;
        let end = if i == num_parts - 1 {
            data.len()
        } else {
            start + base_size
        };
        parts.push(data[start..end].to_vec());
    }

    Ok(parts)
}

/// Reassemble parts (in any order) into the original byte array.
/// Fails if any index is missing or duplicated.
pub fn join(mut indexed_parts: Vec<(usize, Vec<u8>)>, total_parts: usize) -> Result<Vec<u8>> {
    if indexed_parts.len() != total_parts {
        let present: std::collections::HashSet<usize> =
            indexed_parts.iter().map(|(i, _)| *i).collect();
        let missing: Vec<usize> = (0..total_parts).filter(|i| !present.contains(i)).collect();
        return Err(ProtocolError::MissingParts {
            missing,
            total: total_parts,
        });
    }

    // Sort by part index.
    indexed_parts.sort_by_key(|(i, _)| *i);

    for (expected, (got, _)) in indexed_parts.iter().enumerate() {
        if expected != *got {
            return Err(ProtocolError::InvalidPartIndex {
                got: *got,
                total: total_parts,
            });
        }
    }

    let total_size: usize = indexed_parts.iter().map(|(_, c)| c.len()).sum();
    let mut result = Vec::with_capacity(total_size);
    for (_, chunk) in indexed_parts {
        result.extend_from_slice(&chunk);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_evenly_divisible() {
        let parts = split(b"abcdefgh", 4).unwrap();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], b"ab");
        assert_eq!(parts[1], b"cd");
        assert_eq!(parts[2], b"ef");
        assert_eq!(parts[3], b"gh");
    }

    #[test]
    fn split_with_remainder() {
        let parts = split(b"abcdefghij", 3).unwrap();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], b"abc");
        assert_eq!(parts[1], b"def");
        assert_eq!(parts[2], b"ghij");
    }

    #[test]
    fn split_join_roundtrip() {
        let original: Vec<u8> = (0..100).collect();
        let parts = split(&original, 7).unwrap();

        let indexed: Vec<(usize, Vec<u8>)> = parts.into_iter().enumerate().collect();

        let rejoined = join(indexed, 7).unwrap();
        assert_eq!(rejoined, original);
    }

    #[test]
    fn join_rejects_missing_part() {
        // 4 parts expected but only 3 provided.
        let parts = vec![(0, vec![1, 2]), (1, vec![3, 4]), (2, vec![5, 6])];
        let result = join(parts, 4);
        assert!(matches!(result, Err(ProtocolError::MissingParts { .. })));
    }

    #[test]
    fn join_handles_out_of_order_parts() {
        let parts = vec![(2, vec![5, 6]), (0, vec![1, 2]), (1, vec![3, 4])];
        let result = join(parts, 3).unwrap();
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn join_rejects_duplicate_index() {
        // Two parts share index 1, index 2 is missing.
        let parts = vec![(0, vec![1]), (1, vec![2]), (1, vec![3])];
        let result = join(parts, 3);
        assert!(result.is_err());
    }

    #[test]
    fn split_rejects_zero_parts() {
        assert!(split(b"abc", 0).is_err());
    }

    #[test]
    fn split_rejects_empty_data() {
        assert!(split(b"", 4).is_err());
    }

    #[test]
    fn split_rejects_more_parts_than_bytes() {
        // 3 bytes, 5 parts — doesn't make sense.
        assert!(split(b"abc", 5).is_err());
    }
}

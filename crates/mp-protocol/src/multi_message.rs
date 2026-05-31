//! Multi-recipient message format.
//!
//! The General Staff dispatches N different messages to N commanders.
//! These messages are packed into a single byte array:
//!
//! ```text
//! [len_0: 4 bytes LE][msg_0 bytes][len_1: 4 bytes LE][msg_1 bytes]...
//! ```
//!
//! Every commander decrypts the entire payload (M) via AES-GCM but only
//! reads the message at their own index. The other commanders' messages
//! are technically readable in RAM, but by application convention they
//! are never displayed or persisted.

use crate::error::{ProtocolError, Result};

/// Pack N messages into a single byte array.
pub fn pack(messages: &[&[u8]]) -> Vec<u8> {
    let total_size: usize = messages.iter().map(|m| 4 + m.len()).sum();
    let mut out = Vec::with_capacity(total_size);
    for msg in messages {
        let len = msg.len() as u32;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(msg);
    }
    out
}

/// Unpack a combined byte array into the original N messages.
pub fn unpack(data: &[u8]) -> Result<Vec<Vec<u8>>> {
    let mut messages = Vec::new();
    let mut cursor = 0;

    while cursor < data.len() {
        if cursor + 4 > data.len() {
            return Err(ProtocolError::Raid(format!(
                "truncated length prefix at offset {}",
                cursor
            )));
        }
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&data[cursor..cursor + 4]);
        let len = u32::from_le_bytes(len_bytes) as usize;
        cursor += 4;

        if cursor + len > data.len() {
            return Err(ProtocolError::Raid(format!(
                "truncated message at offset {} (need {} more bytes)",
                cursor, len
            )));
        }
        messages.push(data[cursor..cursor + len].to_vec());
        cursor += len;
    }

    Ok(messages)
}

/// Extract a single message at the given index from the packed payload.
pub fn extract_at(data: &[u8], index: usize) -> Result<Vec<u8>> {
    let all = unpack(data)?;
    all.into_iter()
        .nth(index)
        .ok_or_else(|| ProtocolError::InvalidPartIndex {
            got: index,
            total: 0, // unknown here; the error message will be vague
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_unpack_roundtrip() {
        let msgs: Vec<&[u8]> = vec![b"hello", b"hi there", b"hi", b"ciao"];
        let packed = pack(&msgs);
        let unpacked = unpack(&packed).unwrap();
        assert_eq!(unpacked.len(), 4);
        assert_eq!(unpacked[0], b"hello");
        assert_eq!(unpacked[1], b"hi there");
        assert_eq!(unpacked[2], b"hi");
        assert_eq!(unpacked[3], b"ciao");
    }

    #[test]
    fn extract_specific_index() {
        let msgs: Vec<&[u8]> = vec![b"A", b"B", b"C"];
        let packed = pack(&msgs);
        assert_eq!(extract_at(&packed, 0).unwrap(), b"A");
        assert_eq!(extract_at(&packed, 1).unwrap(), b"B");
        assert_eq!(extract_at(&packed, 2).unwrap(), b"C");
    }

    #[test]
    fn empty_message() {
        let msgs: Vec<&[u8]> = vec![b"", b"non-empty", b""];
        let packed = pack(&msgs);
        let unpacked = unpack(&packed).unwrap();
        assert_eq!(unpacked.len(), 3);
        assert_eq!(unpacked[0], b"");
        assert_eq!(unpacked[1], b"non-empty");
        assert_eq!(unpacked[2], b"");
    }

    #[test]
    fn unicode_message() {
        // Turkish text exercises non-ASCII bytes in the payload.
        let m1 = "Doğu sınırına intikal et".as_bytes();
        let m2 = "Hava sahasını kapat".as_bytes();
        let msgs: Vec<&[u8]> = vec![m1, m2];
        let packed = pack(&msgs);
        let unpacked = unpack(&packed).unwrap();
        assert_eq!(unpacked[0], m1);
        assert_eq!(unpacked[1], m2);
    }

    #[test]
    fn rejects_truncated_length() {
        // 2 bytes provided, but 4-byte length prefix expected.
        let bad = vec![1, 2];
        assert!(unpack(&bad).is_err());
    }

    #[test]
    fn rejects_truncated_data() {
        // Header says length=10, but only 3 bytes follow.
        let bad = vec![10, 0, 0, 0, b'a', b'b', b'c'];
        assert!(unpack(&bad).is_err());
    }
}

//! Operation payload types and base64 serde helpers.

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An AES key wrapped under a recipient's RSA public key.
/// The blob is base64-encoded over the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEnvelope {
    #[serde(with = "base64_serde")]
    pub encrypted_blob: Vec<u8>,
}

/// Operation metadata. The 32-byte Merkle root identifies the
/// expected ciphertext layout for tamper detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: Uuid,
    pub name: String,
    #[serde(with = "base64_serde_32")]
    pub merkle_root: [u8; 32],
    pub total_parts: usize,
    pub created_at: DateTime<Utc>,
}

impl Operation {
    pub fn new(name: String, merkle_root: [u8; 32], total_parts: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            merkle_root,
            total_parts,
            created_at: Utc::now(),
        }
    }
}

/// One ciphertext chunk belonging to a specific operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPart {
    pub operation_id: Uuid,
    pub part_index: usize,
    #[serde(with = "base64_serde")]
    pub ciphertext_chunk: Vec<u8>,
}

impl OperationPart {
    pub fn new(operation_id: Uuid, part_index: usize, ciphertext_chunk: Vec<u8>) -> Self {
        Self {
            operation_id,
            part_index,
            ciphertext_chunk,
        }
    }
}

/// Variable-length byte arrays as base64 over the wire.
mod base64_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&B64.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        B64.decode(&s).map_err(serde::de::Error::custom)
    }
}

/// Fixed 32-byte hashes (Merkle root, leaf hash) as base64 over the wire.
mod base64_serde_32 {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&B64.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let s = String::deserialize(d)?;
        let bytes = B64.decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom(format!(
                "expected 32 bytes, got {}",
                bytes.len()
            )));
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_serialization_uses_base64() {
        let root = [0xAB; 32];
        let op = Operation::new("Sand Campaign".into(), root, 4);
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("merkle_root"));
        let parsed: Operation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.merkle_root, root);
        assert_eq!(parsed.name, "Sand Campaign");
        assert_eq!(parsed.total_parts, 4);
    }

    #[test]
    fn part_serialization_roundtrip() {
        let op_id = Uuid::new_v4();
        let part = OperationPart::new(op_id, 2, vec![1, 2, 3, 4, 5]);
        let json = serde_json::to_string(&part).unwrap();
        let parsed: OperationPart = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.operation_id, op_id);
        assert_eq!(parsed.part_index, 2);
        assert_eq!(parsed.ciphertext_chunk, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn key_envelope_roundtrip() {
        let env = KeyEnvelope {
            encrypted_blob: vec![0xDE, 0xAD, 0xBE, 0xEF],
        };
        let json = serde_json::to_string(&env).unwrap();
        let parsed: KeyEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.encrypted_blob, env.encrypted_blob);
    }

    #[test]
    fn rejects_wrong_size_merkle_root() {
        let bad_json = r#"{"id":"00000000-0000-0000-0000-000000000000","name":"x","merkle_root":"AAAAAAAAAAAAAAAAAAAAAA==","total_parts":1,"created_at":"2025-01-01T00:00:00Z"}"#;
        let parsed: std::result::Result<Operation, _> = serde_json::from_str(bad_json);
        assert!(parsed.is_err());
    }
}

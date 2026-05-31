//! HTTP request and response DTOs shared between the general and commander nodes.

use crate::operation::{KeyEnvelope, OperationPart};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchOperationRequest {
    pub operation_id: Uuid,
    pub operation_name: String,
    pub total_parts: usize,
    pub merkle_root_hex: String,
    pub leaf_hash_hex: String,
    pub part_index: usize,
    pub key_envelope: KeyEnvelope,
    pub part: OperationPart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchOperationResponse {
    pub accepted: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchPartRequest {
    pub operation_id: Uuid,
    pub requester_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchPartResponse {
    pub part: OperationPart,
    pub leaf_hash_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::OperationPart;

    #[test]
    fn dispatch_request_serialization() {
        let req = DispatchOperationRequest {
            operation_id: Uuid::new_v4(),
            operation_name: "Sand Campaign".into(),
            total_parts: 4,
            merkle_root_hex: "deadbeef".into(),
            leaf_hash_hex: "cafebabe".into(),
            part_index: 1,
            key_envelope: KeyEnvelope {
                encrypted_blob: vec![1, 2, 3],
            },
            part: OperationPart::new(Uuid::new_v4(), 1, vec![4, 5, 6]),
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: DispatchOperationRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.operation_name, "Sand Campaign");
        assert_eq!(parsed.part_index, 1);
    }
}

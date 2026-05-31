//! HTTP handler functions.

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use uuid::Uuid;

use mp_protocol::api::{
    DispatchOperationRequest, DispatchOperationResponse, ErrorResponse, FetchPartResponse,
};
use mp_storage::commander::CommanderOperation;

use crate::error::NetworkError;
use crate::server::ServerState;

/// `GET /health` — simple liveness probe.
pub async fn health() -> &'static str {
    "ok"
}

/// `POST /api/operations` — accept a dispatched operation and persist it.
pub async fn dispatch_operation(
    State(state): State<ServerState>,
    Json(req): Json<DispatchOperationRequest>,
) -> Response {
    match handle_dispatch(&state, req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "dispatch failed");
            let status = match &e {
                NetworkError::BadRequest(_) => StatusCode::BAD_REQUEST,
                NetworkError::NotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (
                status,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

async fn handle_dispatch(
    state: &ServerState,
    req: DispatchOperationRequest,
) -> Result<DispatchOperationResponse, NetworkError> {
    // Decode merkle root and leaf hash from hex.
    let merkle_root = hex_to_32(&req.merkle_root_hex)
        .ok_or_else(|| NetworkError::BadRequest("invalid merkle_root_hex".into()))?;
    let leaf_hash = hex_to_32(&req.leaf_hash_hex)
        .ok_or_else(|| NetworkError::BadRequest("invalid leaf_hash_hex".into()))?;

    // Persist into the commander's `operations` table.
    let op = CommanderOperation {
        id: req.operation_id,
        name: req.operation_name.clone(),
        encrypted_aes_key: req.key_envelope.encrypted_blob.clone(),
        merkle_root,
        leaf_hash,
        total_parts: req.total_parts,
        part_index: req.part_index,
        received_at: Utc::now(),
    };
    state.operation_repo().create(&op).await?;

    // Persist this commander's part.
    state.part_repo().create(&req.part).await?;

    tracing::info!(
        operation = %req.operation_name,
        part_index = req.part_index,
        "operation accepted"
    );

    Ok(DispatchOperationResponse {
        accepted: true,
        message: None,
    })
}

/// `GET /api/parts/:op_id/:part_idx` — serve a part to a peer.
pub async fn fetch_part(
    State(state): State<ServerState>,
    Path((op_id, part_idx)): Path<(Uuid, i32)>,
) -> Response {
    if part_idx < 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "part_idx negative".into(),
            }),
        )
            .into_response();
    }

    match state.part_repo().find(op_id, part_idx as usize).await {
        Ok(Some(part)) => {
            // Look up the operation to read its leaf hash.
            let op = match state.operation_repo().find_by_id(op_id).await {
                Ok(Some(o)) => o,
                Ok(None) => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(ErrorResponse {
                            error: "operation not found".into(),
                        }),
                    )
                        .into_response();
                }
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: e.to_string(),
                        }),
                    )
                        .into_response();
                }
            };

            // This commander only knows the leaf hash for its own index — we have
            // no per-index leaf info for other parts. In this simplified version
            // we always return our own leaf. TODO: store per-part leaf hashes in
            // the `parts` table so peer responses are fully correct.
            let resp = FetchPartResponse {
                part,
                leaf_hash_hex: hex::encode(op.leaf_hash),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "part not found".into(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

fn hex_to_32(s: &str) -> Option<[u8; 32]> {
    let bytes = hex::decode(s).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Some(out)
}

//! End-to-end: bring up the server, hit it with the client, verify DB state.

use mp_network::{Client, Server, ServerState};
use mp_protocol::api::DispatchOperationRequest;
use mp_protocol::{KeyEnvelope, OperationPart};
use mp_storage::{migrate_commander, pool};
use uuid::Uuid;

fn db_url() -> String {
    std::env::var("COMMANDER_DB_URL")
        .unwrap_or_else(|_| "postgresql://mp:mp_dev_pass@pg-aylin/aylin".into())
}

async fn cleanup(pool: &sqlx::PgPool) {
    sqlx::query("TRUNCATE parts, operations, peer_directory, self_info RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .expect("cleanup failed");
}

async fn spawn_server(port: u16) -> ServerState {
    let pool = pool::create(&db_url()).await.unwrap();
    migrate_commander(&pool).await.unwrap();
    cleanup(&pool).await;

    let state = ServerState { pool };
    let state_for_server = state.clone();

    tokio::spawn(async move {
        let server = Server::new(state_for_server);
        let _ = server.run(port).await;
    });

    // Give the server a moment to start listening.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    state
}

#[tokio::test]
async fn health_endpoint_responds() {
    let _state = spawn_server(18443).await;
    let client = Client::new().unwrap();
    let base = "http://127.0.0.1:18443";
    client.health_check(base).await.unwrap();
}

#[tokio::test]
async fn dispatch_persists_operation_and_part() {
    let state = spawn_server(18444).await;
    let client = Client::new().unwrap();
    let base = "http://127.0.0.1:18444";

    let op_id = Uuid::new_v4();
    let req = DispatchOperationRequest {
        operation_id: op_id,
        operation_name: "TestOp".into(),
        total_parts: 3,
        merkle_root_hex: hex::encode([0xABu8; 32]),
        leaf_hash_hex: hex::encode([0xCDu8; 32]),
        part_index: 1,
        key_envelope: KeyEnvelope {
            encrypted_blob: vec![1, 2, 3, 4],
        },
        part: OperationPart::new(op_id, 1, vec![10, 20, 30]),
    };

    let resp = client.dispatch_operation(base, &req).await.unwrap();
    assert!(resp.accepted);

    // Verify rows landed in the DB.
    let op = state
        .operation_repo()
        .find_by_id(op_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(op.name, "TestOp");
    assert_eq!(op.part_index, 1);

    let part = state.part_repo().find(op_id, 1).await.unwrap().unwrap();
    assert_eq!(part.ciphertext_chunk, vec![10, 20, 30]);
}

#[tokio::test]
async fn fetch_part_returns_stored_part() {
    let state = spawn_server(18445).await;
    let client = Client::new().unwrap();
    let base = "http://127.0.0.1:18445";

    let op_id = Uuid::new_v4();
    let req = DispatchOperationRequest {
        operation_id: op_id,
        operation_name: "FetchOp".into(),
        total_parts: 4,
        merkle_root_hex: hex::encode([0u8; 32]),
        leaf_hash_hex: hex::encode([0u8; 32]),
        part_index: 2,
        key_envelope: KeyEnvelope {
            encrypted_blob: vec![9],
        },
        part: OperationPart::new(op_id, 2, vec![100, 101, 102]),
    };
    client.dispatch_operation(base, &req).await.unwrap();
    let _ = state; // keep alive

    let fetched = client.fetch_part(base, op_id, 2).await.unwrap();
    assert_eq!(fetched.part.ciphertext_chunk, vec![100, 101, 102]);
    assert_eq!(fetched.part.part_index, 2);
}

#[tokio::test]
async fn fetch_nonexistent_part_returns_404() {
    let _state = spawn_server(18446).await;
    let client = Client::new().unwrap();
    let base = "http://127.0.0.1:18446";

    let result = client.fetch_part(base, Uuid::new_v4(), 0).await;
    assert!(matches!(result, Err(mp_network::NetworkError::NotFound)));
}

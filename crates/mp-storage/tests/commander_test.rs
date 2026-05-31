//! Commander DB integration testleri.
//! pg-aylin container'ına gerçek bağlanır.

use mp_storage::{pool, migrate_commander};
use mp_storage::commander::{
    SelfInfo, SelfInfoRepository,
    Peer, PeerRepository,
    CommanderOperation, OperationRepository,
    PartRepository,
};
use mp_protocol::{OperationPart, Rank};
use uuid::Uuid;
use chrono::Utc;

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

#[tokio::test]
async fn test_self_info_crud() {
    let pool = pool::create(&db_url()).await.unwrap();
    migrate_commander(&pool).await.unwrap();
    cleanup(&pool).await;

    let repo = SelfInfoRepository::new(pool.clone());

    let info = SelfInfo {
        id: Uuid::new_v4(),
        commander_id: Uuid::new_v4(),
        full_name: "Aylin Kaya".into(),
        email: "aylinkaya@karakuvvetleri.mil.tr".into(),
        rank: Rank::Tuggeneral,
        password_hash: "$argon2id$fake$hash".into(),
        private_key_pem: "MIIE-fake-private-key".into(),
        created_at: Utc::now(),
    };

    repo.create(&info).await.unwrap();

    let got = repo.get().await.unwrap().unwrap();
    assert_eq!(got.full_name, "Aylin Kaya");
    assert_eq!(got.rank, Rank::Tuggeneral);

    let by_email = repo.find_by_email("aylinkaya@karakuvvetleri.mil.tr").await.unwrap().unwrap();
    assert_eq!(by_email.id, info.id);
}

#[tokio::test]
async fn test_peer_upsert() {
    let pool = pool::create(&db_url()).await.unwrap();
    migrate_commander(&pool).await.unwrap();
    cleanup(&pool).await;

    let repo = PeerRepository::new(pool.clone());

    let peer_id = Uuid::new_v4();
    let mut peer = Peer {
        commander_id: peer_id,
        full_name: "Koray Aydin".into(),
        email: "korayaydin@karakuvvetleri.mil.tr".into(),
        rank: Rank::Tumgeneral,
        public_key_pem: "fake-pem-v1".into(),
        network_address: "https://commander-koray:8443".into(),
        created_at: Utc::now(),
    };

    repo.upsert(&peer).await.unwrap();
    let got = repo.find_by_id(peer_id).await.unwrap().unwrap();
    assert_eq!(got.public_key_pem, "fake-pem-v1");

    peer.public_key_pem = "fake-pem-v2".into();
    repo.upsert(&peer).await.unwrap();

    let updated = repo.find_by_id(peer_id).await.unwrap().unwrap();
    assert_eq!(updated.public_key_pem, "fake-pem-v2");

    let all = repo.list_all().await.unwrap();
    assert_eq!(all.len(), 1);
}

#[tokio::test]
async fn test_commander_operation_crud() {
    let pool = pool::create(&db_url()).await.unwrap();
    migrate_commander(&pool).await.unwrap();
    cleanup(&pool).await;

    let repo = OperationRepository::new(pool.clone());

    let op = CommanderOperation {
        id: Uuid::new_v4(),
        name: "Kum Seferi".into(),
        encrypted_aes_key: vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE],
        merkle_root: [0xAB; 32],
        leaf_hash: [0xCD; 32],
        total_parts: 4,
        part_index: 1,
        received_at: Utc::now(),
    };

    repo.create(&op).await.unwrap();

    let got = repo.find_by_id(op.id).await.unwrap().unwrap();
    assert_eq!(got.name, "Kum Seferi");
    assert_eq!(got.merkle_root, [0xAB; 32]);
    assert_eq!(got.part_index, 1);

    let by_name = repo.find_by_name("Kum Seferi").await.unwrap().unwrap();
    assert_eq!(by_name.id, op.id);

    repo.delete(op.id).await.unwrap();
    assert!(repo.find_by_id(op.id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_parts_crud_and_cascade() {
    let pool = pool::create(&db_url()).await.unwrap();
    migrate_commander(&pool).await.unwrap();
    cleanup(&pool).await;

    let op_repo = OperationRepository::new(pool.clone());
    let parts_repo = PartRepository::new(pool.clone());

    let op = CommanderOperation {
        id: Uuid::new_v4(),
        name: "Test Op".into(),
        encrypted_aes_key: vec![1, 2, 3],
        merkle_root: [0; 32],
        leaf_hash: [0; 32],
        total_parts: 3,
        part_index: 0,
        received_at: Utc::now(),
    };
    op_repo.create(&op).await.unwrap();

    for i in 0..3usize {
        let part = OperationPart::new(op.id, i, vec![i as u8; 16]);
        parts_repo.create(&part).await.unwrap();
    }

    let p1 = parts_repo.find(op.id, 1).await.unwrap().unwrap();
    assert_eq!(p1.ciphertext_chunk, vec![1u8; 16]);

    assert!(parts_repo.find(op.id, 99).await.unwrap().is_none());

    let all = parts_repo.find_all_for_operation(op.id).await.unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].part_index, 0);
    assert_eq!(all[2].part_index, 2);

    op_repo.delete(op.id).await.unwrap();
    let remaining = parts_repo.find_all_for_operation(op.id).await.unwrap();
    assert_eq!(remaining.len(), 0);
}

#[tokio::test]
async fn test_part_uniqueness() {
    let pool = pool::create(&db_url()).await.unwrap();
    migrate_commander(&pool).await.unwrap();
    cleanup(&pool).await;

    let op_repo = OperationRepository::new(pool.clone());
    let parts_repo = PartRepository::new(pool.clone());

    let op = CommanderOperation {
        id: Uuid::new_v4(),
        name: "Op".into(),
        encrypted_aes_key: vec![1],
        merkle_root: [0; 32],
        leaf_hash: [0; 32],
        total_parts: 4,
        part_index: 0,
        received_at: Utc::now(),
    };
    op_repo.create(&op).await.unwrap();

    let p = OperationPart::new(op.id, 0, vec![1, 2, 3]);
    parts_repo.create(&p).await.unwrap();

    let p_dup = OperationPart::new(op.id, 0, vec![4, 5, 6]);
    assert!(parts_repo.create(&p_dup).await.is_err());
}

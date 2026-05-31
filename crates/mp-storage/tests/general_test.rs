//! General DB integration testleri. Gerçek Postgres'e bağlanır.
//!
//! Çalıştırmak için:
//!   GENERAL_DB_URL=postgresql://mp:mp_dev_pass@pg-general/general \
//!     cargo test -p mp-storage --test general_test -- --test-threads=1

use mp_protocol::{Commander, Operation, Rank, Unit};
use mp_storage::general::{
    CommanderRepository, OperationRecipient, OperationRepository, UnitRepository,
};
use mp_storage::{migrate_general, pool};
use uuid::Uuid;

fn db_url() -> String {
    std::env::var("GENERAL_DB_URL")
        .unwrap_or_else(|_| "postgresql://mp:mp_dev_pass@pg-general/general".into())
}

/// Test öncesi DB'yi temizle (tabloları boşalt).
async fn cleanup(pool: &sqlx::PgPool) {
    sqlx::query(
        "TRUNCATE operation_recipients, operations, commanders, units RESTART IDENTITY CASCADE",
    )
    .execute(pool)
    .await
    .expect("cleanup failed");
}

#[tokio::test]
async fn test_unit_crud() {
    let pool = pool::create(&db_url()).await.unwrap();
    migrate_general(&pool).await.unwrap();
    cleanup(&pool).await;

    let repo = UnitRepository::new(pool.clone());
    let unit = Unit::new(2, "107.Topçu".into(), "Alayi".into(), "Siverek".into());

    repo.create(&unit).await.unwrap();

    let found = repo.find_by_id(unit.id).await.unwrap().unwrap();
    assert_eq!(found.name, "107.Topçu");
    assert_eq!(found.corps_number, 2);

    let by_loc = repo.find_by_location("Siverek").await.unwrap();
    assert_eq!(by_loc.len(), 1);

    repo.delete(unit.id).await.unwrap();
    assert!(repo.find_by_id(unit.id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_commander_crud() {
    let pool = pool::create(&db_url()).await.unwrap();
    migrate_general(&pool).await.unwrap();
    cleanup(&pool).await;

    let repo = CommanderRepository::new(pool.clone());
    let cmd = Commander::new(
        "Aylin Kaya".into(),
        "aylinkaya@karakuvvetleri.mil.tr".into(),
        Rank::Tuggeneral,
        "MIIBIjAN-fake-pem-data".into(),
        "https://commander-aylin:8443".into(),
    );

    repo.create(&cmd).await.unwrap();

    let by_id = repo.find_by_id(cmd.id).await.unwrap().unwrap();
    assert_eq!(by_id.full_name, "Aylin Kaya");
    assert_eq!(by_id.rank, Rank::Tuggeneral);

    let by_email = repo
        .find_by_email("aylinkaya@karakuvvetleri.mil.tr")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(by_email.id, cmd.id);

    let by_rank = repo.find_by_rank(Rank::Tuggeneral).await.unwrap();
    assert_eq!(by_rank.len(), 1);
}

#[tokio::test]
async fn test_operation_with_recipients() {
    let pool = pool::create(&db_url()).await.unwrap();
    migrate_general(&pool).await.unwrap();
    cleanup(&pool).await;

    let cmd_repo = CommanderRepository::new(pool.clone());
    let op_repo = OperationRepository::new(pool.clone());

    // 3 komutan kaydet
    let cmds: Vec<Commander> = (0..3)
        .map(|i| {
            Commander::new(
                format!("Komutan {}", i),
                format!("kmd{}@test.tr", i),
                Rank::Tuggeneral,
                "fake-pem".into(),
                format!("https://commander-{}:8443", i),
            )
        })
        .collect();

    for c in &cmds {
        cmd_repo.create(c).await.unwrap();
    }

    // Operasyon oluştur
    let creator = cmds[0].id;
    let op = Operation::new("Kum Seferi".into(), [0xAB; 32], 3);
    op_repo.create(&op, creator).await.unwrap();

    // Alıcıları ekle
    let recipients: Vec<OperationRecipient> = cmds
        .iter()
        .enumerate()
        .map(|(i, c)| OperationRecipient {
            id: Uuid::new_v4(),
            operation_id: op.id,
            commander_id: c.id,
            leaf_hash: [i as u8; 32],
            part_index: i as i32,
        })
        .collect();
    op_repo.add_recipients(&recipients).await.unwrap();

    // Doğrula
    let found_op = op_repo.find_by_id(op.id).await.unwrap().unwrap();
    assert_eq!(found_op.name, "Kum Seferi");
    assert_eq!(found_op.total_parts, 3);

    let found_recipients = op_repo.find_recipients(op.id).await.unwrap();
    assert_eq!(found_recipients.len(), 3);
    // part_index'e göre sıralı dönmeli
    assert_eq!(found_recipients[0].part_index, 0);
    assert_eq!(found_recipients[2].part_index, 2);
}

#[tokio::test]
async fn test_email_uniqueness() {
    let pool = pool::create(&db_url()).await.unwrap();
    migrate_general(&pool).await.unwrap();
    cleanup(&pool).await;

    let repo = CommanderRepository::new(pool.clone());
    let cmd1 = Commander::new(
        "A".into(),
        "same@test.tr".into(),
        Rank::Albay,
        "p".into(),
        "addr1".into(),
    );
    let cmd2 = Commander::new(
        "B".into(),
        "same@test.tr".into(),
        Rank::Albay,
        "p".into(),
        "addr2".into(),
    );

    repo.create(&cmd1).await.unwrap();
    // İkincisi unique constraint'e takılmalı
    assert!(repo.create(&cmd2).await.is_err());
}

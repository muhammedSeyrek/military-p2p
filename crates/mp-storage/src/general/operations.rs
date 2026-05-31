use crate::error::{Result, StorageError};
use chrono::{DateTime, Utc};
use mp_protocol::Operation;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OperationRecipient {
    pub id: Uuid,
    pub operation_id: Uuid,
    pub commander_id: Uuid,
    pub leaf_hash: [u8; 32],
    pub part_index: i32,
}

pub struct OperationRepository {
    pool: PgPool,
}

impl OperationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, op: &Operation, created_by: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO operations
                (id, name, merkle_root, total_parts, status, created_by, dispatched_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(op.id)
        .bind(&op.name)
        .bind(&op.merkle_root[..])
        .bind(op.total_parts as i32)
        .bind("dispatched")
        .bind(created_by)
        .bind(op.created_at)
        .bind(op.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Operation>> {
        let row = sqlx::query_as::<_, OperationRow>(
            r#"
            SELECT id, name, merkle_root, total_parts, created_at
            FROM operations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(TryInto::try_into).transpose()
    }

    pub async fn list_recent(&self, limit: i64) -> Result<Vec<Operation>> {
        let rows = sqlx::query_as::<_, OperationRow>(
            r#"
            SELECT id, name, merkle_root, total_parts, created_at
            FROM operations
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn add_recipients(&self, recipients: &[OperationRecipient]) -> Result<()> {
        // sqlx'in batch insert için zarif yolu: bir transaction içinde döngü.
        let mut tx = self.pool.begin().await?;

        for r in recipients {
            sqlx::query(
                r#"
                INSERT INTO operation_recipients
                    (id, operation_id, commander_id, leaf_hash, part_index)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(r.id)
            .bind(r.operation_id)
            .bind(r.commander_id)
            .bind(&r.leaf_hash[..])
            .bind(r.part_index)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn find_recipients(&self, operation_id: Uuid) -> Result<Vec<OperationRecipient>> {
        let rows = sqlx::query_as::<_, RecipientRow>(
            r#"
            SELECT id, operation_id, commander_id, leaf_hash, part_index
            FROM operation_recipients
            WHERE operation_id = $1
            ORDER BY part_index
            "#,
        )
        .bind(operation_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryInto::try_into).collect()
    }
}

#[derive(sqlx::FromRow)]
struct OperationRow {
    id: Uuid,
    name: String,
    merkle_root: Vec<u8>,
    total_parts: i32,
    created_at: DateTime<Utc>,
}

impl TryFrom<OperationRow> for Operation {
    type Error = StorageError;

    fn try_from(r: OperationRow) -> Result<Self> {
        if r.merkle_root.len() != 32 {
            return Err(StorageError::InvalidData(format!(
                "merkle_root size mismatch: {}",
                r.merkle_root.len()
            )));
        }
        let mut root = [0u8; 32];
        root.copy_from_slice(&r.merkle_root);

        Ok(Operation {
            id: r.id,
            name: r.name,
            merkle_root: root,
            total_parts: r.total_parts as usize,
            created_at: r.created_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct RecipientRow {
    id: Uuid,
    operation_id: Uuid,
    commander_id: Uuid,
    leaf_hash: Vec<u8>,
    part_index: i32,
}

impl TryFrom<RecipientRow> for OperationRecipient {
    type Error = StorageError;

    fn try_from(r: RecipientRow) -> Result<Self> {
        if r.leaf_hash.len() != 32 {
            return Err(StorageError::InvalidData(format!(
                "leaf_hash size mismatch: {}",
                r.leaf_hash.len()
            )));
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&r.leaf_hash);

        Ok(OperationRecipient {
            id: r.id,
            operation_id: r.operation_id,
            commander_id: r.commander_id,
            leaf_hash: hash,
            part_index: r.part_index,
        })
    }
}

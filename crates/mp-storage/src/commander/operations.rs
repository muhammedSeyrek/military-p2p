use crate::error::{Result, StorageError};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CommanderOperation {
    pub id: Uuid,
    pub name: String,

    pub encrypted_aes_key: Vec<u8>,
    pub merkle_root: [u8; 32],
    pub leaf_hash: [u8; 32],
    pub total_parts: usize,
    pub part_index: usize,
    pub received_at: DateTime<Utc>,
}

pub struct OperationRepository {
    pool: PgPool,
}

impl OperationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, op: &CommanderOperation) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO operations
                (id, name, encrypted_aes_key, merkle_root, leaf_hash,
                 total_parts, part_index, received_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(op.id)
        .bind(&op.name)
        .bind(&op.encrypted_aes_key)
        .bind(&op.merkle_root[..])
        .bind(&op.leaf_hash[..])
        .bind(op.total_parts as i32)
        .bind(op.part_index as i32)
        .bind(op.received_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<CommanderOperation>> {
        let row = sqlx::query_as::<_, OpRow>(
            r#"
            SELECT id, name, encrypted_aes_key, merkle_root, leaf_hash,
                   total_parts, part_index, received_at
            FROM operations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(TryInto::try_into).transpose()
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<CommanderOperation>> {
        let row = sqlx::query_as::<_, OpRow>(
            r#"
            SELECT id, name, encrypted_aes_key, merkle_root, leaf_hash,
                   total_parts, part_index, received_at
            FROM operations
            WHERE name = $1
            ORDER BY received_at DESC
            LIMIT 1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        row.map(TryInto::try_into).transpose()
    }

    pub async fn list_recent(&self, limit: i64) -> Result<Vec<CommanderOperation>> {
        let rows = sqlx::query_as::<_, OpRow>(
            r#"
            SELECT id, name, encrypted_aes_key, merkle_root, leaf_hash,
                   total_parts, part_index, received_at
            FROM operations
            ORDER BY received_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM operations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct OpRow {
    id: Uuid,
    name: String,
    encrypted_aes_key: Vec<u8>,
    merkle_root: Vec<u8>,
    leaf_hash: Vec<u8>,
    total_parts: i32,
    part_index: i32,
    received_at: DateTime<Utc>,
}

impl TryFrom<OpRow> for CommanderOperation {
    type Error = StorageError;

    fn try_from(r: OpRow) -> Result<Self> {
        if r.merkle_root.len() != 32 {
            return Err(StorageError::InvalidData(format!(
                "merkle_root size: {}",
                r.merkle_root.len()
            )));
        }
        if r.leaf_hash.len() != 32 {
            return Err(StorageError::InvalidData(format!(
                "leaf_hash size: {}",
                r.leaf_hash.len()
            )));
        }

        let mut merkle_root = [0u8; 32];
        merkle_root.copy_from_slice(&r.merkle_root);
        let mut leaf_hash = [0u8; 32];
        leaf_hash.copy_from_slice(&r.leaf_hash);

        Ok(CommanderOperation {
            id: r.id,
            name: r.name,
            encrypted_aes_key: r.encrypted_aes_key,
            merkle_root,
            leaf_hash,
            total_parts: r.total_parts as usize,
            part_index: r.part_index as usize,
            received_at: r.received_at,
        })
    }
}

use crate::error::Result;
use mp_protocol::OperationPart;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PartRepository {
    pool: PgPool,
}

impl PartRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, part: &OperationPart) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO parts (id, operation_id, part_index, ciphertext_chunk)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(part.operation_id)
        .bind(part.part_index as i32)
        .bind(&part.ciphertext_chunk)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find(
        &self,
        operation_id: Uuid,
        part_index: usize,
    ) -> Result<Option<OperationPart>> {
        let row: Option<(Uuid, i32, Vec<u8>)> = sqlx::query_as(
            r#"
            SELECT operation_id, part_index, ciphertext_chunk
            FROM parts
            WHERE operation_id = $1 AND part_index = $2
            "#,
        )
        .bind(operation_id)
        .bind(part_index as i32)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(op_id, idx, chunk)| OperationPart {
            operation_id: op_id,
            part_index: idx as usize,
            ciphertext_chunk: chunk,
        }))
    }

    pub async fn find_all_for_operation(&self, operation_id: Uuid) -> Result<Vec<OperationPart>> {
        let rows: Vec<(Uuid, i32, Vec<u8>)> = sqlx::query_as(
            r#"
            SELECT operation_id, part_index, ciphertext_chunk
            FROM parts
            WHERE operation_id = $1
            ORDER BY part_index
            "#,
        )
        .bind(operation_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(op_id, idx, chunk)| OperationPart {
                operation_id: op_id,
                part_index: idx as usize,
                ciphertext_chunk: chunk,
            })
            .collect())
    }

    pub async fn delete_for_operation(&self, operation_id: Uuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM parts WHERE operation_id = $1")
            .bind(operation_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}

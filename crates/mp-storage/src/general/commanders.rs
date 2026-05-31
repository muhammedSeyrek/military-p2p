use crate::error::{Result, StorageError};
use mp_protocol::{Commander, Rank};
use sqlx::PgPool;
use uuid::Uuid;

pub struct CommanderRepository {
    pool: PgPool,
}

impl CommanderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, c: &Commander) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO commanders
                (id, full_name, email, rank, public_key_pem, unit_id, network_address, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(c.id)
        .bind(&c.full_name)
        .bind(&c.email)
        .bind(c.rank.as_str())
        .bind(&c.public_key_pem)
        .bind(c.unit_id)
        .bind(&c.network_address)
        .bind(c.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Commander>> {
        let row = sqlx::query_as::<_, CommanderRow>(
            r#"
            SELECT id, full_name, email, rank, public_key_pem, unit_id, network_address, created_at
            FROM commanders
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(TryInto::try_into).transpose()
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<Commander>> {
        let row = sqlx::query_as::<_, CommanderRow>(
            r#"
            SELECT id, full_name, email, rank, public_key_pem, unit_id, network_address, created_at
            FROM commanders
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        row.map(TryInto::try_into).transpose()
    }

    pub async fn list_all(&self) -> Result<Vec<Commander>> {
        let rows = sqlx::query_as::<_, CommanderRow>(
            r#"
            SELECT id, full_name, email, rank, public_key_pem, unit_id, network_address, created_at
            FROM commanders
            ORDER BY full_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn find_by_rank(&self, rank: Rank) -> Result<Vec<Commander>> {
        let rows = sqlx::query_as::<_, CommanderRow>(
            r#"
            SELECT id, full_name, email, rank, public_key_pem, unit_id, network_address, created_at
            FROM commanders
            WHERE rank = $1
            ORDER BY full_name
            "#,
        )
        .bind(rank.as_str())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM commanders WHERE id = $1")
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
struct CommanderRow {
    id: Uuid,
    full_name: String,
    email: String,
    rank: String,
    public_key_pem: String,
    unit_id: Option<Uuid>,
    network_address: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<CommanderRow> for Commander {
    type Error = StorageError;

    fn try_from(r: CommanderRow) -> Result<Self> {
        let rank = Rank::from_str(&r.rank)
            .map_err(|e| StorageError::InvalidData(format!("rank: {}", e)))?;

        Ok(Commander {
            id: r.id,
            full_name: r.full_name,
            email: r.email,
            rank,
            public_key_pem: r.public_key_pem,
            unit_id: r.unit_id,
            network_address: r.network_address,
            created_at: r.created_at,
        })
    }
}

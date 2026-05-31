use crate::error::{Result, StorageError};
use chrono::{DateTime, Utc};
use mp_protocol::Rank;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SelfInfo {
    pub id: Uuid,
    pub commander_id: Uuid,
    pub full_name: String,
    pub email: String,
    pub rank: Rank,

    pub password_hash: String,
    pub private_key_pem: String,
    pub created_at: DateTime<Utc>,
}

pub struct SelfInfoRepository {
    pool: PgPool,
}

impl SelfInfoRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, info: &SelfInfo) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO self_info
                (id, commander_id, full_name, email, rank,
                 password_hash, private_key_pem, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(info.id)
        .bind(info.commander_id)
        .bind(&info.full_name)
        .bind(&info.email)
        .bind(info.rank.as_str())
        .bind(&info.password_hash)
        .bind(&info.private_key_pem)
        .bind(info.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get(&self) -> Result<Option<SelfInfo>> {
        let row = sqlx::query_as::<_, SelfInfoRow>(
            r#"
            SELECT id, commander_id, full_name, email, rank,
                   password_hash, private_key_pem, created_at
            FROM self_info
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(TryInto::try_into).transpose()
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<SelfInfo>> {
        let row = sqlx::query_as::<_, SelfInfoRow>(
            r#"
            SELECT id, commander_id, full_name, email, rank,
                   password_hash, private_key_pem, created_at
            FROM self_info
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        row.map(TryInto::try_into).transpose()
    }
}

#[derive(sqlx::FromRow)]
struct SelfInfoRow {
    id: Uuid,
    commander_id: Uuid,
    full_name: String,
    email: String,
    rank: String,
    password_hash: String,
    private_key_pem: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<SelfInfoRow> for SelfInfo {
    type Error = StorageError;

    fn try_from(r: SelfInfoRow) -> Result<Self> {
        let rank = Rank::from_str(&r.rank)
            .map_err(|e| StorageError::InvalidData(format!("rank: {}", e)))?;

        Ok(SelfInfo {
            id: r.id,
            commander_id: r.commander_id,
            full_name: r.full_name,
            email: r.email,
            rank,
            password_hash: r.password_hash,
            private_key_pem: r.private_key_pem,
            created_at: r.created_at,
        })
    }
}

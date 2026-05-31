use crate::error::{Result, StorageError};
use chrono::{DateTime, Utc};
use mp_protocol::Rank;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Peer {
    pub commander_id: Uuid,
    pub full_name: String,
    pub email: String,
    pub rank: Rank,
    pub public_key_pem: String,
    pub network_address: String,
    pub created_at: DateTime<Utc>,
}

pub struct PeerRepository {
    pool: PgPool,
}

impl PeerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, peer: &Peer) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO peer_directory
                (commander_id, full_name, email, rank,
                 public_key_pem, network_address, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (commander_id) DO UPDATE SET
                full_name       = EXCLUDED.full_name,
                email           = EXCLUDED.email,
                rank            = EXCLUDED.rank,
                public_key_pem  = EXCLUDED.public_key_pem,
                network_address = EXCLUDED.network_address
            "#,
        )
        .bind(peer.commander_id)
        .bind(&peer.full_name)
        .bind(&peer.email)
        .bind(peer.rank.as_str())
        .bind(&peer.public_key_pem)
        .bind(&peer.network_address)
        .bind(peer.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Peer>> {
        let row = sqlx::query_as::<_, PeerRow>(
            r#"
            SELECT commander_id, full_name, email, rank,
                   public_key_pem, network_address, created_at
            FROM peer_directory
            WHERE commander_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(TryInto::try_into).transpose()
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<Peer>> {
        let row = sqlx::query_as::<_, PeerRow>(
            r#"
            SELECT commander_id, full_name, email, rank,
                   public_key_pem, network_address, created_at
            FROM peer_directory
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        row.map(TryInto::try_into).transpose()
    }

    pub async fn list_all(&self) -> Result<Vec<Peer>> {
        let rows = sqlx::query_as::<_, PeerRow>(
            r#"
            SELECT commander_id, full_name, email, rank,
                   public_key_pem, network_address, created_at
            FROM peer_directory
            ORDER BY full_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryInto::try_into).collect()
    }
}

#[derive(sqlx::FromRow)]
struct PeerRow {
    commander_id: Uuid,
    full_name: String,
    email: String,
    rank: String,
    public_key_pem: String,
    network_address: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<PeerRow> for Peer {
    type Error = StorageError;

    fn try_from(r: PeerRow) -> Result<Self> {
        let rank = Rank::from_str(&r.rank)
            .map_err(|e| StorageError::InvalidData(format!("rank: {}", e)))?;

        Ok(Peer {
            commander_id: r.commander_id,
            full_name: r.full_name,
            email: r.email,
            rank,
            public_key_pem: r.public_key_pem,
            network_address: r.network_address,
            created_at: r.created_at,
        })
    }
}

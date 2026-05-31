use crate::error::{Result, StorageError};
use mp_protocol::Unit;
use sqlx::PgPool;
use uuid::Uuid;

pub struct UnitRepository {
    pool: PgPool,
}

impl UnitRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, unit: &Unit) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO units (id, corps_number, name, unit_type, location, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(unit.id)
        .bind(unit.corps_number)
        .bind(&unit.name)
        .bind(&unit.unit_type)
        .bind(&unit.location)
        .bind(unit.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Unit>> {
        let row = sqlx::query_as::<_, UnitRow>(
            r#"
            SELECT id, corps_number, name, unit_type, location, created_at
            FROM units
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    pub async fn find_by_location(&self, location: &str) -> Result<Vec<Unit>> {
        let rows = sqlx::query_as::<_, UnitRow>(
            r#"
            SELECT id, corps_number, name, unit_type, location, created_at
            FROM units
            WHERE location = $1
            ORDER BY name
            "#,
        )
        .bind(location)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn list_all(&self) -> Result<Vec<Unit>> {
        let rows = sqlx::query_as::<_, UnitRow>(
            r#"
            SELECT id, corps_number, name, unit_type, location, created_at
            FROM units
            ORDER BY corps_number, name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Silmek için (test cleanup'ta kullanılır).
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM units WHERE id = $1")
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
struct UnitRow {
    id: Uuid,
    corps_number: i32,
    name: String,
    unit_type: String,
    location: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<UnitRow> for Unit {
    fn from(r: UnitRow) -> Self {
        Unit {
            id: r.id,
            corps_number: r.corps_number,
            name: r.name,
            unit_type: r.unit_type,
            location: r.location,
            created_at: r.created_at,
        }
    }
}

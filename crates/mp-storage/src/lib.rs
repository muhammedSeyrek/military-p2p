pub mod commander;
pub mod error;
pub mod general;
pub mod pool;

pub use error::StorageError;
pub use pool::PgPool;

pub static MIGRATOR_GENERAL: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/general");

pub static MIGRATOR_COMMANDER: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/commander");

pub async fn migrate_general(pool: &PgPool) -> Result<(), StorageError> {
    MIGRATOR_GENERAL
        .run(pool)
        .await
        .map_err(|e| StorageError::Migration(e.to_string()))?;
    Ok(())
}

pub async fn migrate_commander(pool: &PgPool) -> Result<(), StorageError> {
    MIGRATOR_COMMANDER
        .run(pool)
        .await
        .map_err(|e| StorageError::Migration(e.to_string()))?;
    Ok(())
}

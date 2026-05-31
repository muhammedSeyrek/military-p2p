//! Postgres connection pool factory.

use crate::error::Result;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub type PgPool = sqlx::PgPool;

pub async fn create(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .test_before_acquire(true)
        .connect(database_url)
        .await?;

    tracing::info!(url = %sanitize(database_url), "Postgres pool created");
    Ok(pool)
}

pub fn clone(pool: &PgPool) -> PgPool {
    pool.clone()
}

/// URL'den şifreyi maskele (log için).
fn sanitize(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(scheme_end) = url.find("://") {
            let scheme = &url[..scheme_end + 3];
            let host = &url[at_pos..];
            return format!("{}***{}", scheme, host);
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_hides_password() {
        let s = sanitize("postgresql://mp:secret@pg-aylin/aylin");
        assert!(!s.contains("secret"));
        assert!(s.contains("pg-aylin"));
    }

    #[test]
    fn sanitize_handles_url_without_password() {
        let s = sanitize("postgresql://localhost/db");
        assert!(s.contains("localhost"));
    }
}

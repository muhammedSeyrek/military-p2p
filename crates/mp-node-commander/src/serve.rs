//! HTTP server mode.

use anyhow::Result;
use mp_network::{Server, ServerState};
use mp_storage::PgPool;

pub async fn run(pool: PgPool, port: u16) -> Result<()> {
    let state = ServerState { pool };
    let server = Server::new(state);
    tracing::info!(port, "Starting commander HTTP server");
    server.run(port).await?;
    Ok(())
}

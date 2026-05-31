//! axum HTTP server.
//!
//! Currently runs plaintext HTTP. TLS support (`cert` + `tls` modules)
//! is implemented but not wired into the server yet.

use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use mp_storage::commander::{OperationRepository, PartRepository};
use mp_storage::PgPool;

use crate::error::{NetworkError, Result};
use crate::handlers;

#[derive(Clone)]
pub struct ServerState {
    pub pool: PgPool,
}

impl ServerState {
    pub fn operation_repo(&self) -> OperationRepository {
        OperationRepository::new(self.pool.clone())
    }
    pub fn part_repo(&self) -> PartRepository {
        PartRepository::new(self.pool.clone())
    }
}

pub struct Server {
    state: ServerState,
}

impl Server {
    pub fn new(state: ServerState) -> Self {
        Self { state }
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/api/operations", post(handlers::dispatch_operation))
            .route("/api/parts/:op_id/:part_idx", get(handlers::fetch_part))
            .route("/health", get(handlers::health))
            .with_state(self.state.clone())
    }

    /// Start listening on `0.0.0.0:port` over plain HTTP.
    pub async fn run(self, port: u16) -> Result<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| NetworkError::Bind(format!("{}: {}", addr, e)))?;

        tracing::info!(?addr, "HTTP server listening");

        axum::serve(listener, self.router()).await?;
        Ok(())
    }
}

pub fn build_router(state: ServerState) -> Router {
    Server::new(state).router()
}

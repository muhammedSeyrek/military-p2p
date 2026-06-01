//! Network layer — axum HTTP server + reqwest HTTP client.
//!
//! This crate exposes two main components:
//! - [`Server`]: axum-based HTTP server that listens on a port
//! - [`Client`]: reqwest-based HTTP client that talks to other nodes
//!
//! TLS scaffolding (`cert`, `tls` modules) is in place but disabled in the
//! demo build. Production deployments would wire it through `axum_server`.

pub mod cert;
pub mod client;
pub mod error;
pub mod handlers;
pub mod server;
pub mod tls;

pub use client::Client;
pub use error::NetworkError;
pub use server::{Server, ServerState};

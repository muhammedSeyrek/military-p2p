//! Protocol-layer types and helpers for the Military P2P system.
//!
//! - `commander`: `Commander`, `Unit`, `Rank` domain types
//! - `operation`: `Operation`, `OperationPart`, `KeyEnvelope` payload types
//! - `raid`: RAID-0 split/join over byte arrays
//! - `multi_message`: pack/unpack format for per-recipient messages
//! - `api`: HTTP request/response DTOs

pub mod api;
pub mod commander;
pub mod error;
pub mod multi_message;
pub mod operation;
pub mod raid;

pub use commander::{Commander, Rank, Unit};
pub use error::ProtocolError;
pub use operation::{KeyEnvelope, Operation, OperationPart};

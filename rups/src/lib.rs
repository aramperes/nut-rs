#![deny(missing_docs)]

//! # rups
//!
//! The `rups` crate provides a network client implementation
//! for Network UPS Tools (NUT) servers.

pub use config::*;
pub use error::*;
pub use util::*;
pub use var::*;

/// Blocking client implementation for NUT.
pub mod blocking;
/// Async client implementation for NUT, using Tokio.
#[cfg(feature = "async")]
pub mod tokio;

mod cmd;
mod config;
mod error;
/// Bi-directional networking protocol implementation.
#[allow(dead_code)]
mod proto;
#[cfg(feature = "ssl")]
mod ssl;
mod util;
mod var;

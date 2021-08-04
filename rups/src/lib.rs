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
/// NUT protocol implementation (v1.2).
///
/// Reference: <https://networkupstools.org/docs/developer-guide.chunked/ar01s09.html>
#[allow(dead_code)]
#[macro_use]
pub mod proto;
/// Async client implementation for NUT, using Tokio.
#[cfg(feature = "async")]
pub mod tokio;

mod cmd;
mod config;
mod error;
#[cfg(feature = "ssl")]
mod ssl;
mod util;
mod var;

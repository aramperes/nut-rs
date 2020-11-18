#![deny(missing_docs)]

//! # nut-client
//!
//! The `nut-client` crate provides a network client implementation
//! for Network UPS Tools (NUT) servers.

pub use config::*;
pub use error::*;

/// Blocking client implementation for NUT.
pub mod blocking;

mod cmd;
mod config;
mod error;

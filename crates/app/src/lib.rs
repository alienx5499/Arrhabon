//! Use cases and ports for vault commands and queries.
#![forbid(unsafe_code)]

pub mod address;
pub mod commands;
mod error;
#[cfg(any(test, feature = "fakes"))]
pub mod fakes;
pub mod ports;
pub mod queries;
#[cfg(test)]
mod tests;

pub use error::{AppError, PortError};

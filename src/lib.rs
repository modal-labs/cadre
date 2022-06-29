//! Cadre is a simple, self-hosted, high-performance remote configuration
//! service.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod client;
pub mod server;

pub use crate::cli::Args;
pub use crate::client::CadreClient;

//! Cadre is a simple, self-hosted, high-performance, and strongly consistent
//! remote configuration service.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cli;
pub mod client;
pub mod secrets;
pub mod server;
pub mod storage;
pub mod template;

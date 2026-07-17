//! Lunar-calendar MCP server.
//!
//! The crate keeps transport, MCP metadata, and domain engines separate so
//! calendar engines can be upgraded without changing the public tool contract.

pub mod contract;
pub mod domain;
pub mod mcp;
pub mod transport;
pub mod validation;

pub use transport::http::app;

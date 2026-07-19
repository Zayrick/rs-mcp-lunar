//! Model Context Protocol presentation layer.

pub mod protocol;

#[cfg(not(target_arch = "wasm32"))]
mod registry;
#[cfg(not(target_arch = "wasm32"))]
mod server;

pub use crate::contract::{MCP_PATH, SERVER_NAME, SERVER_VERSION, server_info_text};
#[cfg(not(target_arch = "wasm32"))]
pub use registry::tools;
#[cfg(not(target_arch = "wasm32"))]
pub use server::LunarMcpServer;

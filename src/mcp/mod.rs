//! Model Context Protocol presentation layer.

mod registry;
mod server;

pub use registry::{MCP_PATH, SERVER_NAME, SERVER_VERSION, server_info_text, tools};
pub use server::LunarMcpServer;

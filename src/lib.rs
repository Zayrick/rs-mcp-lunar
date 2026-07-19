//! Lunar-calendar MCP server.
//!
//! The crate keeps transport, MCP metadata, and domain engines separate so
//! calendar engines can be upgraded without changing the public tool contract.

pub mod contract;
pub mod domain;
pub mod mcp;
pub mod transport;
pub mod validation;

/// Cloudflare Workers fetch entry point.
///
/// `worker-build` exposes this function through its generated JavaScript/Wasm
/// module.
#[cfg(target_arch = "wasm32")]
#[worker::event(fetch)]
pub async fn fetch(
    request: worker::Request,
    env: worker::Env,
    _context: worker::Context,
) -> worker::Result<worker::Response> {
    transport::cloudflare::handle(request, &env).await
}

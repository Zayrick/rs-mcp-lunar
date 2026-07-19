#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;

#[cfg(not(target_arch = "wasm32"))]
use anyhow::{Context, Result};
#[cfg(not(target_arch = "wasm32"))]
use tokio::net::TcpListener;
#[cfg(not(target_arch = "wasm32"))]
use tracing::info;
#[cfg(not(target_arch = "wasm32"))]
use tracing_subscriber::EnvFilter;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let address = std::env::var("LUNAR_MCP_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8788".to_owned())
        .parse::<SocketAddr>()
        .context("LUNAR_MCP_ADDR must be a valid socket address")?;
    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("failed to bind {address}"))?;

    info!(%address, "Lunar Calendar MCP server listening");
    axum::serve(listener, rs_mcp_lunar::app())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("HTTP server stopped unexpectedly")
}

#[cfg(not(target_arch = "wasm32"))]
async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "failed to install Ctrl+C handler");
    }
}

// `cargo build --target wasm32-unknown-unknown` builds every package target.
// The deployable Worker is the library cdylib; this no-op keeps the native
// server binary target harmless on Wasm while preserving the existing native
// `cargo build --release` workflow.
#[cfg(target_arch = "wasm32")]
fn main() {}

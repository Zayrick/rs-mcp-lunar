use std::net::SocketAddr;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

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

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "failed to install Ctrl+C handler");
    }
}
